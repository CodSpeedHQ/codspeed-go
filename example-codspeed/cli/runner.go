//go:build codspeed
// +build codspeed

package main

import (
	"fmt"
	"io"
	"reflect"
	"time"

	example "example"

	codspeed_testing "github.com/CodSpeedHQ/codspeed-go/testing/testing"
)

type corpusEntry = struct {
	Parent     string
	Path       string
	Data       []byte
	Values     []any
	Generation int
	IsSeed     bool
}

type simpleDeps struct{}

func (d simpleDeps) ImportPath() string                          { return "" }
func (d simpleDeps) MatchString(pat, str string) (bool, error)   { return true, nil }
func (d simpleDeps) SetPanicOnExit0(bool)                        {}
func (d simpleDeps) StartCPUProfile(io.Writer) error             { return nil }
func (d simpleDeps) StopCPUProfile()                             {}
func (d simpleDeps) StartTestLog(io.Writer)                      {}
func (d simpleDeps) StopTestLog() error                          { return nil }
func (d simpleDeps) WriteProfileTo(string, io.Writer, int) error { return nil }

func (d simpleDeps) CoordinateFuzzing(
	fuzzTime time.Duration,
	fuzzN int64,
	minimizeTime time.Duration,
	minimizeN int64,
	parallel int,
	corpus []corpusEntry,
	types []reflect.Type,
	corpusDir,
	cacheDir string,
) error {
	return nil
}
func (d simpleDeps) RunFuzzWorker(fn func(corpusEntry) error) error {
	return nil
}
func (d simpleDeps) ReadCorpus(dir string, types []reflect.Type) ([]corpusEntry, error) {
	return nil, nil
}
func (d simpleDeps) CheckCorpus(vals []any, types []reflect.Type) error {
	return nil
}
func (d simpleDeps) ResetCoverage()    {}
func (d simpleDeps) SnapshotCoverage() {}
func (d simpleDeps) InitRuntimeCoverage() (mode string, tearDown func(coverprofile string, gocoverdir string) (string, error), snapcov func() float64) {
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

	for i := 0; i < len(benchmarks); i++ {
		fmt.Printf("Benchmark %d: %s\n", i, benchmarks[i].Name)
	}

	m := codspeed_testing.MainStart(simpleDeps{}, tests, benchmarks, fuzzTargets, examples)
	m.Run()
}
