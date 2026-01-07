package external

import "testing"

// Suite represents a test suite (similar to testify/suite)
type Suite struct {
	T *testing.T
}

// Run runs the suite with the given testing.T from standard library
// This simulates external libraries like testify that expect *testing.T
func (s *Suite) Run(t *testing.T) {
	s.T = t
	s.T.Log("Running test suite")
}
