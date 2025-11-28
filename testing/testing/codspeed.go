package testing

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

type GoRunnerMetadata struct {
	ProfileFolder       string `json:"profile_folder"`
	RelativePackagePath string `json:"relative_package_path"`
}

func findGoRunnerMetadata() (*GoRunnerMetadata, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return nil, err
	}

	// Search up the directory tree for go-runner.metadata
	currentDir := cwd
	for {
		metadataPath := filepath.Join(currentDir, "go-runner.metadata")
		data, err := os.ReadFile(metadataPath)
		if err == nil {
			var metadata GoRunnerMetadata
			err = json.Unmarshal(data, &metadata)
			if err != nil {
				return nil, err
			}
			return &metadata, nil
		}

		parentDir := filepath.Dir(currentDir)
		if parentDir == currentDir {
			// Reached the root directory
			break
		}
		currentDir = parentDir
	}

	return nil, os.ErrNotExist
}

func getGitRelativePath(absPath string) string {
	canonicalizedAbsPath, err := filepath.EvalSymlinks(absPath)
	if err != nil {
		panic(fmt.Sprintf("failed to evaluate symlinks for path %s: %v", absPath, err))
	}

	cwd, err := os.Getwd()
	if err != nil {
		panic(fmt.Sprintf("failed to get current working directory: %v", err))
	}

	cwdRelativePath, err := filepath.Rel(cwd, canonicalizedAbsPath)
	if err != nil {
		panic(fmt.Sprintf("failed to compute relative path from %s to %s: %v", cwd, canonicalizedAbsPath, err))
	}

	goRunnerMetadata, err := findGoRunnerMetadata()
	if err != nil {
		panic(fmt.Sprintf("failed to find go-runner metadata: %v", err))
	}

	return filepath.Join(goRunnerMetadata.RelativePackagePath, cwdRelativePath)
}

// If the benchmark execution failed, we have to ensure to stop the benchmark, which
// will send the event to the runner to also stop perf. Otherwise we could possibly
// sample a lot of data that isn't relevant. Additionally, we want to ensure that
// the emitted markers are correct (otherwise we'd have a SampleStart without a SampleStop).
func ensureBenchmarkIsStopped(b *B) {
	b.codspeed.instrument_hooks.StopBenchmark()
}

func (b *B) AddBenchmarkMarkers(endTimestamp uint64) {
	if b.startTimestamp >= endTimestamp {
		// This should never happen, unless we have a bug in the timer logic.
		panic(fmt.Sprintf("Invalid benchmark timestamps: start timestamp (%d) is greater than or equal to end timestamp (%d)", b.startTimestamp, endTimestamp))
	}

	b.startTimestamps = append(b.startTimestamps, b.startTimestamp)
	b.stopTimestamps = append(b.stopTimestamps, endTimestamp)

	// Reset to prevent accidental reuse
	b.startTimestamp = 0
}
