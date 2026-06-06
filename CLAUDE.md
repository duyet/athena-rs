# CLAUDE.md - Development Philosophy for athena-rs

## Vision

athena-rs is a powerful, reliable, and elegant tool for managing AWS Athena schemas through templated SQL. Every line of code should reflect our commitment to:

- **Reliability**: No panics in production, comprehensive error handling
- **Performance**: Efficient resource usage, minimal allocations
- **Clarity**: Self-documenting code, clear intent, excellent error messages
- **Maintainability**: Modular design, testable components, future-proof architecture

## Core Principles

### 1. Error Handling

**Never panic in production.**

- ❌ **NEVER** use `.unwrap()`, `.expect()` on operations that can fail
- ✅ **ALWAYS** use `?` operator with proper error context
- ✅ **ALWAYS** provide helpful error messages with `.with_context()`
- ✅ Use `Result<T>` return types for fallible operations

```rust
// ❌ BAD
let value = some_option.unwrap();
let data = resp.query_execution().unwrap().status().unwrap();

// ✅ GOOD
let value = some_option.ok_or_else(|| anyhow!("meaningful error message"))?;
let data = resp.query_execution()
    .and_then(|qe| qe.status())
    .ok_or_else(|| anyhow!("failed to get query execution status"))?;
```

### 2. Performance

**Every allocation matters.**

- Avoid unnecessary `.clone()` - use references where possible
- Cache expensive operations (regex compilation, etc.)
- Use `lazy_static` or `OnceCell` for one-time initialization
- Profile before optimizing, but think about performance from the start

```rust
// ❌ BAD - compiling regex every call
fn parse(sql: String) {
    let re = Regex::new(r"pattern").unwrap();
    // ...
}

// ✅ GOOD - compile once, use many times
lazy_static! {
    static ref SQL_PATTERN: Regex = Regex::new(r"pattern").unwrap();
}
```

### 3. Code Organization

**Every module should have a single, clear purpose.**

- Keep functions focused and small (< 50 lines ideally)
- Extract complex logic into well-named helper functions
- Use descriptive variable and function names
- Group related functionality into modules

### 4. Documentation

**Code tells you HOW, comments tell you WHY.**

- Add module-level documentation explaining purpose and usage
- Document public APIs with examples
- Use `///` for public items, `//` for implementation notes
- Keep comments up-to-date with code changes

```rust
/// Renders SQL templates from the specified path using Tera template engine.
///
/// # Arguments
///
/// * `args` - Build configuration including file path and output options
///
/// # Errors
///
/// Returns an error if:
/// - The template path doesn't exist
/// - Template rendering fails
/// - Output file cannot be created
///
/// # Examples
///
/// ```no_run
/// let args = Build { file: PathBuf::from("./templates"), out: None, context: None, no_pretty: None };
/// build(args)?;
/// ```
pub fn build(args: Build) -> Result<String> {
    // Implementation
}
```

### 5. Testing

**If it's not tested, it's broken.**

- Write tests for all public APIs
- Include edge cases and error conditions
- Use descriptive test names that explain what's being tested
- Keep tests simple and focused on one behavior

### 6. Security

**Trust no input.**

- Validate all user input (paths, S3 URIs, SQL)
- Be careful with environment variable manipulation
- Avoid command injection vulnerabilities
- Use type system to enforce invariants

```rust
// ✅ GOOD - validate S3 paths
fn validate_s3_path(path: &str) -> Result<()> {
    if !path.starts_with("s3://") {
        bail!("Invalid S3 path: must start with s3://");
    }
    // Additional validation...
    Ok(())
}
```

### 7. Dependencies

**Every dependency is a commitment.**

- Minimize dependencies when possible
- Prefer well-maintained, popular crates
- Keep dependencies up-to-date
- Review dependency security advisories regularly

## Code Style

### Formatting

- Use `cargo fmt` for all code
- 100-character line limit for readability
- 4-space indentation (enforced by rustfmt)

### Naming Conventions

- `snake_case` for functions, variables, modules
- `PascalCase` for types, traits
- `SCREAMING_SNAKE_CASE` for constants
- Descriptive names over brevity

### Imports

- Group imports: std, external crates, local modules
- Use explicit imports over glob imports
- Keep imports sorted alphabetically within groups

```rust
use std::path::PathBuf;
use std::collections::HashMap;

use anyhow::{Context, Result};
use log::{debug, info};

use crate::utils::is_dir;
use crate::tera::get_tera;
```

## Architecture Patterns

### Module Structure

```
src/
├── main.rs       # Entry point, minimal logic
├── cli.rs        # CLI argument parsing
├── build.rs      # Template building logic
├── apply.rs      # Athena execution logic
├── tera.rs       # Template engine setup
└── utils.rs      # Shared utilities
```

### Separation of Concerns

- **CLI layer**: Argument parsing only
- **Business logic**: Core functionality, no I/O
- **I/O layer**: File operations, AWS calls, printing
- **Utilities**: Pure functions, no side effects

### Error Propagation

- Use `anyhow::Result` for application errors
- Add context at each level of the call stack
- Let errors bubble up, handle at appropriate level
- Provide actionable error messages to users

## Common Patterns

### Constants Over Magic Values

```rust
// ❌ BAD
sleep(Duration::from_secs(5)).await;

// ✅ GOOD
const POLL_INTERVAL_SECS: u64 = 5;
sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
```

### Builder Pattern for Complex Configs

```rust
// For complex configuration, use builder pattern
let config = ResultConfiguration::builder()
    .set_output_location(output_location)
    .build();
```

### Async Best Practices

- Use `tokio::spawn` for concurrent operations
- Avoid blocking operations in async code
- Use timeout for external calls
- Handle cancellation gracefully

## Git Workflow

### Commit Messages

Follow conventional commits:

```
feat: add support for Athena workgroups
fix: prevent panic when query execution status is null
docs: improve README with more examples
refactor: extract database parsing logic
test: add edge cases for date_range function
chore: update dependencies
```

### Pull Requests

- One logical change per PR
- Include tests for new functionality
- Update documentation as needed
- Ensure CI passes before requesting review

## CI/CD

### Required Checks

All PRs must pass:

- ✅ `cargo fmt --check` - Code formatting
- ✅ `cargo clippy -- -D warnings` - Linting
- ✅ `cargo test` - All tests
- ✅ `cargo doc` - Documentation builds
- ✅ Code coverage threshold

### Release Process

1. Update CHANGELOG.md
2. Bump version in Cargo.toml
3. Create git tag
4. CI automatically publishes to crates.io

## Tools & Configuration

### Recommended Clippy Lints

Enable in `Cargo.toml`:

```toml
[lints.clippy]
unwrap_used = "deny"
expect_used = "warn"
panic = "deny"
```

### Editor Configuration

- Use rust-analyzer for IDE support
- Enable format-on-save
- Configure clippy lints in editor

## Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [AWS SDK for Rust](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html)

## Questions?

When in doubt:
1. **Prefer safety over performance** (optimize later if needed)
2. **Prefer clarity over cleverness** (readable code > clever code)
3. **Prefer explicit over implicit** (no magic, clear intent)
4. **Prefer boring over exciting** (tried and true > cutting edge)

---

*"Simplicity is the ultimate sophistication." - Leonardo da Vinci*
