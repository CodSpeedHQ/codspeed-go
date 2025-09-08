package example

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestTestifyWithT(t *testing.T) {
	useAssertWithT(t)
	useRequireWithT(t)
}

func TestTestifyWithNew(t *testing.T) {
	assert := assert.New(t)
	require := require.New(t)

	useAssert(assert)
	useRequire(require)
}

func BenchmarkTestifyWithT(b *testing.B) {
	for b.Loop() {
		useAssertWithT(b)
		useRequireWithT(b)
	}
}

func BenchmarkTestifyWithNew(b *testing.B) {
	assert := assert.New(b)
	require := require.New(b)

	for b.Loop() {
		useAssert(assert)
		useRequire(require)
	}
}

func useAssertWithT(t assert.TestingT) {
	assert.Equal(t, 42, 42, "they should be equal")
	assert.NotEqual(t, 42, 24, "they should not be equal")
	assert.True(t, true, "True is true!")
}

func useAssert(assert *assert.Assertions) {
	assert.Equal(42, 42, "they should be equal")
	assert.NotEqual(42, 24, "they should not be equal")
	assert.True(true, "True is true!")
}

func useRequireWithT(t require.TestingT) {
	require.Equal(t, "hello", "hello", "they should be equal")
	require.NotEqual(t, "hello", "world", "they should not be equal")
}

func useRequire(require *require.Assertions) {
	require.Equal("hello", "hello", "they should be equal")
	require.NotEqual("hello", "world", "they should not be equal")
}
