// Fibonacci recursive (n=35) - Go
// Matches AWS Lambda baseline implementation
// Expected result: 9227465

package main

func fibonacci(n int) int {
	if n <= 1 {
		return n
	}
	return fibonacci(n-1) + fibonacci(n-2)
}

func main() {
	result := fibonacci(35)
	_ = result // Silent for benchmarking
}
