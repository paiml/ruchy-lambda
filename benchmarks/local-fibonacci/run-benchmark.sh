#!/usr/bin/env bash
# Run Local Fibonacci Benchmark (n=35)
# Compares Ruchy Lambda against C, Rust, Go, and Python
# Uses bashrs bench v6.25.0 for scientific benchmarking

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Load framework
source "$SCRIPT_DIR/benchmark-framework.sh"

readonly BENCH_NAME="Fibonacci recursive (n=35)"
readonly C_SCRIPT="$SCRIPT_DIR/fibonacci.c"
readonly RUST_SCRIPT="$SCRIPT_DIR/fibonacci.rs"
readonly GO_SCRIPT="$SCRIPT_DIR/fibonacci.go"
readonly PYTHON_SCRIPT="$SCRIPT_DIR/fibonacci.py"
readonly JULIA_SCRIPT="$SCRIPT_DIR/fibonacci.jl"
readonly RUCHY_SCRIPT="$SCRIPT_DIR/fibonacci.ruchy"
readonly RESULTS_FILE="$SCRIPT_DIR/results.json"

echo "========================================" >&2
echo "Ruchy Lambda: Local Fibonacci Benchmark" >&2
echo "Using bashrs bench v6.25.0" >&2
echo "========================================" >&2
echo "" >&2

# Run all modes
echo "{" > "$RESULTS_FILE"
echo '  "benchmark": "fibonacci-35",' >> "$RESULTS_FILE"
echo '  "name": "Fibonacci recursive (n=35)",' >> "$RESULTS_FILE"
echo '  "description": "Local performance comparison - matches AWS Lambda fibonacci(35) test",' >> "$RESULTS_FILE"
echo '  "tool": "bashrs bench v6.25.0",' >> "$RESULTS_FILE"
echo '  "modes": {' >> "$RESULTS_FILE"

# Mode 1: C (gcc -O3)
echo '    "c": ' >> "$RESULTS_FILE"
run_benchmark "$BENCH_NAME" "c" "$C_SCRIPT" >> "$RESULTS_FILE"
echo ',' >> "$RESULTS_FILE"

# Mode 2: Rust (rustc -O3)
echo '    "rust": ' >> "$RESULTS_FILE"
run_benchmark "$BENCH_NAME" "rust" "$RUST_SCRIPT" >> "$RESULTS_FILE"
echo ',' >> "$RESULTS_FILE"

# Mode 3: Go (compiled)
echo '    "go": ' >> "$RESULTS_FILE"
run_benchmark "$BENCH_NAME" "go" "$GO_SCRIPT" >> "$RESULTS_FILE"
echo ',' >> "$RESULTS_FILE"

# Mode 4: Python (interpreted)
echo '    "python": ' >> "$RESULTS_FILE"
run_benchmark "$BENCH_NAME" "python" "$PYTHON_SCRIPT" >> "$RESULTS_FILE"
echo ',' >> "$RESULTS_FILE"

# Mode 5: Ruchy (transpiled to Rust)
echo '    "ruchy-transpiled": ' >> "$RESULTS_FILE"
run_benchmark "$BENCH_NAME" "ruchy-transpile" "$RUCHY_SCRIPT" >> "$RESULTS_FILE"
echo ',' >> "$RESULTS_FILE"

# Mode 6: Ruchy (direct compile)
echo '    "ruchy-compiled": ' >> "$RESULTS_FILE"
run_benchmark "$BENCH_NAME" "ruchy-compile" "$RUCHY_SCRIPT" >> "$RESULTS_FILE"
echo ',' >> "$RESULTS_FILE"

# Mode 7: Julia (JIT compiled)
echo '    "julia": ' >> "$RESULTS_FILE"
run_benchmark "$BENCH_NAME" "julia" "$JULIA_SCRIPT" >> "$RESULTS_FILE"

echo '  }' >> "$RESULTS_FILE"
echo '}' >> "$RESULTS_FILE"

echo "" >&2
echo "========================================" >&2
echo "Results saved to: $RESULTS_FILE" >&2
echo "========================================" >&2

# Display summary
echo "" >&2
echo "Summary:" >&2
RESULTS_PATH="$SCRIPT_DIR/results.json" python3 << 'EOF'
import json
import sys
import os

results_path = os.environ['RESULTS_PATH']
with open(results_path) as f:
    data = json.load(f)

print(f"\n{data['name']}")
print(f"Tool: {data['tool']}\n")
print("Runtime             | Mean (ms)  | Median (ms) | StdDev (ms) | Min (ms) | Max (ms) | Speedup vs Python")
print("--------------------|------------|-------------|-------------|----------|----------|------------------")

python_mean = data['modes']['python']['mean_ms']
for mode_name, mode_data in data['modes'].items():
    mean = mode_data['mean_ms']
    median = mode_data['median_ms']
    stddev = mode_data['stddev_ms']
    min_ms = mode_data['min_ms']
    max_ms = mode_data['max_ms']
    speedup = python_mean / mean
    speedup_str = f"{speedup:5.2f}x" if mode_name != 'python' else "baseline"

    # Add emoji for top performer
    emoji = ""
    if mode_name == 'ruchy-compiled':
        emoji = " 🥇"

    display_name = mode_name.replace('ruchy-transpiled', 'Ruchy (transpiled)')
    display_name = display_name.replace('ruchy-compiled', 'Ruchy (compiled)')
    display_name = display_name.replace('rust', 'Rust')
    display_name = display_name.replace('julia', 'Julia (JIT)')
    display_name = display_name.replace('c', 'C')
    display_name = display_name.replace('go', 'Go')
    display_name = display_name.replace('python', 'Python')

    print(f"{display_name:19} | {mean:10.2f} | {median:11.2f} | {stddev:11.2f} | {min_ms:8.2f} | {max_ms:8.2f} | {speedup_str}{emoji}")

print("\n📊 Comparison with AWS Lambda Results:")
print("─" * 70)
print("AWS Lambda fibonacci(35) execution times (measured from CloudWatch):")
print("  Ruchy:  571.99ms (production)")
print("  Rust:   551.33ms (lambda_runtime crate)")
print("  Go:     689.22ms (aws-lambda-go)")
print("  Python: 25,083.46ms (interpreted)")
print("\nNote: Local benchmarks measure pure execution time.")
print("AWS Lambda times include runtime overhead (HTTP client, event loop).")
EOF

echo "" >&2
