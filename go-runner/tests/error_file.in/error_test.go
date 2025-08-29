package error_file

import "testing"

func BenchmarkErrorFile(b *testing.B) {
	b.Error("this_should_be_in_stdout")
}
