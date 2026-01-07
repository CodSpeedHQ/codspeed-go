# Example: logr Type Mismatch

This example demonstrates a type mismatch issue that occurs when patching imports for packages that use `github.com/go-logr/logr` types across multiple packages.

## The Issue

When the runner patches imports to replace `github.com/go-logr/logr` with `github.com/CodSpeedHQ/codspeed-go/pkg/logr`:

1. Source files in the root package (`logging.go`) get patched
2. Source files in sub-packages (`internal/global/logging.go`) get patched
3. Test files also get patched and renamed (`logging_test.go` → `logging_codspeed.go`)
4. However, when compiling the `global` package benchmarks, there's a type mismatch:
   - Some code references `*"github.com/go-logr/logr".Logger`
   - Other code references `*"github.com/CodSpeedHQ/codspeed-go/pkg/logr".Logger`

## How to Test

### Standard Go (should work):
```bash
cd go-runner/testdata/projects/example-logr-type-mismatch
go test -bench=. ./...
```

### CodSpeed Go Runner (currently broken):
```bash
cd go-runner
cargo run -- test -bench=. testdata/projects/example-logr-type-mismatch
```

Expected behavior: Should compile and run benchmarks successfully
Actual behavior: Compilation fails with type mismatch errors between original and patched logr.Logger types

## Related

This reproduces the issue seen in the `opentelemetry-go` integration test where:
- `./internal_logging.go:24:10: cannot use &l (value of type *"github.com/go-logr/logr".Logger) as *"github.com/CodSpeedHQ/codspeed-go/pkg/logr".Logger`
- `./internal_logging_codspeed.go:25:13: cannot use stdr.New(...) (value of struct type "github.com/go-logr/logr".Logger) as "github.com/CodSpeedHQ/codspeed-go/pkg/logr".Logger`
