package slogassert

import testing "github.com/CodSpeedHQ/codspeed-go/compat/testing"

func TestNullHandler(t *testing.T) {
	l := NullLogger()

	// Just verify this doesn't crash.
	l.With("x", "y").WithGroup("nope").Debug("no")
}
