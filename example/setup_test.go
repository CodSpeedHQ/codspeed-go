package example

import (
	"runtime"
	"testing"
	"time"
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

	// The `testing` package starts the timer automatically at the beginning of the benchmark. If we try to
	// stop the timer before the expensiveOperation, and then start it again afterwards we'll run into an issue
	// where the difference from the function start to the first b.StopTimer() call is included in the timing.
	// ```
	// 	for i := 0; i < b.N; i++ {
	// 		b.StopTimer()
	// 		expensiveOperation()
	// 		b.StartTimer()
	// 		b.Log("Running iteration", i, b.Elapsed())
	//
	// 		result = actualWork()
	// 		b.Log("Running iteration", i, b.Elapsed())
	// 	}
	// ```
	// This will print two timings:
	// setup_test.go:26: Running iteration 0 782ns
	// setup_test.go:29: Running iteration 0 7.560911ms
	//
	// However, the first timing (782ns) is misleading because it's not related to the benchmark. To avoid this, we have to
	// stop the timer in the beginning of the benchmark function.
	b.StopTimer()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		expensiveOperation()

		b.StartTimer()
		result = actualWork()
		b.StopTimer()
	}
	runtime.KeepAlive(result)
}

func BenchmarkWithOutlierMeasurementTraditional(b *testing.B) {
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		b.StopTimer()
		time.Sleep(10 * time.Millisecond)
		b.StartTimer()

		time.Sleep(1 * time.Millisecond)
	}
}

func BenchmarkWithOutlierMeasurementModern(b *testing.B) {
	b.ResetTimer()
	for b.Loop() {
		b.StopTimer()
		time.Sleep(10 * time.Millisecond)
		b.StartTimer()

		time.Sleep(1 * time.Millisecond)
	}
}

func BenchmarkWithoutStartupModern(b *testing.B) {
	time.Sleep(10 * time.Millisecond)
	b.ResetTimer()
	for b.Loop() {
		time.Sleep(1 * time.Millisecond)
	}
}

func BenchmarkWithoutStartupTraditional(b *testing.B) {
	time.Sleep(10 * time.Millisecond)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		time.Sleep(1 * time.Millisecond)
	}
}

func BenchmarkWithStopTraditional(b *testing.B) {
	time.Sleep(2 * time.Millisecond)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		time.Sleep(1 * time.Millisecond)
	}

	b.StopTimer()
}

func BenchmarkWithStopModern(b *testing.B) {
	time.Sleep(2 * time.Millisecond)

	b.ResetTimer()
	for b.Loop() {
		time.Sleep(1 * time.Millisecond)
	}

	b.StopTimer()
}
