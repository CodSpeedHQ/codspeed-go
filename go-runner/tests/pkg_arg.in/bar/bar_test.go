package bar

import "testing"

func BenchmarkBar1(b *testing.B) {
	b.Log("bar_bench_1")
	for i := 0; i < b.N; i++ {
		// Some work
		_ = i * 4
	}
}

func BenchmarkBar2(b *testing.B) {
	b.Log("bar_bench_2")
	for i := 0; i < b.N; i++ {
		// Some work
		_ = i * 5
	}
}
