# Ruchy Lambda Benchmarks

This directory contains benchmark results and reports for the Ruchy Lambda runtime.

## Directory Structure

```
benchmarks/
├── reports/          # Versioned benchmark results (committed to git)
│   └── cold-start-YYYY-MM-DD-vX.Y.Z.json
└── README.md         # This file
```

## Benchmark Reports

All benchmark results are **version controlled** to track performance over time.

### Naming Convention

Reports use the format: `{benchmark-type}-{date}-v{version}.json`

Examples:
- `cold-start-2025-11-04-v0.1.0.json` - Cold start measurements
- `full-request-2025-11-05-v0.1.0.json` - Full request latency
- `memory-usage-2025-11-06-v0.1.0.json` - Memory consumption

### Report Types

1. **Cold Start** (`cold-start-*.json`)
   - Process initialization time
   - Binary size
   - Comparison vs C++/Rust/Go baselines
   - Target: <8ms

2. **Full Request** (future: `full-request-*.json`)
   - End-to-end request latency
   - Includes handler execution
   - Warm vs cold invocations

3. **Memory Usage** (future: `memory-*.json`)
   - Peak memory consumption
   - Memory footprint over time

## Running Benchmarks

```bash
# Cold start benchmark (local)
./scripts/benchmark-cold-start.sh

# Results saved to: local-benchmark-results.json
# Commit to: benchmarks/reports/cold-start-$(date +%Y-%m-%d)-v{VERSION}.json
```

## Current Results

### v0.1.0 (2025-11-04) - Phase 3 Complete

**Binary Size:** 316KB

**Cold Start Performance:**
- Average: 2ms
- Min: 2ms
- Max: 3ms
- Iterations: 10

**Comparison vs lambda-perf:**
- C++: 13.54ms → Ruchy **6.77x faster** ✅
- Rust: 16.98ms → Ruchy **8.49x faster** ✅
- Go: 45.77ms → Ruchy **22.89x faster** ✅

**Target Achievement:**
- ✅ Target: <8ms
- ✅ Achieved: 2ms (4x better!)

## Historical Tracking

| Version | Date | Cold Start (avg) | Binary Size | vs C++ | vs Rust | vs Go |
|---------|------|------------------|-------------|--------|---------|-------|
| v0.1.0 | 2025-11-04 | 2ms | 316KB | 6.77x | 8.49x | 22.89x |

## Next Steps

1. **AWS Lambda deployment** - Validate in real cloud environment
2. **ARM64 Graviton2** - Test on AWS Graviton2 processors
3. **Full request latency** - End-to-end including handler execution
4. **Memory profiling** - Track memory consumption
5. **lambda-perf submission** - Submit results to lambda-perf project
