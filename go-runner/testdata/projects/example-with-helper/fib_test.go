package examplewithhelper

import "testing"

func BenchmarkFib(b *testing.B) {
	BenchmarkFibHelper(b)
}
