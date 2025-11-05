#!/bin/bash
# Deploy Ruchy Lambda to AWS for REAL testing
# Measures: cold start, CPU time, memory usage

set -euo pipefail

readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly FUNCTION_NAME_MINIMAL="ruchy-lambda-minimal"
readonly FUNCTION_NAME_FIBONACCI="ruchy-lambda-fibonacci"
readonly REGION="${AWS_REGION:-us-east-1}"
readonly RUNTIME="provided.al2023"
readonly ARCHITECTURE="x86_64"
readonly MEMORY_SIZE="128"  # Minimum for fair comparison

printf "🚀 Deploying Ruchy Lambda to AWS\\n"
printf "====================================\\n"
printf "Region: %s\\n" "$REGION"
printf "Architecture: %s\\n" "$ARCHITECTURE"
printf "Memory: %sMB\\n\\n" "$MEMORY_SIZE"

# Check AWS credentials
printf "🔐 Checking AWS credentials...\\n"
if ! aws sts get-caller-identity &>/dev/null; then
    printf "❌ AWS credentials not configured\\n" >&2
    printf "Run: aws configure\\n" >&2
    exit 1
fi

ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
printf "✅ Authenticated as account: %s\\n\\n" "$ACCOUNT_ID"

# Create IAM role for Lambda (if not exists)
ROLE_NAME="ruchy-lambda-execution-role"
printf "📝 Creating IAM role: %s...\\n" "$ROLE_NAME"

if ! aws iam get-role --role-name "$ROLE_NAME" &>/dev/null; then
    # Create trust policy
    cat > /tmp/trust-policy.json << EOF
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Principal": {"Service": "lambda.amazonaws.com"},
    "Action": "sts:AssumeRole"
  }]
}
EOF

    aws iam create-role \
        --role-name "$ROLE_NAME" \
        --assume-role-policy-document file:///tmp/trust-policy.json \
        --description "Execution role for Ruchy Lambda benchmarks"

    # Attach basic Lambda execution policy
    aws iam attach-role-policy \
        --role-name "$ROLE_NAME" \
        --policy-arn "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"

    printf "✅ IAM role created\\n"
    printf "⏳ Waiting for role to propagate...\\n"
    sleep 10
else
    printf "✅ IAM role already exists\\n"
fi

ROLE_ARN="arn:aws:iam::${ACCOUNT_ID}:role/${ROLE_NAME}"
printf "Role ARN: %s\\n\\n" "$ROLE_ARN"

# Function to deploy Lambda
deploy_function() {
    local HANDLER_TYPE=$1
    local FUNCTION_NAME=$2

    printf "\\n📦 Deploying %s handler...\\n" "$HANDLER_TYPE"
    printf "========================================\\n"

    # Build package
    "$PROJECT_ROOT/scripts/build-lambda-package.sh" "$HANDLER_TYPE"

    local PACKAGE_PATH="$PROJECT_ROOT/target/lambda-packages/ruchy-lambda-${HANDLER_TYPE}.zip"

    # Create or update function
    if aws lambda get-function --function-name "$FUNCTION_NAME" --region "$REGION" &>/dev/null; then
        printf "📤 Updating existing function...\\n"
        aws lambda update-function-code \
            --function-name "$FUNCTION_NAME" \
            --zip-file "fileb://$PACKAGE_PATH" \
            --region "$REGION" \
            --architectures "$ARCHITECTURE" \
            --output json > /dev/null

        printf "✅ Function updated\\n"
    else
        printf "📤 Creating new function...\\n"
        aws lambda create-function \
            --function-name "$FUNCTION_NAME" \
            --runtime "$RUNTIME" \
            --role "$ROLE_ARN" \
            --handler "bootstrap" \
            --zip-file "fileb://$PACKAGE_PATH" \
            --region "$REGION" \
            --architectures "$ARCHITECTURE" \
            --memory-size "$MEMORY_SIZE" \
            --timeout 30 \
            --description "Ruchy Lambda - $HANDLER_TYPE handler for performance testing" \
            --output json > /dev/null

        printf "✅ Function created\\n"
    fi

    # Wait for function to be active
    printf "⏳ Waiting for function to be active...\\n"
    aws lambda wait function-active-v2 \
        --function-name "$FUNCTION_NAME" \
        --region "$REGION"

    printf "✅ %s deployed successfully\\n" "$FUNCTION_NAME"
}

# Deploy BOTH handlers
deploy_function "minimal" "$FUNCTION_NAME_MINIMAL"
deploy_function "fibonacci" "$FUNCTION_NAME_FIBONACCI"

printf "\\n🎉 Deployment complete!\\n\\n"
printf "📊 To invoke and measure performance:\\n"
printf "   ./scripts/measure-aws-performance.sh\\n"
