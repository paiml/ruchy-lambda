#!/bin/bash
# Build Python baseline Lambda function
# Source: lambda-perf python312

set -euo pipefail

echo "🔨 Building Python baseline Lambda..."

# Create deployment package (no dependencies needed)
zip function.zip index.py

echo "✅ Python baseline built: function.zip"
echo "Package size: $(ls -lh function.zip | awk '{print $5}')"
