# Contributing to financeindia

First off, thank you for considering contributing to `financeindia`! It's people like you who make it a great tool for everyone.

## How Can I Contribute?

### Reporting Bugs
- Use the GitHub issue tracker to report bugs.
- Provide a clear and concise description of the issue.
- Include steps to reproduce the bug and any relevant logs or error messages.

### Suggesting Enhancements
- Enhancement suggestions are tracked as GitHub issues.
- Explain the behavior you want and why it's useful.

### Pull Requests
1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and ensure they follow the project's coding style.
4. Add tests for your changes.
5. Submit a pull request with a clear description of the modifications.

## Development Setup

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (2024 edition)
- [Python](https://www.python.org/downloads/) (3.8+)
- [maturin](https://github.com/PyO3/maturin) (for building PyO3 extensions)

### Building from Source
```bash
# Clone the repository
git clone https://github.com/yourusername/financeindia.git
cd financeindia

# Build the extension in develop mode
maturin develop
```

## Design Principles
- **Efficiency**: Keep the codebase lightweight and the binary size small.
- **Safety**: Leverage Rust's safety guarantees to avoid crashes and memory issues.
- **Performance**: Prioritize fast data parsing and minimal network overhead.
