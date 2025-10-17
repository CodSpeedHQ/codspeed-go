# Test Execution Strategy

This document explains how go-runner executes benchmarks for both internal and external test packages.

## Overview

Go supports two testing patterns:
- **Internal tests** (`package foo`) - Same package as the code being tested
- **External tests** (`package foo_test`) - Separate package for black-box testing

Each pattern requires a different execution strategy because of Go's package import rules.

## Internal Test Execution

**Example:**
```go
// fib.go
package fib

func Fibonacci(n int) int { ... }

// fib_test.go
package fib  // ← Same package

import "testing"

func BenchmarkFibonacci(b *testing.B) {
    Fibonacci(10)  // ← Direct call (same package)
}
```

**Strategy:**

1. **Rename test files**: `fib_test.go` → `fib_codspeed.go`
   - Why: `go build` only compiles `*_test.go` files with `go test`, not `go build`

2. **Patch imports**: Replace `import "testing"` with CodSpeed compat package
   - Done by: `patcher::patch_imports()` in `src/builder/patcher.rs`

3. **Create codspeed sub-package**: `codspeed/runner.go`
   - The sub-package can import the parent package
   - Template: `src/builder/template.go` with Handlebars

4. **Import and reference benchmarks** in runner:
   ```go
   import benchmarkfib_12345 "example.com/pkg/fib"

   var benchmarks = []InternalBenchmark{
       {"BenchmarkFibonacci", benchmarkfib_12345.BenchmarkFibonacci},
   }
   ```

5. **Build and run**: `go build -tags=codspeed ./codspeed`
   - Executed by: `builder::build_binary()` in `src/builder/mod.rs`

**Key files:**
- `src/builder/templater.rs` - Orchestrates the process (line 105-121: internal test handling)
- `src/builder/template.go` - Handlebars template (line 12-14: conditional imports)

## External Test Execution

**Example:**
```go
// fib.go
package fib

func Fibonacci(n int) int { ... }

// fib_integration_test.go
package fib_test  // ← Different package (_test suffix)

import "testing"
import "example.com/pkg/fib"  // ← Must import the package

func BenchmarkFibonacci(b *testing.B) {
    fib.Fibonacci(10)  // ← Qualified call (external package)
}
```

**Strategy:**

1. **Move test files to codspeed/**: `fib_integration_test.go` → `codspeed/fib_integration_codspeed.go`
   - Why: Avoids package conflicts (can't have `package fib` and `package fib_test` in same directory)

2. **Rename package**: `package fib_test` → `package main`
   - Why: Runner is `package main`, all files in codspeed/ must use same package
   - Done by: `patcher::patch_package_for_source()` in `src/builder/patcher.rs` (line 168-193)

3. **Patch imports**: Replace testing imports with CodSpeed compat packages

4. **Create runner in same package**: `codspeed/runner.go` as `package main`
   - Template: Same `template.go` but with different conditionals

5. **Direct benchmark references** (no import needed):
   ```go
   // NO import needed - benchmarks are in the same package (main)

   var benchmarks = []InternalBenchmark{
       {"BenchmarkFibonacci", BenchmarkFibonacci},  // ← Direct reference
   }
   ```

6. **Build entire directory**: `go build -tags=codspeed ./codspeed`
   - Compiles all `.go` files in the directory together

**Key files:**
- `src/builder/templater.rs` - Orchestrates the process (line 88-104: external test handling)
- `src/builder/template.go` - Template conditionals (line 12-14: skip imports, line 78-79: direct vs qualified names)

## Special Case: Package Main

When benchmarks exist in `package main`:

**Problem**: Go doesn't allow importing `package main`

**Solution**: Rename all `.go` files in the package to `package main_compat`
- Done by: `patcher::patch_all_packages_in_dir()` in `src/builder/patcher.rs` (line 138-164)
- Triggered in: `src/builder/templater.rs` (line 79-87)

This makes the package importable by the runner.

## Template Conditionals

The `template.go` file uses Handlebars to generate different code based on test type:

```go
// Import only for internal tests
{{#each benchmarks}}
    {{#unless is_external}}{{import_alias}} "{{module_path}}"{{/unless}}
{{/each}}

// Reference: qualified (internal) vs direct (external)
{{#each benchmarks}}
    {"{{name}}",
    {{#if is_external}}{{name}}{{else}}{{qualified_name}}{{/if}}},
{{/each}}
```

The `is_external` flag is set during discovery in `src/builder/discovery.rs`.

## Build Process

All paths converge to a single build command:

```rust
// src/builder/mod.rs:build_binary()
go build -tags=codspeed -o <binary> ./<codspeed_dir>
```

The key difference is what files are in the `codspeed/` directory:
- **Internal tests**: Only `runner.go`
- **External tests**: `runner.go` + renamed test files (`*_codspeed.go`)
