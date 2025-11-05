# Pure Ruchy Lambda Runtime

**Status**: ✅ Hybrid Implementation - Functional (transpiler bugs fixed in v3.208.0!)

## Vision

A Ruchy-based Lambda runtime demonstrating Ruchy as a systems programming language.

**Current Reality**: Hybrid approach (~40% Ruchy, ~60% Rust) due to parser limitations.

## Current Status

✅ **Working**:
- Runtime API in Ruchy (`lib.ruchy`) - struct, impl, methods
- HTTP client in Rust (`http_client.rs`) - TcpStream, I/O
- Transpilation pipeline - Ruchy → Rust at build time
- Full Lambda Runtime API support (next_event, post_response)
- Compiles and builds successfully

✅ **Fixed in v3.208.0**:
- ✅ Arithmetic operations work correctly (was: `+` → `format!()`)
- ✅ Method names preserved (was: `add()` → `insert()`)
- ✅ Standalone functions generated (was: functions disappeared)
- ✅ `pub fun` → `pub fn` (was: lost visibility)
- ✅ No spurious `.cloned()` calls

⚠️ **Remaining Minor Issues**:
- Unnecessary braces around return values (cosmetic, non-blocking)
- `vec![0u8; 1024]` syntax not supported (can use `Vec::with_capacity()`)
- `mod http_client;` declarations not supported (use build.rs workaround)

📋 **Issue**: https://github.com/paiml/ruchy/issues/137 (bugs fixed, can close!)

## Testing

**Integration tests** (5/5 passing):
```bash
cargo test --test integration_tests
# ✅ test_runtime_can_be_created
# ✅ test_hybrid_architecture
# ✅ test_runtime_next_event
# ✅ test_runtime_post_response
# ✅ test_transpilation_quality
```

**Build verification**:
```bash
# Library builds successfully
cargo build --release -p ruchy-lambda-runtime-pure
# ✅ Compiles with 2 warnings (unnecessary braces)

# Standalone binary builds successfully
cd examples && ruchy transpile bootstrap.ruchy -o bootstrap_generated.rs
rustc --edition 2021 -C opt-level=3 -C lto=fat ... bootstrap_generated.rs -o bootstrap_optimized
# ✅ Result: 338KB optimized binary (vs 400KB production runtime)
```

**Quality gate status**:
```bash
cargo clippy -- -D warnings
# ⚠️ FAILS: 2 warnings (unnecessary braces, unused imports)
# Workaround: #![allow(clippy::all)] in generated code
```

## Architecture

### Hybrid Approach (Current)

```
crates/runtime-pure/
├── Cargo.toml              # Crate configuration
├── build.rs                # Transpiles lib.ruchy + injects http_client.rs
├── src/
│   ├── lib.ruchy           # Ruchy runtime API (struct, impl, methods)
│   ├── http_client.rs      # Rust HTTP client (TcpStream, I/O)
│   └── lib_generated.rs    # Generated Rust (transpiled + injected)
└── examples/
    └── bootstrap.ruchy     # Pure Ruchy Lambda bootstrap example
```

**Composition**: ~40% Ruchy, ~60% Rust (HTTP client)

### What's in Ruchy

**`src/lib.ruchy`** - Runtime API:
```ruchy
pub struct Runtime {
    api_endpoint: String,
}

impl Runtime {
    pub fun new() -> Runtime { ... }
    pub fun next_event(&self) -> (String, String) { ... }
    pub fun post_response(&self, request_id: &str, body: &str) -> bool { ... }
}
```

### What's in Rust

**`src/http_client.rs`** - Low-level I/O:
- `http_get(endpoint, path) -> Result<(request_id, body), String>`
- `http_post(endpoint, path, body) -> Result<(), String>`
- HTTP/1.1 request building
- TcpStream connection handling
- Response parsing (extract Lambda-Runtime-Aws-Request-Id header)

### How It Works

1. **Build time**: `build.rs` runs `ruchy transpile src/lib.ruchy`
2. **Inject module**: `build.rs` reads `http_client.rs` and wraps it in `mod http_client { ... }`
3. **Fix transpiler bugs**: Replace `http_client.http_get(` → `http_client::http_get(`
4. **Output**: `src/lib_generated.rs` (transpiled Ruchy + injected Rust)
5. **Cargo compiles**: Standard Rust compilation

## Usage

```rust
use ruchy_lambda_runtime_pure::Runtime;

fn main() {
    let runtime = Runtime::new();

    loop {
        let (request_id, event_body) = runtime.next_event();
        println!("Received event: {}", event_body);

        let response = handler(&event_body);
        runtime.post_response(&request_id, &response);
    }
}

fn handler(event: &str) -> String {
    format!("{{\"statusCode\":200,\"body\":\"Processed: {}\"}}", event)
}
```

## Building

```bash
# Build library (transpiles at build time)
cargo build -p ruchy-lambda-runtime-pure

# View generated code
cat crates/runtime-pure/src/lib_generated.rs | head -50

# Build standalone bootstrap executable (Pure Ruchy!)
cd crates/runtime-pure/examples
ruchy transpile bootstrap.ruchy -o bootstrap_generated.rs
cargo build --release -p ruchy-lambda-runtime-pure  # Build library first
rustc --edition 2021 -C opt-level=3 -C lto=fat -C codegen-units=1 -C strip=symbols \
  -L ../../../target/release/deps \
  --extern ruchy_lambda_runtime_pure=../../../target/release/libruchy_lambda_runtime_pure.rlib \
  bootstrap_generated.rs -o bootstrap_optimized

# Result: 338KB optimized binary!
ls -lh bootstrap_optimized
```

### Why not `ruchy compile`?

`ruchy compile` **cannot link external crates**, which blocks standalone compilation:

```bash
# This fails:
ruchy compile bootstrap.ruchy -o bootstrap
# error[E0432]: unresolved import `ruchy_lambda_runtime_pure`
#   use of unresolved module or unlinked crate `ruchy_lambda_runtime_pure`

# This also fails:
ruchy compile src/lib.ruchy -o runtime
# error[E0433]: use of unresolved module or unlinked crate `http_client`
```

**Workaround**: Use `ruchy transpile` + `rustc`/`cargo` with explicit linking (as shown above).

**Tracked in**: https://github.com/paiml/ruchy/issues/137

## Future: Pure Ruchy Implementation

**Goal**: 90%+ Ruchy once parser supports:
1. `vec![]` macro syntax
2. `mod` declarations for Rust interop
3. Module path separator (`::` not `.`)
4. Complex `use` statements
5. No stub generation for `std::*` types

**Then we can**:
- Rewrite `http_client.rs` in pure Ruchy
- Eliminate build.rs post-processing hacks
- Demonstrate Ruchy as viable systems language

## Performance

Expected performance identical to hand-written Rust:
- Ruchy transpiles to idiomatic Rust
- Zero runtime overhead
- Ruchy achieves 82% of C performance (fibonacci benchmark)

## Related

- **Ruchy compiler**: `../../ruchy` (v3.182.0+, 4,031 tests)
- **Parser limitations**: https://github.com/paiml/ruchy/issues/137
- **Production runtime**: `../runtime` (hand-written Rust, battle-tested)
- **Bootstrap**: `../bootstrap` (uses production runtime)
