package main

import "testing"

func helper(b *testing.B) {
	// This function receives testing.B as parameter
}

func BenchmarkInvalid(b *testing.B) {
	helper(b) // This is invalid - passing testing.B to another function
	for i := 0; i < b.N; i++ {
		// Some work
	}
}
