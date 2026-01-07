package example_test

import (
	"io"
	"log"

	"github.com/go-logr/stdr"

	example "example.com/logr-test"
)

func ExampleSetLogger() {
	logger := stdr.New(log.New(io.Discard, "", 0))
	example.SetLogger(logger)
}
