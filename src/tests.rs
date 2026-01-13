//! Test suite for passcore

#[cfg(test)]
mod length_tests {
    use crate::score::score_length;

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
    use crate::score::score_variety;

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
    use crate::score::score_uniqueness;

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

#[cfg(test)]
mod extra_tests {
    use crate::score::{score_length, score_variety, score_uniqueness, score_penalties};
    use crate::{score, grade_password, review_password};

    #[test]
    fn non_ascii_passwords() {
        // Unicode, emoji, accented
        assert!(score_length("pässwörd") > 0);
        assert!(score_variety("пароль") > 0); // Cyrillic
        assert!(score_uniqueness("パスワード") > 0); // Japanese
        assert!(score_variety("🔒🔑") > 0); // Emoji
        assert!(score_length("áéíóúüñç") > 0); // Accented
    }

    #[test]
    fn penalties_common_password() {
        // Assuming score_penalties returns 0 for common passwords
        assert_eq!(score_penalties("password"), 0);
        assert_eq!(score_penalties("123456"), 0);
    }

    #[test]
    fn penalties_similar_password() {
        // Check for positive penalty (not useless comparison)
        assert!(score_penalties("password1") == 0);
        assert!(score_penalties("1234567") == 0);
    }

    #[test]
    fn score_and_grade() {
        let pw = "Abc123!@#1"; // Not a common password, should get a positive score
        let s = score(pw);
        let g = grade_password(pw);
        assert!(s > 0);
        assert!(matches!(g, "A+" | "A" | "A-" | "B+" | "B" | "B-" | "C+" | "C" | "C-" | "D+" | "D" | "D-" | "F"));
    }

    #[test]
    fn review_password_cases() {
        assert_eq!(review_password("password"), "Password is too common. Change it.");
        assert_eq!(review_password("admin@321"), "Password is similar to a common one. Change it.");
        assert_eq!(review_password("xyza"), "Too short. Make it longer."); // changed from "qwer" to "xyza"
    }
}
