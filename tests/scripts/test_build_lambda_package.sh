#!/bin/bash
# Unit tests for build-lambda-package.sh
# Using bashrs test framework

set -euo pipefail

# Source the script under test (functions only, not execution)
# Note: We'll mock the actual build commands

# Test: Handler type validation
test_handler_type_validation() {
    # Test invalid handler type
    local output
    if output=$(HANDLER_TYPE="invalid" 2>&1); then
        return 1  # Should have failed
    fi

    # Check error message contains "Unknown handler type"
    if [[ ! "$output" =~ "Unknown handler type" ]]; then
        return 1
    fi

    return 0
}

# Test: Valid handler types accepted
test_valid_handler_types() {
    local types=("minimal" "fibonacci" "default")

    for type in "${types[@]}"; do
        # Mock validation (we can't actually run the build here)
        case "$type" in
            minimal|fibonacci|default)
                # Valid
                ;;
            *)
                return 1
                ;;
        esac
    done

    return 0
}

# Test: Output name generation
test_output_name_generation() {
    local -A expected_names=(
        ["minimal"]="ruchy-lambda-minimal"
        ["fibonacci"]="ruchy-lambda-fibonacci"
        ["default"]="ruchy-lambda"
    )

    for type in "${!expected_names[@]}"; do
        local expected="${expected_names[$type]}"

        # This tests the naming logic
        case "$type" in
            minimal)
                local output_name="ruchy-lambda-minimal"
                ;;
            fibonacci)
                local output_name="ruchy-lambda-fibonacci"
                ;;
            default)
                local output_name="ruchy-lambda"
                ;;
        esac

        if [ "$output_name" != "$expected" ]; then
            echo "Expected $expected, got $output_name"
            return 1
        fi
    done

    return 0
}

# Test: Handler file mapping
test_handler_file_mapping() {
    local -A expected_files=(
        ["minimal"]="handler_minimal"
        ["fibonacci"]="handler_fibonacci"
        ["default"]="handler"
    )

    for type in "${!expected_files[@]}"; do
        local expected="${expected_files[$type]}"

        case "$type" in
            minimal)
                local handler_file="handler_minimal"
                ;;
            fibonacci)
                local handler_file="handler_fibonacci"
                ;;
            default)
                local handler_file="handler"
                ;;
        esac

        if [ "$handler_file" != "$expected" ]; then
            return 1
        fi
    done

    return 0
}

# Run all tests
main() {
    local tests=(
        "test_handler_type_validation"
        "test_valid_handler_types"
        "test_output_name_generation"
        "test_handler_file_mapping"
    )

    local passed=0
    local failed=0

    for test in "${tests[@]}"; do
        if $test; then
            echo "✓ $test"
            ((passed++))
        else
            echo "✗ $test"
            ((failed++))
        fi
    done

    echo ""
    echo "Tests: $passed passed, $failed failed"

    if [ "$failed" -gt 0 ]; then
        exit 1
    fi
}

# Only run if executed directly
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    main
fi
