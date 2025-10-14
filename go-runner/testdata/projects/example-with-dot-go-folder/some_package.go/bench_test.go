package somepackage

import "testing"

func BenchmarkInsideFolder(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = i * 3
	}
}
