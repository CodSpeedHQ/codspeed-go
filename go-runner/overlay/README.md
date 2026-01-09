# Go Benchmark Overlay Patches

This directory contains CodSpeed instrumentation overlays for Go's standard `testing` package benchmarks.

## Files

- `benchmark1.24.0.go`, `benchmark1.25.0.go` - Modified versions of Go's `testing/benchmark.go` with CodSpeed instrumentation
- `benchmark1.24.0.patch`, `benchmark1.25.0.patch` - Patch files showing differences from upstream
- `codspeed.go` - CodSpeed-specific benchmark extensions
- `instrument-hooks.go` - Bindings to the instrument-hooks library

## Supporting a new Go Version

To generate the patch files, run this (or use the existing patches):
```bash
# Download the unpatched file:
export VERSION=1.25.0
wget -O benchmark$VERSION.go https://github.com/golang/go/raw/refs/tags/go$VERSION/src/testing/benchmark.go

# Then compare against the patched file (same version!):
diff -a -u -N benchmark$VERSION.go overlay/benchmark$VERSION.go > overlay/benchmark$VERSION.patch
```

You can then download the latest benchmark file and apply the patch:
```bash
wget -O overlay/benchmark$VERSION.go https://github.com/golang/go/raw/refs/tags/go$VERSION/src/testing/benchmark.go
patch overlay/benchmark$VERSION.go < overlay/benchmark$VERSION.patch
```

Then manually fix any conflicts or issues that arise.
