# Pure Ruchy Runtime - Test Results v3.207.0

**Date**: 2025-11-05
**Status**: ✅ All tests passing (with documented transpiler limitations)

---

## Summary

- ✅ **5/5 integration tests passing**
- ✅ **Library builds successfully** (2 warnings)
- ✅ **Standalone binaries build successfully** (325KB-338KB)
- ⚠️ **Transpiler bugs remain** (arithmetic, cloning, functions)
- ⚠️ **ruchy compile limitation remains** (cannot link external crates)

---

## Build Tests

### Integration Tests

```bash
$ cargo test -p ruchy-lambda-runtime-pure --test integration_tests

running 5 tests
test test_runtime_can_be_created ... ok
test test_hybrid_architecture ... ok
test test_runtime_next_event ... ok
test test_runtime_post_response ... ok
test test_transpilation_quality ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured
```

**Result**: ✅ **5/5 PASSING**

---

### Library Build

```bash
$ cargo build --release -p ruchy-lambda-runtime-pure

Compiling ruchy-lambda-runtime-pure v3.207.0
warning: unused import: `self`
warning: unnecessary braces around block return value
Finished `release` profile [optimized] target(s) in 0.54s
```

**Result**: ✅ **Builds successfully** (2 warnings from transpiler)

---

### Standalone Binary Builds

```bash
$ cd crates/runtime-pure/examples
$ ruchy transpile bootstrap.ruchy -o bootstrap_generated.rs
$ cargo build --release -p ruchy-lambda-runtime-pure

# Speed optimization (opt-level=3)
$ rustc --edition 2021 -C opt-level=3 -C lto=fat -C codegen-units=1 -C strip=symbols \
  -L ../../../target/release/deps \
  --extern ruchy_lambda_runtime_pure=../../../target/release/libruchy_lambda_runtime_pure.rlib \
  bootstrap_generated.rs -o bootstrap_optimized

$ ls -lh bootstrap_optimized
-rwxrwxr-x 1 noah noah 338K Nov  5 13:08 bootstrap_optimized

# Size optimization (opt-level=z)
$ rustc --edition 2021 -C opt-level=z -C lto=fat -C codegen-units=1 -C strip=symbols \
  -L ../../../target/release/deps \
  --extern ruchy_lambda_runtime_pure=../../../target/release/libruchy_lambda_runtime_pure.rlib \
  bootstrap_generated.rs -o bootstrap_size_opt

$ ls -lh bootstrap_size_opt
-rwxrwxr-x 1 noah noah 325K Nov  5 13:08 bootstrap_size_opt
```

**Result**: ✅ **Both binaries build successfully**

---

## Binary Size Comparison

| Configuration | Binary Size | vs Production | Optimization |
|---------------|-------------|---------------|--------------|
| **Size optimized (opt-level=z)** | **325KB** | **-19%** | Best for Lambda |
| Speed optimized (opt-level=3) | 338KB | -15% | Balanced |
| **Production (hand-written Rust)** | **400KB** | baseline | Current |

**Best result**: **325KB with opt-level=z** (19% smaller than production!)

**Estimated cold start improvement**:
```
325KB ÷ 133 MB/s = 2.44ms (binary load)
+ ~5.5ms (runtime init)
= ~7.94ms estimated cold start

vs 400KB production: 8.50ms
Improvement: 0.56ms (6.6% faster)
```

---

## Transpiler Bug Verification

### Bug #1: Arithmetic → String Concatenation ✅ CONFIRMED

**Test**: `/tmp/test_v3207_math.ruchy`
```ruchy
impl Calculator {
    pub fun add(&mut self, amount: i32) {
        self.value = self.value + amount;
    }
}
```

**Transpiled**:
```rust
fn add(&mut self, amount: i32) {
    { self.value = format!("{}{}", self.value, & amount) }
}
```

**Compilation error**:
```
error[E0308]: mismatched types
expected `i32`, found `String`
```

**Status**: ❌ **STILL BROKEN** - Cannot do arithmetic in Ruchy

---

### Bug #2: Spurious `.cloned()` Calls ✅ CONFIRMED

**Test**: `/tmp/test_v3207_math.ruchy`
```ruchy
println!("Result: {}", calc.get());  // get() returns i32
```

**Transpiled**:
```rust
println!("Result: {}", calc.get().cloned());  // i32 has no .cloned()!
```

**Compilation error**:
```
error[E0599]: `i32` is not an iterator
```

**Status**: ❌ **STILL BROKEN** - Cannot call methods that return primitives

---

### Bug #3: Functions Disappear ✅ CONFIRMED

**Test**: `/tmp/test_v3207_function.ruchy`
```ruchy
fun square(n: i32) -> i32 {
    n * n
}

fun main() {
    let result = square(5);
    println!("5 squared = {}", result);
}
```

**Transpiled**:
```rust
fn main() {
    println!("5 squared = {}", result)  // Where's square()?
}
```

**Status**: ❌ **STILL BROKEN** - Standalone functions not generated

---

### Bug #4: Unnecessary Braces ✅ CONFIRMED

**Test**: `/tmp/test_v3207_basic.ruchy`
```ruchy
impl Point {
    pub fun get_x(&self) -> i32 {
        self.x
    }
}
```

**Transpiled**:
```rust
impl Point {
    fn get_x(&self) -> i32 {
        { self.x }  // Unnecessary braces
    }
}
```

**Warnings**: 3 warnings (unnecessary braces)

**Status**: ⚠️ **STILL PRESENT** - Code quality issue

---

### Bug #5: `pub fun` → `fn` ✅ CONFIRMED

**Test**: `/tmp/test_v3207_basic.ruchy`
```ruchy
pub fun new(x_val: i32, y_val: i32) -> Point { ... }
```

**Transpiled**:
```rust
fn new(x_val: i32, y_val: i32) -> Point { ... }  // Missing `pub`
```

**Status**: ⚠️ **STILL PRESENT** - Visibility loss

---

## ruchy compile Limitation

```bash
$ ruchy compile bootstrap.ruchy -o bootstrap

✗ Compilation failed:
error[E0432]: unresolved import `ruchy_lambda_runtime_pure`
use of unresolved module or unlinked crate `ruchy_lambda_runtime_pure`
```

**Status**: ❌ **STILL BROKEN** - Cannot link external crates

**Workaround**: Use `ruchy transpile` + `rustc` (as documented)

---

## What Works in v3.207.0

### ✅ Successfully Working

1. **Library build** - Transpiles and compiles correctly
2. **Integration tests** - All 5 tests passing
3. **Standalone binaries** - Can build with `ruchy transpile` + `rustc`
4. **Binary size** - 325KB (19% smaller than production!)
5. **String operations** - Concatenation works correctly
6. **Struct/impl** - Basic struct and impl blocks work
7. **Control flow** - if/else, loops work

### ⚠️ Known Limitations

1. **Arithmetic operations** - BROKEN (`+` becomes string concat)
2. **Method chaining on primitives** - BROKEN (spurious `.cloned()`)
3. **Standalone functions** - BROKEN (not generated)
4. **`pub` visibility** - LOST (pub fun → fn)
5. **Code quality** - Unnecessary braces (fails strict clippy)
6. **ruchy compile** - Cannot link external crates

### 💡 Why ruchy-lambda Works

Our code **only uses working features**:
- ✅ String concatenation (not arithmetic)
- ✅ Struct methods (no standalone functions)
- ✅ Direct returns (no method chaining on primitives)
- ✅ Build.rs post-processing (fixes transpiler bugs)

---

## Regression Analysis

**Changes from v3.205.0 → v3.207.0**: None detected

- ✅ All tests still passing
- ❌ All bugs still present
- ✅ Binary sizes identical (325KB/338KB)
- ✅ Build process unchanged

**Conclusion**: No regressions, no improvements

---

## Recommendations

### For ruchy-lambda v3.207.0

- ✅ **Ship as-is** - All tests passing, smaller binary achieved
- ✅ **Document limitations** - TRANSPILER-LIMITATIONS.md complete
- ⚠️ **Don't expand Ruchy usage** - Stay within working subset
- ⚠️ **Wait for transpiler fixes** - Block on Ruchy compiler improvements

### For Future Versions

**When transpiler bugs fixed**:
1. Remove build.rs post-processing hacks
2. Expand Ruchy usage to 90%+ (from current 40%)
3. Rewrite HTTP client in pure Ruchy
4. Pass strict quality gates (`clippy -D warnings`)

**Priorities for Ruchy compiler team** (GitHub #137):
1. 🔥 Fix arithmetic bug (BLOCKING)
2. 🔥 Fix spurious `.cloned()` (BLOCKING)
3. 🔥 Fix function generation (BLOCKING)
4. ⚠️ Preserve `&self` references
5. ⚠️ Preserve `pub` visibility
6. ⚠️ Remove unnecessary braces

---

## GitHub Issues

- **#137**: Parser limitations & transpiler bugs
  - Latest update: All 3 BLOCKING bugs confirmed in v3.207.0
  - Test cases provided: `/tmp/test_v3207_*.ruchy`

- **#138**: Profiling tools (ruchy profile --binary, ruchy analyze)
  - Includes `ruchy compile` enhancements

---

## Test Files

All test files available for reproduction:

- `/tmp/test_v3207_basic.ruchy` - Basic struct/impl (WORKS with warnings)
- `/tmp/test_v3207_math.ruchy` - Arithmetic bug (FAILS)
- `/tmp/test_v3207_function.ruchy` - Function disappears (FAILS)
- `crates/runtime-pure/examples/bootstrap.ruchy` - Production code (WORKS)

---

## Conclusion

**v3.207.0 Status**: ✅ **STABLE**

- All tests passing (5/5)
- Binary size optimized (325KB, 19% smaller)
- Transpiler bugs documented and tracked
- Workarounds in place and functioning

**Ready for**: Continued development within documented constraints
**Blocked by**: Ruchy transpiler bugs (arithmetic, cloning, functions)

---

**Tested by**: Noah (ruchy-lambda maintainer)
**Environment**: Ruchy v3.182.0+, ruchy-lambda v3.207.0
**Date**: 2025-11-05
