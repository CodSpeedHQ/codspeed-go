package example

import (
	testing "github.com/CodSpeedHQ/codspeed-go/testing/testing"
)

func BenchmarkFibonacci10(b *testing.B) {
	b.Run("fibonacci(10)", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			fibonacci(10)
		}
	})
	b.Run("fibonacci(20)", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			fibonacci(20)
		}
	})
	// b.RunParallel(func(b *testing.PB) {
	// 	for b.Next() {
	// 		fibonacci(30)
	// 	}
	// })
}

func BenchmarkFibonacci20(b *testing.B) {
	for b.Loop() {
		fibonacci(20)
	}
}

func BenchmarkFibonacci30(b *testing.B) {
	for i := 0; i < b.N; i++ {
		fibonacci(30)
	}
}
