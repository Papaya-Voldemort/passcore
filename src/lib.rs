use crate::score::{score_length, score_penalties, score_uniqueness, score_variety};

pub mod score;
mod tests;

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
    let penalty = score_penalties(password);
    if penalty == 0 {
        score = 0;
    } else {
        score += penalty;
    }
    score
}

/// The grade_password function takes a password as input and outputs a letter grade.<br>
/// The grade_password function will never log your password <br>
///
/// # Example
/// ``` rust
/// use passcore::{grade_password, score};
///
/// let password = "password";
/// let grade = grade_password(password);
/// println!("Your password's grade is {}", grade)
/// ```
pub fn grade_password(password: &str) -> &str {
    let score = score(password);
    if score >= 995 { "A+" }
    else if score >= 980 { "A" }
    else if score >= 950 { "A-" }
    else if score >= 900 { "B+" }
    else if score >= 850 { "B" }
    else if score >= 800 { "B-" }
    else if score >= 750 { "C+" }
    else if score >= 700 { "C" }
    else if score >= 650 { "C-" }
    else if score >= 600 { "D+" }
    else if score >= 550 { "D" }
    else if score >= 450 { "D-" }
    else { "F" }
}

/// The review_password function takes a password as input and outputs suggestions for improvement.<br>
/// The review_password function will never log your password <br>
///
/// # Example
/// ``` rust
/// use passcore::review_password;
///
/// let password = "password";
/// print!("{}", review_password(password))
/// ```
pub fn review_password(password: &str) -> &str {
    let length = score_length(password) / 2;
    let variety = score_variety(password);
    let uniqueness = score_uniqueness(password);
    let penalty = score_penalties(password);

    let (min_name, min_value) = if length <= variety && length <= uniqueness && length <= penalty {
        ("length", length)
    } else if variety <= length && variety <= uniqueness && variety <= penalty {
        ("variety", variety)
    } else if uniqueness <= length && uniqueness <= variety && uniqueness <= penalty {
        ("uniqueness", uniqueness)
    } else {
        ("penalty", penalty)
    };

    if min_name == "penalty" {
        if min_value == 0 {
            "Password is too common. Change it."
        } else {
            "Password is similar to a common one. Change it."
        }
    } else if min_name == "length" {
        "Too short. Make it longer."
    } else if min_name == "variety" {
        "Add more character types."
    } else if min_name == "uniqueness" {
        "Use more unique characters."
    } else {
        panic!("You password broke the function! Please report this error!");
    }
}

// Test modules moved to src/tests.rs
