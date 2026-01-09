package testing

import (
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"runtime"
	"strings"
	"time"
)

func findGitRoot() (string, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return "", err
	}

	// Search up the directory tree for .git
	currentDir := cwd
	for {
		gitRoot := filepath.Join(currentDir, ".git")
		if _, err := os.Stat(gitRoot); err == nil {
			return currentDir, nil
		}

		parentDir := filepath.Dir(currentDir)
		if parentDir == currentDir {
			// Reached the root directory
			break
		}
		currentDir = parentDir
	}

	return "", os.ErrNotExist
}

func getGitRelativePath(absPath string) string {
	canonicalizedAbsPath, err := filepath.EvalSymlinks(absPath)
	if err != nil {
		panic(fmt.Sprintf("failed to evaluate symlinks for path %s: %v", absPath, err))
	}

	gitRoot, err := findGitRoot()
	if err != nil {
		panic(fmt.Sprintf("failed to find git root: %v", err))
	}

	gitRelativePath, err := filepath.Rel(gitRoot, canonicalizedAbsPath)
	if err != nil {
		panic(fmt.Sprintf("failed to compute relative path from git root %s to %s: %v", gitRoot, canonicalizedAbsPath, err))
	}

	return gitRelativePath
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

func removeFolderFromPath(path string, folder string) string {
	parts := strings.Split(path, string(os.PathSeparator))

	var newParts []string
	for _, p := range parts {
		if p != folder {
			newParts = append(newParts, p)
		}
	}

	return filepath.Join(newParts...)
}

func saveCodspeedResults(b *B, r BenchmarkResult, benchName string) {
	type RawResults struct {
		Name                   string          `json:"name"`
		Uri                    string          `json:"uri"`
		Pid                    int             `json:"pid"`
		CodspeedTimePerRoundNs []time.Duration `json:"codspeed_time_per_round_ns"`
		CodspeedItersPerRound  []int64         `json:"codspeed_iters_per_round"`
	}

	// Find the filename of the benchmark file
	var benchFile string
	if b.benchFunc != nil {
		pc := reflect.ValueOf(b.benchFunc).Pointer()
		fn := runtime.FuncForPC(pc)
		if fn == nil {
			return
		}

		file, _ := fn.FileLine(pc)
		benchFile = file
	}

	if benchFile == "" {
		panic("Could not determine benchmark file name")
	}
	relativeBenchFile := getGitRelativePath(benchFile)

	// Build custom bench name with :: separator
	var nameParts []string
	current := &b.common
	for current.parent != nil {
		// Extract the sub-benchmark part by removing parent prefix
		parentName := current.parent.name
		if strings.HasPrefix(current.name, parentName+"/") {
			subName := strings.TrimPrefix(current.name, parentName+"/")
			nameParts = append([]string{subName}, nameParts...)
		} else {
			nameParts = append([]string{current.name}, nameParts...)
		}

		if current.parent.name == "Main" {
			break
		}
		current = current.parent
	}
	benchName = strings.Join(nameParts, "::")
	benchUri := fmt.Sprintf("%s::%s", relativeBenchFile, benchName)

	rawResults := RawResults{
		Name:                   benchName,
		Uri:                    benchUri,
		Pid:                    os.Getpid(),
		CodspeedTimePerRoundNs: r.CodspeedTimePerRoundNs,
		CodspeedItersPerRound:  r.CodspeedItersPerRound,
	}

	profileDir := "@@CODSPEED_PROFILE_DIR@@" // NOTE: This will be replaced by the go-runner
	if err := os.MkdirAll(filepath.Join(profileDir, "raw_results"), 0755); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create raw results directory: %v\n", err)
		return
	}
	// Generate random filename to avoid any overwrites
	randomBytes := make([]byte, 16)
	if _, err := rand.Read(randomBytes); err != nil {
		fmt.Fprintf(os.Stderr, "failed to generate random filename: %v\n", err)
		return
	}
	rawResultsFile := filepath.Join(profileDir, "raw_results", fmt.Sprintf("%s.json", hex.EncodeToString(randomBytes)))
	file, err := os.Create(rawResultsFile)
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to create raw results file: %v\n", err)
		return
	}
	output, err := json.MarshalIndent(rawResults, "", "  ")
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to marshal raw results: %v\n", err)
		file.Close()
		return
	}
	// FIXME: Don't overwrite the file if it already exists
	if _, err := file.Write(output); err != nil {
		fmt.Fprintf(os.Stderr, "failed to write raw results: %v\n", err)
		file.Close()
		return
	}
	defer file.Close()

	// Send pid and executed benchmark to the runner
	b.codspeed.instrument_hooks.SetExecutedBenchmark(uint32(os.Getpid()), benchUri)
}
