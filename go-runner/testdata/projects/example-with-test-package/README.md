# Example: Test Package Name with `_test` Suffix

This example demonstrates the issue reported in https://github.com/CodSpeedHQ/codspeed-go/issues/23

## The Issue

In Go, it's common to use the `_test` suffix for package names in test files to perform "black-box testing". This means testing only the public API of a package without accessing internal implementation details.

For example:
- `math.go` declares `package math`
- `math_test.go` declares `package math_test` (note the `_test` suffix)

This is a recommended Go testing pattern (see [testpackage linter](https://github.com/maratori/testpackage)), but the codspeed-go runner doesn't recognize benchmarks in `*_test` packages correctly.

## How to Test

### Standard Go (should work):
```bash
cd go-runner/testdata/projects/example-with-test-package
go test -bench=.
```

Expected output:
```
BenchmarkAdd-X      xxxxx    xxx ns/op
BenchmarkMultiply-X xxxxx    xxx ns/op
```

### CodSpeed Go Runner (before we supported external test packages)
```bash
cd go-runner/testdata/projects/example-with-test-package
export CODSPEED_LOG=debug
/path/to/codspeed-go/target/debug/codspeed-go-runner test -bench=.
```

Expected behavior: Should discover and run both benchmarks (BenchmarkAdd and BenchmarkMultiply).

Actual behavior:
```
[DEBUG codspeed_go_runner::builder::discovery] Skipping package without test files: math
[DEBUG codspeed_go_runner::builder::discovery] Skipping package without test files: math_test
[INFO  codspeed_go_runner] Discovered 0 packages
[INFO  codspeed_go_runner] Total benchmarks discovered: 0
```

The runner skips the `math_test` package even though it contains valid benchmarks, resulting in 0 benchmarks discovered.

## Files

- `math.go` - Implementation with `package math`
- `math_test.go` - Tests with `package math_test` (black-box testing pattern)
- `go.mod` - Minimal Go module definition
