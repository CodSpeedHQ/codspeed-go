package mylib_test

import (
	"testing"

	"example-external-test-unexported-access"
)

// setupTest is a helper function in the external test package
// that calls unexported functions from the mylib internal test file
func setupTest(value int) {
	mylib.SetTestState(value)
}

// getTestState is a helper function that reads unexported state from internal test
func getTestState() int {
	return mylib.GetTestState()
}

// cleanupTest is a helper that resets state using internal test function
func cleanupTest() {
	mylib.SetTestState(0)
}

func BenchmarkPublicFunction(b *testing.B) {
	setupTest(5)
	defer cleanupTest()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		result := mylib.PublicFunction(getTestState())
		_ = result
	}
}

func BenchmarkPublicFunctionMultiple(b *testing.B) {
	for j := 1; j <= 10; j++ {
		setupTest(j)
		for i := 0; i < b.N; i++ {
			result := mylib.PublicFunction(j)
			_ = result
		}
	}
	cleanupTest()
}
