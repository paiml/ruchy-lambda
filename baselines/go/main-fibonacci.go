package main

import (
	"context"
	"fmt"

	"github.com/aws/aws-lambda-go/lambda"
)

// Fibonacci recursive implementation
// Source: ruchy-book bench-007-fibonacci.go
func fibonacci(n int) int {
	if n <= 1 {
		return n
	}
	return fibonacci(n-1) + fibonacci(n-2)
}

type testResponse struct {
	StatusCode int    `json:"statusCode"`
	Body       string `json:"body"`
}

func handleRequest(ctx context.Context) (testResponse, error) {
	// Calculate fibonacci(35) - standard Lambda benchmark
	result := fibonacci(35)

	return testResponse{
		StatusCode: 200,
		Body:       fmt.Sprintf("fibonacci(35)=%d", result),
	}, nil
}

func main() {
	lambda.Start(handleRequest)
}
