package examplewithhelper

import "testing"

func Fib(n int) int {
	if n <= 1 {
		return n
	}
	return Fib(n-1) + Fib(n-2)
}

func BenchmarkFibHelper(b *testing.B) {
	for i := 0; i < b.N; i++ {
		Fib(10)
	}
}
