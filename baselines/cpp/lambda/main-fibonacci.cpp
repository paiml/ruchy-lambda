#include <aws/lambda-runtime/runtime.h>
#include <string>

using namespace aws::lambda_runtime;

// Fibonacci recursive implementation
// Source: ruchy-book bench-007-fibonacci.c
int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

static invocation_response handler(invocation_request const&)
{
    // Calculate fibonacci(35) - standard Lambda benchmark
    int result = fibonacci(35);

    std::string response = "fibonacci(35)=" + std::to_string(result);
    return invocation_response::success(response, "text/plain");
}

int main()
{
    run_handler(handler);
    return 0;
}
