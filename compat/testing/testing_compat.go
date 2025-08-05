//go:build !codspeed
// +build !codspeed

package codspeed

import (
	stdtesting "testing"
)

type B = stdtesting.B
type BenchmarkResult = stdtesting.BenchmarkResult
type Cover = stdtesting.Cover
type CoverBlock = stdtesting.CoverBlock
type F = stdtesting.F
type InternalBenchmark = stdtesting.InternalBenchmark
type InternalExample = stdtesting.InternalExample
type InternalFuzzTarget = stdtesting.InternalFuzzTarget
type InternalTest = stdtesting.InternalTest
type M = stdtesting.M
type PB = stdtesting.PB
type T = stdtesting.T
type TB = stdtesting.TB

func AllocsPerRun(runs int, f func()) (avg float64) {
	return stdtesting.AllocsPerRun(runs, f)
}

func CoverMode() string {
	return stdtesting.CoverMode()
}

func Coverage() float64 {
	return stdtesting.Coverage()
}

func Init() {
	stdtesting.Init()
}

func Main(matchString func(pat, str string) (bool, error), tests []stdtesting.InternalTest, benchmarks []stdtesting.InternalBenchmark, examples []stdtesting.InternalExample) {
	stdtesting.Main(matchString, tests, benchmarks, examples)
}

func RegisterCover(c stdtesting.Cover) {
	stdtesting.RegisterCover(c)
}

func RunBenchmarks(matchString func(pat, str string) (bool, error), benchmarks []stdtesting.InternalBenchmark) {
	stdtesting.RunBenchmarks(matchString, benchmarks)
}

func RunExamples(matchString func(pat, str string) (bool, error), examples []stdtesting.InternalExample) (ok bool) {
	return stdtesting.RunExamples(matchString, examples)
}

func RunTests(matchString func(pat, str string) (bool, error), tests []stdtesting.InternalTest) (ok bool) {
	return stdtesting.RunTests(matchString, tests)
}

func Short() bool {
	return stdtesting.Short()
}

func Testing() bool {
	return stdtesting.Testing()
}

func Verbose() bool {
	return stdtesting.Verbose()
}
