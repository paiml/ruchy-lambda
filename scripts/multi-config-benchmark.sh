#!/bin/bash
# multi-config-benchmark.sh
# Test across multiple memory configurations

set -euo pipefail

MEMORY_SIZES=(128 256 512 1024)

echo "🔍 Building optimized binary..."
cargo build --profile release-ultra --target x86_64-unknown-linux-musl

echo ""
echo "📊 Running multi-configuration benchmarks..."
echo ""

for mem in "${MEMORY_SIZES[@]}"; do
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "🔍 Benchmarking ${mem}MB configuration..."
  echo ""

  ./target/release/profiler benchmark \
    -m "$mem" \
    -a x86_64 \
    -o "results-${mem}mb.json"

  echo ""
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Results Summary:"
echo ""
for mem in "${MEMORY_SIZES[@]}"; do
  echo "  ${mem}MB:"
  if command -v jq &> /dev/null; then
    jq -r '"    Avg: \(.stats.avg_ms)ms, P50: \(.stats.p50_ms)ms, P99: \(.stats.p99_ms)ms"' "results-${mem}mb.json"
  else
    echo "    (Install jq to see detailed stats)"
  fi
done

echo ""
echo "✅ Multi-configuration profiling complete!"

exit 0
