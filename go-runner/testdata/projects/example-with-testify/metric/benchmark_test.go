package metric

import (
	"testing"

	"github.com/stretchr/testify/suite"
)

// MetricTestSuite demonstrates the type mismatch issue with testify
type MetricTestSuite struct {
	suite.Suite
}

// TestWithSuite demonstrates using testify/suite in a test
func TestWithSuite(t *testing.T) {
	// This will fail when imports are patched because:
	// - t is *"github.com/CodSpeedHQ/codspeed-go/testing/testing".T (after patching)
	// - suite.Run expects *"testing".T from standard library (unpatched)
	suite.Run(t, new(MetricTestSuite))
}

func BenchmarkWithTestifySuite(b *testing.B) {
	// This reproduces the exact error from opentelemetry-go
	// where testify/suite.Run expects *testing.T but receives CodSpeed's version
	suite.Run(&testing.T{}, new(MetricTestSuite))
}
