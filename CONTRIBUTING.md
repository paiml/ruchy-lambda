# Contributing to Ruchy Lambda

Thank you for your interest in contributing to Ruchy Lambda! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Quality Standards](#quality-standards)
5. [Testing Requirements](#testing-requirements)
6. [Pull Request Process](#pull-request-process)
7. [Coding Guidelines](#coding-guidelines)
8. [License](#license)

---

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- Be respectful and considerate
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Report any unacceptable behavior to the maintainers

## Getting Started

### Prerequisites

```bash
# Install Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-tarpaulin  # Code coverage
cargo install cargo-mutants    # Mutation testing

# Install Ruchy transpiler
git clone https://github.com/paiml/ruchy
cd ruchy
cargo install --path .
```

### Fork and Clone

```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/YOUR_USERNAME/ruchy-lambda
cd ruchy-lambda

# Add upstream remote
git remote add upstream https://github.com/paiml/ruchy-lambda
```

### Build and Test

```bash
# Build the project
cargo build

# Run all tests
cargo test --workspace

# Run specific test suite
cargo test -p ruchy-lambda-runtime
cargo test -p ruchy-lambda-bootstrap

# Check code quality
cargo clippy --all-targets --all-features
cargo fmt --check
```

---

## Development Workflow

We follow **Extreme TDD** (Test-Driven Development):

### RED → GREEN → REFACTOR

1. **RED Phase**: Write failing tests first
   ```rust
   #[test]
   fn test_new_feature() {
       // Test the feature you want to add
       assert!(new_feature_works());
   }
   ```

2. **GREEN Phase**: Implement minimum code to pass tests
   ```rust
   fn new_feature_works() -> bool {
       // Implement just enough to pass
       true
   }
   ```

3. **REFACTOR Phase**: Optimize while keeping tests passing
   ```rust
   fn new_feature_works() -> bool {
       // Optimize, clean up, improve
       optimized_implementation()
   }
   ```

### Branching Strategy

- **main**: Production-ready code (always stable)
- **feature/xxx**: New features
- **fix/xxx**: Bug fixes
- **docs/xxx**: Documentation updates

**Important**: We work directly on main for this project (no branching per .claude/CLAUDE.md)

### Commit Messages

Use conventional commits:

```
feat: add response streaming support
fix: correct request_id extraction from headers
docs: update architecture documentation
test: add mutation tests for logger
perf: optimize HTTP client connection pooling
```

---

## Quality Standards

### Toyota Way Principles

All contributions must meet our quality standards:

1. **Zero Defects**: TDG Grade ≥A
2. **High Test Coverage**: ≥85%
3. **Strong Mutation Score**: ≥85%
4. **Low Complexity**: Cyclomatic ≤15, Cognitive ≤20
5. **No Technical Debt**: Zero SATD violations

### Quality Gates (Must Pass)

```bash
# 1. All tests pass
cargo test --workspace

# 2. Code coverage ≥85%
cargo tarpaulin --workspace --out Html
# Open tarpaulin-report.html and verify ≥85%

# 3. Mutation score ≥85%
cd crates/runtime
cargo mutants --no-shuffle
# Verify mutation score ≥85%

# 4. No clippy warnings
cargo clippy --all-targets --all-features -- -D warnings

# 5. Code formatted
cargo fmt --check

# 6. Documentation builds
cargo doc --no-deps --workspace
```

---

## Testing Requirements

### Test Categories

1. **Unit Tests**: Test individual functions/methods
   ```rust
   #[test]
   fn test_parse_headers() {
       let input = "Content-Length: 42\r\n";
       let result = parse_header(input);
       assert_eq!(result, ("Content-Length", "42"));
   }
   ```

2. **Integration Tests**: Test component interactions
   ```rust
   #[test]
   fn test_runtime_api_integration() {
       let runtime = Runtime::new().unwrap();
       let (req_id, body) = runtime.next_event().unwrap();
       assert!(!req_id.is_empty());
   }
   ```

3. **Property-Based Tests**: Test invariants
   ```rust
   use proptest::prelude::*;

   proptest! {
       #[test]
       fn test_header_parsing(s in "\\PC*") {
           let result = parse_header(&s);
           // Verify properties hold for all inputs
       }
   }
   ```

4. **AWS Validation Tests**: Test real AWS deployment
   ```bash
   cargo test -p ruchy-lambda-bootstrap \
     --test aws_validation_tests -- --ignored
   ```

### Test Isolation

Use `#[serial]` for tests that:
- Modify environment variables
- Use shared global state
- Require sequential execution

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_with_env_var() {
    env::set_var("TEST_VAR", "value");
    // test code
    env::remove_var("TEST_VAR");
}
```

---

## Pull Request Process

### Before Submitting

1. **Update from upstream**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run full test suite**:
   ```bash
   cargo test --workspace
   cargo clippy --all-targets --all-features
   cargo fmt
   ```

3. **Check quality gates**:
   - Code coverage ≥85%
   - Mutation score ≥85% (if modifying runtime)
   - No clippy warnings
   - All tests passing

### PR Template

When creating a PR, include:

**Title**: Brief description (e.g., "feat: add response streaming")

**Description**:
```markdown
## Summary
Brief overview of changes

## Motivation
Why is this change needed?

## Changes
- Added feature X
- Fixed bug Y
- Updated documentation Z

## Testing
- Added 12 unit tests
- Integration tests passing
- Mutation score: 87.3%
- Code coverage: 89.2%

## Checklist
- [x] Tests added/updated
- [x] Documentation updated
- [x] Quality gates passing
- [x] CHANGELOG.md updated (if applicable)
```

### Review Process

1. Automated checks run (CI/CD)
2. Maintainer review (may request changes)
3. Address feedback
4. Maintainer approval
5. Merge to main

---

## Coding Guidelines

### Rust Style

Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

```rust
// Good: Clear, idiomatic Rust
pub fn next_event(&self) -> Result<(String, String), Error> {
    let (headers, body) = self.client.get(&url)?;
    let request_id = extract_request_id(&headers)?;
    Ok((request_id, body))
}

// Bad: Unclear, non-idiomatic
pub fn get_event(&self) -> (String, String) {
    let x = self.client.get(&url).unwrap();
    (x.0, x.1)
}
```

### Performance Considerations

- **Minimize allocations**: Use references where possible
- **Avoid async overhead**: Use blocking I/O (Lambda processes one event at a time)
- **Zero-copy**: Pass event data by reference
- **Small binary**: Minimize dependencies

### Documentation

All public APIs must have documentation:

```rust
/// Fetches the next event from Lambda Runtime API
///
/// This method blocks until an event is available (long-polling).
///
/// # Returns
///
/// A tuple of (request_id, event_body) where:
/// - `request_id`: Unique ID from Lambda-Runtime-Aws-Request-Id header
/// - `event_body`: Raw user event payload (JSON string)
///
/// # Errors
///
/// Returns `Error::MissingRequestId` if header not found
/// Returns `Error::HttpError` if network request fails
///
/// # Examples
///
/// ```no_run
/// let runtime = Runtime::new()?;
/// let (request_id, body) = runtime.next_event()?;
/// ```
pub fn next_event(&self) -> Result<(String, String), Error> {
    // implementation
}
```

### Error Handling

Use `Result<T, Error>` for fallible operations:

```rust
// Good: Explicit error handling
pub fn parse_header(line: &str) -> Result<(String, String), Error> {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(Error::InvalidHeader);
    }
    Ok((parts[0].to_string(), parts[1].trim().to_string()))
}

// Bad: Using unwrap() in library code
pub fn parse_header(line: &str) -> (String, String) {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    (parts[0].to_string(), parts[1].to_string())  // Will panic!
}
```

---

## License

By contributing to Ruchy Lambda, you agree that your contributions will be licensed under both:

- **MIT License** (LICENSE-MIT or http://opensource.org/licenses/MIT)
- **Apache License 2.0** (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at the user's option.

Your contributions must be your own work and you must have the right to submit them under these licenses.

---

## Questions?

- **Issues**: https://github.com/paiml/ruchy-lambda/issues
- **Discussions**: https://github.com/paiml/ruchy-lambda/discussions

Thank you for contributing to Ruchy Lambda!

---

**Version**: 1.0.0
**Last Updated**: 2025-11-04
