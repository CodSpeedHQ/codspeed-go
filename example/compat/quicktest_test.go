package example

import (
	"testing"

	qt "github.com/frankban/quicktest"
)

func TestQuicktest(t *testing.T) {
	t.Run("numbers", func(t *testing.T) {
		c := qt.New(t)

		c.Assert("hello world", qt.Contains, "world")
		c.Assert([]int{3, 5, 7, 99}, qt.Contains, 7)
		c.Assert([]int{3, 5, 8}, qt.All(qt.Not(qt.Equals)), 0)
	})
}

func BenchmarkQuicktest(b *testing.B) {
	for b.Loop() {
		c := qt.New(b)
		c.Assert("hello world", qt.Contains, "world")
		c.Assert([]int{3, 5, 7, 99}, qt.Contains, 7)
		c.Assert([]int{3, 5, 8}, qt.All(qt.Not(qt.Equals)), 0)
	}
}
