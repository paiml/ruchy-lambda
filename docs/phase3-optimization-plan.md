# Phase 3 Optimization Plan: Binary Size Reduction

## Current State (Baseline)

**Binary Size**: 505KB (stripped release build)

**Breakdown** (from cargo bloat):
- std library: 280KB (55%)
- tokio async runtime: 77KB (15%)
- our code: 11KB (2%)
- other (serde, mio, etc): 137KB (27%)

**Target**: <100KB (5x reduction needed)

## Analysis

### Why Tokio Is Unnecessary

Lambda Runtime API is **inherently synchronous**:

```
Loop forever:
  1. GET /runtime/invocation/next  (BLOCKS until event)
  2. handler(event)                (synchronous processing)
  3. POST /runtime/invocation/{id}/response
  4. Repeat
```

**Key Insight**: We process ONE event at a time. No concurrency needed!

### Async vs Blocking Comparison

| Approach | Binary Size | Complexity | Performance |
|----------|-------------|------------|-------------|
| **Current (tokio)** | 505KB | High (async runtime) | Excellent |
| **Proposed (blocking)** | ~200-250KB | Low (std only) | Identical |

**Lambda invocation pattern:**
- Single-threaded
- One event at a time
- No concurrent I/O
- Long-polling GET is naturally blocking

**Conclusion**: Async adds 77KB+ overhead with ZERO benefit for Lambda's use case.

## Optimization Strategy

### Option 1: Remove Tokio (Blocking I/O) ⭐ **RECOMMENDED**

**Changes:**
1. Replace `tokio::net::TcpStream` with `std::net::TcpStream`
2. Change `async fn` to `fn` (remove async/await)
3. Replace `tokio::main` with regular `main()`
4. Keep HTTP client logic identical (just blocking)

**Expected Savings:**
- tokio: -77KB
- Reduced std usage: -50KB
- **Total: ~127KB saved → ~378KB binary**

**Pros:**
- ✅ Largest size reduction
- ✅ Simpler code (no async complexity)
- ✅ Identical performance (Lambda is synchronous)
- ✅ Easier to debug (no async stack traces)

**Cons:**
- ❌ Lose async for future features (but Lambda doesn't need it)

### Option 2: Replace Tokio with Smol

**Expected Savings:** ~50KB (tokio 77KB → smol 20KB)

**Pros:**
- ✅ Keep async capability
- ✅ Smaller async runtime

**Cons:**
- ❌ Still overhead for unused async
- ❌ Only 50KB savings vs 127KB for blocking

### Option 3: Keep Tokio, Optimize Features

**Expected Savings:** ~20-30KB

**Pros:**
- ✅ No code changes

**Cons:**
- ❌ Minimal savings
- ❌ Won't reach <100KB target

## Decision: Option 1 (Blocking I/O)

**Rationale:**
1. Lambda processes one event at a time → no concurrency needed
2. Largest size reduction (127KB+)
3. Simpler codebase
4. Identical performance

## Implementation Plan

### Step 1: Create Blocking HTTP Client

Replace `crates/runtime/src/http_client.rs`:

```rust
// Before (async with tokio):
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct HttpClient { /* ... */ }

impl HttpClient {
    pub async fn get(&self, path: &str) -> Result<String, HttpError> {
        let mut stream = TcpStream::connect(&self.endpoint).await?;
        // ...
    }
}
```

```rust
// After (blocking with std):
use std::net::TcpStream;
use std::io::{Read, Write};

pub struct HttpClient { /* ... */ }

impl HttpClient {
    pub fn get(&self, path: &str) -> Result<String, HttpError> {
        let mut stream = TcpStream::connect(&self.endpoint)?;
        // ...
    }
}
```

### Step 2: Update Runtime API

`crates/runtime/src/lib.rs`:

```rust
// Before:
pub async fn next_event(&self) -> Result<String> { /* ... */ }

// After:
pub fn next_event(&self) -> Result<String> { /* ... */ }
```

### Step 3: Update Bootstrap

`crates/bootstrap/src/main.rs`:

```rust
// Before:
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new()?;
    loop {
        let event = runtime.next_event().await?;
        // ...
    }
}

// After:
fn main() -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new()?;
    loop {
        let event = runtime.next_event()?;
        // ...
    }
}
```

### Step 4: Remove Tokio Dependency

`Cargo.toml`:

```toml
# Remove:
tokio = { version = "1.40", features = ["rt", "rt-multi-thread", "macros", "net", "io-util"] }
```

### Step 5: Update Tests

All test functions: `async fn` → `fn`

## Expected Results

**Before:**
- Binary: 505KB
- Dependencies: tokio, mio, serde, once_cell

**After:**
- Binary: ~200-250KB (2.5x reduction)
- Dependencies: serde, once_cell (minimal)

**Still short of <100KB target**, but major progress. Further optimizations:
- Phase 3b: Optimize serde features
- Phase 3c: LTO (Link-Time Optimization)
- Phase 3d: Strip std library further

## Risks

1. **Tests might need updates** - All async test functions become blocking
2. **OnceCell usage** - Verify still works without tokio
3. **Error handling** - I/O errors slightly different (no .await)

## Validation

**Success Criteria:**
- [ ] Binary size <250KB (intermediate target)
- [ ] All 65 tests passing
- [ ] Performance unchanged (<8ms cold start)
- [ ] PMAT quality gates pass
- [ ] Mock server tests work (blocking I/O)

## Next Steps

1. Implement blocking HTTP client
2. Update runtime to remove async
3. Update bootstrap main()
4. Run tests (expect failures, then fix)
5. Measure binary size
6. If <250KB, continue to <100KB optimizations
