#!/bin/bash
# local-benchmark.sh
# Local development profiling workflow for Ruchy Lambda

set -euo pipefail

echo "🔍 Building optimized binary..."
cargo build --profile release-ultra --target x86_64-unknown-linux-musl

echo ""
echo "🔧 Building profiler..."
cargo build --package ruchy-lambda-profiler --release

echo ""
echo "📊 Running profiler benchmark..."
./target/release/profiler benchmark \
  -m 128 \
  -a x86_64 \
  -o benchmark-results.json

echo ""
echo "📈 Comparing against fastest runtimes..."
./target/release/profiler compare -i benchmark-results.json

echo ""
echo "📄 Generating lambda-perf report..."
./target/release/profiler report \
  -i benchmark-results.json \
  -o lambda-perf-report.json

echo ""
echo "✅ Profiling complete!"
echo "   Results: benchmark-results.json"
echo "   Lambda-perf: lambda-perf-report.json"

exit 0
