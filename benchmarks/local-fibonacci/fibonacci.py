#!/usr/bin/env python3
# Fibonacci recursive (n=35) - Python
# Matches AWS Lambda baseline implementation
# Expected result: 9227465

def fibonacci(n):
    """Calculate nth Fibonacci number recursively"""
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

def main():
    result = fibonacci(35)
    # Silent for benchmarking

if __name__ == "__main__":
    main()
