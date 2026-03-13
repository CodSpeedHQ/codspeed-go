# AGENTS.md - codspeed-go

## Project Overview

codspeed-go is the CodSpeed Go benchmark runner. It patches Go's `testing` package at build time using Go's `-overlay` flag, injecting instrumentation into benchmarks without requiring user code changes.

## Build & Test

```bash
cargo check                    # type-check
cargo test                     # unit + integration tests (uses rstest + insta)
cargo nextest run --all        # parallel test execution (used in CI)
cargo clippy                   # lint
cargo fmt                      # format

# Run runner against example project (from example/ directory)
cargo run -- test -bench . -benchtime 3s ./...

# Run a single integration test
cargo test test_name_here
```

Pre-commit hooks enforce: `go-mod-tidy`, `go-fmt`, `cargo fmt`, `cargo check --all-targets`, `clippy -D warnings`.

CI tests against Go 1.24.x and 1.25.x. Go 1.24 tests require `GOEXPERIMENT=synctest`.

## Architecture

Rust workspace with a single crate: `go-runner/` (`codspeed-go-runner`), edition 2024, toolchain 1.90.0.

**Flow:** `main.rs` parses CLI args → `runner::run()` generates overlay + runs `go test` → Go benchmarks write raw JSON results to `$CODSPEED_PROFILE_FOLDER/raw_results/` → `collect_walltime_results()` aggregates into `results/{pid}.json`.

**Overlay mechanism:** Three files are overlaid into `$GOROOT/src/testing/`:
- `benchmark.go` — replaces the standard `testing.B` implementation (version-specific: 1.24 or 1.25+)
- `codspeed.go` — CodSpeed measurement logic, result saving, `codspeed` struct with per-round measurements
- `instrument-hooks.go` — cgo FFI bindings to the C instrument-hooks library (downloaded at runtime)

The overlay uses `@@PLACEHOLDER@@` strings that the Rust runner substitutes at runtime (`@@INSTRUMENT_HOOKS_DIR@@`, `@@CODSPEED_PROFILE_DIR@@`, `@@GO_RUNNER_VERSION@@`).

**CLI parser:** Custom hand-rolled parser in `cli.rs` because Go uses single-dash flags (`-bench`, `-benchtime`) which clap/structopt don't support.

## Runner Modes

- **walltime** — wall-clock measurement with warmup, multiple rounds. Used on bare metal runners.
- **simulation** — single iteration under instrumentation (valgrind/callgrind). Used on CodSpeed infrastructure.
- **memory** — memory profiling mode.

Set via `CODSPEED_RUNNER_MODE` env var (default: `walltime`).

## Integration Tests

Tests in `go-runner/src/integration_tests.rs` use real Go projects from `go-runner/testdata/projects/` (git submodules). Uses `insta` for snapshot testing with redactions for non-deterministic fields (PID, version, stats). Accept new snapshots with `cargo insta review`.

## Key Environment Variables

- `CODSPEED_RUNNER_MODE` — `walltime` (default), `simulation`, or `memory`
- `CODSPEED_PROFILE_FOLDER` — where results are written (default: `/tmp`)
- `CODSPEED_LOG` — log level filter (default: `info`)

## Gotchas

- `instrument-hooks.go` requires cgo (`import "C"`). The runner sets `CGO_ENABLED=1` and checks for a C compiler before building. Without this, Go silently excludes the file causing "undefined: InstrumentHooks" errors.
- The runner uses `$GOROOT/bin/go` directly (not PATH) to avoid infinite recursion with the runner binary intercepting `go test`.
- The runner sets custom `GOCACHE` and `GOMODCACHE` to temp dirs to avoid cache conflicts.
- Overlay patches are maintained as `.patch` files alongside the full `.go` files in `go-runner/overlay/`. Use `update-patch.sh` to regenerate.

## Release Process

Update version in `go-runner/Cargo.toml`, generate changelog with `git cliff --tag "v$VERSION" -o CHANGELOG.md`, commit, create annotated tag (`git tag -a`), push with `--follow-tags`. See `RELEASE.md` for details.
