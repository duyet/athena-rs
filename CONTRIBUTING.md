# Contributing to athena-rs

Thank you for your interest in contributing to athena-rs! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.64.0 or later
- AWS CLI (for testing with real AWS Athena)
- Git

### Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/athena-rs.git
   cd athena-rs
   ```

3. Build the project:
   ```bash
   cargo build
   ```

4. Run tests:
   ```bash
   cargo test
   ```

## Development Workflow

### Before You Start

- Read [CLAUDE.md](./CLAUDE.md) for the project's development philosophy
- Check existing issues and pull requests to avoid duplication
- For major changes, open an issue first to discuss the proposal

### Making Changes

1. Create a new branch from `master`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes following our coding standards (see below)

3. Add tests for new functionality

4. Ensure all tests pass:
   ```bash
   cargo test
   ```

5. Format your code:
   ```bash
   cargo fmt
   ```

6. Run clippy and fix any warnings:
   ```bash
   cargo clippy --all-targets --all-features
   ```

7. Commit your changes with a descriptive message:
   ```bash
   git commit -m "feat: add support for custom Athena catalogs"
   ```

### Commit Message Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Test updates
- `chore:` - Maintenance tasks
- `perf:` - Performance improvements

Examples:
```
feat: add support for Athena workgroups
fix: prevent panic when query status is null
docs: improve README with more examples
refactor: extract database parsing logic
test: add edge cases for date_range function
chore: update dependencies
```

### Pull Request Process

1. Update documentation if needed (README.md, code comments, etc.)
2. Ensure your PR description clearly describes the problem and solution
3. Link any related issues in the PR description
4. Request review from maintainers
5. Address review feedback promptly

### Code Review

All submissions require review. We use GitHub pull requests for this purpose. Reviewers will check for:

- Code quality and style
- Test coverage
- Documentation updates
- Breaking changes
- Performance implications

## Coding Standards

### Rust Style Guide

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting (enforced in CI)
- Fix all `cargo clippy` warnings
- Write clear, self-documenting code

### Error Handling

- **NEVER** use `.unwrap()` or `.expect()` in production code
- Use `?` operator with proper error context
- Provide helpful error messages with `.with_context()`
- Return `Result<T>` for fallible operations

```rust
// ❌ BAD
let value = some_option.unwrap();

// ✅ GOOD
let value = some_option.ok_or_else(|| anyhow!("meaningful error message"))?;
```

### Testing

- Write tests for all public APIs
- Include edge cases and error conditions
- Use descriptive test names
- Keep tests simple and focused

```rust
#[test]
fn test_build_sql_from_template_with_variables() {
    // Arrange
    let template = "CREATE TABLE {{ table_name }}";

    // Act
    let result = build_sql(template);

    // Assert
    assert!(result.contains("CREATE TABLE"));
}
```

### Documentation

- Add module-level documentation
- Document all public APIs
- Include examples in doc comments
- Use `///` for public items, `//` for implementation notes

```rust
/// Renders SQL templates from the specified path using Tera template engine.
///
/// # Arguments
///
/// * `args` - Build configuration including file path and output options
///
/// # Errors
///
/// Returns an error if template rendering fails or output file cannot be created
///
/// # Examples
///
/// ```no_run
/// let args = Build { file: PathBuf::from("./templates"), ... };
/// let sql = build(args)?;
/// ```
pub fn build(args: Build) -> Result<String> {
    // Implementation
}
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'
```

### Test Coverage

We aim for high test coverage. Check coverage locally:

```bash
cargo tarpaulin --out Html
```

## CI/CD

All pull requests must pass:

- ✅ `cargo fmt --check` - Code formatting
- ✅ `cargo clippy` - Linting
- ✅ `cargo test` - All tests
- ✅ `cargo doc` - Documentation builds

These checks run automatically on pull requests.

## Reporting Issues

### Bug Reports

When reporting bugs, include:

- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- Environment (OS, Rust version, etc.)
- Relevant logs or error messages

### Feature Requests

When requesting features:

- Describe the use case
- Explain why this would be valuable
- Suggest possible implementation approaches
- Consider backward compatibility

## Getting Help

- Read the [README](./README.md)
- Check [existing issues](https://github.com/duyet/athena-rs/issues)
- Open a new issue for questions
- Review [CLAUDE.md](./CLAUDE.md) for development philosophy

## Code of Conduct

### Our Standards

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Assume good intentions

### Unacceptable Behavior

- Harassment or discriminatory language
- Trolling or insulting comments
- Publishing private information
- Other unprofessional conduct

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

Feel free to open an issue or reach out to the maintainers.

---

Thank you for contributing to athena-rs! 🚀
