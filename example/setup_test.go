package example

import (
	"runtime"
	"testing"
)

func BenchmarkLargeSetup(b *testing.B) {
	expensiveOperation()
	b.ResetTimer()

	var result uint64
	for i := 0; i < b.N; i++ {
		result = actualWork()
	}
	runtime.KeepAlive(result)
}

func BenchmarkLargeSetupInLoop(b *testing.B) {
	var result uint64
	for i := 0; i < b.N; i++ {
		b.StopTimer()
		expensiveOperation()
		b.StartTimer()

		result = actualWork()
	}
	runtime.KeepAlive(result)
}
