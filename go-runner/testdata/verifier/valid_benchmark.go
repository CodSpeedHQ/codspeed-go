package main

import "testing"

func BenchmarkValid(b *testing.B) {
	for i := 0; i < b.N; i++ {
		// Some work
	}
	b.ReportAllocs()
}
