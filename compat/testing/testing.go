//go:build codspeed
// +build codspeed

package codspeed

import (
	codspeed_testing "github.com/CodSpeedHQ/codspeed-go/testing/testing"
)

type B = codspeed_testing.B
type BenchmarkResult = codspeed_testing.BenchmarkResult
type Cover = codspeed_testing.Cover
type CoverBlock = codspeed_testing.CoverBlock
type F = codspeed_testing.F
type InternalBenchmark = codspeed_testing.InternalBenchmark
type InternalExample = codspeed_testing.InternalExample
type InternalFuzzTarget = codspeed_testing.InternalFuzzTarget
type InternalTest = codspeed_testing.InternalTest
type M = codspeed_testing.M
type PB = codspeed_testing.PB
type T = codspeed_testing.T
type TB = codspeed_testing.TB

func AllocsPerRun(runs int, f func()) (avg float64) {
	return codspeed_testing.AllocsPerRun(runs, f)
}

func CoverMode() string {
	return codspeed_testing.CoverMode()
}

func Coverage() float64 {
	return codspeed_testing.Coverage()
}

func Init() {
	codspeed_testing.Init()
}

func Main(matchString func(pat, str string) (bool, error), tests []codspeed_testing.InternalTest, benchmarks []codspeed_testing.InternalBenchmark, examples []codspeed_testing.InternalExample) {
	codspeed_testing.Main(matchString, tests, benchmarks, examples)
}

func RegisterCover(c codspeed_testing.Cover) {
	codspeed_testing.RegisterCover(c)
}

func RunBenchmarks(matchString func(pat, str string) (bool, error), benchmarks []codspeed_testing.InternalBenchmark) {
	codspeed_testing.RunBenchmarks(matchString, benchmarks)
}

func RunExamples(matchString func(pat, str string) (bool, error), examples []codspeed_testing.InternalExample) (ok bool) {
	return codspeed_testing.RunExamples(matchString, examples)
}

func RunTests(matchString func(pat, str string) (bool, error), tests []codspeed_testing.InternalTest) (ok bool) {
	return codspeed_testing.RunTests(matchString, tests)
}

func Short() bool {
	return codspeed_testing.Short()
}

func Testing() bool {
	return codspeed_testing.Testing()
}

func Verbose() bool {
	return codspeed_testing.Verbose()
}
