package examplewithvendor

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

// TestAdd demonstrates using testify assert (vendored dependency)
func TestAdd(t *testing.T) {
	assert.Equal(t, 5, Add(2, 3))
}

// BenchmarkLocalFib should be executed
func BenchmarkLocalFib(b *testing.B) {
	for i := 0; i < b.N; i++ {
		LocalFib(10)
	}
}

// BenchmarkAddWithAssert uses testify assert in a benchmark
func BenchmarkAddWithAssert(b *testing.B) {
	for i := 0; i < b.N; i++ {
		result := Add(2, 3)
		assert.Equal(b, 5, result)
	}
}
