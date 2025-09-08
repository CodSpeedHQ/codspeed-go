//go:build codspeed
// +build codspeed

package slogtest

import (
	"log/slog"

	testing "github.com/CodSpeedHQ/codspeed-go/testing/testing"
	slogtest "github.com/CodSpeedHQ/codspeed-go/testing/testing/slogtest"
)

func Run(t *testing.T, newHandler func(*testing.T) slog.Handler, result func(*testing.T) map[string]any) {
	slogtest.Run(t, newHandler, result)
}
func TestHandler(h slog.Handler, results func() []map[string]any) error {
	return slogtest.TestHandler(h, results)
}
