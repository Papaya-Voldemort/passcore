# Passcore Scoring Model — v2 Specification

> **Status:** Draft · **Library:** `passcore` · **Language:** Rust (`no_std`-friendly)

Fast, explainable, high-quality password scoring built as a lightweight alternative to zxcvbn-style checkers.

---

## Table of Contents

1. [Core Philosophy](#1-core-philosophy)
2. [Goals and Non-Goals](#2-goals-and-non-goals)
3. [Design Principles](#3-design-principles)
4. [Scoring Pipeline Overview](#4-scoring-pipeline-overview)
5. [Stage 1 — Normalization](#5-stage-1--normalization)
6. [Stage 2 — Length Potential](#6-stage-2--length-potential)
7. [Stage 3 — Diversity Bonus](#7-stage-3--diversity-bonus)
8. [Stage 4 — Predictability Penalties](#8-stage-4--predictability-penalties)
9. [Penalty Precedence and Dedup Rules](#9-penalty-precedence-and-dedup-rules)
10. [Score Floors and Hard Caps](#10-score-floors-and-hard-caps)
11. [Passphrase Handling](#11-passphrase-handling)
12. [Dictionary and Data Strategy](#12-dictionary-and-data-strategy)
13. [Grading Bands](#13-grading-bands)
14. [Review Generation](#14-review-generation)
15. [Calibration Test Suite](#15-calibration-test-suite)
16. [Performance Requirements](#16-performance-requirements)
17. [Module Layout](#17-module-layout)
18. [Public API Surface](#18-public-api-surface)
19. [Revision Notes](#19-revision-notes)

---

## 1. Core Philosophy

Passcore scores passwords based on **guessability**, not visual complexity.

The model follows one rule:

> **Start with the strength potential created by length, then subtract points for human predictability.**

This is the single defining idea of Passcore v2. Everything else is implementation detail.

| Common Mistake | Passcore v2 Approach |
|---|---|
| "Has uppercase → add points" | Length creates potential; uppercase barely matters |
| "Has symbol → looks strong" | Predictable patterns destroy strength regardless of symbols |
| "Unique chars → strong" | Structure matters far more than character uniqueness ratio |
| "Short + complex = decent" | Short passwords have a hard ceiling no matter what |

**The master formula:**

```text
score = clamp(length_potential + diversity_bonus - predictability_penalties, 0, 1000)
```

---

## 2. Goals and Non-Goals

### Goals

- **Fast** — mostly O(n) over password length; bounded extras for pattern checks
- **Explainable** — every major score change maps to a clear human reason
- **Passphrase-friendly** — long memorable phrases score well without tricks
- **Predictability-aware** — common and human-made patterns are penalized hard
- **Stable** — scores feel intuitive across diverse real-world examples
- **Portable** — runs in Rust backends, CLIs, WASM, and `no_std` embedded contexts

### Non-Goals

Passcore is **not** trying to be:

- A full password cracker or entropy simulator
- A large dictionary engine or ML classifier
- A breach-checking service (use HIBP for that)
- A black-box scorer with unexplainable outputs

---

## 3. Design Principles

**1. Length is the foundation.**
Length should be the strongest positive factor. A 24-character passphrase beats a short symbol-soup password.

**2. Predictability matters more than cosmetics.**
Attackers guess patterns, not character classes. Human habits cost real points.

**3. Character variety is a bonus, not the core.**
Uppercase, digits, and symbols help a little but never outweigh obvious predictability.

**4. Natural repetition is not the same as weak structure.**
`correct-horse-battery-staple` repeats letters. That is fine. `abcabcabc` repeats a chunk. That is not.

**5. Common secrets must fail fast.**
Exact common passwords score near zero. Near-common transformations are hit almost as hard.

**6. Review text reflects the real weakness.**
Feedback points to the largest active risk, not the lowest raw sub-score from an unrelated category.

**7. Penalties do not double-count the same weakness.**
When multiple checks fire on the same root cause, only the strongest applies. See Section 9.

---

## 4. Scoring Pipeline Overview

```
Input password
      │
      ▼
┌─────────────────────┐
│  1. Normalize        │  → build comparison form only; original untouched
└─────────────────────┘
      │
      ▼
┌─────────────────────┐
│  2. Length Potential │  → piecewise linear: 0–960 pts
└─────────────────────┘
      │
      ▼
┌─────────────────────┐
│  3. Diversity Bonus  │  → +0 to +100 pts
└─────────────────────┘
      │
      ▼
┌─────────────────────┐
│  4. Predictability   │  → subtract 0–750+ pts, dedup rules applied
│     Penalties        │
└─────────────────────┘
      │
      ▼
┌─────────────────────┐
│  5. Hard Caps        │  → enforce short-password ceilings
└─────────────────────┘
      │
      ▼
┌─────────────────────┐
│  6. Clamp + Grade    │  → score in [0, 1000], grade A+ to F
└─────────────────────┘
      │
      ▼
┌─────────────────────┐
│  7. Review Message   │  → single most relevant feedback string
└─────────────────────┘
```

---

## 5. Stage 1 — Normalization

The original password is **never modified** for scoring or display. Normalization builds a separate comparison form used only for commonness and pattern checks.

### Length Measurement

> **Length is always measured in Unicode scalar values**, i.e., `.chars().count()` in Rust — not `.len()` (bytes).
>
> The ASCII fast path may use `.len()` only after confirming all bytes are `< 128`. For any non-ASCII input, fall back to `.chars().count()`.

This ensures passwords with accented letters, CJK characters, or emoji are measured consistently.

### Normalization Steps

1. Lowercase all ASCII characters.
2. Apply conservative leetspeak substitutions to the comparison form only:

| Symbol | Maps To | Default? |
|---|---|---|
| `0` | `o` | Yes |
| `3` | `e` | Yes |
| `4` | `a` | Yes |
| `5` | `s` | Yes |
| `7` | `t` | Yes |
| `@` | `a` | Yes |
| `$` | `s` | Yes |
| `1` | `i` | No — `extended` feature flag only |

3. Strip separators (`-`, `_`, `.`, space) **only for Tier A-exact lookups**. See below.

### Separator Handling

This resolves a contradiction in prior drafts.

**Tier A-exact lookup:** Strip separators before comparing. `pass-word` normalizes to `password` and matches the exact list.

**Near-common transformation check (Tier B):** Do **not** strip separators before checking. Separator insertion is instead listed explicitly as one of the Tier B transforms. This keeps the categories clean and non-overlapping: a password either exactly matches a common password after normalization, or it is a detectable transformation of one.

The rule: separator stripping is a Tier A-exact concern only.

---

## 6. Stage 2 — Length Potential

Length is the single most important factor. The potential score is determined by a piecewise linear function over Unicode scalar count.

### Calibrated Curve

This curve is steeper than earlier drafts to ensure that genuinely strong short-to-medium passwords can reach meaningful scores, while still rewarding length heavily.

| Length (chars) | Potential |
|---|---|
| 0 | 0 |
| 4 | 25 |
| 6 | 70 |
| 8 | 180 |
| 10 | 320 |
| 12 | 480 |
| 14 | 620 |
| 16 | 720 |
| 18 | 790 |
| 20 | 840 |
| 24 | 890 |
| 28 | 930 |
| 32+ | 960 |

### Why This Curve?

The prior draft topped out at 900 (at 32+ chars) with 12-char max potential of ~340. That made it impossible for a strong 12-char password to reach B- range even with max diversity — an unintended bias toward passphrases over random strings. This new curve is calibrated so:

| Scenario | Max Score | Grade Ceiling |
|---|---|---|
| 12-char high-variety random | 480 + 100 = 580 | B- |
| 16-char high-variety random | 720 + 100 = 820 | A- |
| 20-char high-variety random | 840 + 100 = 940 | A+ |
| 29-char passphrase | ~935 + 100 = ~1000 | A+ |

Both passphrases and strong random strings can reach A range — just at different lengths.

### Implementation

```rust
fn length_potential(char_count: usize) -> i32 {
    const BREAKPOINTS: &[(usize, i32)] = &[
        (0,  0),
        (4,  25),
        (6,  70),
        (8,  180),
        (10, 320),
        (12, 480),
        (14, 620),
        (16, 720),
        (18, 790),
        (20, 840),
        (24, 890),
        (28, 930),
        (32, 960),
    ];

    if char_count >= 32 {
        return 960;
    }
    for window in BREAKPOINTS.windows(2) {
        let (lo_len, lo_pts) = window[0];
        let (hi_len, hi_pts) = window[1];
        if char_count >= lo_len && char_count < hi_len {
            let span = (hi_len - lo_len) as f32;
            let t = (char_count - lo_len) as f32 / span;
            return (lo_pts as f32 + t * (hi_pts - lo_pts) as f32) as i32;
        }
    }
    0
}
```

This is predictable, branch-light, and heap-allocation free.

---

## 7. Stage 3 — Diversity Bonus

Diversity helps, but only modestly. **Maximum diversity bonus: 100 points.**

### Character Class Buckets

| Class | Examples |
|---|---|
| Lowercase | `a–z` |
| Uppercase | `A–Z` |
| Digits | `0–9` |
| Symbols / Separators | `!@#$%^&*-_. ` etc. |

### Bonus Schedule

| Condition | Bonus |
|---|---|
| 1 class present | +0 |
| 2 classes present | +25 |
| 3 classes present | +45 |
| All 4 classes present | +60 |
| Separators used between 2+ word-like tokens | +15 |

**Hard cap: +100 total from all diversity sources.**

### Removed: "Mixed Token Style" Bonus

A previous draft included "+10 to +25 for mixed token style without obvious pattern." This has been removed. The phrase "without obvious pattern" is not mechanically definable — two implementations could score the same password differently, violating the explainability goal. The four-class schedule plus separator bonus is sufficient and fully specified.

### Separator Credit Rule

Grant +15 if:
- The password contains 2+ separator characters (space, `-`, `_`, `.`)
- Those separators divide the password into 2+ segments of length ≥ 2 each

---

## 8. Stage 4 — Predictability Penalties

This is where Passcore wins. The goal is to quickly find the human shortcuts attackers actually exploit.

**Read Section 9 before implementing.** Penalties are not all cumulative — dedup and suppression rules apply.

---

### 4.1 Exact Common Password

**Trigger:** Normalized form (with separators stripped) exactly matches a Tier A-exact entry.

```
penalty = force score to 0
grade   = F
review  = "This password is too common."
```

**Hard stop.** Return immediately. No other checks run.

---

### 4.2 Near-Common Transformation

**Trigger:** Password is a shallow transformation of a Tier A-exact or Tier A-base entry via any Tier B transform.

| Transform | Example |
|---|---|
| Case flip | `Password` |
| Digit suffix (1–4 digits) | `password1`, `password123` |
| Symbol suffix (1–2 symbols) | `password!` |
| Year suffix (1900–2099) | `password2025`, `letmein1999` |
| Leetspeak | `p@ssw0rd` |
| Doubled word | `passpass` |
| Separator insertion | `pass-word`, `pass_word` |
| Combined transforms | `P@ssw0rd1!` |

```
penalty = -500 to -750
```

Scale: closer to -750 for single simple transforms (`password1`). Closer to -500 for longer or stacked transforms where base word confidence is lower.

**When this fires: suppress Checks 4.7 and 4.8.** See Section 9.

---

### 4.3 Repeated Single-Character Runs

**Trigger:** A single character appears consecutively 4 or more times.

Examples: `aaaaaa`, `111111`, `!!!!!!`

```
penalty = min(20 * (run_length - 3), 180)
```

| Run Length | Penalty |
|---|---|
| 4 | -20 |
| 6 | -60 |
| 8 | -100 |
| 12+ | -180 (cap) |

---

### 4.4 Repeated Chunk Patterns

**Trigger:** A substring of length 2–4 repeats 2+ times and accounts for 40% or more of the total password length.

Examples: `abcabcabc`, `passpass`, `catcatcat42`, `12121212`

```
penalty = chunk_length * occurrence_count * 15
cap     = -260
```

Check chunk sizes 2, 3, and 4. Apply only the **largest** matching penalty — do not stack multiple chunk sizes against the same password.

**When this fires on a doubled chunk: suppress Check 4.9 for the same structure.** See Section 9.

---

### 4.5 Sequential Runs

**Trigger:** 4+ consecutive characters in alphabetical or numeric sequence, ascending or descending.

Examples: `abcd`, `dcba`, `1234`, `9876`

Count only **maximal non-overlapping runs**. Do not count `abcde` as both `abcd` and `bcde`.

```
penalty per maximal run = 40
cap across all runs     = -180
```

---

### 4.6 Keyboard Walks

**Trigger:** 4+ character path along common keyboard rows, columns, or diagonals.

Common patterns (implement with full static adjacency table):

```
Row:    qwerty  asdf  zxcv  uiop  hjkl
Column: 1qaz  2wsx  qazwsx
Reverse of any of the above
```

Count only **maximal non-overlapping walks**.

```
penalty per walk = 50 + (walk_length - 4) * 15
cap              = -220
```

| Walk Length | Penalty |
|---|---|
| 4 | -50 |
| 6 | -80 |
| 8 | -110 |

---

### 4.7 Date and Year Patterns

**Trigger:** Password contains a recognizable date or year pattern.

| Pattern | Example |
|---|---|
| 4-digit year (1900–2099) | `2026`, `1987` |
| 2-digit year at end | `dragon99` |
| Month/day combos | `0101`, `1225`, `0704` |
| Full date formats | `12/25/99`, `2025-01-01` |

```
base penalty     = -40 to -80
+40 additional   if attached directly to a Tier A-base word
```

**Suppressed when Check 4.2 fires.** See Section 9.

---

### 4.8 Common Base Word + Suffix

**Trigger:** Password matches `[Tier A-base word][padding]` where padding is digits, symbols, or a year. Requires a separate Tier A-base list (see Section 12).

Examples: `dragon99`, `monkey123`, `summer2026!`, `football7`

```
penalty = -120 to -300
```

Scale: higher penalty for shorter, more-common base words with minimal padding. Lower for less-common words with substantial padding.

**Suppressed when Check 4.2 fires.** See Section 9.

---

### 4.9 Mirrored and Doubled Structures

**Trigger:** Password is a palindrome, has a mirrored half, or is a chunk doubled.

Examples: `abcddcba`, `hellohello`, `pass123pass123`

```
penalty = -100 to -220
```

Scale with how much of the password the pattern covers. Full mirror or double gets -220; partial gets -100.

**Suppressed when Check 4.4 fires on the same repeated structure.** See Section 9.

---

## 9. Penalty Precedence and Dedup Rules

Without these rules, a password gets punished multiple times for the same root weakness. Apply in order during scoring.

### Rule 1 — Exact Common Password Is a Hard Stop

If Check 4.1 fires: return immediately with score 0, grade F. No other checks run at all.

### Rule 2 — Near-Common Suppresses Date and Base-Word Checks

```
Check 4.2 fires?
  → suppress Check 4.7 (date/year pattern)
  → suppress Check 4.8 (common base word + suffix)
```

Rationale: the transformation detection already captured the year suffix and word+suffix as part of the same failure. Adding them separately would double-count one weakness.

### Rule 3 — Repeated Chunk Suppresses Mirrored/Doubled for Same Structure

```
Check 4.4 fires on a doubled chunk (e.g., passpass, abcabc)?
  → suppress Check 4.9 for that structure
  → allow Check 4.9 if the mirror is structurally independent (e.g., abcddcba has
     a palindrome structure, no repeated chunk)
```

### Rule 4 — Sequence and Keyboard Walks Use Maximal Non-Overlapping Spans

```
abcdef  → one run of length 6, not three runs of length 4
qwerty  → one walk of length 6, not two overlapping walks of length 4–5
```

Apply the penalty once per maximal span and advance past it.

### Rule 5 — Date Penalty Does Not Stack With Base-Word Penalty

If both Checks 4.7 and 4.8 would fire for the same password (e.g., `dragon2025`):

```
Apply Check 4.8 + the date attachment modifier (+40)
Suppress standalone Check 4.7
```

---

## 10. Score Floors and Hard Caps

Short passwords cannot game the system regardless of character class choices.

| Condition | Ceiling |
|---|---|
| length < 4 chars | score ≤ 20 (always F) |
| length < 6 chars | score ≤ 80 (always F) |
| length < 8 chars | score ≤ 220 (D or below) |
| Exact common password | forced to 0 |

Applied **after** all other calculations.

---

## 11. Passphrase Handling

A good passphrase should score highly if it is long, made of multiple nontrivial chunks, and not a famous phrase.

### Passphrase Candidate Detection

A password qualifies as a passphrase candidate under **either** condition:

**Condition A — Separator-Based:**
- Length ≥ 14 chars
- Contains 2+ separator characters (space, `-`, `_`, `.`)
- Produces 2+ token segments of length ≥ 2 each

**Condition B — Length-Only:**
- Length ≥ 20 chars
- Does not trigger chunk repetition, sequence, or keyboard walk checks

The prior draft required all three criteria simultaneously, which excluded valid passphrases like `orchid-castle` (13 chars, one separator) and `BlueRiverWindow42` (17 chars, no separator). Using two independent conditions catches more real cases.

### Passphrase-Friendly Rules

| Concern | Rule |
|---|---|
| Repeated vowels or letters | Do **not** penalize natural letter recurrence |
| Separator characters | Count toward diversity bonus (+15), never penalized as a pattern |
| Famous phrases | Check against Tier A-phrases list; penalize if matched |
| Repeated word tokens | Penalize only if the exact same word token appears 2+ times |

### Target Scores for Well-Known Passphrases

| Password | Expected Score | Grade |
|---|---|---|
| `correct-horse-battery-staple` | 920–1000 | A+ |
| `moonlight-river-cabin-echo-42` | 940–1000 | A+ |
| `toast museum orbit pepper window` | 930–1000 | A+ |
| `orchid-castle-river` | 700–790 | A- |
| `BlueRiverWindow42` | 750–850 | A- |

---

## 12. Dictionary and Data Strategy

### Dictionary Split

The prior draft used a single "Tier A" list for both exact password failures and base-word detection. These are now separated.

**Tier A-exact — Exact common password list**
Used only in Check 4.1. These entries represent passwords that are themselves unacceptable.

```
password    123456    abc123     qwerty    111111
iloveyou    admin     letmein    welcome   monkey
```

**Tier A-base — Common base word list**
Used in Checks 4.7 and 4.8. These are roots that commonly appear in constructed passwords. `dragon` alone is weak but not an instant fail; `dragon99` is a textbook attack target.

```
dragon    monkey    summer    football    princess
shadow    baseball  sunshine  master      hello
```

Why separate them? Treating `dragon` the same as `password` produces confusing scores. `dragon` alone as a password is bad but qualitatively different from `password`. The split allows these to fail differently and for the right reasons.

**Tier A-phrases — Famous phrase list (small)**
A small list of known passphrases and famous quotes, to prevent them from scoring well on length alone.

```
correcthorsebatterystaple
tobeoornottobe
illbeback
maytheforcebeithyou
```

**Tier B — Transformation rules**
Applied against Tier A-exact and Tier A-base at runtime. See Check 4.2.

**Tier C — Extended mode (feature flag: `extended`)**
Deeper dictionary matching enabled as a Cargo feature. Default crate stays fast.

```toml
passcore = { version = "0.2", features = ["extended"] }
```

---

## 13. Grading Bands

| Score | Grade | Human Meaning |
|---|---|---|
| 900–1000 | **A+** | Excellent |
| 840–899 | **A** | Strong |
| 780–839 | **A−** | Strong with minor room to improve |
| 720–779 | **B+** | Good |
| 650–719 | **B** | Solid |
| 580–649 | **B−** | Acceptable, some risk |
| 500–579 | **C+** | Mediocre |
| 420–499 | **C** | Weak |
| 340–419 | **C−** | Weak, should be changed |
| 260–339 | **D+** | Poor |
| 180–259 | **D** | Very poor |
| 100–179 | **D−** | Near-failing |
| 0–99 | **F** | Unacceptable |

---

## 14. Review Generation

The review message is driven by the **largest active weakness**, not the lowest raw sub-score.

### Priority Order

1. Exact common password
2. Near-common transformation
3. Password too short
4. Repeated chunk or heavy repetition
5. Sequence or keyboard pattern
6. Common word + year/digit suffix
7. Low diversity (only if all above are absent)
8. Strong or Excellent

### Message Templates

| Trigger | Message |
|---|---|
| Exact common | "This password is too common. Change it completely." |
| Near-common | "This looks like a common password with minor changes. Make it less predictable." |
| Too short | "This password is too short. Add much more length." |
| Repeated chunk | "This password repeats a pattern. Avoid repeated blocks or long runs." |
| Sequence/keyboard | "This password contains a predictable sequence or keyboard pattern." |
| Word + suffix | "Using a common word with numbers or symbols is easy to guess." |
| Low diversity | "This password would be stronger with a bit more variety." |
| Strong (650–799) | "This password is strong." |
| Excellent (800+) | "This is an excellent password." |

Only one message is surfaced per evaluation.

---

## 15. Calibration Test Suite

All expected ranges are derived directly from the formula. Each row below includes its derivation so failures can be diagnosed.

**Derivation method:** `length_potential(n) + max_diversity(100) - expected_penalties = approximate ceiling. Adjust downward for non-ideal diversity.`

If a result falls outside the stated range: fix the code, not the test.

---

### Failing (F)

| Password | Chars | Derivation | Expected Score |
|---|---|---|---|
| `password` | 8 | Tier A-exact → hard 0 | 0 |
| `123456` | 6 | Tier A-exact → hard 0 | 0 |
| `abc123` | 6 | Tier A-exact → hard 0 | 0 |
| `qwerty` | 6 | Tier A-exact or near-common + keyboard | 0–30 |
| `password1` | 9 | Near-common -750; 250+100-750 = -400, clamp | 0–60 |
| `P@ssw0rd123` | 11 | Near-common -750; 400+100-750 = -250, clamp | 30–100 |
| `letmein2026` | 11 | Near-common -750 | 0–80 |
| `aaaaaaaaaaaa` | 12 | 480+100=580; run penalty -180 | 10–80 |
| `abcabcabcabc` | 12 | 480+100=580; chunk penalty -260 | 20–80 |

---

### Weak (D Range)

| Password | Chars | Derivation | Expected Score |
|---|---|---|---|
| `dragon99` | 8 | 180+60=240; base word+year: -260; cap at 220 | 100–180 |
| `Summer2024!` | 11 | 400+100=500; word+year+symbol: -240 | 130–220 |
| `monkey123` | 9 | 250+60=310; base word+digits: -240 | 80–160 |
| `football7` | 9 | 250+60=310; base word+digit: -220 | 80–150 |

---

### Moderate (C Range)

| Password | Chars | Derivation | Expected Score |
|---|---|---|---|
| `Tr0ub4dor&3` | 11 | 400+100=500; leet near-common maybe -300 | 280–420 |
| `S7v!qLm2` | 8 | 180+80=260; no patterns; short cap 220 | 200–280 |
| `BlueSky#99Rain` | 14 | 620+100=720; two words + year: -200 | 420–540 |

---

### Strong (B to A-)

| Password | Chars | Derivation | Expected Score |
|---|---|---|---|
| `S7v!qLm2#R8p` | 12 | 480+100=580; no detected patterns | 520–600 |
| `xK8@mNq3!vLpT` | 14 | 620+100=720; no detected patterns | 660–740 |
| `jZ9#wQp2$rVm7nX` | 16 | 720+100=820; no detected patterns | 760–830 |
| `orchid-castle-river` | 19 | ~815+90=905; some word concern; maybe -100 | 700–800 |

---

### Excellent (A to A+)

| Password | Chars | Derivation | Expected Score |
|---|---|---|---|
| `aB3!xK9#mQ2$nR7@pL5` | 20 | 840+100=940; no patterns | 880–960 |
| `correct-horse-battery-staple` | 29 | ~935+100=1000; Tier A-phrases check | 920–1000 |
| `moonlight-river-cabin-echo-42` | 30 | ~940+100=1000 | 940–1000 |
| `toast museum orbit pepper window` | 32 | 960+100=1000 | 930–1000 |
| `BlueRiverWindow42` | 18 | 790+100=890; Condition B passphrase | 750–860 |

---

### Edge Cases

| Password | Expected Behavior |
|---|---|
| `a` | Score ≤ 5, F |
| 20× `a` | Score ≤ 60 despite length — run penalty -180 dominates |
| `correct-horse-correct-horse` | Repeated token; B range (600–700) |
| `abcdefghijklmnopqrstuvwxyz` | Seq penalty -180; still C range from length |
| `AAABBBCCCDDDEEE` | Chunk repeat; D range |
| `pass-word` | Tier A-exact after separator strip → 0, F |

---

## 16. Performance Requirements

### Target Complexity

| Operation | Complexity |
|---|---|
| Core scoring | O(n) |
| Chunk repetition (sizes 2–4) | O(n × 3) — amortized O(n) |
| Dictionary lookup (sorted slice) | O(log d) |
| Keyboard walk detection | O(n) with static adjacency table |
| **Total** | **O(n) amortized** |

### Bounded Check Windows

| Check | Constraint |
|---|---|
| Repeated chunk sizes | 2–4 characters only |
| Sequence detection window | 4–8 characters |
| Keyboard walk window | 4–8 characters |
| Year/date suffix scan | Last 1–8 characters |
| Common suffix check | Last 1–6 characters |

### Implementation Guidance

- Length: `.chars().count()`; use `.len()` fast path only after confirming all bytes `< 128`
- Tier A-exact and Tier A-base: static sorted `&[&str]` slices for binary search, or compile-time perfect hash
- No heap allocation in the hot path
- Keyboard adjacency: `static` fixed-size lookup table
- Tier C and extended checks: `#[cfg(feature = "extended")]`
- All `f32` math confined to the length interpolation; everything else is integer arithmetic

---

## 17. Module Layout

```
passcore/
├── src/
│   ├── lib.rs          # Public re-exports, crate docs, top-level score()
│   ├── score.rs        # Scoring orchestration + pipeline
│   ├── patterns.rs     # Repetition, sequences, keyboard walks, dates, chunks
│   ├── common.rs       # Normalization, separator handling, Tier A/B matching
│   ├── diversity.rs    # Character class counting, separator credit, bonus calc
│   ├── review.rs       # Priority-ordered feedback message selection
│   └── grade.rs        # score_to_grade(score: u32) -> Grade
├── benches/
│   └── score_bench.rs  # Criterion benchmarks against calibration corpus
├── tests/
│   └── calibration.rs  # Automated calibration suite (mirrors Section 15)
└── Cargo.toml
```

### Module Responsibilities

| Module | Owns |
|---|---|
| `score.rs` | Pipeline; dedup rules; clamp; returns `ScoreResult` |
| `patterns.rs` | All predictability penalty calculations |
| `common.rs` | Normalization, separator handling, Tier A/B matching, transforms |
| `diversity.rs` | Class counting, separator detection, bonus calculation |
| `review.rs` | Priority-ordered feedback message selection |
| `grade.rs` | `score_to_grade(score: u32) -> Grade` |

---

## 18. Public API Surface

```rust
/// The complete result of scoring a password.
#[derive(Debug, Clone)]
pub struct ScoreResult {
    /// Final score in [0, 1000].
    pub score: u32,
    /// Letter grade derived from score.
    pub grade: Grade,
    /// Single most relevant feedback message.
    pub review: &'static str,
    /// Scoring breakdown. Requires `debug` feature.
    #[cfg(feature = "debug")]
    pub breakdown: ScoreBreakdown,
}

/// Score a password. Primary entry point.
///
/// Length is measured in Unicode scalar values (`.chars().count()`).
///
/// # Example
/// ```rust
/// let result = passcore::score("correct-horse-battery-staple");
/// assert!(result.score > 900);
/// assert_eq!(result.grade, Grade::APlus);
/// ```
pub fn score(password: &str) -> ScoreResult;

/// Score with configurable options.
pub fn score_with_options(password: &str, opts: &ScoreOptions) -> ScoreResult;

/// Letter grades.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    F,
    DMinus, D, DPlus,
    CMinus, C, CPlus,
    BMinus, B, BPlus,
    AMinus, A, APlus,
}

/// Optional scoring configuration.
#[derive(Debug, Clone, Default)]
pub struct ScoreOptions {
    /// Enable Tier C extended dictionary matching (slower).
    pub extended_mode: bool,
    /// Enable aggressive `1 → i` leetspeak normalization.
    pub deep_leet: bool,
}

/// Per-stage breakdown. Requires `debug` feature.
#[cfg(feature = "debug")]
#[derive(Debug, Clone)]
pub struct ScoreBreakdown {
    pub length_potential: i32,
    pub diversity_bonus: i32,
    pub penalty_total: i32,
    pub penalties_fired: Vec<&'static str>,
    pub penalties_suppressed: Vec<&'static str>,
    pub pre_cap_score: i32,
    pub final_score: u32,
}
```

---

## 19. Revision Notes

Changes from the prior draft and their rationale, for future maintainers.

| Issue | Prior Spec | This Version |
|---|---|---|
| **Length curve** | Topped at 900 at 32+ chars; 12-char max potential ~340 | Steeper curve; 12-char max 580, 20-char max 940. Removes unintended passphrase bias. |
| **Calibration examples** | Multiple examples showed scores impossible under the formula | All ranges recomputed from the formula with derivations shown. |
| **Penalty stacking** | No dedup rules; same weakness penalized multiple times | Section 9 adds explicit precedence and suppression rules. |
| **Mixed token bonus** | "+10 to +25 for mixed token style without obvious pattern" | Removed — not mechanically definable; breaks explainability. |
| **Separator contradiction** | Stripped for exact match AND listed as near-common transform | Exact-match strips separators; near-common handles separator insertion separately. |
| **Length definition** | Unspecified | `.chars().count()` with ASCII fast path. Documented in Section 5. |
| **Passphrase detection** | Required ≥ 16 chars AND 2+ separators AND 3 segments | Two independent conditions (A and B); catches more valid passphrases. |
| **Dictionary split** | Single "Tier A" for both exact failures and base words | `Tier A-exact` and `Tier A-base` are separate with distinct behaviors. |