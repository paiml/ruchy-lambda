#!/bin/bash
# Build Lambda deployment package
# Usage: ./scripts/build-lambda-package.sh [minimal|fibonacci|default]

set -euo pipefail

readonly HANDLER_TYPE="${1:-default}"
readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly BOOTSTRAP_DIR="$PROJECT_ROOT/crates/bootstrap"

printf "🔨 Building Ruchy Lambda - Handler: %s\\n" "$HANDLER_TYPE"
printf "========================================\\n\\n"

# Switch handler based on type
case "$HANDLER_TYPE" in
    minimal)
        printf "📝 Using minimal handler (lambda-perf style)\\n"
        HANDLER_FILE="handler_minimal"
        OUTPUT_NAME="ruchy-lambda-minimal"
        ;;
    fibonacci)
        printf "📝 Using fibonacci handler (CPU benchmark)\\n"
        HANDLER_FILE="handler_fibonacci"
        OUTPUT_NAME="ruchy-lambda-fibonacci"
        ;;
    default)
        printf "📝 Using default handler\\n"
        HANDLER_FILE="handler"
        OUTPUT_NAME="ruchy-lambda"
        ;;
    *)
        printf "❌ Unknown handler type: %s\\n" "$HANDLER_TYPE" >&2
        printf "Usage: %s [minimal|fibonacci|default]\\n" "$0" >&2
        exit 1
        ;;
esac

# Create temporary main.rs that uses the selected handler
cd "$BOOTSTRAP_DIR"

# Backup original main.rs
cp src/main.rs src/main.rs.backup

# Update main.rs to use selected handler
# Match any existing handler path pattern (handler*_generated.rs)
sed -i "s|#\[path = \"[^\"]*_generated.rs\"\]|#[path = \"${HANDLER_FILE}_generated.rs\"]|" src/main.rs

# Build optimized binary
# Use generic x86-64 target to ensure compatibility with AWS Lambda
# AWS Lambda uses baseline x86-64 without modern CPU extensions
# Use release-ultra profile for maximum size optimization (opt-level='z', lto=fat, codegen-units=1)
printf "\\n🚀 Building optimized binary with release-ultra profile...\\n"
RUSTFLAGS="-C target-cpu=x86-64" cargo build --profile release-ultra -p ruchy-lambda-bootstrap

# Restore original main.rs
mv src/main.rs.backup src/main.rs

# Binary is in target/release-ultra/ for custom profiles
BINARY_PATH="$PROJECT_ROOT/target/release-ultra/bootstrap"

# Strip binary (release-ultra profile already strips, but ensure it's done)
printf "\\n✂️  Stripping debug symbols...\\n"
strip "$BINARY_PATH" 2>/dev/null || true

# Get binary size
BINARY_SIZE=$(stat -c%s "$BINARY_PATH")
BINARY_SIZE_KB=$((BINARY_SIZE / 1024))

printf "\\n📦 Binary size: %dKB\\n" "$BINARY_SIZE_KB"

# Create deployment package
PACKAGE_DIR="$PROJECT_ROOT/target/lambda-packages"
mkdir -p "$PACKAGE_DIR"

PACKAGE_PATH="$PACKAGE_DIR/${OUTPUT_NAME}.zip"

printf "\\n📦 Creating Lambda package: %s\\n" "$PACKAGE_PATH"
cd "$PROJECT_ROOT/target/release-ultra"
zip -j "$PACKAGE_PATH" bootstrap

printf "\\n✅ Package created successfully!\\n"
printf "   Path: %s\\n" "$PACKAGE_PATH"
printf "   Size: %s\\n" "$(ls -lh "$PACKAGE_PATH" | awk '{print $5}')"
printf "\\n"
printf "📤 Ready to deploy to AWS Lambda!\\n"
