package main

import "testing"

func BenchmarkValidMethods(b *testing.B) {
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		// Some work
	}
	b.StopTimer()
	b.ReportAllocs()
	b.StartTimer()
}
