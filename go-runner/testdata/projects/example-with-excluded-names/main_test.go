package main

import (
	"testing"
)

func BenchmarkProcessData(b *testing.B) {
	for i := 0; i < b.N; i++ {
		ProcessData("example-target")
	}
}
