# Contributing to Fish

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

Thank you for your interest in contributing to Fish!

## Contribution Workflow

1. Fork the repository on GitHub: `https://github.com/requla11/fish`
2. Clone your fork locally:
   ```bash
   git clone https://github.com/<YOUR_USERNAME>/fish.git
   cd fish
   git checkout -b feat/my-feature dev
   ```
3. Make your changes following the project guidelines:
   - Ensure code is written in English.
   - Run tests: `cargo test --workspace`.
   - Verify formatting: `cargo fmt --all -- --check`.
   - Check linter: `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
4. Commit your changes with clear English commit messages.
5. Push to your fork and submit a Pull Request targeting the **`dev`** branch.
