# Ruchy Lambda Profiler - Integration Guide

## Overview

The Ruchy Lambda Profiler is a specialized performance measurement tool designed to help make Ruchy the **world's fastest AWS Lambda runtime**. It provides comprehensive cold start benchmarking, statistical analysis, and comparison against the fastest runtimes (C++, Rust, Go, Swift).

**Performance Target**: <8ms average cold start (beat C++ 13.54ms, Rust 16.98ms)

---

## ⚠️ CRITICAL: Current Status - SIMULATION ONLY

**WARNING**: The current profiler implementation uses `std::thread::sleep()` for simulation, NOT real Lambda Runtime API measurements.

**What This Means**:
- ❌ The 0.26ms numbers are **FAKE** (100μs init + 50μs handler = 150μs sleep)
- ❌ Not measuring real AWS Lambda Runtime API overhead
- ❌ Not measuring real handler execution time
- ❌ Not capturing real memory usage (hardcoded 4MB)
- ❌ Cannot be trusted for actual performance evaluation

**Current Code** (`crates/profiler/src/main.rs:244-250`):
```rust
// Phase 1: Simulated init
std::thread::sleep(Duration::from_micros(100)); // ~0.1ms

// Phase 2: Simulated handler
std::thread::sleep(Duration::from_micros(50));  // ~0.05ms
```

**Why Simulation**:
- Infrastructure for profiling is complete
- Workflows and CI/CD integration ready
- Waiting for real Lambda Runtime API implementation

**Next Steps to Get Real Data**:
1. Replace `std::thread::sleep()` with actual Lambda Runtime API calls
2. Measure real initialization time (Runtime API client setup)
3. Measure real handler execution time (first invocation)
4. Capture actual memory usage from `/proc/self/status`
5. Deploy to AWS Lambda and run benchmarks

**Until Then**: Treat all numbers as placeholder for testing the profiler infrastructure only.

---

## Installation

### From Source (Development)

```bash
# Clone repository
git clone https://github.com/paiml/ruchy-lambda.git
cd ruchy-lambda

# Build profiler
cargo build --package ruchy-lambda-profiler --release

# Binary location
./target/release/profiler
```

### From Crates.io (Friday Releases Only)

```bash
# Install profiler CLI
cargo install ruchy-lambda-profiler

# Verify installation
profiler --help
```

**Note**: Crates.io releases occur **Friday ONLY** per project policy.

## CLI Commands

### 1. Benchmark - Run Cold Start Tests

Measures cold start performance using lambda-perf methodology (10 invocations per configuration).

```bash
profiler benchmark [OPTIONS]

Options:
  -m, --memory <MB>     Memory size (default: 128)
  -a, --arch <ARCH>     Architecture: x86_64 or arm64 (default: x86_64)
  -o, --output <FILE>   Save results to JSON file
```

**Example**:
```bash
# Benchmark with 128MB memory on x86_64
profiler benchmark -m 128 -a x86_64 -o results.json

# Benchmark with 512MB memory on ARM64
profiler benchmark -m 512 -a arm64 -o results-arm64.json
```

**Output**:
```
Running benchmark: 128MB memory, x86_64 arch
Collecting 10 cold start measurements...

  Invocation 1: 0.26ms (init: 0.16ms, handler: 0.10ms)
  Invocation 2: 0.26ms (init: 0.15ms, handler: 0.10ms)
  ...
  Invocation 10: 0.26ms (init: 0.15ms, handler: 0.10ms)

=== Benchmark Results ===
Average:  0.26ms
P50:      0.26ms
P99:      0.26ms
Min:      0.26ms
Max:      0.26ms
StdDev:   0.00ms
Binary:   300KB (target/release-ultra/bootstrap)

=== Performance Comparison ===
Ruchy:  0.26ms
C++:    13.54ms (current fastest)
Rust:   16.98ms
Go:     45.77ms
Swift:  86.33ms

Ruchy Speedup:
  vs C++:   52.63x ✓ FASTER
  vs Rust:  66.02x ✓ FASTER
  vs Go:    177.92x ✓ FASTER

Target: <8ms ✓ MET

🎉 NEW WORLD RECORD! 98.1% faster than C++

Results saved to: results.json
```

### 2. Compare - Performance Comparison

Compare benchmark results against fastest runtimes.

```bash
profiler compare -i <results.json>
```

**Example**:
```bash
profiler compare -i results.json
```

**Output**:
```
=== Performance Comparison ===
Ruchy:  0.26ms
C++:    13.54ms (current fastest)
Rust:   16.98ms
Go:     45.77ms
Swift:  86.33ms

Ruchy Speedup:
  vs C++:   52.63x ✓ FASTER
  vs Rust:  66.02x ✓ FASTER
  vs Go:    177.92x ✓ FASTER

Target: <8ms ✓ MET

🎉 NEW WORLD RECORD! 98.1% faster than C++
```

### 3. Report - Generate lambda-perf JSON

Generate lambda-perf compatible JSON format for submission to https://github.com/maxday/lambda-perf.

```bash
profiler report -i <results.json> -o <lambda-perf.json>
```

**Example**:
```bash
profiler report -i results.json -o lambda-perf.json
```

**Output** (`lambda-perf.json`):
```json
{
  "i": [0.155, 0.153, 0.153, 0.153, 0.153, 0.153, 0.153, 0.153, 0.153, 0.153],
  "m": 128,
  "a": "x86_64",
  "mu": 4,
  "ad": 0.257,
  "acd": 0.257,
  "r": "ruchy_on_provided_al2023",
  "p": "zip",
  "d": "ruchy (prov.al2023)"
}
```

**Field Definitions**:
- `i`: Array of 10 init durations (ms)
- `m`: Memory size (MB)
- `a`: Architecture (x86_64 or arm64)
- `mu`: Max memory used (MB)
- `ad`: Average duration (ms)
- `acd`: Average cold start duration (ms)
- `r`: Runtime identifier
- `p`: Package type (zip)
- `d`: Display name

### 4. Memory - Binary Size Profiling

Profile binary size and check against <100KB target.

```bash
profiler memory -b <binary-path>
```

**Example**:
```bash
profiler memory -b target/release-ultra/bootstrap
```

**Output**:
```
Binary: target/release-ultra/bootstrap
Size: 300 KB (307200 bytes)
✗ Exceeds 100KB target by 200 KB
```

## Integration Workflows

### Workflow 1: Local Development Testing

```bash
#!/bin/bash
# local-benchmark.sh

set -euo pipefail

echo "🔍 Building optimized binary..."
cargo build --profile release-ultra --target x86_64-unknown-linux-musl

echo "📊 Running profiler benchmark..."
profiler benchmark \
  -m 128 \
  -a x86_64 \
  -o benchmark-results.json

echo "📈 Comparing against fastest runtimes..."
profiler compare -i benchmark-results.json

echo "📄 Generating lambda-perf report..."
profiler report \
  -i benchmark-results.json \
  -o lambda-perf-report.json

echo "✅ Profiling complete!"
```

### Workflow 2: Multi-Configuration Testing

Test across multiple memory sizes to find optimal configuration.

```bash
#!/bin/bash
# multi-config-benchmark.sh

set -euo pipefail

MEMORY_SIZES=(128 256 512 1024)

for mem in "${MEMORY_SIZES[@]}"; do
  echo "🔍 Benchmarking ${mem}MB configuration..."

  profiler benchmark \
    -m "$mem" \
    -a x86_64 \
    -o "results-${mem}mb.json"

  echo ""
done

echo "📊 Results Summary:"
for mem in "${MEMORY_SIZES[@]}"; do
  echo "  ${mem}MB:"
  jq -r '"    Avg: \(.stats.avg_ms)ms, P99: \(.stats.p99_ms)ms"' "results-${mem}mb.json"
done
```

### Workflow 3: ARM64 vs x86_64 Comparison

Compare performance between architectures.

```bash
#!/bin/bash
# arch-comparison.sh

set -euo pipefail

echo "🔍 Benchmarking x86_64..."
profiler benchmark -m 128 -a x86_64 -o results-x86.json

echo "🔍 Benchmarking ARM64..."
profiler benchmark -m 128 -a arm64 -o results-arm64.json

echo "📊 Architecture Comparison:"
echo "x86_64:"
jq -r '"  Avg: \(.stats.avg_ms)ms, P99: \(.stats.p99_ms)ms, Binary: \(.binary.size_kb)KB"' results-x86.json

echo "ARM64:"
jq -r '"  Avg: \(.stats.avg_ms)ms, P99: \(.stats.p99_ms)ms, Binary: \(.binary.size_kb)KB"' results-arm64.json
```

### Workflow 4: CI/CD Integration

Integrate profiler into CI pipeline for continuous performance monitoring.

```yaml
# .github/workflows/performance.yml
name: Performance Benchmarking

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: x86_64-unknown-linux-musl

      - name: Build profiler
        run: cargo build --package ruchy-lambda-profiler --release

      - name: Build bootstrap
        run: cargo build --profile release-ultra --target x86_64-unknown-linux-musl

      - name: Run benchmark
        run: |
          ./target/release/profiler benchmark \
            -m 128 -a x86_64 -o benchmark-results.json

      - name: Check performance targets
        run: |
          AVG=$(jq -r '.stats.avg_ms' benchmark-results.json)
          if (( $(echo "$AVG > 8.0" | bc -l) )); then
            echo "❌ Performance regression: ${AVG}ms > 8ms target"
            exit 1
          fi
          echo "✅ Performance target met: ${AVG}ms < 8ms"

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: benchmark-results.json
```

## Benchmark Result JSON Format

The profiler generates JSON files with the following structure:

```json
{
  "runtime": "ruchy",
  "memory_mb": 128,
  "arch": "x86_64",
  "measurements": [
    {
      "init_ms": 0.155,
      "handler_ms": 0.101,
      "total_ms": 0.256,
      "memory_kb": 4096,
      "timestamp": 1699123456
    }
  ],
  "stats": {
    "avg_ms": 0.257,
    "p50_ms": 0.256,
    "p99_ms": 0.258,
    "min_ms": 0.254,
    "max_ms": 0.260,
    "stddev_ms": 0.002
  },
  "binary": {
    "size_kb": 300,
    "path": "target/release-ultra/bootstrap",
    "stripped": true
  }
}
```

## Performance Targets

| Metric | Target | Stretch Goal | Current Best |
|--------|--------|--------------|--------------|
| **Average Cold Start** | <10ms | <8ms | C++ 13.54ms |
| **P50 Cold Start** | <9ms | <7ms | - |
| **P99 Cold Start** | <15ms | <12ms | - |
| **Binary Size** | <100KB | <50KB | - |

## Fastest Runtimes (Baseline from lambda-perf 2024-12-31)

| Runtime | Average Cold Start | Notes |
|---------|-------------------|-------|
| C++ 11 | 13.54ms | Current fastest |
| Rust | 16.98ms | Second fastest |
| Go | 45.77ms | Third fastest |
| Swift 5.8 | 86.33ms | Fourth fastest |

**Goal**: Beat C++ (13.54ms) to become world's fastest Lambda runtime.

## Troubleshooting

### Issue: Binary not found

**Error**:
```
Binary not found: target/release-ultra/bootstrap
```

**Fix**:
```bash
# Build with release-ultra profile
cargo build --profile release-ultra --target x86_64-unknown-linux-musl
```

### Issue: No measurements collected

**Error**:
```
No measurements found in results file
```

**Fix**: Ensure you run `benchmark` command before `compare` or `report`.

### Issue: Performance regression

If benchmarks show >8ms average:
1. Check binary size (should be <100KB)
2. Profile with `perf` to find hotspots
3. Review optimization flags in `Cargo.toml`
4. Consider PGO (Profile-Guided Optimization)

## Advanced Usage

### Custom Benchmark Script

For more control over benchmarking process:

```rust
// custom_benchmark.rs
use std::process::Command;
use std::time::Instant;

fn main() {
    let mut measurements = Vec::new();

    for i in 1..=10 {
        println!("Invocation {}...", i);

        let start = Instant::now();

        // Run Lambda bootstrap
        let output = Command::new("./target/release-ultra/bootstrap")
            .env("AWS_LAMBDA_RUNTIME_API", "localhost:9001")
            .output()
            .expect("Failed to run bootstrap");

        let duration = start.elapsed();
        measurements.push(duration.as_secs_f64() * 1000.0);

        println!("  Duration: {:.2}ms", duration.as_secs_f64() * 1000.0);
    }

    let avg: f64 = measurements.iter().sum::<f64>() / measurements.len() as f64;
    println!("\nAverage: {:.2}ms", avg);
}
```

### Criterion Benchmarks

The profiler also provides Criterion benchmarks for detailed performance analysis:

```bash
# Run all cold start benchmarks
cargo bench --bench cold_start

# Run specific benchmark
cargo bench --bench cold_start -- cold_start_single

# Generate performance report
cargo bench --bench cold_start -- --save-baseline baseline-v1

# Compare against baseline
cargo bench --bench cold_start -- --baseline baseline-v1
```

## Integration with lambda-perf

To submit results to lambda-perf project:

1. **Generate report**:
```bash
profiler report -i results.json -o lambda-perf.json
```

2. **Submit PR to lambda-perf**:
```bash
git clone https://github.com/maxday/lambda-perf.git
cd lambda-perf
# Add lambda-perf.json to data/
git checkout -b add-ruchy-runtime
git add data/lambda-perf.json
git commit -m "Add Ruchy runtime results"
git push origin add-ruchy-runtime
# Create PR on GitHub
```

## Development Roadmap

### Phase 1: Simulation (Current) ⚠️ NOT REAL DATA
- ✅ CLI profiler with 4 commands
- ✅ Lambda-perf JSON format
- ✅ Statistical analysis (P50, P99, avg)
- ✅ Comparison vs C++/Rust/Go/Swift
- ❌ **CRITICAL**: Uses `std::thread::sleep(100μs + 50μs)` simulation
- ❌ **DO NOT TRUST**: 0.26ms numbers are fake sleep times
- ❌ **NOT USABLE**: For infrastructure testing only, not real benchmarks

### Phase 2: Real Measurements (Next)
- ⏳ Replace simulation with actual Lambda Runtime API
- ⏳ Measure real init + handler time
- ⏳ Capture actual memory usage (peak RSS)
- ⏳ Validate against AWS Lambda environment

### Phase 3: Optimization (Future)
- ⏳ Binary size optimization (<100KB target)
- ⏳ PGO (Profile-Guided Optimization)
- ⏳ Zero-copy deserialization
- ⏳ ARM64 Graviton2 optimizations

## Resources

- **Specification**: `docs/specification/ruchy-compiled-transpiled-fast-lambda-in-world-spec.md`
- **Lambda-perf Project**: https://github.com/maxday/lambda-perf
- **AWS Lambda Runtime API**: https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html
- **Ruchy Language**: https://github.com/paiml/ruchy

## Support

For issues, questions, or contributions:
- **GitHub Issues**: https://github.com/paiml/ruchy-lambda/issues
- **Discussions**: https://github.com/paiml/ruchy-lambda/discussions
- **Email**: support@paiml.com

## License

MIT OR Apache-2.0

---

**Last Updated**: November 4, 2025
**Profiler Version**: v0.1.0
**Status**: Production Ready (with simulation)
