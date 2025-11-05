// Fibonacci recursive (n=35) - Rust
// Matches AWS Lambda baseline implementation
// Expected result: 9227465

fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fn main() {
    let result = fibonacci(35);
    let _ = result; // Silent for benchmarking
}
