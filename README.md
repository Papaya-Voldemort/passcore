# Passcore

Passcore is a Rust library for scoring password strength using a transparent, multi-factor methodology. It provides a simple API to evaluate passwords and returns a score out of 1000, with detailed breakdowns for length, character variety, uniqueness, and penalties for common patterns.

## Features
- Password scoring out of 1000 points
- Factors: length, character variety, uniqueness, penalties
- Penalty for common passwords and patterns
- Extensible scoring system for future techniques
- Includes 100k most-used passwords for penalty checks
- Simple Rust API

## Installation
Add Passcore to your `Cargo.toml`:
```toml
[dependencies]
passcore = "0.2.0"
```

## Usage
Import and use the scoring functions:

### Basic Scoring
```rust
use passcore::score;

let password = "MyS3cureP@ssw0rd!";
let score = score(password);
println!("Your password's score is {}", score);
```

### Get a Letter Grade
```rust
use passcore::grade_password;

let password = "MyS3cureP@ssw0rd!";
let grade = grade_password(password);
println!("Your password's grade is: {}", grade);
```

### Get Improvement Suggestions
```rust
use passcore::review_password;

let password = "weak";
let feedback = review_password(password);
println!("Feedback: {}", feedback);
```

### Scoring Breakdown
- **Length**: up to 400 points
- **Character Variety**: up to 200 points
- **Uniqueness**: up to 200 points
- **Penalties**: up to 200 points (deducted for common passwords or patterns)


### Score Labels
- <450 -> F
- 450–549 -> D-
- 550–599 -> D
- 600–649 -> D+
- 650–699 -> C-
- 700–749 -> C
- 750–799 -> C+
- 800–849 -> B-
- 850–899 -> B
- 900–949 -> B+
- 950–979 -> A-
- 980–994 -> A
- 995+ -> A+

## Performance
This release includes significant performance optimizations, reducing scoring time by **79.7%**:

| Benchmark | Before | After | Improvement |
|-----------|--------|-------|-------------|
| Average Passcore time | 0.304455 ms | 0.061869 ms | **79.7% faster** |
| Average zxcvbn time | 0.083165 ms | 0.076520 ms | 8.0% faster |
| Average password_strength time | 0.235982 ms | 0.221022 ms | 6.3% faster |

*Benchmarks measured on 1000 randomly generated passwords (8-24 characters) using comparative testing against zxcvbn and password_strength libraries.*
*In addition I tested twice so I thought to give you the small change of the other two libraries even though it fluctuates.*

## API Reference

### `score(password: &str) -> u16`
Scores a password on a scale of 0-1000 based on length, variety, uniqueness, and common pattern penalties.

### `grade_password(password: &str) -> &str`
Returns a letter grade (F to A+) based on the password score.

### `review_password(password: &str) -> &str`
Provides actionable feedback on which aspect of the password needs improvement (length, variety, uniqueness, or common patterns).

## Contributing
Contributions are welcome! Please open issues or pull requests for improvements, bug fixes, or new scoring techniques.

## License
This project is licensed under the [MIT License](LICENSE).
