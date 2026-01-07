# Example: External Test Package Calling Internal Test Functions

## The Issue

When running benchmarks in an external test package (`package foo_test`) that calls functions defined in internal test files (`foo_test.go` in `package foo`), the codspeed-go runner fails with compilation errors like:

```
Error: codspeed/main_test_codspeed.go:XX:YY: undefined: mylib.SetTestState
Error: codspeed/main_test_codspeed.go:XX:YY: undefined: mylib.GetTestState
```

### Why This Happens

In Go's testing architecture:

1. **Internal Test Files** (`foo_test.go` in `package foo`):
   - Part of the same package as source code
   - Can define test-only helper functions and variables
   - Only compiled when building with `go test`, not with `go build`

2. **External Test Files** (`foo_test.go` in `package foo_test`):
   - Separate package for black-box testing
   - Can call functions from both the package's source AND internal test files
   - Only available during `go test` builds

3. **codspeed-go's Problem**:
   - The runner renames internal test files from `*_test.go` to `*_codspeed.go`
   - It then uses `go build` (not `go test`) to compile
   - When `go build` compiles, the original `*_test.go` files are ignored
   - The renamed `*_codspeed.go` files are compiled, but they're now regular files with test code
   - External test files that reference functions defined only in internal test files fail

### Example Structure

```
mylib/
├── main.go                 # package mylib - main implementation
├── internal_test.go        # package mylib - internal test helpers (SetTestState, GetTestState)
├── main_test.go            # package mylib_test - external test benchmarks
├── go.mod                  # Module definition
└── README.md              # This file
```

The `internal_test.go` file:
- Declares `package mylib` (internal test)
- Defines helper functions: `SetTestState()`, `GetTestState()`
- These are only available when `go test` is used

The `main_test.go` file:
- Declares `package mylib_test` (external test package)
- Calls `mylib.SetTestState()` and `mylib.GetTestState()` in helper functions
- Has benchmarks that use these helpers
- Works with `go test` because internal test functions are compiled
- Fails with codspeed-go because those functions aren't compiled with `go build`

## How to Test

### Standard Go (works fine):
```bash
cd go-runner/testdata/projects/example-external-test-unexported-access
go test -bench=.
```

Expected behavior: Both benchmarks run successfully.

### CodSpeed Go Runner (currently fails):
```bash
cd go-runner
cargo run -- test -bench=. testdata/projects/example-external-test-unexported-access
```

Expected error:
```
Error: codspeed/main_codspeed.go:XX:YY: undefined: mylib.SetTestState
Error: codspeed/main_codspeed.go:XX:YY: undefined: mylib.GetTestState
```

## Root Cause Analysis

1. **Discovery** finds external test package (`package mylib_test`) in `main_test.go`
2. **Patcher** also processes internal test file `internal_test.go` (renames to `internal_codspeed.go`)
3. **Templater** moves external test file to `codspeed/main_test_codspeed.go`
4. **Build** uses `go build` which:
   - Compiles `internal_codspeed.go` (renamed internal test file) as regular code
   - Compiles `codspeed/main_test_codspeed.go` (external test moved to subdirectory)
   - BUT: The internal test code functions are now in the parent package (`mylib`), not in `package mylib`'s test namespace
   - The external test now can't access these functions because they're in a subdirectory and the package namespace is confused

## Real-World Impact

This affects projects like OpenTelemetry Go (and others) that have:
- Internal test files defining mock/setup functions (e.g., `SetHostIDProvider`)
- External test files in separate packages that call those mock functions
- Benchmark functions in the external test files

## Potential Solutions

1. **Keep internal tests as `*_test.go`**: Don't rename internal test files - find a way to compile test code differently
2. **Merge test setup into main code**: Move internal test helper functions to main package (not ideal)
3. **Use build tags**: Use `//go:build codspeed` to conditionally include test helpers when building with codspeed-go
4. **Generate wrapper code**: Create wrapper functions in the runner that provide the test helpers
