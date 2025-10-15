// External test package (black-box testing) with benchmarks
// This file tests the fib package from an external perspective
package fib_test

import (
	"example-with-test-package/fib"
	"testing"
)

// TestFibonacciIntegration is an integration test with benchmarks
// This ensures we handle external test packages correctly
func TestFibonacciIntegration(t *testing.T) {
	t.Parallel()

	// Test that Fibonacci works correctly for a range of values
	testCases := []struct {
		input    int
		expected int
	}{
		{0, 0},
		{1, 1},
		{5, 5},
		{10, 55},
		{15, 610},
	}

	for _, tc := range testCases {
		result := fib.Fibonacci(tc.input)
		if result != tc.expected {
			t.Errorf("Fibonacci(%d) = %d; want %d", tc.input, result, tc.expected)
		}
	}
}

// TestFibonacciEdgeCases tests edge cases
func TestFibonacciEdgeCases(t *testing.T) {
	if fib.Fibonacci(0) != 0 {
		t.Error("Fibonacci(0) should return 0")
	}
	if fib.Fibonacci(1) != 1 {
		t.Error("Fibonacci(1) should return 1")
	}
}

func BenchmarkFibonacci20(b *testing.B) {
	for i := 0; i < b.N; i++ {
		fib.Fibonacci(20)
	}
}
