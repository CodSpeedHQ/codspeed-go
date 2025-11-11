package main

import (
	"testing"
)

func BenchmarkGetValue(b *testing.B) {
	for i := 0; i < b.N; i++ {
		GetValue()
	}
}
