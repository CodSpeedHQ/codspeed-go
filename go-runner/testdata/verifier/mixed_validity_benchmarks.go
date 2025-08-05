package main

import "testing"

func helper(b *testing.B) {
	// Helper function
}

func BenchmarkValid(b *testing.B) {
	for i := 0; i < b.N; i++ {
		// Some work
	}
}

func BenchmarkInvalid(b *testing.B) {
	helper(b) // Invalid
	for i := 0; i < b.N; i++ {
		// Some work
	}
}
