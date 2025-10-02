package timing

import (
	"testing"
	"time"
)

func busyWait(duration time.Duration) {
	start := time.Now()
	for time.Since(start) < duration {
		// Busy wait loop
	}
}

func BenchmarkSleep100ns(b *testing.B) {
	for i := 0; i < b.N; i++ {
		busyWait(100 * time.Nanosecond)
	}
}

func BenchmarkSleep100ns_Loop(b *testing.B) {
	for b.Loop() {
		busyWait(100 * time.Nanosecond)
	}
}

func BenchmarkSleep1us(b *testing.B) {
	for i := 0; i < b.N; i++ {
		busyWait(1 * time.Microsecond)
	}
}

func BenchmarkSleep1us_Loop(b *testing.B) {
	for b.Loop() {
		busyWait(1 * time.Microsecond)
	}
}

func BenchmarkSleep10us(b *testing.B) {
	for i := 0; i < b.N; i++ {
		busyWait(10 * time.Microsecond)
	}
}

func BenchmarkSleep10us_Loop(b *testing.B) {
	for b.Loop() {
		busyWait(10 * time.Microsecond)
	}
}

func BenchmarkSleep100us(b *testing.B) {
	for i := 0; i < b.N; i++ {
		busyWait(100 * time.Microsecond)
	}
}

func BenchmarkSleep100us_Loop(b *testing.B) {
	for b.Loop() {
		busyWait(100 * time.Microsecond)
	}
}

func BenchmarkSleep1ms(b *testing.B) {
	for i := 0; i < b.N; i++ {
		busyWait(1 * time.Millisecond)
	}
}

func BenchmarkSleep1ms_Loop(b *testing.B) {
	for b.Loop() {
		busyWait(1 * time.Millisecond)
	}
}

func BenchmarkSleep10ms(b *testing.B) {
	for i := 0; i < b.N; i++ {
		busyWait(10 * time.Millisecond)
	}
}

func BenchmarkSleep10ms_Loop(b *testing.B) {
	for b.Loop() {
		busyWait(10 * time.Millisecond)
	}
}

func BenchmarkSleep50ms(b *testing.B) {
	for i := 0; i < b.N; i++ {
		busyWait(50 * time.Millisecond)
	}
}

func BenchmarkSleep50ms_Loop(b *testing.B) {
	for b.Loop() {
		busyWait(50 * time.Millisecond)
	}
}
