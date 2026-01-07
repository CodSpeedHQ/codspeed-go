package example

import (
	"log"
	"os"
	"sync/atomic"

	"github.com/go-logr/logr"
	"github.com/go-logr/stdr"
)

// globalLogger holds a reference to the logr.Logger
var globalLogger = func() *atomic.Pointer[logr.Logger] {
	l := stdr.New(log.New(os.Stderr, "", log.LstdFlags))

	p := new(atomic.Pointer[logr.Logger])
	p.Store(&l)
	return p
}()

// SetLogger sets the global Logger to l.
func SetLogger(l logr.Logger) {
	globalLogger.Store(&l)
}

// GetLogger returns the global logger.
func GetLogger() logr.Logger {
	return *globalLogger.Load()
}
