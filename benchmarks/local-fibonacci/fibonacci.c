// Fibonacci recursive (n=35) - C
// Matches AWS Lambda baseline implementation
// Expected result: 9227465

#include <stdio.h>

int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main() {
    int result = fibonacci(35);
    // Silent for benchmarking
    return 0;
}
