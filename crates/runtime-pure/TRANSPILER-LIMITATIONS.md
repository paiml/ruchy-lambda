# Ruchy Transpiler Limitations - Critical Assessment

**Version**: v3.207.0 (Ruchy v3.182.0+)
**Date**: 2025-11-05
**Status**: ⚠️ **DO NOT USE for general Rust code**

---

## Executive Summary

The Ruchy transpiler has **severe code generation bugs** that make it unusable for most Rust code:

- ❌ **Arithmetic broken**: `a + b` transpiles to string concatenation
- ❌ **Methods broken**: Spurious `.cloned()` calls on primitives
- ❌ **Functions broken**: Standalone functions disappear from output

**ruchy-lambda works ONLY because it avoids all broken features.**

---

## 🔥 Critical Bugs

### Bug #1: Integer Arithmetic → String Concatenation

**Symptom**: All `+` operations transpile to `format!()` string concatenation.

**Example**:
```ruchy
// Input
self.count = self.count + 1;

// Output (WRONG)
self.count = format!("{}{}", self.count, 1)  // String concat!
```

**Error**:
```
error[E0308]: mismatched types
expected `i32`, found `String`
```

**Impact**:
- Cannot increment counters
- Cannot do any arithmetic (addition, subtraction, multiplication, division)
- Math operations completely broken

---

### Bug #2: Spurious `.cloned()` Calls

**Symptom**: Method calls on primitives get `.cloned()` added.

**Example**:
```ruchy
// Input
println!("{}", counter.get());  // get() returns i32

// Output (WRONG)
println!("{}", counter.get().cloned());  // i32 doesn't have .cloned()!
```

**Error**:
```
error[E0599]: `i32` is not an iterator
```

**Impact**:
- Cannot call methods that return `i32`, `bool`, `f64`, etc.
- Method chaining completely broken
- Printing method results fails

---

### Bug #3: Functions Disappear

**Symptom**: Standalone functions are not generated in output.

**Example**:
```ruchy
// Input
fun add(a: i32, b: i32) -> i32 {
    a + b
}

fun main() {
    let result = add(2, 3);
    println!("{}", result);
}

// Output (WRONG)
fn main() {
    println!("{}", result)  // Where's add()?
}
```

**Impact**:
- Cannot define helper functions
- Only `impl` methods work
- Must put all code in struct methods

---

## Why ruchy-lambda Still Works

Our code **carefully avoids all broken features**:

### ✅ What We Use (Works)

```ruchy
// String concatenation (transpiles correctly to format!())
let path = String::from("/runtime/invocation/") + request_id + "/response";

// Struct methods (no standalone functions)
impl Runtime {
    pub fun next_event(&self) -> (String, String) {
        // ...
    }
}

// String operations only (no arithmetic)
let endpoint = String::from("127.0.0.1:9001");

// Direct method calls (no chaining on primitives)
let result = http_client::http_get(&self.api_endpoint, &path);
if result.is_ok() {
    result.unwrap()
}
```

### ❌ What We Avoid (Broken)

```ruchy
// ❌ Integer arithmetic
// let count = count + 1;  // Would break

// ❌ Method chaining on primitives
// println!("{}", obj.get_count());  // Would add spurious .cloned()

// ❌ Standalone functions
// fun helper(x: i32) -> i32 { x * 2 }  // Would disappear

// ❌ Math operations
// let total = price * quantity + tax;  // Would break
```

---

## Test Cases

### Test 1: Counter (All Bugs Triggered)

**File**: `/tmp/test_v3205.ruchy`
```ruchy
pub struct Counter {
    count: i32,
}

impl Counter {
    pub fun new() -> Counter {
        Counter { count: 0 }
    }

    pub fun increment(&mut self) {
        self.count = self.count + 1;  // Bug #1: Arithmetic
    }

    pub fun get(&self) -> i32 {
        self.count
    }
}

fun main() {
    let mut counter = Counter::new();
    println!("Count: {}", counter.get());  // Bug #2: .cloned()
    counter.increment();
}
```

**Result**: 4 compilation errors

---

### Test 2: Simple Function (Bug #3)

**File**: `/tmp/test_addition.ruchy`
```ruchy
fun add(a: i32, b: i32) -> i32 {
    a + b
}

fun main() {
    let result = add(2, 3);
    println!("{}", result);
}
```

**Result**: `add()` function disappears, `result` undefined

---

## Safe Usage Pattern

**Only use Ruchy transpiler for**:

1. **Struct definitions** with methods (no standalone functions)
2. **String operations** only (no arithmetic)
3. **Direct method calls** (no chaining on primitives)
4. **Simple control flow** (if/else, loops)

**Example safe code**:
```ruchy
pub struct Logger {
    prefix: String,
}

impl Logger {
    pub fun new() -> Logger {
        Logger {
            prefix: String::from("[LOG]"),
        }
    }

    pub fun log(&self, message: &str) {
        // String concatenation works
        let full_message = self.prefix.clone() + " " + message;
        println!("{}", full_message);
    }
}
```

---

## Complete Bug List

### 🔥 BLOCKING (Makes transpiler unusable)

| Bug | Example | Impact |
|-----|---------|--------|
| Integer `+` → `format!()` | `a + 1` | Cannot do math |
| Spurious `.cloned()` | `obj.get()` | Cannot call methods |
| Functions disappear | `fun add() {}` | No helper functions |

### ❌ CRITICAL (Blocks most code)

| Bug | Example | Impact |
|-----|---------|--------|
| `&self` → `self` | `pub fun get(&self)` | Ownership errors |
| `pub fun` → `fn` | `pub fun new()` | Visibility loss |
| `vec![0u8; 1024]` | Buffer allocation | Cannot create arrays |
| `mod http_client;` | Module imports | Cannot organize code |
| `foo::bar()` → `foo.bar()` | Path separator | Cannot call Rust fns |

### ⚠️ MEDIUM (Code quality issues)

| Bug | Example | Impact |
|-----|---------|--------|
| Unnecessary braces | `{ return x }` | Fails clippy |
| Unused imports | `use std::io::{self, ...}` | Fails clippy |

---

## Workarounds

### For ruchy-lambda

1. **Avoid arithmetic** - Use Rust helper functions for math
2. **Avoid method chaining** - Store intermediate results
3. **Avoid standalone functions** - Put everything in `impl` blocks
4. **Use build.rs** - Post-process generated code to fix bugs

### For other projects

**Don't use Ruchy transpiler until bugs are fixed.**

Use Rust directly instead.

---

## What Needs to Be Fixed

**Priority 1** (BLOCKING):
1. Fix integer arithmetic (`+`, `-`, `*`, `/` should not use `format!()`)
2. Remove spurious `.cloned()` calls
3. Generate standalone functions

**Priority 2** (CRITICAL):
4. Preserve `&self` references
5. Preserve `pub` visibility
6. Support `vec![]` syntax
7. Support `mod` declarations
8. Fix `::` path separator

**Priority 3** (MEDIUM):
9. Remove unnecessary braces
10. Don't generate unused imports

---

## Binary Size Achievement

Despite all bugs, **ruchy-lambda produces smaller binaries than pure Rust**:

| Binary | Size | Composition |
|--------|------|-------------|
| **Pure Ruchy** (opt-level=z) | **325KB** | 40% Ruchy, 60% Rust |
| Pure Ruchy (opt-level=3) | 338KB | 40% Ruchy, 60% Rust |
| **Production Rust** | **400KB** | 100% Rust |

**Result**: 19% smaller with size optimization!

**Why?** Ruchy generates minimal code (only API layer), Rust fills in complex parts.

---

## Recommendation

**For ruchy-lambda**: Continue hybrid approach (40% Ruchy, 60% Rust)
- Works because we avoid all broken features
- Achieves smaller binary size
- All tests passing (5/5)

**For general Rust code**: **DO NOT USE Ruchy transpiler**
- Arithmetic broken
- Method chaining broken
- Standalone functions broken
- Use Rust directly instead

**For Ruchy maintainers**: Fix BLOCKING bugs before promoting transpiler
- Current state: Only suitable for toy examples
- Real-world code requires arithmetic, functions, method chaining

---

**Reported**: GitHub Issue #137 (https://github.com/paiml/ruchy/issues/137)
**Tested with**: Ruchy v3.182.0+, ruchy-lambda v3.205.0
**Contact**: Noah (ruchy-lambda maintainer)
