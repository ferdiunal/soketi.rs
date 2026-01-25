# Contributing to Soketi.rs

First off, thank you for considering contributing to Soketi.rs! It's people like you that make Soketi.rs such a great tool.

## Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold this code.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible:

- Use a clear and descriptive title
- Describe the exact steps which reproduce the problem
- Provide specific examples to demonstrate the steps
- Describe the behavior you observed after following the steps
- Explain which behavior you expected to see instead and why
- Include logs and error messages
- Include your configuration (remove sensitive data)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

- Use a clear and descriptive title
- Provide a step-by-step description of the suggested enhancement
- Provide specific examples to demonstrate the steps
- Describe the current behavior and explain which behavior you expected to see instead
- Explain why this enhancement would be useful

### Pull Requests

- Fill in the required template
- Follow the Rust style guide
- Include appropriate test coverage
- Update documentation as needed
- End all files with a newline

## Development Setup

### Prerequisites

- Rust 1.75 or higher
- Docker and Docker Compose (for testing)
- Redis (for adapter testing)
- PostgreSQL or MySQL (for database testing)

### Setting Up Development Environment

```bash
# Clone the repository
git clone https://github.com/ferdiunal/soketi.rs.git
cd soketi.rs

# Install dependencies
cargo build

# Run tests
cargo test

# Run with example config
cargo run -- --config-file config.json
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test

# Run integration tests (requires services)
docker-compose up -d redis postgres
cargo test --test integration_tests
```

### Code Style

We use `rustfmt` and `clippy` to maintain code quality:

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy -- -D warnings

# Fix clippy issues automatically
cargo clippy --fix
```

### Documentation

- Add doc comments to all public APIs
- Update README.md if needed
- Update relevant documentation in `docs/`
- Include examples for new features

### Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line

Example:
```
Add Redis adapter support

- Implement Redis pub/sub for message distribution
- Add configuration options for Redis connection
- Include tests for Redis adapter

Fixes #123
```

## Project Structure

```
soketi.rs/
├── src/                    # Source code
│   ├── adapters/          # Message distribution adapters
│   ├── app_managers/      # App configuration managers
│   ├── cache_managers/    # Caching implementations
│   ├── channels/          # Channel managers
│   ├── queues/            # Webhook queue managers
│   └── ...
├── tests/                 # Integration tests
├── examples/              # Example code
├── docs/                  # Documentation
├── deployment/            # Deployment configurations
└── demo-chat/            # Demo application
```

## Testing Guidelines

### Unit Tests

- Write unit tests for all new functionality
- Place tests in the same file as the code (using `#[cfg(test)]`)
- Use descriptive test names

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_works_correctly() {
        // Test implementation
    }
}
```

### Integration Tests

- Place integration tests in `tests/` directory
- Test real-world scenarios
- Use Docker Compose for external dependencies

### Performance Tests

- Mark performance tests with `#[ignore]`
- Document expected performance characteristics
- Run performance tests separately

## Documentation Guidelines

### Code Documentation

```rust
/// Brief description of the function
///
/// More detailed explanation if needed.
///
/// # Arguments
///
/// * `param1` - Description of param1
/// * `param2` - Description of param2
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```
/// let result = function(arg1, arg2);
/// ```
pub fn function(param1: Type1, param2: Type2) -> ReturnType {
    // Implementation
}
```

### User Documentation

- Write in clear, simple language
- Include code examples
- Provide both English and Turkish versions when possible
- Update relevant guides in `docs/`

## Release Process

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create a git tag: `git tag v1.0.0`
4. Push tag: `git push origin v1.0.0`
5. GitHub Actions will automatically build and publish

## Community

- Join our [GitHub Discussions](https://github.com/ferdiunal/soketi.rs/discussions)
- Follow [@ferdiunal](https://github.com/ferdiunal) on GitHub
- Star the repository if you find it useful!

## Questions?

Feel free to ask questions in:
- [GitHub Discussions](https://github.com/ferdiunal/soketi.rs/discussions)
- [GitHub Issues](https://github.com/ferdiunal/soketi.rs/issues)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- GitHub contributors page

Thank you for contributing to Soketi.rs! 🚀
