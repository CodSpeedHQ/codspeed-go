//go:build darwin

package example

import "testing"

func BenchmarkFibonacciDarwin(b *testing.B) {
	for b.Loop() {
		fibonacci(25)
	}
}
