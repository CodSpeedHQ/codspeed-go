package example

import (
	"strconv"
	"strings"
	"testing"
)

func rleBad(s string) string {
	if len(s) == 0 {
		return ""
	}
	var out string
	run := 1
	for i := 1; i <= len(s); i++ {
		if i < len(s) && s[i] == s[i-1] {
			run++
			continue
		}
		out += string(s[i-1])
		out += strconv.Itoa(run)
		out += "|"
		run = 1
	}
	return out
}

func BenchmarkRLEBad(b *testing.B) {
	in := strings.Repeat("AAAABBBCCDAA", 200)
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		_ = rleBad(in)
	}
}

func leakyFunction() {
	s := make([]string, 3)
	for i := 0; i < 10000000; i++ {
		s = append(s, "magical pandas")
	}
}

func BenchmarkLeakyFunction(b *testing.B) {
	for i := 0; i < b.N; i++ {
		leakyFunction()
	}
}
