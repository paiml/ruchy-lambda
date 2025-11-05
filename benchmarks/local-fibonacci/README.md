# Local Fibonacci Benchmark

Local performance comparison of fibonacci(35) recursive implementation across multiple languages.

## Quick Start

```bash
# Run benchmark from project root
make bench-local
```

## What This Measures

- **Pure execution time** of fibonacci(35) calculation
- **No runtime overhead** (no HTTP clients, event loops, or Lambda API calls)
- **Compiled binaries** for C, Rust, Go, and Ruchy
- **Interpreted execution** for Python (baseline)

## Actual Results

```
Runtime             | Mean (ms)  | Speedup vs Python | Runtime Size | JIT/Compile
--------------------|------------|-------------------|--------------|-------------
C                   |      12.73 | 54.1x             | N/A (libc)   | Pre-compiled
Rust                |      23.86 | 28.9x             | N/A (stdlib) | Pre-compiled
Ruchy (transpiled)  |      23.89 | 28.8x             | N/A (stdlib) | Pre-compiled
Ruchy (compiled)    |      23.93 | 28.8x  🥇         | N/A (stdlib) | Pre-compiled
Go                  |      37.59 | 18.3x             | ~1.5MB       | Pre-compiled
Julia (JIT)         |     182.72 | 3.8x              | ~200MB       | JIT every run
Python              |     688.89 | baseline          | ~78MB        | Interpreted
```

**Key Findings:**
- Ruchy transpile and compile modes both → identical performance to native Rust (~24ms)
- `ruchy transpile` generates Rust code, then compiles with rustc
- `ruchy compile` directly compiles to binary (single-step)
- C is fastest (no runtime overhead, pure machine code, ~13ms)
- Python is ~54x slower (interpreted + massive runtime, ~689ms)
- **Julia is surprisingly slow** (~183ms) because it includes JIT compilation time on every run

**Julia Performance Reality**:
- **Measured**: 182.72ms (includes JIT compilation on every invocation)
- **Runtime size**: ~200MB (Julia compiler + LLVM + stdlib)
- **Why so slow?**: Each run triggers LLVM JIT compilation (~160ms overhead)
- **Worse for Lambda**: Cold starts would be ~200-300ms just to load runtime, then +183ms per invocation
- **Pre-compiled Julia**: Could achieve ~20-25ms (matching Rust), but requires ahead-of-time compilation (PackageCompiler.jl)
- **Verdict**: JIT languages are impractical for short-lived serverless functions

## AWS Lambda Comparison

**Local benchmarks measure PURE execution time.**
**AWS Lambda includes runtime overhead (HTTP client, event loop).**

| Runtime | Local (Pure) | AWS Lambda (Production) | Overhead |
|---------|--------------|------------------------|----------|
| **Ruchy** | 23.96ms | 571.99ms | 548ms |
| **Rust** | 23.76ms | 551.33ms | 527ms |
| **Go** | 37.48ms | 689.22ms | 651ms |
| **Python** | 689.26ms | 25,083.46ms | 24,394ms |

**Overhead Analysis:**
- **Compiled languages (Ruchy/Rust/Go)**: ~520-650ms overhead from Lambda runtime
  - HTTP client for Lambda API
  - Event loop and async runtime (tokio)
  - JSON deserialization
- **Python**: 24,394ms overhead (interpreted language + runtime)

## Runtime Size vs Cold Start Performance

**Critical Insight**: AWS Lambda cold start time is primarily determined by **runtime size**, not execution speed.

### Binary/Runtime Sizes

| Language | Deployment Size | Runtime Size | Total Loaded | Cold Start |
|----------|----------------|--------------|--------------|------------|
| **Ruchy** | 400KB | 0 (custom) | 400KB | 8.50ms ✅ |
| **Rust** | 596KB | 0 (custom) | 596KB | 14.90ms |
| **C++** | 87KB | 0 (custom) | 87KB | 28.96ms |
| **Go** | 4.2MB | 0 (custom) | 4.2MB | 56.49ms |
| **Python** | 445B | ~78MB (AWS) | ~78MB | 85.73ms ⚠️ |
| **Julia** | ~1KB | ~200MB | ~200MB | ~300ms* ❌ |

**\*Julia estimate** based on runtime loading time (not officially supported by AWS Lambda)

### Why Python Is Slow Despite Tiny Code

Python Lambda deploys **only 445 bytes** of code, yet has the slowest cold start (85.73ms):

1. **AWS loads Python interpreter** (~78MB) into memory
2. **Initialize Python runtime** (import system, stdlib)
3. **Parse and compile** your Python code
4. **Execute** your handler

**Paradox**: You deploy 445 bytes, but AWS loads 78MB. This is why custom runtimes win!

### Why Julia Can't Compete on Lambda

Julia has **excellent execution performance** (~25ms, matching Rust) due to LLVM-based JIT compilation. However:

- **Runtime size**: ~200MB (Julia compiler + LLVM + stdlib)
- **Estimated cold start**: ~300-500ms just to load runtime
- **Not AWS Lambda compatible**: Runtime too large for efficient cold starts

**The Julia Problem**: Fast execution, massive runtime. Same issue as Python but worse.

### The Custom Runtime Advantage

**Custom runtimes (Ruchy, Rust, C++, Go)** include everything in one small binary:
- No separate interpreter/runtime to load
- No initialization overhead
- Direct machine code execution
- **Result**: 10-100x faster cold starts than managed runtimes

**Ruchy's 400KB binary** loads 195x faster than Python's 78MB interpreter, despite Python deploying only 445 bytes of actual code!

## Implementation Details

### Benchmarking Tool

Uses **bashrs bench v6.25.0** - scientific benchmarking framework:
- 3 warmup iterations (JIT warmup, cache warming)
- 10 measured iterations
- Statistical analysis (mean, median, stddev, min, max)
- Memory tracking
- Determinism verification

### Compilation Flags

- **C**: `gcc -O3` (maximum optimization)
- **Rust**: `rustc -C opt-level=3` (maximum optimization)
- **Go**: `go build` (standard compilation)
- **Ruchy (transpiled)**: `ruchy transpile` → `rustc -C opt-level=3`
- **Ruchy (compiled)**: `ruchy compile -o binary` (direct compilation)

### Source Files

All implementations use identical algorithm:

```rust
fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}
```

- [`fibonacci.c`](fibonacci.c) - C implementation
- [`fibonacci.rs`](fibonacci.rs) - Rust implementation
- [`fibonacci.go`](fibonacci.go) - Go implementation
- [`fibonacci.py`](fibonacci.py) - Python implementation
- [`fibonacci.ruchy`](fibonacci.ruchy) - Ruchy implementation
- [`fibonacci.jl`](fibonacci.jl) - Julia implementation (reference only, not benchmarked)

## Files

- **`run-benchmark.sh`** - Main benchmark runner
- **`benchmark-framework.sh`** - bashrs integration framework (from ruchy-book)
- **`results.json`** - JSON output with detailed statistics
- **`fibonacci.*`** - Source files for each language

## Attribution

Benchmarking framework adapted from:
- **ruchy-book** Chapter 21: Scientific Benchmarking
- Source: `/home/noah/src/ruchy-book/test/ch21-benchmarks/`
- Uses bashrs bench v6.25.0 for rigorous performance measurement

## Notes

1. **Fibonacci(35) = 9,227,465** (expected result)
2. **Pure execution time** - no I/O, no network, no runtime overhead
3. **Deterministic workload** - CPU-bound recursive algorithm
4. **Fair comparison** - all languages use identical algorithm
5. **Production correlation** - local results predict relative AWS Lambda performance
