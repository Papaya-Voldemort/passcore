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
passcore = "0.1.0"
```

## Usage
Import and use the scoring function:
```rust
use passcore::score;

let password = "MyS3cureP@ssw0rd!";
let score = score(password);
println!("Your password's score is {}", score);
```

### Scoring Breakdown
- **Length**: up to 400 points
- **Character Variety**: up to 200 points
- **Uniqueness**: up to 200 points
- **Penalties**: up to 200 points (deducted for common passwords or patterns)


### Score Labels
- <300 → Weak
- 300–600 → Okay
- 600–800 → Strong
- 800+ → Very Strong
- 980+ → God Like

## Contributing
Contributions are welcome! Please open issues or pull requests for improvements, bug fixes, or new scoring techniques.

## License
This project is licensed under the [MIT License](LICENSE).
# passcore
