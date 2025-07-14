package main

import "testing"

func BenchmarkFibonacci10(b *testing.B) {
	for i := 0; i < b.N; i++ {
		fibonacci(10)
	}
}

func BenchmarkFibonacci20(b *testing.B) {
	for i := 0; i < b.N; i++ {
		fibonacci(20)
	}
}

func BenchmarkFibonacci30(b *testing.B) {
	for i := 0; i < b.N; i++ {
		fibonacci(30)
	}
}
