# 30 Performance Tips for Passcore (Easy → Hard)

> **Note**: These tips are written for **full Unicode support** (not ASCII-only).
> Line numbers reference `score.rs` as of the current version.

>FPS Before: 0.113863 ms (mean for 1000 random passwords)
> FPS After: TBD
---

## STD-Only Tips (25 tips)

### Easy (Minimal Code Changes)

---

- [x] **1. You're already using `chars().count()` correctly for Unicode!**
  
  **Location**: `score.rs`, line 32
  
  **What it does**: You have `password.chars().count()` which correctly counts Unicode characters (called "grapheme clusters" or code points), not bytes.
  
  **Why it matters**: The string `"café"` is 4 characters but 5 bytes (the `é` takes 2 bytes in UTF-8). Using `.len()` would return 5, but `.chars().count()` correctly returns 4.
  
  **Current code** (already correct):
  ```rust
  let length = password.chars().count();
  ```
  
  **Status**: ✅ Already implemented! No changes needed.

---

- [x] **2. Early exit in `score_variety` when all character types found**
  
  **Location**: `score.rs`, lines 77-79
  
  **What it does**: Once you've found at least one lowercase, one uppercase, one digit, and one symbol, there's no reason to keep checking the rest of the password.
  
  **Why it matters**: For a password like `"Abc123!xxxxxxxxxxxxxxxxxxx"`, after checking the first 7 characters you already know all 4 types exist. Without early exit, you'd check all 27 characters.
  
  **Current code** (already correct):
  ```rust
  for c in password.chars() {
      if lower_count && upper_count && digit_count && symbol_count {
          break;  // ← This is the early exit!
      }
      // ... rest of checks
  }
  ```
  
  **Status**: ✅ Already implemented! No changes needed.

---

- [x] **3. Cache first and last character outside the loop in `score_penalties`**
  
  **Location**: `score.rs`, lines 173-174 and 177-178
  
  **What's happening now**: You calculate the first and last character of the user's password ONCE (good!), but then you calculate the first and last character of EACH common password INSIDE the loop (wasteful!).
  
  **Current code**:
  ```rust
  // Line 173-174: Calculated once (good!)
  let first = normalized.chars().next();
  let last = normalized.chars().rev().next();
  
  for common in candidates {
      // Line 177-178: Calculated 100,000 times! (bad!)
      let cfirst = common.chars().next();
      let clast = common.chars().rev().next();
      // ...
  }
  ```
  
  **The problem**: Every time you call `.chars()`, Rust creates an iterator object. Calling `.next()` on it is fast, but creating ~100,000 iterators adds up.
  
  **The fix**: Pre-compute the first/last characters when building the `PASSWORD_SET`. Store them alongside each password:
  
  ```rust
  // Instead of HashSet<String>, use:
  pub static PASSWORD_DATA: Lazy<Vec<PasswordEntry>> = Lazy::new(|| {
      include_str!("../data/100k-most-used-passwords-NCSC.txt")
          .lines()
          .map(|line| {
              let pw = line.trim().to_lowercase();
              let first = pw.chars().next();
              let last = pw.chars().last();
              PasswordEntry { password: pw, first, last }
          })
          .collect()
  });
  ```
  
  **Expected speedup**: 5-15% faster in `score_penalties`

---

- [Nope I can't] **4. Use `&'static str` instead of `String` in `HashSet`**
  
  **Location**: `score.rs`, lines 4-11
  
  **What's happening now**: You use `include_str!` to embed the password file at compile time. This gives you a `&'static str` (a reference to text that lives forever in your program). But then you call `.to_lowercase()` which creates a NEW `String` for each password.
  
  **Current code**:
  ```rust
  pub static PASSWORD_SET: Lazy<HashSet<String>> = Lazy::new(|| {
      let content = include_str!("../data/100k-most-used-passwords-NCSC.txt");
      content
          .lines()
          .map(|line| line.trim().to_lowercase())  // Creates 100k Strings!
          .collect()
  });
  ```
  
  **The problem**: You're allocating ~100,000 `String` objects at startup. Each `String` has:
  - A pointer (8 bytes)
  - A length (8 bytes)  
  - A capacity (8 bytes)
  - The actual text data on the heap
  
  **Why you can't fully fix this for Unicode**: The `.to_lowercase()` is necessary for case-insensitive matching, and Unicode lowercase conversion (like `Ü` → `ü`) can change byte length. So you DO need `String` here.
  
  **Partial fix**: If your password list is already lowercase, skip the conversion:
  ```rust
  content
      .lines()
      .map(|line| line.trim().to_string())  // Still allocates, but faster
      .collect()
  ```
  
  **Better fix**: Pre-process your password list file to be lowercase, then use `&'static str`:
  ```rust
  pub static PASSWORD_SET: Lazy<HashSet<&'static str>> = Lazy::new(|| {
      include_str!("../data/100k-most-used-passwords-lowercase.txt")
          .lines()
          .map(|line| line.trim())
          .collect()
  });
  ```
  
  **Expected speedup**: 20-40% faster startup, ~10% less memory

---

- [x] **5. Avoid double allocation in `score_penalties` normalization**
  
  **Location**: `score.rs`, line 161
  
  **What's happening now**: 
  ```rust
  let normalized = password.trim().to_lowercase();
  ```
  
  **The problem**: `to_lowercase()` creates a new `String` every time `score_penalties` is called. If you score 1000 passwords, you create 1000 temporary strings.
  
  **Why this is tricky for Unicode**: Unicode lowercasing can change string length! The German `ẞ` (capital eszett) becomes `ss` (2 characters). So you can't lowercase "in place."
  
  **Partial fixes**:
  
  1. **Check if trimming is needed first**:
     ```rust
     let trimmed = password.trim();
     // Only allocate if the password has uppercase
     let normalized = if trimmed.chars().any(|c| c.is_uppercase()) {
         trimmed.to_lowercase()
     } else {
         trimmed.to_string()
     };
     ```
  
  2. **Use `Cow` (Copy-on-Write)** to avoid allocation when possible:
     ```rust
     use std::borrow::Cow;
     
     let trimmed = password.trim();
     let normalized: Cow<str> = if trimmed.chars().any(|c| c.is_uppercase()) {
         Cow::Owned(trimmed.to_lowercase())
     } else {
         Cow::Borrowed(trimmed)
     };
     ```
  
  **Expected speedup**: 10-30% for mostly-lowercase passwords

---

- [x] **6. Keep using `is_lowercase()` and `is_uppercase()` for full Unicode**
  
  **Location**: `score.rs`, lines 80-85
  
  **Current code**:
  ```rust
  if c.is_lowercase() {
      lower_count = true;
  } else if c.is_uppercase() {
      upper_count = true;
  }
  ```
  
  **Why you should KEEP this**: You want full Unicode support! These methods correctly handle:
  - `ñ` (Spanish) → lowercase ✓
  - `Ü` (German) → uppercase ✓
  - `α` (Greek alpha) → lowercase ✓
  - `Ω` (Greek omega) → uppercase ✓
  - `中` (Chinese) → neither (no case concept) ✓
  
  **What NOT to do**: Don't switch to `is_ascii_lowercase()` which only handles a-z.
  
  **Small optimization**: The `else if` chain means a lowercase letter checks 1 condition, uppercase checks 2, digit checks 3, symbol checks 4. Consider putting the most common type first. For passwords, lowercase is most common, so your current order is good!
  
  **Status**: ✅ Keep as-is for Unicode support.

---

- [x] **7. Replace `HashSet<char>` with `HashMap<char, ()>` or keep for Unicode**
  
  **Location**: `score.rs`, lines 127-131
  
  **Current code**:
  ```rust
  let mut set: HashSet<char> = HashSet::new();
  for c in password.chars() {
      set.insert(c);
  }
  ```
  
  **Why you should KEEP this for Unicode**: A `HashSet<char>` can store ANY Unicode character (over 140,000 possible values). If you switched to a `[bool; 128]` array (ASCII only), you'd miss:
  - Accented letters: é, ñ, ü
  - Emoji: 🔐, 🔑, 💪
  - Non-Latin scripts: 中文, العربية, עברית
  
  **Small optimization you CAN do**: Pre-allocate the HashSet:
  ```rust
  let mut set: HashSet<char> = HashSet::with_capacity(password.chars().count());
  ```
  
  **Why this helps**: Without `with_capacity`, the HashSet starts small and grows by reallocating as you add characters. If you know roughly how many characters, pre-allocating avoids these reallocations.
  
  **Expected speedup**: 5-10% for longer passwords

---

- [x] **8. Keep using `chars()` for Unicode iteration**
  
  **Location**: Multiple places in `score.rs`
  
  **What you might be tempted to do**: Use `password.as_bytes()` or `password.bytes()` because iterating bytes is faster than iterating chars.
  
  **Why you should NOT do this for Unicode**: 
  
  ```rust
  let password = "Ñoño";  // Spanish word
  
  // WRONG - bytes:
  for b in password.bytes() {
      // Sees: [195, 145, 111, 195, 177, 111] - 6 bytes
      // The Ñ is bytes [195, 145], not a single value!
  }
  
  // CORRECT - chars:
  for c in password.chars() {
      // Sees: ['Ñ', 'o', 'ñ', 'o'] - 4 characters
  }
  ```
  
  **Status**: ✅ Already implemented! No changes needed.

---

- [ ] **9. Skip `trim()` if passwords are guaranteed pre-trimmed**
  
  **Location**: `score.rs`, line 161
  
  **Current code**:
  ```rust
  let normalized = password.trim().to_lowercase();
  ```
  
  **What `trim()` does**: Removes whitespace (spaces, tabs, newlines) from start and end.
  
  **The question**: Will your library ever receive passwords with leading/trailing spaces?
  
  - If **yes** (user might paste " password "), keep `trim()`
  - If **no** (caller already trims), you can remove it
  
  **Small optimization**: Check if trim is needed:
  ```rust
  let needs_trim = password.starts_with(char::is_whitespace) 
                || password.ends_with(char::is_whitespace);
  let trimmed = if needs_trim { password.trim() } else { password };
  ```
  
  **Expected speedup**: Minimal (1-2%), but cleaner if you document the requirement.

---

- [ ] **10. Use `HashSet::contains` with the right type**
  
  **Location**: `score.rs`, line 163
  
  **Current code**:
  ```rust
  if PASSWORD_SET.contains(&normalized) {
  ```
  
  **What's happening**: `normalized` is a `String`, and `PASSWORD_SET` is a `HashSet<String>`. The types match, so this works.
  
  **The issue**: If you changed `PASSWORD_SET` to `HashSet<&'static str>` (tip #4), this would break because you can't compare `String` directly with `&str` in a HashSet.
  
  **The fix (if you implement tip #4)**:
  ```rust
  // Option 1: Borrow as &str
  if PASSWORD_SET.contains(normalized.as_str()) {
  
  // Option 2: Use Borrow trait (HashSet already supports this)
  if PASSWORD_SET.contains(&normalized as &str) {
  ```
  
  **Status**: Current code is fine. Only needs change if you implement tip #4.

---

### Medium (Moderate Refactoring)

---

- [ ] **11. Pre-filter candidates by length before Levenshtein**
  
  **Location**: `score.rs`, lines 165-170
  
  **Current code**:
  ```rust
  let candidates = PASSWORD_SET.iter().filter(|common| {
      let clen = common.len();  // ⚠️ This is BYTE length, not char length!
      (clen as isize - len as isize).abs() <= 3
  });
  ```
  
  **Problem 1 - Unicode bug!**: You're comparing `common.len()` (bytes) with `len` which is... let's check line 162:
  ```rust
  let len = normalized.len();  // Also bytes!
  ```
  
  For ASCII passwords this works, but for Unicode it's inconsistent. The password "café" has 4 chars but 5 bytes.
  
  **Fix for Unicode correctness**:
  ```rust
  let len = normalized.chars().count();  // Character count
  
  let candidates = PASSWORD_SET.iter().filter(|common| {
      let clen = common.chars().count();  // Character count
      clen.abs_diff(len) <= 3
  });
  ```
  
  **Problem 2 - Performance**: Calling `.chars().count()` on 100,000 passwords is slow!
  
  **Better solution**: Pre-compute lengths when building `PASSWORD_SET`:
  ```rust
  pub static PASSWORD_DATA: Lazy<Vec<(String, usize)>> = Lazy::new(|| {
      include_str!("../data/100k-most-used-passwords-NCSC.txt")
          .lines()
          .map(|line| {
              let pw = line.trim().to_lowercase();
              let len = pw.chars().count();
              (pw, len)
          })
          .collect()
  });
  ```
  
  **Expected speedup**: 30-50% in `score_penalties`

---

- [ ] **12. Use a match statement instead of if-else chain for character types**
  
  **Location**: `score.rs`, lines 80-89
  
  **Current code**:
  ```rust
  if c.is_lowercase() {
      lower_count = true;
  } else if c.is_uppercase() {
      upper_count = true;
  } else if c.is_ascii_digit() {
      digit_count = true;
  } else {
      symbol_count = true;
  }
  ```
  
  **Note about your digit check**: You use `is_ascii_digit()` which only matches 0-9. There are other Unicode digits like:
  - `٤` (Arabic-Indic 4)
  - `④` (Circled 4)
  - `۴` (Extended Arabic 4)
  
  **If you want full Unicode digits**, use `is_numeric()` instead:
  ```rust
  } else if c.is_numeric() {
      digit_count = true;
  ```
  
  **Alternative structure using match** (same performance, arguably cleaner):
  ```rust
  match c {
      _ if c.is_lowercase() => lower_count = true,
      _ if c.is_uppercase() => upper_count = true,
      _ if c.is_numeric() => digit_count = true,
      _ => symbol_count = true,
  }
  ```
  
  **Expected speedup**: None (this is about correctness/clarity, not speed)

---

- [ ] **13. Add `#[inline]` hint to `levenshtein_with_cutoff`**
  
  **Location**: `score.rs`, line 195
  
  **Current code**:
  ```rust
  fn levenshtein_with_cutoff(a: &str, b: &str, threshold: usize) -> usize {
  ```
  
  **What inlining does**: Normally when you call a function, the CPU has to:
  1. Save current state
  2. Jump to the function's code
  3. Execute the function
  4. Jump back
  5. Restore state
  
  Inlining copies the function's code directly into the caller, avoiding the jump overhead.
  
  **The change**:
  ```rust
  #[inline]
  fn levenshtein_with_cutoff(a: &str, b: &str, threshold: usize) -> usize {
  ```
  
  **Or more aggressive**:
  ```rust
  #[inline(always)]
  fn levenshtein_with_cutoff(a: &str, b: &str, threshold: usize) -> usize {
  ```
  
  **Why this might NOT help**: The compiler often inlines small functions automatically. This function is medium-sized, so the hint might help.
  
  **Expected speedup**: 0-5% (compiler may already inline it)

---

- [ ] **14. Pre-allocate vectors in `levenshtein_with_cutoff`**
  
  **Location**: `score.rs`, lines 196-197
  
  **Current code**:
  ```rust
  let mut v0: Vec<usize> = (0..=b.len()).collect();
  let mut v1 = vec![0; b.len() + 1];
  ```
  
  **What's happening**: Every time you call this function, you allocate two vectors on the heap. For 100,000 password comparisons, that's 200,000 allocations!
  
  **The problem for Unicode**: `b.len()` returns BYTES, but you're iterating with `.chars()`. For the password "München" (7 chars, 8 bytes), you'd allocate 9-element vectors but only use 8 positions.
  
  **Fix for correctness**:
  ```rust
  let b_chars: Vec<char> = b.chars().collect();
  let mut v0: Vec<usize> = (0..=b_chars.len()).collect();
  let mut v1 = vec![0; b_chars.len() + 1];
  ```
  
  **Fix for performance** (use thread-local storage):
  ```rust
  use std::cell::RefCell;
  
  thread_local! {
      static LEVENSHTEIN_BUFFER: RefCell<(Vec<usize>, Vec<usize>)> = 
          RefCell::new((Vec::with_capacity(64), Vec::with_capacity(64)));
  }
  
  fn levenshtein_with_cutoff(a: &str, b: &str, threshold: usize) -> usize {
      LEVENSHTEIN_BUFFER.with(|buf| {
          let mut buf = buf.borrow_mut();
          buf.0.clear();
          buf.1.clear();
          // ... use buf.0 and buf.1 instead of v0 and v1
      })
  }
  ```
  
  **Expected speedup**: 20-40% in `score_penalties`

---

- [ ] **15. Reuse allocations with thread-local storage**
  
  **Location**: `score.rs`, line 161 and lines 196-197
  
  **The pattern**: Several places allocate temporary data:
  - `normalized` string in `score_penalties`
  - `v0` and `v1` vectors in `levenshtein_with_cutoff`
  - The `HashSet` in `score_uniqueness`
  
  **What thread-local storage does**: Instead of allocating fresh memory each call, you keep a buffer that persists across calls. Each thread gets its own buffer (thread-safe without locks).
  
  **Example for `score_uniqueness`**:
  ```rust
  use std::cell::RefCell;
  use std::collections::HashSet;
  
  thread_local! {
      static UNIQUENESS_SET: RefCell<HashSet<char>> = RefCell::new(HashSet::with_capacity(64));
  }
  
  pub fn score_uniqueness(password: &str) -> u16 {
      if password.is_empty() {
          return 0;
      }
      
      UNIQUENESS_SET.with(|set| {
          let mut set = set.borrow_mut();
          set.clear();  // Reuse the allocation, just clear contents
          
          for c in password.chars() {
              set.insert(c);
          }
          
          let unique_ratio = set.len() as f32 / password.chars().count() as f32;
          (unique_ratio * 200.0).round() as u16
      })
  }
  ```
  
  **Expected speedup**: 15-30% for repeated calls

---

- [ ] **16. Group passwords by length for faster filtering**
  
  **Location**: `score.rs`, lines 4-11
  
  **Current approach**: One big `HashSet` of 100,000 passwords.
  
  **Better approach**: Group by character length:
  ```rust
  use std::collections::HashMap;
  
  pub static PASSWORDS_BY_LENGTH: Lazy<HashMap<usize, Vec<String>>> = Lazy::new(|| {
      let mut map: HashMap<usize, Vec<String>> = HashMap::new();
      
      for line in include_str!("../data/100k-most-used-passwords-NCSC.txt").lines() {
          let pw = line.trim().to_lowercase();
          let len = pw.chars().count();
          map.entry(len).or_default().push(pw);
      }
      
      map
  });
  ```
  
  **Then in `score_penalties`**:
  ```rust
  let len = normalized.chars().count();
  
  // Only check passwords within ±3 of our length
  for check_len in len.saturating_sub(3)..=len + 3 {
      if let Some(passwords) = PASSWORDS_BY_LENGTH.get(&check_len) {
          for common in passwords {
              // ... Levenshtein check
          }
      }
  }
  ```
  
  **Why this is faster**: Most passwords are 6-12 characters. If the user's password is 20 characters, you skip ~90% of the list immediately.
  
  **Expected speedup**: 40-70% in `score_penalties`

---

- [ ] **17. Use `chars().count()` consistently (you already do!)**
  
  **Location**: Check all places where you measure string length
  
  **Current usage**:
  - `score_length`: Uses `password.chars().count()` ✅
  - `score_uniqueness`: Uses `password.len()` on line 135 ⚠️
  - `score_penalties`: Uses `normalized.len()` on line 162 ⚠️
  
  **The bug in `score_uniqueness`**:
  ```rust
  let unique_ratio = set.len() as f32 / password.len() as f32;
  //                                     ^^^^^^^^^^^^^^
  //                                     This is BYTES, not characters!
  ```
  
  For "café" with 4 unique chars and 5 bytes, you'd calculate 4/5 = 0.8 instead of 4/4 = 1.0.
  
  **Fix**:
  ```rust
  let unique_ratio = set.len() as f32 / password.chars().count() as f32;
  ```
  
  **Status**: This is a bug fix, not just optimization!

---

- [ ] **18. Simplify `std::cmp::min` chains**
  
  **Location**: `score.rs`, lines 205-208
  
  **Current code**:
  ```rust
  v1[j + 1] = std::cmp::min(
      std::cmp::min(v1[j] + 1, v0[j + 1] + 1),
      v0[j] + cost,
  );
  min = std::cmp::min(min, v1[j + 1]);
  ```
  
  **Cleaner version**:
  ```rust
  v1[j + 1] = (v1[j] + 1)
      .min(v0[j + 1] + 1)
      .min(v0[j] + cost);
  min = min.min(v1[j + 1]);
  ```
  
  **Why this is better**:
  1. Easier to read
  2. Exactly the same performance (compiles to identical code)
  3. No need to import `std::cmp::min`
  
  **Expected speedup**: None (same compiled code), but more readable.

---

- [ ] **19. Cache password scores for repeated lookups**
  
  **Location**: `lib.rs`, `score()` function
  
  **The scenario**: If the same password is scored multiple times (e.g., user types, sees score, types more, sees score), you recalculate everything.
  
  **Simple caching with `HashMap`**:
  ```rust
  use std::collections::HashMap;
  use std::sync::Mutex;
  use once_cell::sync::Lazy;
  
  static SCORE_CACHE: Lazy<Mutex<HashMap<String, u16>>> = 
      Lazy::new(|| Mutex::new(HashMap::new()));
  
  pub fn score(password: &str) -> u16 {
      // Check cache first
      if let Some(&cached) = SCORE_CACHE.lock().unwrap().get(password) {
          return cached;
      }
      
      // Calculate score
      let mut score = 0;
      score += score_length(password);
      score += score_variety(password);
      score += score_uniqueness(password);
      let penalty = score_penalties(password);
      if penalty == 0 {
          score = 0;
      } else {
          score += penalty;
      }
      
      // Store in cache
      SCORE_CACHE.lock().unwrap().insert(password.to_string(), score);
      score
  }
  ```
  
  **Caution**: This uses memory for each unique password scored. Consider using an LRU cache that evicts old entries.
  
  **Expected speedup**: Huge (100%) for repeated passwords, 0% for unique passwords.

---

- [ ] **20. Convert collected `Vec` to `Box<[T]>` for static data**
  
  **Location**: `score.rs`, lines 4-11
  
  **What's the difference?**
  - `Vec<T>`: Stores pointer + length + capacity (24 bytes overhead)
  - `Box<[T]>`: Stores pointer + length only (16 bytes overhead)
  
  **Current structure**: Your `HashSet<String>` internally uses a `Vec`.
  
  **When this matters**: If you switch to a `Vec<String>` or `Vec<(String, usize)>` for the password list.
  
  **How to do it**:
  ```rust
  pub static PASSWORD_LIST: Lazy<Box<[String]>> = Lazy::new(|| {
      include_str!("../data/100k-most-used-passwords-NCSC.txt")
          .lines()
          .map(|line| line.trim().to_lowercase())
          .collect::<Vec<_>>()
          .into_boxed_slice()  // Convert Vec to Box<[T]>
  });
  ```
  
  **Expected improvement**: 8 bytes saved. Negligible unless you have many such structures.

---

### Hard (Significant Architectural Changes)

---

- [ ] **21. Build a trie (prefix tree) for password lookup**
  
  **Location**: Replace `PASSWORD_SET` entirely
  
  **What's a trie?** A tree where each node is a character. The path from root to a node spells out a prefix.
  
  ```
  Example trie for ["cat", "car", "card", "dog"]:
  
       (root)
       /    \
      c      d
      |      |
      a      o
     / \     |
    t   r    g
        |
        d
  ```
  
  **Why it's good for fuzzy matching**: 
  - You can bail out early if no path matches
  - You can track "edit budget" as you traverse
  - Common prefixes are shared (memory efficient)
  
  **Basic trie structure for Unicode**:
  ```rust
  use std::collections::HashMap;
  
  #[derive(Default)]
  struct TrieNode {
      children: HashMap<char, TrieNode>,  // char for Unicode support!
      is_end: bool,
  }
  
  impl TrieNode {
      fn insert(&mut self, word: &str) {
          let mut node = self;
          for c in word.chars() {
              node = node.children.entry(c).or_default();
          }
          node.is_end = true;
      }
      
      fn contains(&self, word: &str) -> bool {
          let mut node = self;
          for c in word.chars() {
              match node.children.get(&c) {
                  Some(child) => node = child,
                  None => return false,
              }
          }
          node.is_end
      }
  }
  ```
  
  **Estimated effort**: ~100-200 lines of code
  
  **Expected speedup**: 50-80% for exact match, more complex for fuzzy match

---

- [ ] **22. Implement perfect hashing (STD only)**
  
  **Location**: Replace `PASSWORD_SET` hashing
  
  **What's perfect hashing?** A hash function specifically designed for your data that guarantees NO collisions. Every password maps to a unique bucket.
  
  **Why it's hard**: You need to analyze all 100k passwords at compile time to find a perfect hash function.
  
  **STD-only approach** (minimal perfect hashing):
  1. Sort passwords
  2. Use binary search instead of hashing
  
  ```rust
  pub static PASSWORD_LIST: Lazy<Box<[String]>> = Lazy::new(|| {
      let mut list: Vec<String> = include_str!("../data/100k-most-used-passwords-NCSC.txt")
          .lines()
          .map(|line| line.trim().to_lowercase())
          .collect();
      list.sort();  // Sort for binary search
      list.into_boxed_slice()
  });
  
  fn contains_password(pw: &str) -> bool {
      PASSWORD_LIST.binary_search_by(|p| p.as_str().cmp(pw)).is_ok()
  }
  ```
  
  **Binary search performance**: O(log n) = ~17 comparisons for 100k passwords
  **HashSet performance**: O(1) average, but with hashing overhead
  
  **Expected result**: Might be faster or slower depending on CPU cache behavior. Benchmark to know!

---

- [ ] **23. Pre-compute password fingerprints for ultra-fast filtering**
  
  **Location**: New data structure alongside `PASSWORD_SET`
  
  **What's a fingerprint?** Quick-to-compare summary of a password:
  - Length (number of characters)
  - First character
  - Last character
  - Character set bitmap (which types of chars present)
  
  **Structure**:
  ```rust
  struct PasswordFingerprint {
      password: String,
      length: u8,
      first_char: char,
      last_char: char,
      has_lower: bool,
      has_upper: bool,
      has_digit: bool,
      has_symbol: bool,
  }
  ```
  
  **Fast pre-filtering**:
  ```rust
  fn might_be_similar(user: &Fingerprint, common: &Fingerprint) -> bool {
      // Quick checks that rule out most candidates
      if user.length.abs_diff(common.length) > 3 { return false; }
      if user.first_char != common.first_char && user.last_char != common.last_char { 
          return false; 
      }
      true  // Might be similar, do full Levenshtein
  }
  ```
  
  **Why this is fast**: These checks are simple integer/char comparisons. You can reject 90%+ of passwords without expensive string operations.
  
  **Expected speedup**: 70-90% in `score_penalties`

---

- [ ] **24. Use SIMD for parallel character classification**
  
  **Location**: `score_variety` function
  
  **What's SIMD?** "Single Instruction, Multiple Data" - process 16, 32, or 64 bytes at once using special CPU instructions.
  
  **The challenge for Unicode**: SIMD works on fixed-size byte chunks, but Unicode characters are variable-length (1-4 bytes). This makes SIMD tricky for Unicode.
  
  **Where SIMD CAN help**: 
  - Scanning for ASCII-only passwords (fast path)
  - Counting bytes quickly
  - Finding if ANY non-ASCII exists
  
  **Basic approach**:
  ```rust
  use std::arch::x86_64::*; // or aarch64 for ARM Macs
  
  fn is_all_ascii(s: &str) -> bool {
      // Check if high bit is set in any byte
      s.bytes().all(|b| b & 0x80 == 0)
  }
  
  fn score_variety_fast(password: &str) -> u16 {
      if is_all_ascii(password) {
          // Fast path: ASCII-only, can use byte operations
          score_variety_ascii(password)
      } else {
          // Slow path: Has Unicode, use char iteration
          score_variety_unicode(password)
      }
  }
  ```
  
  **Estimated effort**: 200+ lines, requires unsafe code
  
  **Expected speedup**: 2-5x for ASCII passwords, no change for Unicode

---

- [ ] **25. Implement BK-tree for efficient fuzzy matching**
  
  **Location**: Replace the candidate filtering in `score_penalties`
  
  **What's a BK-tree?** A tree structure where nodes are organized by edit distance. If you want all words within distance 2 of "password", you only need to visit a small portion of the tree.
  
  **How it works**:
  ```
  Root: "book"
     |
     +-- distance 1: "look", "books"
     |      |
     |      +-- distance 1 from "look": "lock", "took"
     |
     +-- distance 2: "cook", "hook"
     |
     +-- distance 4: "password"
  ```
  
  **To find words within distance 2 of "look"**:
  1. Start at "book", distance = 1
  2. Check children at distances 1±2 = [-1, 3] → check distances 0,1,2,3
  3. Recursively check those subtrees
  
  **Structure**:
  ```rust
  struct BKNode {
      word: String,
      children: HashMap<usize, BKNode>,  // key = edit distance
  }
  
  impl BKNode {
      fn find_within_distance(&self, target: &str, max_dist: usize, results: &mut Vec<&str>) {
          let dist = levenshtein(&self.word, target);
          if dist <= max_dist {
              results.push(&self.word);
          }
          
          // Only check children at distances [dist-max_dist, dist+max_dist]
          let min_child = dist.saturating_sub(max_dist);
          let max_child = dist + max_dist;
          
          for d in min_child..=max_child {
              if let Some(child) = self.children.get(&d) {
                  child.find_within_distance(target, max_dist, results);
              }
          }
      }
  }
  ```
  
  **Estimated effort**: 100-150 lines
  
  **Expected speedup**: 80-95% in `score_penalties` - instead of checking 100k passwords, typically check ~100-500

---

## Library-Assisted Tips (5 tips)

---

- [ ] **26. Use `phf` crate for compile-time perfect hash maps**
  
  **What it does**: Generates a perfect hash function at compile time. Zero runtime overhead for lookups.
  
  **Add to Cargo.toml**:
  ```toml
  [dependencies]
  phf = { version = "0.11", features = ["macros"] }
  
  [build-dependencies]
  phf_codegen = "0.11"
  ```
  
  **The challenge**: Your password list has 100k entries. `phf` macros work better for smaller sets (hundreds). For large sets, use `phf_codegen` in a build script.
  
  **Create `build.rs`**:
  ```rust
  use std::env;
  use std::fs::File;
  use std::io::{BufWriter, Write};
  use std::path::Path;
  
  fn main() {
      let path = Path::new(&env::var("OUT_DIR").unwrap()).join("passwords.rs");
      let mut file = BufWriter::new(File::create(&path).unwrap());
      
      let passwords: Vec<&str> = include_str!("data/100k-most-used-passwords-NCSC.txt")
          .lines()
          .map(|l| l.trim())
          .collect();
      
      write!(&mut file, "static PASSWORDS: phf::Set<&'static str> = ").unwrap();
      let mut set = phf_codegen::Set::new();
      for pw in &passwords {
          set.entry(pw);
      }
      set.build(&mut file).unwrap();
      write!(&mut file, ";\n").unwrap();
  }
  ```
  
  **Expected speedup**: 40-60% for exact match lookups

---

- [ ] **27. Use `ahash` for faster runtime hashing**
  
  **What it does**: Replaces Rust's default SipHash with AHash, which is 2-3x faster.
  
  **Add to Cargo.toml**:
  ```toml
  [dependencies]
  ahash = "0.8"
  ```
  
  **Change your code**:
  ```rust
  use ahash::AHashSet;
  
  pub static PASSWORD_SET: Lazy<AHashSet<String>> = Lazy::new(|| {
      include_str!("../data/100k-most-used-passwords-NCSC.txt")
          .lines()
          .map(|line| line.trim().to_lowercase())
          .collect()
  });
  ```
  
  **Why AHash is faster**: SipHash is designed to prevent hash-flooding attacks. AHash prioritizes speed for trusted data (like your password list).
  
  **Expected speedup**: 20-40% for HashSet operations

---

- [ ] **28. Use `memchr` for fast byte scanning**
  
  **What it does**: Provides SIMD-accelerated byte searching.
  
  **Add to Cargo.toml**:
  ```toml
  [dependencies]
  memchr = "2"
  ```
  
  **Use case for your project**: Quickly check if a password contains any digit:
  ```rust
  use memchr::memchr;
  
  fn contains_digit(s: &str) -> bool {
      // Check for any byte 0x30-0x39 (ASCII '0'-'9')
      s.as_bytes().iter().any(|&b| b >= b'0' && b <= b'9')
      
      // OR with memchr (faster for long strings):
      // memchr::memchr2(b'0', b'1', s.as_bytes()).is_some() || ...
  }
  ```
  
  **Note**: This only finds ASCII digits. For Unicode digits, you still need `.chars()`.
  
  **Expected speedup**: Minimal for password-length strings, more for bulk text

---

- [ ] **29. Use `bstr` for byte string operations**
  
  **What it does**: Work with bytes directly while still having string-like operations.
  
  **Add to Cargo.toml**:
  ```toml
  [dependencies]
  bstr = "1"
  ```
  
  **Use case**: When you're certain data is UTF-8 but want to avoid validation overhead:
  ```rust
  use bstr::ByteSlice;
  
  fn fast_lowercase_ascii(s: &str) -> String {
      s.as_bytes()
          .to_ascii_lowercase()  // bstr method
          .to_str()
          .unwrap()
          .to_string()
  }
  ```
  
  **Caution for Unicode**: `bstr` operations are byte-based. For true Unicode support, keep using standard `str` methods.
  
  **Expected speedup**: 10-20% for ASCII-heavy operations

---

- [ ] **30. Use `rayon` for parallel batch scoring**
  
  **What it does**: Automatically parallelizes iteration across CPU cores.
  
  **Add to Cargo.toml**:
  ```toml
  [dependencies]
  rayon = "1"
  ```
  
  **Use case**: Scoring many passwords at once:
  ```rust
  use rayon::prelude::*;
  
  /// Score multiple passwords in parallel
  pub fn score_batch(passwords: &[&str]) -> Vec<u16> {
      passwords
          .par_iter()           // Parallel iterator!
          .map(|pw| score(pw))
          .collect()
  }
  ```
  
  **When this helps**: 
  - Scoring 100+ passwords at once: Big win
  - Scoring 1 password: Overhead makes it slower
  
  **Expected speedup**: 2-8x for batch operations (depending on CPU cores)

---

## Summary Table

| Tip | Location | Effort | Unicode-Safe? | Expected Impact |
|-----|----------|--------|---------------|-----------------|
| 1 | score.rs:32 | ✅ Done | ✅ Yes | Already correct |
| 2 | score.rs:77-79 | ✅ Done | ✅ Yes | Already correct |
| 3 | score.rs:173-178 | Easy | ✅ Yes | 5-15% |
| 4 | score.rs:4-11 | Easy | ⚠️ Tricky | 20-40% startup |
| 5 | score.rs:161 | Easy | ✅ Yes | 10-30% |
| 6 | score.rs:80-85 | N/A | ✅ Keep | Already correct |
| 7 | score.rs:127-131 | Easy | ✅ Yes | 5-10% |
| 8 | Multiple | N/A | ❌ Don't | Keep chars() |
| 9 | score.rs:161 | Easy | ✅ Yes | 1-2% |
| 10 | score.rs:163 | Easy | ✅ Yes | Needed for #4 |
| 11 | score.rs:165-170 | Medium | ⚠️ Fix bug | 30-50% |
| 12 | score.rs:80-89 | Easy | ✅ Yes | Clarity only |
| 13 | score.rs:195 | Easy | ✅ Yes | 0-5% |
| 14 | score.rs:196-197 | Medium | ⚠️ Fix bug | 20-40% |
| 15 | Multiple | Medium | ✅ Yes | 15-30% |
| 16 | score.rs:4-11 | Medium | ✅ Yes | 40-70% |
| 17 | score.rs:135,162 | Easy | ✅ Fix bugs! | Bug fix |
| 18 | score.rs:205-208 | Easy | ✅ Yes | Clarity only |
| 19 | lib.rs | Medium | ✅ Yes | 100% repeat |
| 20 | score.rs:4-11 | Easy | ✅ Yes | 8 bytes |
| 21 | New structure | Hard | ✅ Yes | 50-80% |
| 22 | New structure | Hard | ✅ Yes | Varies |
| 23 | New structure | Hard | ✅ Yes | 70-90% |
| 24 | score_variety | Hard | ⚠️ ASCII fast path | 2-5x ASCII |
| 25 | New structure | Hard | ✅ Yes | 80-95% |
| 26 | Cargo.toml | Medium | ✅ Yes | 40-60% |
| 27 | Cargo.toml | Easy | ✅ Yes | 20-40% |
| 28 | Cargo.toml | Easy | ⚠️ ASCII only | Minimal |
| 29 | Cargo.toml | Easy | ⚠️ Careful | 10-20% |
| 30 | Cargo.toml | Easy | ✅ Yes | 2-8x batch |

---

## Critical Bugs Found! 🐛

While analyzing your code, I found these Unicode-related bugs:

1. **`score_uniqueness` line 135**: Uses `password.len()` (bytes) instead of `password.chars().count()` (characters)

2. **`score_penalties` line 162**: Uses `normalized.len()` (bytes) instead of `.chars().count()` (characters)

3. **`levenshtein_with_cutoff` lines 196-197**: Uses `b.len()` (bytes) but iterates with `b.chars()`, causing mismatched vector sizes

These will give wrong results for passwords with non-ASCII characters like "café", "naïve", or "日本語"!
