package external

// Explicitly using standard library testing package
// This simulates an external library that is already compiled
// and was NOT patched by CodSpeed (like github.com/stretchr/testify)
import stdtesting "testing"

// Suite represents a test suite (similar to testify/suite)
type Suite struct {
	T *stdtesting.T
}

// Run runs the suite with the given testing.T from standard library
// This simulates external libraries like testify that expect *testing.T
// The key is that this function signature uses the STANDARD LIBRARY testing.T
// not the CodSpeed version, because external packages are already compiled
func (s *Suite) Run(t *stdtesting.T) {
	s.T = t
	s.T.Log("Running test suite")
}
