package mylib

import (
	"testing"
)

// state is unexported and only accessible to test files
var testState = 0

// SetTestState is an unexported function defined in internal test file
// This function is only available during test builds
func SetTestState(value int) {
	testState = value
}

// GetTestState is an unexported function defined in internal test file
func GetTestState() int {
	return testState
}

// InternalTestHelper is another test helper in the internal test file
func InternalTestHelper(b *testing.B, value int) {
	SetTestState(value)
}

func BenchmarkSomething(b *testing.B) {
	b.Run("test", func(b *testing.B) {
		InternalTestHelper(b, 10)
	})
}
