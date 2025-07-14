package cli

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

type BenchmarkStats struct {
	MinNs              float64 `json:"min_ns"`
	MaxNs              float64 `json:"max_ns"`
	MeanNs             float64 `json:"mean_ns"`
	StdevNs            float64 `json:"stdev_ns"`
	Q1Ns               float64 `json:"q1_ns"`
	MedianNs           float64 `json:"median_ns"`
	Q3Ns               float64 `json:"q3_ns"`
	Rounds             uint64  `json:"rounds"`
	TotalTime          float64 `json:"total_time"`
	IqrOutlierRounds   uint64  `json:"iqr_outlier_rounds"`
	StdevOutlierRounds uint64  `json:"stdev_outlier_rounds"`
	IterPerRound       uint64  `json:"iter_per_round"`
	WarmupIters        uint64  `json:"warmup_iters"`
}

type BenchmarkMetadata struct {
	Name string `json:"name"`
	URI  string `json:"uri"`
}

type BenchmarkConfig struct {
	WarmupTimeNs   *int64 `json:"warmup_time_ns"`
	MinRoundTimeNs *int64 `json:"min_round_time_ns"`
	MaxTimeNs      *int64 `json:"max_time_ns"`
	MaxRounds      *int64 `json:"max_rounds"`
}

type CodspeedWalltimeBenchmark struct {
	Metadata BenchmarkMetadata `json:"metadata"`
	Config   BenchmarkConfig   `json:"config"`
	Stats    BenchmarkStats    `json:"stats"`
}

type Creator struct {
	Name    string `json:"name"`
	Version string `json:"version"`
	PID     int    `json:"pid"`
}

type Instrument struct {
	Type string `json:"type"`
}

type CodspeedReport struct {
	Creator    Creator                     `json:"creator"`
	Instrument Instrument                  `json:"instrument"`
	Benchmarks []CodspeedWalltimeBenchmark `json:"benchmarks"`
}

type RawWalltimeBenchmark struct {
	Name         string
	URI          string
	RoundTimesNs []float64
	MeanNs       float64
	MedianNs     float64
	StdevNs      float64
	IterPerRound uint64
}

func computeQuantile(data []float64, quantile float64) float64 {
	n := len(data)
	if n == 0 {
		return 0.0
	}

	pos := quantile * float64(n-1)
	k := int(pos)
	d := pos - float64(k)

	if k+1 < n {
		return data[k] + d*(data[k+1]-data[k])
	}
	return data[k]
}

func computeIQRAndOutliers(timesNs []float64, mean, stdev float64) (q1, q3 float64, iqrOutlierRounds, stdevOutlierRounds uint64) {
	sortedTimes := make([]float64, len(timesNs))
	copy(sortedTimes, timesNs)
	sort.Float64s(sortedTimes)

	q1 = computeQuantile(sortedTimes, 0.25)
	q3 = computeQuantile(sortedTimes, 0.75)

	iqr := q3 - q1
	const (
		iqrOutlierFactor   = 1.5
		stdevOutlierFactor = 3.0
	)

	for _, x := range sortedTimes {
		if x < q1-iqrOutlierFactor*iqr || x > q3+iqrOutlierFactor*iqr {
			iqrOutlierRounds++
		}
		if x < mean-stdevOutlierFactor*stdev || x > mean+stdevOutlierFactor*stdev {
			stdevOutlierRounds++
		}
	}

	return q1, q3, iqrOutlierRounds, stdevOutlierRounds
}

func escapeBackslashes(input string) string {
	return strings.ReplaceAll(input, "\\", "\\\\")
}

func writeCodspeedBenchmarksToJSON(benchmarks []CodspeedWalltimeBenchmark) error {
	creator := Creator{
		Name:    "codspeed-go",
		Version: "0.1.0",
		PID:     os.Getpid(),
	}

	instrument := Instrument{
		Type: "walltime",
	}

	report := CodspeedReport{
		Creator:    creator,
		Instrument: instrument,
		Benchmarks: benchmarks,
	}

	// Determine the output directory
	directory := "."
	if profileFolder := os.Getenv("CODSPEED_PROFILE_FOLDER"); profileFolder != "" {
		directory = profileFolder
	}

	// Create the results directory
	resultsPath := filepath.Join(directory, "results")
	if err := os.MkdirAll(resultsPath, 0o755); err != nil {
		return fmt.Errorf("creating directory %s: %w", resultsPath, err)
	}

	// Create the output file path
	fileName := fmt.Sprintf("%d.json", os.Getpid())
	filePath := filepath.Join(resultsPath, fileName)

	// Marshal and write the JSON report
	jsonData, err := json.MarshalIndent(report, "", "  ")
	if err != nil {
		return fmt.Errorf("marshaling JSON: %w", err)
	}

	if err := os.WriteFile(filePath, jsonData, 0o644); err != nil {
		return fmt.Errorf("writing file %s: %w", filePath, err)
	}

	fmt.Printf("JSON written to %s\n", filePath)
	return nil
}

func min(data []float64) float64 {
	if len(data) == 0 {
		return 0
	}
	min := data[0]
	for _, v := range data[1:] {
		if v < min {
			min = v
		}
	}
	return min
}

func max(data []float64) float64 {
	if len(data) == 0 {
		return 0
	}
	max := data[0]
	for _, v := range data[1:] {
		if v > max {
			max = v
		}
	}
	return max
}

func sum(data []float64) float64 {
	total := 0.0
	for _, v := range data {
		total += v
	}
	return total
}

func GenerateCodspeedWalltimeReport(rawWalltimeBenchmarks []RawWalltimeBenchmark) error {
	var codspeedWalltimeBenchmarks []CodspeedWalltimeBenchmark

	for _, rawBenchmark := range rawWalltimeBenchmarks {
		metadata := BenchmarkMetadata{
			Name: escapeBackslashes(rawBenchmark.Name),
			URI:  escapeBackslashes(rawBenchmark.URI),
		}

		config := BenchmarkConfig{
			WarmupTimeNs:   nil,
			MinRoundTimeNs: nil,
			MaxTimeNs:      nil,
			MaxRounds:      nil,
		}

		totalTime := sum(rawBenchmark.RoundTimesNs)
		mean := rawBenchmark.MeanNs
		median := rawBenchmark.MedianNs
		stdev := rawBenchmark.StdevNs

		q1, q3, iqrOutlierRounds, stdevOutlierRounds := computeIQRAndOutliers(
			rawBenchmark.RoundTimesNs, mean, stdev)

		stats := BenchmarkStats{
			MinNs:              min(rawBenchmark.RoundTimesNs),
			MaxNs:              max(rawBenchmark.RoundTimesNs),
			MeanNs:             mean,
			StdevNs:            stdev,
			Q1Ns:               q1,
			MedianNs:           median,
			Q3Ns:               q3,
			Rounds:             uint64(len(rawBenchmark.RoundTimesNs)),
			TotalTime:          totalTime,
			IqrOutlierRounds:   iqrOutlierRounds,
			StdevOutlierRounds: stdevOutlierRounds,
			IterPerRound:       rawBenchmark.IterPerRound,
			WarmupIters:        0, // TODO: implement warmup_iters
		}

		codspeedBenchmark := CodspeedWalltimeBenchmark{
			Metadata: metadata,
			Config:   config,
			Stats:    stats,
		}

		codspeedWalltimeBenchmarks = append(codspeedWalltimeBenchmarks, codspeedBenchmark)
	}

	return writeCodspeedBenchmarksToJSON(codspeedWalltimeBenchmarks)
}
