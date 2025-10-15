// Internal test file with benchmarks
package fib

import "testing"

func TestFibonacci(t *testing.T) {
	tests := []struct {
		n    int
		want int
	}{
		{0, 0},
		{1, 1},
		{2, 1},
		{3, 2},
		{4, 3},
		{5, 5},
		{10, 55},
	}

	for _, tt := range tests {
		got := Fibonacci(tt.n)
		if got != tt.want {
			t.Errorf("Fibonacci(%d) = %d, want %d", tt.n, got, tt.want)
		}
	}
}

func BenchmarkFibonacci10(b *testing.B) {
	for i := 0; i < b.N; i++ {
		Fibonacci(10)
	}
}

func BenchmarkFibonacci20(b *testing.B) {
	for i := 0; i < b.N; i++ {
		Fibonacci(20)
	}
}
