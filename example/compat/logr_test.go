package example

import (
	"fmt"
	"testing"

	"github.com/go-logr/logr/testr"
)

func TestLogrTestr(t *testing.T) {
	logger := testr.NewWithOptions(t, testr.Options{Verbosity: 2})

	logger.Info("starting test", "test", "logr")
	logger.V(0).Info("V(0).info")
	logger.V(1).Info("V(1).info")
	logger.Error(fmt.Errorf("test error"), "error")

	childLogger := logger.WithValues("request_id", "12345")
	childLogger.Info("child logger message")

	loggerWithName := childLogger.WithName("child")
	loggerWithName.Info("named child logger")
}

func BenchmarkLogrTestr(b *testing.B) {
	t := &testing.T{}
	logger := testr.New(t)

	for b.Loop() {
		// Log operations within the benchmark
		logger.Info("benchmark message")
		logger.V(1).Info("verbose benchmark")
		childLogger := logger.WithValues("id", "test")
		childLogger.Info("child message")
	}
}
