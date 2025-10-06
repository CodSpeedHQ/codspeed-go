package main

import "testing"

func BenchmarkFib(b *testing.B) {
	BenchmarkFibHelper(b)
}
