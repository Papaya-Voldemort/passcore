use once_cell::sync::Lazy;
use std::collections::HashSet;

pub static PASSWORD_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    // Load file and collect lines into a HashSet
    let content = include_str!("../data/100k-most-used-passwords-NCSC.txt");
    content
        .lines()
        .map(|line| line.trim().to_lowercase())
        .collect()
});

/// The score_length function takes a password as input, and outputs its **Length** score.
///
/// Length is worth 400 Points in the total password score (* = or less):<br>
/// Very Low - 4 chars* - 10 points<br>
/// Low - 8 chars* - 50 points<br>
/// Medium - 12 chars* - 150 points<br>
/// High - 16 chars* - 250 points<br>
/// Very High - 24 chars* - 360 points<br>
/// Amazing - 40 chars* - up to 400 points<br>
///
/// # Example
/// ```rust
/// use passcore::score::score_length;
/// let password = "password";
/// let length_score = score_length(password); // Would score 50 for 8 chars
/// println!("Your password's length scores {}", length_score)
/// ```
pub fn score_length(password: &str) -> u16 {
    let length = password.len();
    let score;
    if length == 0 {
        score = 0;
    } else if length <= 4 {
        score = length * 2 + 2;
    } else if length <= 8 {
        score = length * 6 + 2;
    } else if length <= 12 {
        score = length * 12 + 6;
    } else if length <= 16 {
        score = length * 15 + 10;
    } else if length <= 24 {
        score = length * 15;
    } else if length <= 39 {
        score = length * 5 / 2 + 300
    } else {
        score = 400;
    }
    score.min(400) as u16
}

/// The score_variety function takes a password as input, and outputs its **Variety** score.
///
/// Character types include:<br>
/// Lowercase: abcdefghijklmnopqrstuvwxyz <br>
/// Uppercase: ABCDEFGHIJKLMNOPQRSTUVWXYZ <br>
/// Digit: 0123456789 <br>
/// Symbols: ! @ # $ % ^ & * ( ) - _ = + [ ] { } \ | ; : ' " , < . > / ? ` ~ <br>
///
/// Character Variety is worth 200 Points of the total password score:<br>
/// One Type of Character - 25 Points<br>
/// Two Types - 70 Points<br>
/// Three Types - 130 Points<br>
/// Four Types - 200 Points<br>
///
/// # Example
/// ```rust
/// use passcore::score::score_variety;
/// let password = "password";
/// let variety_score = score_variety(password); // Would score 25 for one type of character (lowercase)
/// println!("Your password's variety score is {}", variety_score)
/// ```
pub fn score_variety(password: &str) -> u16 {
    let mut lower_count = false;
    let mut upper_count = false;
    let mut digit_count = false;
    let mut symbol_count = false;

    for c in password.chars() {
        if c.is_lowercase() {
            lower_count = true;
        } else if c.is_uppercase() {
            upper_count = true;
        } else if c.is_ascii_digit() {
            digit_count = true;
        } else {
            symbol_count = true;
        }
    }

    let mut types = 0;
    if lower_count {
        types += 1;
    }
    if upper_count {
        types += 1;
    }
    if digit_count {
        types += 1;
    }
    if symbol_count {
        types += 1;
    }

    let score = match types {
        0 => 0,
        1 => 25,
        2 => 70,
        3 => 130,
        4 => 200,
        _ => 0, // safety
    };

    score as u16
}

/// The score_uniqueness function takes a password as input and outputs its **Uniqueness** score.
///
/// This function puts one copy of each character into a HashSet,
/// then takes the total number of items and divides it by the total length of the password.
///
/// # Example
/// ```rust
/// use passcore::score::score_uniqueness;
/// let password = "password";
/// let uniqueness_score = score_uniqueness(password); // Would score 150 for 6 unique letters
/// println!("Your password's uniqueness score is {}", uniqueness_score)
/// ```
pub fn score_uniqueness(password: &str) -> u16 {
    if password.is_empty() {
        return 0;
    }

    let mut set: HashSet<char> = HashSet::new();
    for c in password.chars() {
        set.insert(c);
    }

    let unique_ratio = set.len() as f32 / password.len() as f32;

    // Map ratio to 0–200 points
    (unique_ratio * 200.0).round() as u16
}

/// The score_penalties function takes a password as input and outputs its **Penalties**.
///
/// This function checks your password against 100k common passwords using levenshtein distance.<br>
/// Match an item on the list loss 200. Get close lose 50.
///
/// # Example
/// ```rust
/// use passcore::score::score_penalties;
/// let password = "password";
/// let penalties = score_penalties(password); // Would lose 200 points because it directly matches an item on the list
/// println!("Your password's penalties are {}", penalties)
/// ```
pub fn score_penalties(password: &str) -> u16 {
    let normalized = password.trim().to_lowercase();
    let len = normalized.len();

    if PASSWORD_SET.contains(&normalized) {
        return 0;
    }

    let candidates = PASSWORD_SET.iter().filter(|common| {
        let clen = common.len();
        (clen as isize - len as isize).abs() <= 3
    });

    let first = normalized.chars().next();
    let last = normalized.chars().rev().next();

    for common in candidates {
        let cfirst = common.chars().next();
        let clast = common.chars().rev().next();

        if first == cfirst || last == clast {
            if levenshtein_with_cutoff(&normalized, common, 2) <= 2 {
                return 50;
            }
        }
    }

    200
}


/// Function for modified levenshtein distance with cutoff for speed.
fn levenshtein_with_cutoff(a: &str, b: &str, threshold: usize) -> usize {
    let mut v0: Vec<usize> = (0..=b.len()).collect();
    let mut v1 = vec![0; b.len() + 1];

    for (i, ca) in a.chars().enumerate() {
        v1[0] = i + 1;
        let mut min = v1[0];

        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            v1[j + 1] = std::cmp::min(
                std::cmp::min(v1[j] + 1, v0[j + 1] + 1),
                v0[j] + cost,
            );
            min = std::cmp::min(min, v1[j + 1]);
        }

        if min > threshold {
            return threshold + 1;
        }
        std::mem::swap(&mut v0, &mut v1);
    }
    let result = v0[b.len()];
    if result > threshold { threshold + 1 } else { result }
}
