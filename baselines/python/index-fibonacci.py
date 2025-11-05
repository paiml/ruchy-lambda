#!/usr/bin/env python3
# Fibonacci Lambda handler - Python 3.12
# Source: ruchy-book bench-007-fibonacci.py

def fibonacci(n):
    """Calculate nth Fibonacci number recursively"""
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

def handler(event, context):
    # Calculate fibonacci(35) - standard Lambda benchmark
    result = fibonacci(35)

    return {
        'statusCode': 200,
        'body': f'fibonacci(35)={result}'
    }
