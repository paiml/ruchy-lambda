#!/bin/bash
# arch-comparison.sh
# Compare ARM64 vs x86_64 performance

set -euo pipefail

echo "🔍 Building profiler..."
cargo build --package ruchy-lambda-profiler --release

echo ""
echo "📊 Running architecture comparison..."
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Benchmarking x86_64..."
echo ""
./target/release/profiler benchmark -m 128 -a x86_64 -o results-x86.json

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Benchmarking ARM64..."
echo ""
./target/release/profiler benchmark -m 128 -a arm64 -o results-arm64.json

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Architecture Comparison:"
echo ""
echo "x86_64:"
if command -v jq &> /dev/null; then
  jq -r '"  Avg: \(.stats.avg_ms)ms, P50: \(.stats.p50_ms)ms, P99: \(.stats.p99_ms)ms, Binary: \(.binary.size_kb)KB"' results-x86.json
else
  echo "  (Install jq to see detailed stats)"
fi

echo ""
echo "ARM64:"
if command -v jq &> /dev/null; then
  jq -r '"  Avg: \(.stats.avg_ms)ms, P50: \(.stats.p50_ms)ms, P99: \(.stats.p99_ms)ms, Binary: \(.binary.size_kb)KB"' results-arm64.json
else
  echo "  (Install jq to see detailed stats)"
fi

echo ""
echo "✅ Architecture comparison complete!"

exit 0
