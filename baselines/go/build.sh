#!/bin/bash
# Build Go baseline Lambda function
# Source: lambda-perf go_on_provided_al2023

set -euo pipefail

echo "🔨 Building Go baseline Lambda..."

# Build for Linux ARM64 (or x86_64)
GOOS=linux GOARCH=amd64 go build -tags lambda.norpc -o bootstrap main.go

# Create deployment package
zip function.zip bootstrap

echo "✅ Go baseline built: function.zip"
echo "Binary size: $(ls -lh function.zip | awk '{print $5}')"
