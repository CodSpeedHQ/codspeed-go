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

type codspeed struct {
	instrument_hooks *InstrumentHooks

	codspeedTimePerRoundNs []time.Duration
	codspeedItersPerRound  []int64

	startTimestamp  uint64
	startTimestamps []uint64
	stopTimestamps  []uint64

	// Indicates whether a measurement has been saved already. This aims to prevent saving measurements
	// twice, because `b.Loop()` saves them internally as well but is also called from runN
	savedMeasurement bool

	// The start time of the first b.Loop() call. This includes the benchmark execution
	// time, including the overhead of start/stop the timer each loop iteration.
	loopStartTime time.Time
}

// This is used to determine the maximum time a benchmark is allowed to run, with the overhead of our
// modifications in the testing package included. When using a benchtime=3s and multiplier of 3, the
// benchmark can run at most 9s.
const BenchMaxTimeMult = 3

// Modified version of the `stopOrScaleLoop` function to also take into account the
// overhead of start/stop the timer each loop iteration.
//
// If we have large setups/teardowns within the loop, they won't count as benchmark time
// which could cause the benchmark to run for too long. If 10k iterations took 1s with
// the upstream testing package, then it could take 5s with CodSpeed. To limit the overhead,
// we exit early if we exceed the maximum time
//
// There are 2 differences compared to the upstream version:
// 1. We check if we exceeded the time upon entry
// 2. We use the _actual_ time to determine b.loop.n to avoid running too long
func (b *B) stopOrScaleBLoopCodspeed() bool {
	// The total duration must be at most N times the requested benchtime
	actualT := time.Since(b.loopStartTime)
	if actualT >= b.benchTime.d*BenchMaxTimeMult {
		return false
	}

	t := b.Elapsed()
	if t >= b.benchTime.d {
		// We've reached the target
		return false
	}

	// Loop scaling
	goalns := b.benchTime.d.Nanoseconds()
	prevIters := int64(b.loop.n)
	b.loop.n = uint64(predictN(goalns, prevIters, actualT.Nanoseconds(), prevIters))
	if b.loop.n&loopPoisonMask != 0 {
		// The iteration count should never get this high, but if it did we'd be
		// in big trouble.
		panic("loop iteration target overflow")
	}
	return true
}

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

func (b *B) sendAccumulatedTimestamps() {
	for i := 0; i < len(b.startTimestamps); i++ {
		b.instrument_hooks.AddBenchmarkTimestamps(
			b.startTimestamps[i],
			b.stopTimestamps[i],
		)
	}
	b.startTimestamps = b.startTimestamps[:0]
	b.stopTimestamps = b.stopTimestamps[:0]
}

func (b *B) SaveMeasurement() {
	if b.savedMeasurement {
		return
	}
	b.savedMeasurement = true

	// WARN: This function must not be called if the timer is on, because we
	// would read an incomplete b.duration value.
	if b.timerOn {
		panic("SaveMeasurement called with timer on")
	}

	// For b.N loops: This will be called in runN which sets b.N to the number of iterations.
	// For b.Loop() loops: loopSlowPath sets b.N to 0 to prevent b.N loops within b.Loop. However, since
	// we're starting/stopping the timer for each iteration in the b.Loop() loop, we can use 1 as
	// the number of iterations for this round.
	timeSinceStart := highPrecisionTimeSince(b.start)

	// If this gets called from b.Loop(), we have to take the duration compared to the previous StartTimer,
	// if it's called from runN, we can use b.duration
	duration := time.Duration(0)
	if b.N == 0 {
		duration = timeSinceStart
	} else {
		duration = b.duration
	}

	b.codspeedItersPerRound = append(b.codspeedItersPerRound, max(int64(b.N), 1))
	b.codspeedTimePerRoundNs = append(b.codspeedTimePerRoundNs, duration)
}

// StopTimer stops timing a test. This can be used to pause the timer
// while performing steps that you don't want to measure.
func (b *B) StopTimerWithoutMarker() {
	if b.timerOn {
		timeSinceStart := highPrecisionTimeSince(b.start)
		b.duration += timeSinceStart
		// runtime.ReadMemStats(&memStats)
		// b.netAllocs += memStats.Mallocs - b.startAllocs
		// b.netBytes += memStats.TotalAlloc - b.startBytes
		b.timerOn = false
		// If we hit B.Loop with the timer stopped, fail.
		// b.loop.i |= loopPoisonTimer
	}
}

// StartTimer starts timing a test. This function is called automatically
// before a benchmark starts, but it can also be used to resume timing after
// a call to [B.StopTimer].
func (b *B) StartTimerWithoutMarker() {
	if !b.timerOn {
		// runtime.ReadMemStats(&memStats)
		// b.startAllocs = memStats.Mallocs
		// b.startBytes = memStats.TotalAlloc
		b.start = highPrecisionTimeNow()
		b.timerOn = true
		b.savedMeasurement = false
		// b.loop.i &^= loopPoisonTimer
	}
}

func runBenchmarkWithWarmup(b *B) {
	warmupD := b.benchTime.d / 10
	warmupN := int64(1)
	for n := int64(1); !b.failed && b.duration < warmupD && n < 1e9; {
		last := n
		// Predict required iterations.
		goalns := warmupD.Nanoseconds()
		prevIters := int64(b.N)
		n = int64(predictN(goalns, prevIters, b.duration.Nanoseconds(), last))
		b.runN(int(n))
		warmupN = n
	}

	// Reset the fields from the warmup run
	b.ResetTimer()

	// Final run:
	benchD := b.benchTime.d
	benchN := predictN(benchD.Nanoseconds(), int64(b.N), b.duration.Nanoseconds(), warmupN)

	// When we have a very slow benchmark (e.g. taking 500ms), we have to:
	// 1. Reduce the number of rounds to not slow down the process (e.g. by executing a 1s bench 100 times)
	// 2. Not end up with roundN of 0 when dividing benchN (which can be < 100) by rounds
	const minRounds = 100
	var rounds int
	var roundN int
	if benchN < minRounds {
		rounds = benchN
		roundN = 1
	} else {
		rounds = minRounds
		roundN = benchN / int(rounds)
	}

	b.codspeed.instrument_hooks.StartBenchmark()
	for range rounds {
		b.runN(int(roundN))
	}
	b.codspeed.instrument_hooks.StopBenchmark()
	b.sendAccumulatedTimestamps()
}
