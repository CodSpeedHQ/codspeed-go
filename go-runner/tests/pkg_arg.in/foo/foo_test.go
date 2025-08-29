package foo

import "testing"

func BenchmarkFoo1(b *testing.B) {
	b.Log("foo_bench_1_should_not_be_in_stdout")
	for i := 0; i < b.N; i++ {
		// Some work
		_ = i * 2
	}
}

func BenchmarkFoo2(b *testing.B) {
	b.Log("foo_bench_2_should_not_be_in_stdout")
	for i := 0; i < b.N; i++ {
		// Some work
		_ = i * 3
	}
}