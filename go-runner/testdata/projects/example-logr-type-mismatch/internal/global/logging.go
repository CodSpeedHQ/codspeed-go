package global

import (
	"log"
	"os"
	"sync/atomic"

	"github.com/go-logr/logr"
	"github.com/go-logr/stdr"
)

// internalLogger holds a reference to the logr.Logger used within the global package
var internalLogger = func() *atomic.Pointer[logr.Logger] {
	l := stdr.New(log.New(os.Stderr, "", log.LstdFlags))

	p := new(atomic.Pointer[logr.Logger])
	p.Store(&l)
	return p
}()

// SetInternalLogger sets the internal Logger to l.
func SetInternalLogger(l logr.Logger) {
	internalLogger.Store(&l)
}

// GetInternalLogger returns the internal logger.
func GetInternalLogger() logr.Logger {
	return *internalLogger.Load()
}
