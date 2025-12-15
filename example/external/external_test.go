// This file uses the _test package suffix, which is a common Go pattern
// for black-box testing. This tests the public API of the math package
// without accessing internal implementation details.
package external_test

import (
	"example"
	"testing"
)

func BenchmarkExternalFib(b *testing.B) {
	for i := 0; i < b.N; i++ {
		example.Fibonacci(5)
	}
}
