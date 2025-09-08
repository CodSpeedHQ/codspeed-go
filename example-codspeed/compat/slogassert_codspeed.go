package compat

import (
	"log/slog"

	testing "github.com/CodSpeedHQ/codspeed-go/compat/testing"
	slogassert "github.com/CodSpeedHQ/codspeed-go/pkg/slogassert"
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
