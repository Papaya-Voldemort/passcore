use crate::score::{score_length, score_penalties, score_uniqueness, score_variety};

pub mod score;


/// The score function takes a password as input and outputs a score.<br>
/// It makes use of 4 scoring functions located in the score.rs file.<br>
/// The score function will never log your password <br>
///
/// # Example
/// ``` rust
/// use passcore::score;
///
/// let password = "password";
/// let psw_score = score(password);
/// println!("Your password's score is {}", psw_score)
/// ```
pub fn score(password: &str) -> u16 {
    let mut score = 0;
    score += score_length(password);
    score += score_variety(password);
    score += score_uniqueness(password);
    score += score_penalties(password);
    score
}

#[cfg(test)]
mod length_tests {
    use super::*;

    #[test]
    fn empty_password() {
        assert_eq!(score_length(""), 0);
    }

    #[test]
    fn very_short_password() {
        assert_eq!(score_length("abcd"), 10); // 4*2 + 2
    }

    #[test]
    fn short_password() {
        assert_eq!(score_length("abcdefgh"), 50); // 8*6 + 2
    }

    #[test]
    fn medium_password() {
        assert_eq!(score_length("abcdefghijkl"), 150); // 12*12 + 6
    }

    #[test]
    fn long_password() {
        assert_eq!(score_length("abcdefghijklmnop"), 250); // 16*15 + 10
    }

    #[test]
    fn extra_long_password() {
        assert_eq!(score_length("abcdefghijklmnopqrstuvwx"), 360); // 24*15
    }

    #[test]
    fn ramp_up_password() {
        assert_eq!(score_length("abcdefghijklmnopqrstuvwxyz1234"), 375); // 30*2.5 + 300
    }

    #[test]
    fn maxed_password() {
        assert_eq!(score_length("abcdefghijklmnopqrstuvwxyz1234567890abcd"), 400);
    }

    #[test]
    fn over_max_password() {
        let pw = "abcdefghijklmnopqrstuvwxyz1234567890abcdefghijklmn";
        assert_eq!(score_length(pw), 400);
    }
}

#[cfg(test)]
mod variety_tests {
    use super::*;

    #[test]
    fn no_characters() {
        assert_eq!(score_variety(""), 0);
    }

    #[test]
    fn only_lowercase() {
        assert_eq!(score_variety("abcdef"), 25);
    }

    #[test]
    fn only_uppercase() {
        assert_eq!(score_variety("ABCDEF"), 25);
    }

    #[test]
    fn only_digits() {
        assert_eq!(score_variety("123456"), 25);
    }

    #[test]
    fn only_symbols() {
        assert_eq!(score_variety("!@#$%"), 25);
    }

    #[test]
    fn two_types() {
        assert_eq!(score_variety("abc123"), 70); // lower + digits
        assert_eq!(score_variety("ABC!@#"), 70); // upper + symbols
    }

    #[test]
    fn three_types() {
        assert_eq!(score_variety("Abc123"), 130); // lower + upper + digits
        assert_eq!(score_variety("Abc!@#"), 130); // lower + upper + symbols
    }

    #[test]
    fn four_types() {
        assert_eq!(score_variety("Abc123!@#"), 200); // lower + upper + digits + symbols
    }
}

#[cfg(test)]
mod uniqueness_tests {
    use super::*;

    #[test]
    fn empty_password() {
        assert_eq!(score_uniqueness(""), 0);
    }

    #[test]
    fn all_same_characters() {
        assert_eq!(score_uniqueness("aaaa"), 50); // 1 unique / 4 * 200 = 50
        assert_eq!(score_uniqueness("11111111"), 25); // 1/8 * 200 = 25
    }

    #[test]
    fn all_unique_characters() {
        assert_eq!(score_uniqueness("abcd"), 200); // 4/4 * 200
        assert_eq!(score_uniqueness("a1B!"), 200); // 4/4 * 200
    }

    #[test]
    fn some_repeats() {
        assert_eq!(score_uniqueness("aabbcc"), 100); // 3 unique / 6 * 200 = 100
        assert_eq!(score_uniqueness("abcabc123"), 133); // 6 unique / 9 * 200 ≈ 133
    }

    #[test]
    fn longer_password_with_repeats() {
        let pw = "abcabcabcabc123123!!!";
        let score = score_uniqueness(pw);
        // 7 unique chars: a,b,c,1,2,3,!; len = 21 → 7/21*200 ≈ 67
        assert_eq!(score, 67);
    }
}
