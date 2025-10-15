// This file uses the _test package suffix, which is a common Go pattern
// for black-box testing. This tests the public API of the math package
// without accessing internal implementation details.
package math_test

import "testing"
import "example-with-test-package"

func BenchmarkAdd(b *testing.B) {
	for i := 0; i < b.N; i++ {
		math.Add(5, 10)
	}
}

func BenchmarkMultiply(b *testing.B) {
	for i := 0; i < b.N; i++ {
		math.Multiply(5, 10)
	}
}
