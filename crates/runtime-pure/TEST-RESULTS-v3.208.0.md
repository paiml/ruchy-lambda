# Pure Ruchy Runtime - Test Results v3.208.0

**Date**: 2025-11-05
**Status**: ✅ All tests passing (NEW transpiler bug discovered)

---

## Summary

- ✅ **5/5 integration tests passing**
- ✅ **Binary: 325KB** (19% smaller than production)
- 🔥 **NEW BUG: Method name mangling** (`add()` → `insert()`)
- ❌ **All previous bugs still present**

---

## Critical Finding: Method Name Mangling

### NEW BUG Discovered in v3.208.0

**Symptom**: Method names change during transpilation

**Example**:
```ruchy
calc.add(5);  // Ruchy source
```

**Transpiles to**:
```rust
calc.insert(5);  // WRONG name!
```

**Error**:
```
error[E0599]: no method named `insert` found for struct `Calculator`
```

**Impact**: **Method invocations completely broken**

---

## Test Results

### Integration Tests

```bash
$ cargo test -p ruchy-lambda-runtime-pure --test integration_tests

running 5 tests
test test_runtime_can_be_created ... ok
test test_hybrid_architecture ... ok
test test_runtime_next_event ... ok
test test_runtime_post_response ... ok
test test_transpilation_quality ... ok

test result: ok. 5 passed; 0 failed; 0 ignored
```

**Result**: ✅ **5/5 PASSING**

---

### Binary Build

```bash
$ ruchy transpile bootstrap.ruchy -o bootstrap_generated.rs
$ cargo build --release -p ruchy-lambda-runtime-pure
$ rustc --edition 2021 -C opt-level=z -C lto=fat -C codegen-units=1 \
    -C strip=symbols ... bootstrap_generated.rs -o bootstrap_size_opt

$ ls -lh bootstrap_size_opt
-rwxrwxr-x 1 noah noah 325K Nov  5 14:35 bootstrap_size_opt
```

**Result**: ✅ **325KB** (same as v3.207.0, 19% smaller than production)

---

## Transpiler Bug Status

### 🔥 NEW BUG (v3.208.0)

| Bug | Example | Impact |
|-----|---------|--------|
| **Method name mangling** | `calc.add()` → `calc.insert()` | **BLOCKING** |

### ❌ STILL BROKEN (from v3.205.0/v3.207.0)

| Bug | Example | Impact |
|-----|---------|--------|
| Arithmetic → format!() | `a + b` → `format!()` | **BLOCKING** |
| Spurious .cloned() | `get()` → `get().cloned()` | **BLOCKING** |
| Functions disappear | `fun add() {}` → (missing) | **BLOCKING** |
| &self → self | `&self` → `self` | CRITICAL |
| pub fun → fn | `pub fun` → `fn` | CRITICAL |

### ⚠️ STILL PRESENT (code quality)

| Bug | Impact |
|-----|--------|
| Unnecessary braces | Fails clippy |
| Unused imports | Fails clippy |

---

## Comprehensive Test Case

**File**: `/tmp/test_v3208_comprehensive.ruchy`

```ruchy
pub struct Calculator {
    value: i32,
}

impl Calculator {
    pub fun new() -> Calculator {
        Calculator { value: 0 }
    }

    pub fun add(&mut self, amount: i32) {
        self.value = self.value + amount;
    }

    pub fun get(&self) -> i32 {
        self.value
    }
}

fun multiply(a: i32, b: i32) -> i32 {
    a * b
}

fun main() {
    let mut calc = Calculator::new();
    calc.add(5);
    println!("Value: {}", calc.get());

    let result = multiply(3, 4);
    println!("3 * 4 = {}", result);
}
```

### Transpiled Output (showing all bugs)

```rust
#[derive(Clone)]
pub struct Calculator {
    value: i32,
}
impl Calculator {
    fn new() -> Calculator {  // ❌ Lost 'pub'
        { Calculator { value: 0 } }  // ⚠️ Unnecessary braces
    }
    fn add(&mut self, amount: i32) {  // ❌ Lost 'pub'
        { self.value = format!("{}{}", self.value, & amount) }  // ❌ Arithmetic bug
    }
    fn get(&self) -> i32 {
        { self.value }  // ⚠️ Unnecessary braces
    }
}
fn main() {
    let mut calc = Calculator::new();
    calc.insert(5);  // ❌ NEW: 'add' became 'insert'!
    println!("Value: {}", calc.get().cloned());  // ❌ Spurious .cloned()
    println!("3 * 4 = {}", result);  // ❌ 'multiply' missing, 'result' undefined
}
```

### Compilation Errors

```
error[E0425]: cannot find value `result` in this scope
error[E0308]: mismatched types (expected `i32`, found `String`)
error[E0599]: no method named `insert` found for struct `Calculator`
error[E0599]: `i32` is not an iterator
```

**Result**: 4 compilation errors

---

## Why ruchy-lambda Still Works

### Lucky Code Subset

Our code avoids **ALL broken features**:

```ruchy
// ✅ WORKS: String concatenation (not arithmetic)
let path = String::from("/runtime/invocation/") + request_id + "/response";

// ✅ WORKS: Direct method calls (no chaining)
let result = http_client::http_get(&endpoint, &path);

// ✅ WORKS: Simple struct methods (no standalone functions)
impl Runtime {
    pub fun next_event(&self) -> (String, String) {
        // Only String operations, no math
    }
}
```

### What We Avoid

```ruchy
// ❌ Would break: Arithmetic
// let count = count + 1;

// ❌ Would break: Method chaining on primitives
// println!("{}", obj.get());

// ❌ Would break: Standalone functions
// fun helper(x: i32) -> i32 { x * 2 }

// ❌ Would break: Method names get mangled
// calc.add(5);  // Becomes calc.insert(5)
```

---

## Binary Size Comparison

| Configuration | Binary Size | vs Production | Change from v3.207.0 |
|---------------|-------------|---------------|----------------------|
| v3.208.0 (opt-level=z) | 325KB | -19% | No change |
| v3.207.0 (opt-level=z) | 325KB | -19% | - |
| Production (Rust) | 400KB | baseline | - |

**Conclusion**: Binary size **unchanged** from v3.207.0

---

## Regression Analysis

**v3.207.0 → v3.208.0**:

### ❌ NEW REGRESSIONS

1. **Method name mangling** - `add()` → `insert()` (BLOCKING)

### ✅ NO CHANGES

- All tests still passing (5/5)
- Binary size identical (325KB)
- All previous bugs still present

### 📊 Transpiler Status

**v3.205.0**: 5 BLOCKING bugs
**v3.207.0**: 5 BLOCKING bugs
**v3.208.0**: **6 BLOCKING bugs** (method name mangling added)

**Trend**: 🔴 **DEGRADING** - Getting worse, not better

---

## Recommendations

### Immediate Actions

1. ⚠️ **DO NOT UPGRADE** Ruchy compiler if avoidable
   - v3.208.0 has MORE bugs than v3.207.0
   - Method name mangling makes transpiler unusable

2. 🔥 **UPDATE DOCUMENTATION** - Add method name mangling to limitations

3. ❌ **DO NOT USE** transpiler for any new code

### For ruchy-lambda

- ✅ **v3.208.0 is safe** - Tests passing, binary size good
- ⚠️ **Stay within working subset** - Don't expand Ruchy usage
- 🔴 **Transpiler degrading** - Wait for compiler fixes before expanding

### For Ruchy Compiler Team

**URGENT**: Fix BLOCKING bugs before adding features
- 🔥 Method name mangling (NEW in v3.208.0)
- 🔥 Arithmetic → format!()
- 🔥 Spurious .cloned()
- 🔥 Functions disappear

**Current state**: Transpiler getting WORSE with each version

---

## GitHub Issues

- **#137**: Updated with v3.208.0 findings
  - NEW BUG: Method name mangling
  - All previous bugs confirmed
  - Test case: `/tmp/test_v3208_comprehensive.ruchy`

- **#138**: Profiling tools request (not affected)

---

## Conclusion

**v3.208.0 Status**: ⚠️ **DEGRADED**

- ✅ Tests still passing (5/5)
- ✅ Binary size unchanged (325KB)
- 🔴 **NEW BUG** discovered (method name mangling)
- 🔴 **Transpiler quality declining**

**Recommendation**:
- ✅ Safe to use for ruchy-lambda (we avoid broken features)
- ❌ NOT SAFE for general Rust code
- ⚠️ Monitor transpiler quality closely

---

**Tested by**: Noah (ruchy-lambda maintainer)
**Environment**: Ruchy v3.182.0+, ruchy-lambda v3.208.0
**Date**: 2025-11-05
