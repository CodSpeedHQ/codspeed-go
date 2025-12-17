package example

import (
	"errors"
	"log"
	"testing"

	"github.com/go-logr/stdr"
)

func TestStdr(t *testing.T) {
	logger := stdr.New(log.Default())

	logger.Info("info message", "key", "value")
	logger.Error(errors.New("test error"), "error message")

	namedLogger := logger.WithName("myapp")
	namedLogger.Info("named logger message")

	childLogger := namedLogger.WithValues("request_id", "12345")
	childLogger.Info("child logger message")

	childLogger.V(1).Info("verbose message")
}

func TestStdrWithOptions(t *testing.T) {
	options := stdr.Options{
		LogCaller: stdr.Error,
		Depth:     1,
	}

	logger := stdr.NewWithOptions(log.Default(), options)

	logger.Info("info with options")
	logger.Error(errors.New("err"), "error with options")
}

func BenchmarkStdr(b *testing.B) {
	logger := stdr.New(log.Default())

	for b.Loop() {
		logger.Info("benchmark message", "iteration", b.N)
		childLogger := logger.WithValues("id", "test")
		childLogger.Info("child message")
	}
}

func BenchmarkStdrWithName(b *testing.B) {
	logger := stdr.New(log.Default()).WithName("bench")

	for b.Loop() {
		logger.Info("named benchmark", "count", b.N)
		logger.V(1).Info("verbose benchmark")
	}
}
