package cli

import (
	"context"
	"fmt"
	"log"
	"os"
	"os/exec"
	"strings"
)

const (
	exitCodeError = 1
)

// Main is the entry point for the CLI application.
func Main() {
	if err := run(os.Args[1:]); err != nil {
		log.Printf("Error: %v", err)
		os.Exit(exitCodeError)
	}
}

func run(args []string) error {
	if len(args) == 0 {
		printUsage()
		return nil
	}

	switch args[0] {
	case "run":
		return runBenchmark(args[1:])
	default:
		return fmt.Errorf("unknown command: %q. Available commands: run", args[0])
	}
}

func printUsage() {
	fmt.Println("codspeed-go - A CLI for running Go benchmarks")
	fmt.Println("Usage:")
	fmt.Println("  codspeed-go run [benchmark-pattern] - Run benchmarks and generate CodSpeed report")
}

func buildBenchmarkArgs(pattern string) []string {
	args := []string{"test", "-bench=."}
	if pattern != "" {
		args[1] = "-bench=" + pattern
	}
	return args
}

func runBenchmark(args []string) error {
	pattern := strings.Join(args, " ")
	cmdArgs := append(buildBenchmarkArgs(pattern), "-json")

	log.Printf("Running benchmark command: go %s", strings.Join(cmdArgs, " "))

	cmd := exec.CommandContext(context.Background(), "go", cmdArgs...)
	cmd.Stderr = os.Stderr

	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return fmt.Errorf("creating stdout pipe: %w", err)
	}

	if err := cmd.Start(); err != nil {
		return fmt.Errorf("starting benchmark: %w", err)
	}

	log.Printf("Parsing benchmark results...")
	parser := NewBenchmarkParser()
	results, err := parser.ParseJSONStream(stdout)
	if err != nil {
		_ = cmd.Wait()
		return fmt.Errorf("parsing benchmark results: %w", err)
	}

	if err := cmd.Wait(); err != nil {
		return fmt.Errorf("running benchmark: %w", err)
	}

	if len(results) == 0 {
		log.Printf("No benchmark results found")
		return nil
	}

	log.Printf("Found %d benchmark results, generating report...", len(results))
	rawBenchmarks := parser.ConvertToRawWalltimeBenchmarks(results)
	if err := GenerateCodspeedWalltimeReport(rawBenchmarks); err != nil {
		return fmt.Errorf("generating CodSpeed report: %w", err)
	}

	log.Printf("Successfully processed %d benchmark results", len(results))
	return nil
}
