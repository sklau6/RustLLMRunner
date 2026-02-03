# Contributing to Rust LLM Runner

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Development Setup

1. **Install Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/rust-llm-runner.git
   cd rust-llm-runner
   ```

3. **Build the project**
   ```bash
   cargo build
   ```

4. **Run tests**
   ```bash
   cargo test
   ```

## Code Style

- Follow Rust standard formatting: `cargo fmt`
- Run clippy for linting: `cargo clippy`
- Write documentation for public APIs
- Add tests for new features

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and formatting
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## Commit Messages

- Use clear and descriptive commit messages
- Start with a verb in present tense (e.g., "Add", "Fix", "Update")
- Reference issues when applicable

## Testing

- Write unit tests for new functionality
- Ensure all tests pass before submitting PR
- Add integration tests for API endpoints

## Documentation

- Update README.md for user-facing changes
- Add inline documentation for code
- Update examples if API changes

## Questions?

Feel free to open an issue for any questions or concerns.
