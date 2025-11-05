#!/bin/bash
# Measure REAL AWS Lambda performance
# Collects: cold start, duration, memory usage

set -euo pipefail

readonly FUNCTION_NAME_MINIMAL="ruchy-lambda-minimal"
readonly FUNCTION_NAME_FIBONACCI="ruchy-lambda-fibonacci"
readonly REGION="${AWS_REGION:-us-east-1}"
readonly ITERATIONS=10
readonly RESULTS_DIR="benchmarks/reports/aws"

mkdir -p "$RESULTS_DIR"

printf "📊 Measuring AWS Lambda Performance (REAL DATA)\\n"
printf "=================================================\\n"
printf "Region: %s\\n" "$REGION"
printf "Iterations: %d per function\\n\\n" "$ITERATIONS"

# Function to invoke Lambda and extract metrics
invoke_and_measure() {
    local FUNCTION_NAME=$1
    local HANDLER_TYPE=$2

    printf "\\n🧪 Testing: %s\\n" "$FUNCTION_NAME"
    printf "----------------------------------------\\n"

    declare -a durations
    declare -a memory_used
    declare -a init_durations

    # Force cold start by updating environment variable
    printf "❄️  Forcing cold start...\\n"
    aws lambda update-function-configuration \
        --function-name "$FUNCTION_NAME" \
        --region "$REGION" \
        --environment "Variables={FORCE_COLD_START=$(date +%s)}" \
        --output json > /dev/null 2>&1

    aws lambda wait function-updated-v2 \
        --function-name "$FUNCTION_NAME" \
        --region "$REGION"

    sleep 2

    for i in $(seq 1 "$ITERATIONS"); do
        printf "  Iteration %d/%d..." "$i" "$ITERATIONS"

        # Invoke function
        RESPONSE=$(aws lambda invoke \
            --function-name "$FUNCTION_NAME" \
            --region "$REGION" \
            --payload '{}' \
            --log-type Tail \
            --output json \
            /tmp/lambda-response.txt 2>&1)

        # Extract duration (ms)
        DURATION=$(printf "%s" "$RESPONSE" | jq -r '.Duration // 0')
        durations+=("$DURATION")

        # Extract memory used (MB)
        MEMORY=$(printf "%s" "$RESPONSE" | jq -r '.MemoryUsed // 0')
        memory_used+=("$MEMORY")

        # Extract init duration if present (cold start)
        INIT=$(printf "%s" "$RESPONSE" | jq -r '.InitDuration // 0')
        if [ "$INIT" != "0" ] && [ "$INIT" != "null" ]; then
            init_durations+=("$INIT")
            printf " cold start: %sms, duration: %sms, memory: %sMB\\n" "$INIT" "$DURATION" "$MEMORY"
        else
            printf " duration: %sms, memory: %sMB\\n" "$DURATION" "$MEMORY"
        fi

        # Force new cold start for next iteration
        if [ "$i" -lt "$ITERATIONS" ]; then
            aws lambda update-function-configuration \
                --function-name "$FUNCTION_NAME" \
                --region "$REGION" \
                --environment "Variables={FORCE_COLD_START=$(date +%s)-${i}}" \
                --output json > /dev/null 2>&1

            sleep 1
        fi
    done

    # Calculate statistics
    local duration_sum=0
    local duration_min=${durations[0]}
    local duration_max=${durations[0]}

    for val in "${durations[@]}"; do
        duration_sum=$(echo "$duration_sum + $val" | bc)
        if (( $(echo "$val < $duration_min" | bc -l) )); then
            duration_min=$val
        fi
        if (( $(echo "$val > $duration_max" | bc -l) )); then
            duration_max=$val
        fi
    done

    local duration_avg=$(echo "scale=2; $duration_sum / ${#durations[@]}" | bc)

    # Memory statistics
    local memory_sum=0
    local memory_max=${memory_used[0]}

    for val in "${memory_used[@]}"; do
        memory_sum=$((memory_sum + val))
        if [ "$val" -gt "$memory_max" ]; then
            memory_max=$val
        fi
    done

    local memory_avg=$((memory_sum / ${#memory_used[@]}))

    # Init duration (cold start) statistics
    local init_avg="N/A"
    if [ "${#init_durations[@]}" -gt 0 ]; then
        local init_sum=0
        for val in "${init_durations[@]}"; do
            init_sum=$(echo "$init_sum + $val" | bc)
        done
        init_avg=$(echo "scale=2; $init_sum / ${#init_durations[@]}" | bc)
    fi

    printf "\\n📈 Results for %s:\\n" "$HANDLER_TYPE"
    printf "  Cold Start:  %s ms (avg of %d)\\n" "$init_avg" "${#init_durations[@]}"
    printf "  Duration:    %s ms (avg), %s ms (min), %s ms (max)\\n" "$duration_avg" "$duration_min" "$duration_max"
    printf "  Memory:      %d MB (avg), %d MB (max)\\n" "$memory_avg" "$memory_max"

    # Save results
    local REPORT_FILE="$RESULTS_DIR/${HANDLER_TYPE}-$(date +%Y-%m-%d).json"
    cat > "$REPORT_FILE" << EOF
{
  "function_name": "$FUNCTION_NAME",
  "handler_type": "$HANDLER_TYPE",
  "region": "$REGION",
  "iterations": $ITERATIONS,
  "cold_start_ms": {
    "avg": "$init_avg",
    "count": ${#init_durations[@]}
  },
  "duration_ms": {
    "avg": "$duration_avg",
    "min": "$duration_min",
    "max": "$duration_max"
  },
  "memory_mb": {
    "avg": $memory_avg,
    "max": $memory_max
  },
  "timestamp": "$(date -Iseconds)"
}
EOF

    printf "\\n💾 Saved: %s\\n" "$REPORT_FILE"
}

# Measure both functions
invoke_and_measure "$FUNCTION_NAME_MINIMAL" "minimal"
invoke_and_measure "$FUNCTION_NAME_FIBONACCI" "fibonacci"

printf "\\n✅ Performance measurement complete!\\n"
printf "\\n📊 Reports saved to: %s/\\n" "$RESULTS_DIR"
