package global

import (
	"testing"
)

func BenchmarkLogger(b *testing.B) {
	logger := GetInternalLogger()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		logger.Info("benchmark message")
	}
}
