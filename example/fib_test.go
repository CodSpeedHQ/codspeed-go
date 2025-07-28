package example

import "testing"

func BenchmarkFibonacci10(b *testing.B) {
	b.Run("fibonacci(10)", func(b *testing.B) {
		b.Run("fibonacci(10)", func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				fibonacci(10)
			}
		})

	})
}

func BenchmarkFibonacci20_Loop(b *testing.B) {
	for b.Loop() {
		fibonacci(20)
	}
}

func BenchmarkFibonacci20_bN(b *testing.B) {
	for i := 0; i < b.N; i++ {
		fibonacci(20)
	}
}

// func BenchmarkFibonacci30(b *testing.B) {
// 	b.Run("fibonacci(30)", func(b *testing.B) {
// 		this shouldn't be executed
// 	})
// }
