# Example: Testing.T Type Mismatch with testify/suite

This example demonstrates the type mismatch issue that occurs when CodSpeed's import patching conflicts with external libraries like testify that expect the standard library's `*testing.T` type.

## The Issue

This reproduces the exact error seen in the opentelemetry-go project:
```
./benchmark_codspeed.go:19:12: cannot use t (variable of type *"github.com/CodSpeedHQ/codspeed-go/testing/testing".T)
    as *"testing".T value in argument to suite.Run
./benchmark_codspeed.go:25:12: cannot use &testing.T{} (value of type *"github.com/CodSpeedHQ/codspeed-go/testing/testing".T)
    as *"testing".T value in argument to suite.Run
```

Seen in production at: https://github.com/CodSpeedHQ/codspeed-go (opentelemetry-go customer project)

### Root Cause

1. CodSpeed patches test files: `import "testing"` → `import testing "github.com/CodSpeedHQ/codspeed-go/compat/testing"`
2. This makes all `*testing.T` references in the test file use the CodSpeed version
3. However, external packages like `github.com/stretchr/testify/suite` are compiled against the standard library `testing` package
4. When test code calls `suite.Run(t)`, it passes CodSpeed's `*testing.T`, but `suite.Run` expects standard library's `*testing.T`
5. Go's type system treats these as incompatible types → compile error

### Why testify Specifically?

CodSpeed already patches some testing-related imports (see `patcher.rs`):
- `github.com/go-quicktest/qt` → patched
- `github.com/go-logr/logr/slogassert` → patched
- `github.com/go-logr/logr/testr` → patched
- `github.com/go-logr/stdr` → patched

But **NOT** `github.com/stretchr/testify`, which is extremely popular and used in opentelemetry-go.

### Real-World Impact

This pattern appears in many projects using:
- **testify/suite**: `suite.Run(t *testing.T)` expects standard library type
- **testify/require**: Various assertion methods that take `testing.TB`
- **testify/assert**: Various assertion methods that take `testing.TB`
- Any external library that accepts `*testing.T` or `*testing.B` as parameters

## Project Structure

```
example-with-testify/
├── go.mod               # Requires github.com/stretchr/testify v1.10.0
└── metric/
    └── benchmark_test.go  # Uses testify/suite.Run()
```

## How to Test

### Standard Go (works correctly):
```bash
cd /home/not-matthias/Documents/work/wgit/codspeed-go/go-runner/testdata/projects/example-with-testify
go test -v ./metric
go test -bench=. ./metric
```

### CodSpeed Go Runner (fails with type mismatch):
```bash
cd /home/not-matthias/Documents/work/wgit/codspeed-go/go-runner
cargo run -- test -bench=. ./metric
```

**Expected behavior**: Benchmark should compile and run

**Actual behavior**:
```
error: cannot use t (variable of type *"github.com/CodSpeedHQ/codspeed-go/testing/testing".T)
       as *"testing".T value in argument to suite.Run
```

## Potential Solutions

1. **Patch testify imports** (similar to logr/testr):
   - Add testify-compatible wrappers in codspeed-go
   - Patch `github.com/stretchr/testify/suite` imports in test files
   - Requires maintaining compatibility with testify API

2. **Type wrapper/adapter**:
   - Create conversion functions between types
   - Inject at call sites (complex AST manipulation)

3. **Skip patching for testify tests**:
   - Detect testify usage and skip those test files
   - Tests run without CodSpeed instrumentation (loses metrics)

4. **Upstream collaboration**:
   - Work with testify to accept interface instead of concrete type
   - Long-term solution but requires ecosystem changes

## Notes

This is a fundamental limitation of Go's type system - two types with the same structure but different package paths are incompatible. The patched `*testing.T` from `github.com/CodSpeedHQ/codspeed-go/testing/testing` and standard library `*testing.T` are distinct, incompatible types.
