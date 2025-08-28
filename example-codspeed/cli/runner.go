//go:build codspeed
// +build codspeed

package main

import (
	"io"
	"reflect"
	"regexp"
	"time"

	codspeed_testing "github.com/CodSpeedHQ/codspeed-go/testing/testing"

	// Import parent package containing the benchmarks
	example "example"
)

// TestDeps is an implementation of the testing.testDeps interface,
// suitable for passing to [testing.MainStart].
//
// It is partially copied from `testing/internal/testdeps/deps.go`, and
// only implements the minimum required functionality.
type TestDeps struct{}

func (d TestDeps) ImportPath() string { return "" }

var matchPat string
var matchRe *regexp.Regexp

func (TestDeps) MatchString(pat, str string) (result bool, err error) {
	if matchRe == nil || matchPat != pat {
		matchPat = pat
		matchRe, err = regexp.Compile(matchPat)
		if err != nil {
			return
		}
	}
	return matchRe.MatchString(str), nil
}

func (d TestDeps) SetPanicOnExit0(bool)                        {}
func (d TestDeps) StartCPUProfile(io.Writer) error             { return nil }
func (d TestDeps) StopCPUProfile()                             {}
func (d TestDeps) StartTestLog(io.Writer)                      {}
func (d TestDeps) StopTestLog() error                          { return nil }
func (d TestDeps) WriteProfileTo(string, io.Writer, int) error { return nil }

type corpusEntry = struct {
	Parent     string
	Path       string
	Data       []byte
	Values     []any
	Generation int
	IsSeed     bool
}

func (d TestDeps) CoordinateFuzzing(fuzzTime time.Duration, fuzzN int64, minimizeTime time.Duration, minimizeN int64, parallel int, corpus []corpusEntry, types []reflect.Type, corpusDir, cacheDir string) error {
	return nil
}
func (d TestDeps) RunFuzzWorker(fn func(corpusEntry) error) error { return nil }
func (d TestDeps) ReadCorpus(dir string, types []reflect.Type) ([]corpusEntry, error) {
	return nil, nil
}
func (d TestDeps) CheckCorpus(vals []any, types []reflect.Type) error {
	return nil
}
func (d TestDeps) ResetCoverage()    {}
func (d TestDeps) SnapshotCoverage() {}
func (d TestDeps) InitRuntimeCoverage() (mode string, tearDown func(coverprofile string, gocoverdir string) (string, error), snapcov func() float64) {
	return "", nil, nil
}

func main() {
	var tests = []codspeed_testing.InternalTest{}
	var fuzzTargets = []codspeed_testing.InternalFuzzTarget{}
	var examples = []codspeed_testing.InternalExample{}
	var benchmarks = []codspeed_testing.InternalBenchmark{
		{
			Name: "BenchmarkFibonacci10",
			F:    example.BenchmarkFibonacci10,
		},
		{
			Name: "BenchmarkFibonacci20",
			F:    example.BenchmarkFibonacci20,
		},
	}

	m := codspeed_testing.MainStart(TestDeps{}, tests, benchmarks, fuzzTargets, examples)
	m.Run()
}
