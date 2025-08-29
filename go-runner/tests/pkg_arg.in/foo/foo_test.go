package foo

import "testing"

func BenchmarkFoo1(b *testing.B) {
	b.Log("foo_bench_1")
	for i := 0; i < b.N; i++ {
		// Some work
		_ = i * 2
	}
}

func BenchmarkFoo2(b *testing.B) {
	b.Log("foo_bench_2")
	for i := 0; i < b.N; i++ {
		// Some work
		_ = i * 3
	}
}
