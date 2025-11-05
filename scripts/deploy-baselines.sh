#!/bin/bash
# Deploy all baseline Lambda functions for fair comparison
# Baselines sourced from lambda-perf: https://github.com/maxday/lambda-perf

set -euo pipefail

readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly REGION="${AWS_REGION:-us-east-1}"
readonly RUNTIME_CUSTOM="provided.al2023"
readonly RUNTIME_PYTHON="python3.12"
readonly MEMORY_SIZE="128"
readonly ROLE_NAME="ruchy-lambda-execution-role"

printf "🚀 Deploying Baseline Lambda Functions\\n"
printf "========================================\\n"
printf "Region: %s\\n" "$REGION"
printf "Memory: %sMB\\n\\n" "$MEMORY_SIZE"

# Check AWS credentials
if ! aws sts get-caller-identity &>/dev/null; then
    printf "❌ AWS credentials not configured\\n" >&2
    exit 1
fi

ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
ROLE_ARN="arn:aws:iam::${ACCOUNT_ID}:role/${ROLE_NAME}"

printf "✅ Authenticated as account: %s\\n\\n" "$ACCOUNT_ID"

# Function to deploy Lambda
deploy_baseline() {
    local LANG=$1
    local FUNCTION_NAME="baseline-${LANG}"
    local RUNTIME=$2
    local BUILD_DIR="${PROJECT_ROOT}/baselines/${LANG}"

    printf "\\n📦 Deploying %s baseline...\\n" "$LANG"
    printf "========================================\\n"

    # Build package
    cd "$BUILD_DIR"
    if [ -f "build.sh" ]; then
        printf "Building %s...\\n" "$LANG"
        ./build.sh
    fi

    local PACKAGE_PATH="${BUILD_DIR}/function.zip"

    if [ ! -f "$PACKAGE_PATH" ]; then
        printf "❌ Package not found: %s\\n" "$PACKAGE_PATH" >&2
        return 1
    fi

    printf "Package size: %s\\n" "$(ls -lh $PACKAGE_PATH | awk '{print $5}')"

    # Check if function exists
    if aws lambda get-function --function-name "$FUNCTION_NAME" &>/dev/null; then
        printf "Updating existing function...\\n"
        aws lambda update-function-code \
            --function-name "$FUNCTION_NAME" \
            --zip-file "fileb://${PACKAGE_PATH}" \
            --region "$REGION" \
            > /dev/null
    else
        printf "Creating new function...\\n"
        aws lambda create-function \
            --function-name "$FUNCTION_NAME" \
            --runtime "$RUNTIME" \
            --role "$ROLE_ARN" \
            --handler "bootstrap" \
            --zip-file "fileb://${PACKAGE_PATH}" \
            --timeout 30 \
            --memory-size "$MEMORY_SIZE" \
            --region "$REGION" \
            > /dev/null
    fi

    printf "✅ %s baseline deployed: %s\\n" "$LANG" "$FUNCTION_NAME"
}

# Deploy all baselines
printf "Building and deploying baselines...\\n\\n"

# Go
deploy_baseline "go" "$RUNTIME_CUSTOM"

# Rust
deploy_baseline "rust" "$RUNTIME_CUSTOM"

# C++
# deploy_baseline "cpp" "$RUNTIME_CUSTOM"  # Requires Docker build

# Python
deploy_baseline "python" "$RUNTIME_PYTHON"

printf "\\n========================================\\n"
printf "✅ All baselines deployed!\\n\\n"

printf "Verify deployments:\\n"
printf "  aws lambda list-functions --query \"Functions[?starts_with(FunctionName, 'baseline-')].[FunctionName, Runtime, LastModified]\" --output table\\n\\n"

printf "Invoke baselines:\\n"
printf "  aws lambda invoke --function-name baseline-go response.json\\n"
printf "  aws lambda invoke --function-name baseline-rust response.json\\n"
printf "  aws lambda invoke --function-name baseline-python response.json\\n"
