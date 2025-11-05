# Ruchy → Rust Transpiler Specification

**Version**: 1.0.0
**Status**: DRAFT (Extreme TDD - Specification written FIRST)
**Date**: 2025-11-04

This document defines the contract for the Ruchy → Rust transpiler, written using extreme TDD principles before implementation.

## Overview

The Ruchy transpiler converts Ruby-like Ruchy code into optimized Rust code for AWS Lambda, targeting <8ms cold start times and <100μs invocation overhead.

## Interface Contract

### Command Line Interface

```bash
# Basic transpilation
ruchy transpile <input.ruchy> -o <output.rs>

# With optimization flags
ruchy transpile <input.ruchy> -o <output.rs> --optimize

# Watch mode (for development)
ruchy transpile <input.ruchy> -o <output.rs> --watch

# Version
ruchy --version

# Help
ruchy --help
```

### Input Format

**File Extension**: `.ruchy`

**Example Input** (`hello_world.ruchy`):
```ruby
# Lambda handler function
def handler(event)
  request_id = event["requestContext"]["requestId"]
  message = "Hello from Ruchy Lambda! Request ID: #{request_id}"

  {
    "statusCode" => 200,
    "body" => message
  }
end

# Entry point
Lambda.start(handler: :handler)
```

### Output Format

**File Extension**: `.rs`

**Requirements**:
1. Valid Rust code that compiles with `rustc`
2. Integrates with `ruchy-lambda-runtime` crate
3. Uses `LambdaEvent` and `Runtime` types
4. Implements proper error handling (`Result<T, E>`)
5. Preserves handler logic exactly
6. Generates performant code (minimal allocations)

**Example Output** (see `examples/hello_world.expected.rs`)

## Translation Rules

### 1. Function Definitions

**Ruchy:**
```ruby
def handler(event)
  # body
end
```

**Rust:**
```rust
fn handler(event: LambdaEvent) -> Result<Value, Box<dyn Error>> {
    // body
}
```

### 2. Hash Literals

**Ruchy:**
```ruby
{
  "statusCode" => 200,
  "body" => "Hello"
}
```

**Rust:**
```rust
json!({
    "statusCode": 200,
    "body": "Hello"
})
```

### 3. String Interpolation

**Ruchy:**
```ruby
"Hello #{name}!"
```

**Rust:**
```rust
format!("Hello {}!", name)
```

### 4. Hash Access

**Ruchy:**
```ruby
event["requestContext"]["requestId"]
```

**Rust:**
```rust
event.request_context.request_id
```

### 5. Entry Point

**Ruchy:**
```ruby
Lambda.start(handler: :handler)
```

**Rust:**
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new()?;

    loop {
        let event_json = runtime.next_event().await?;
        let event: LambdaEvent = serde_json::from_str(&event_json)?;
        let request_id = event.request_context.request_id.clone();

        let response = handler(event)?;
        let response_json = serde_json::to_string(&response)?;

        runtime.post_response(&request_id, &response_json).await?;
    }
}
```

## Performance Requirements

### Cold Start Time
- **Target**: <8ms on ARM64 Graviton2
- **Approach**:
  - Minimal dependencies
  - Lazy initialization
  - Zero-copy where possible

### Invocation Overhead
- **Target**: <100μs per invocation
- **Approach**:
  - No allocations in hot path
  - Efficient string handling
  - Reuse of event loop structures

### Binary Size
- **Target**: <100KB (stretch goal from 415KB current)
- **Approach**:
  - Minimal runtime dependencies
  - No unnecessary stdlib features
  - Aggressive LTO and optimization

## Validation Tests

All transpiled code must pass these validations:

1. **Compilation**: `cargo build` succeeds
2. **Structure**: Contains required functions (handler, main)
3. **Logic Preservation**: Handler behavior matches Ruchy source
4. **Performance**: No unnecessary allocations in handler
5. **Error Handling**: Uses `Result<T, E>` properly
6. **Integration**: Works with `ruchy-lambda-runtime`
7. **Imports**: Correct `use` statements
8. **Idioms**: Follows Rust conventions

See `crates/runtime/tests/transpiler_validation_tests.rs` for full test suite.

## Error Handling

### Transpiler Errors

The transpiler should return clear error messages:

```
Error: Unsupported syntax at line 12
  |
12| class MyClass
  | ^^^^^
  |
  | Ruchy Lambda does not support classes. Use functions instead.
```

### Runtime Errors

Generated code should handle errors gracefully:

```rust
// Bad - panic
let id = event["id"].unwrap();

// Good - propagate
let id = event.get("id").ok_or("Missing id")?;
```

## Optimization Flags

### `--optimize`

Enables aggressive optimizations:
- Inlines small functions
- Removes unused code paths
- Applies const folding
- Uses zero-copy where possible

### `--no-std` (future)

Generates no_std compatible code for even smaller binaries.

## Integration with Cargo

### `build.rs` Integration

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=src/handler.ruchy");

    let status = std::process::Command::new("ruchy")
        .args(["transpile", "src/handler.ruchy", "-o", "src/handler.rs"])
        .status()
        .expect("Failed to run transpiler");

    if !status.success() {
        panic!("Transpilation failed");
    }
}
```

### Cargo.toml

```toml
[build-dependencies]
# Future: ruchy-transpiler crate for programmatic access
```

## Future Enhancements

1. **Type Inference**: Infer Rust types from Ruchy usage
2. **Macros**: Support Ruchy macros → Rust macros
3. **Async/Await**: Native async support in Ruchy
4. **Error Messages**: Better error reporting with suggestions
5. **IDE Integration**: LSP server for Ruchy
6. **Incremental Compilation**: Only transpile changed files

## Compatibility

### Ruchy Language Subset

For Lambda runtime, Ruchy supports:
- ✅ Functions (`def`)
- ✅ Hash literals (`{}`)
- ✅ String interpolation (`#{}`)
- ✅ Basic control flow (`if`, `loop`)
- ❌ Classes (not needed for Lambda)
- ❌ Modules (use Rust modules)
- ❌ Metaprogramming (compile-time only)

### Rust Version

- **Minimum**: rustc 1.70.0
- **Recommended**: rustc 1.75.0+
- **Edition**: 2021

## Example: Full Translation

### Input: `fibonacci.ruchy`

```ruby
def fibonacci(n)
  return 0 if n == 0
  return 1 if n == 1
  fibonacci(n - 1) + fibonacci(n - 2)
end

def handler(event)
  n = event["body"]["n"].to_i
  result = fibonacci(n)

  {
    "statusCode" => 200,
    "body" => result.to_s
  }
end

Lambda.start(handler: :handler)
```

### Output: `fibonacci.rs`

```rust
use ruchy_lambda_runtime::{LambdaEvent, Runtime};
use serde_json::{json, Value};
use std::error::Error;

fn fibonacci(n: i64) -> i64 {
    if n == 0 { return 0; }
    if n == 1 { return 1; }
    fibonacci(n - 1) + fibonacci(n - 2)
}

fn handler(event: LambdaEvent) -> Result<Value, Box<dyn Error>> {
    let body: Value = serde_json::from_str(&event.body)?;
    let n = body["n"].as_i64().unwrap_or(0);
    let result = fibonacci(n);

    Ok(json!({
        "statusCode": 200,
        "body": result.to_string()
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new()?;

    loop {
        let event_json = runtime.next_event().await?;
        let event: LambdaEvent = serde_json::from_str(&event_json)?;
        let request_id = event.request_context.request_id.clone();

        let response = handler(event)?;
        let response_json = serde_json::to_string(&response)?;

        runtime.post_response(&request_id, &response_json).await?;
    }
}
```

## Testing Strategy

### Unit Tests

Test individual translation rules:
```rust
#[test]
fn test_translate_hash_literal() {
    let input = r#"{"key" => "value"}"#;
    let output = transpile(input);
    assert!(output.contains("json!"));
}
```

### Integration Tests

Test full file transpilation:
```rust
#[test]
fn test_transpile_hello_world() {
    let result = transpile_file("examples/hello_world.ruchy");
    assert!(result.is_ok());
    assert!(compiles(&result.unwrap()));
}
```

### Property-Based Tests

Test invariants:
```rust
#[test]
fn property_transpiled_always_compiles() {
    for source in valid_ruchy_sources() {
        let transpiled = transpile(source);
        assert!(compiles(transpiled));
    }
}
```

## References

- **Lambda Runtime API**: [AWS Documentation](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html)
- **Performance Spec**: `docs/performance-requirements.md`
- **Runtime Crate**: `crates/runtime/`
- **Example Handlers**: `examples/`

## Status

- ✅ Specification complete (extreme TDD)
- ✅ Example Ruchy code written
- ✅ Expected Rust output defined
- ✅ Validation tests written
- ⏳ Transpiler implementation (pending)
- ⏳ Build script integration (pending)

This specification was written FIRST using extreme TDD principles to define the contract before implementation.
