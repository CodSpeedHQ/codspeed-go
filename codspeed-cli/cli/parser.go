package cli

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"regexp"
	"strconv"
	"strings"
	"time"
)

type TestEvent struct {
	Time    time.Time `json:"Time"`
	Action  string    `json:"Action"`
	Package string    `json:"Package"`
	Test    string    `json:"Test,omitempty"`
	Output  string    `json:"Output,omitempty"`
	Elapsed float64   `json:"Elapsed,omitempty"`
}

type BenchmarkResult struct {
	Name       string
	Iterations uint64
	NsPerOp    float64
	Package    string
}

// BenchmarkParser parses go test JSON output to extract benchmark results.
type BenchmarkParser struct {
	benchmarkRegex *regexp.Regexp
}

var benchmarkPattern = regexp.MustCompile(`^([a-zA-Z0-9_]+)-(\d+)\s+\t\s*(\d+)\s+(\d+(?:\.\d+)?)\s+ns/op`)

func NewBenchmarkParser() *BenchmarkParser {
	return &BenchmarkParser{
		benchmarkRegex: benchmarkPattern,
	}
}

// ParseJSONStream parses a stream of JSON test events and extracts benchmark results.
func (p *BenchmarkParser) ParseJSONStream(reader io.Reader) ([]BenchmarkResult, error) {
	var results []BenchmarkResult
	scanner := bufio.NewScanner(reader)

	for scanner.Scan() {
		line := scanner.Text()
		if line == "" {
			continue
		}

		var event TestEvent
		if err := json.Unmarshal([]byte(line), &event); err != nil {
			return nil, fmt.Errorf("parsing JSON line: %w", err)
		}

		if p.isBenchmarkResult(&event) {
			if benchmark := p.parseBenchmarkOutput(event.Output, event.Package); benchmark != nil {
				results = append(results, *benchmark)
			}
		}
	}

	if err := scanner.Err(); err != nil {
		return nil, fmt.Errorf("reading input: %w", err)
	}

	return results, nil
}

func (p *BenchmarkParser) isBenchmarkResult(event *TestEvent) bool {
	return event.Action == "output" &&
		event.Test != "" &&
		strings.Contains(event.Output, "ns/op")
}

func (p *BenchmarkParser) parseBenchmarkOutput(output, packageName string) *BenchmarkResult {
	output = strings.TrimSpace(output)

	matches := p.benchmarkRegex.FindStringSubmatch(output)
	if len(matches) != 5 {
		return nil
	}

	benchmarkName := matches[1]

	iterations, err := strconv.ParseUint(matches[3], 10, 64)
	if err != nil {
		return nil
	}

	nsPerOp, err := strconv.ParseFloat(matches[4], 64)
	if err != nil {
		return nil
	}

	return &BenchmarkResult{
		Name:       benchmarkName,
		Iterations: iterations,
		NsPerOp:    nsPerOp,
		Package:    packageName,
	}
}

func (p *BenchmarkParser) ConvertToRawWalltimeBenchmarks(results []BenchmarkResult) []RawWalltimeBenchmark {
	if len(results) == 0 {
		return nil
	}

	rawBenchmarks := make([]RawWalltimeBenchmark, 0, len(results))

	for _, result := range results {
		rawBenchmark := RawWalltimeBenchmark{
			Name:         result.Name,
			URI:          fmt.Sprintf("%s.%s", result.Package, result.Name),
			RoundTimesNs: []float64{result.NsPerOp},
			MeanNs:       result.NsPerOp,
			MedianNs:     result.NsPerOp,
			StdevNs:      0.0, // No variance with single measurement
			IterPerRound: result.Iterations,
		}

		rawBenchmarks = append(rawBenchmarks, rawBenchmark)
	}

	return rawBenchmarks
}
