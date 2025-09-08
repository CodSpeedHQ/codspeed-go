package example

import (
	"log/slog"
	"testing"

	"github.com/thejerf/slogassert"
)

func TestWithSlogAssert(t *testing.T) {
	handler := slogassert.NewDefault(t)
	slog.Info("This is a test log message")
	handler.AssertMessage("This is a test log message")
	handler.AssertEmpty()
}

func BenchmarkWithSlogAssert(b *testing.B) {
	for b.Loop() {
		handler := slogassert.NewDefault(b)
		slog.Info("This is a test log message")
		handler.AssertMessage("This is a test log message")
		handler.AssertEmpty()
	}
}
