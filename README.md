<div align="center">
<h1>codspeed-go</h1>

[![CI](https://github.com/CodSpeedHQ/codspeed-go/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/CodSpeedHQ/codspeed-go/actions/workflows/ci.yml)
[![Discord](https://img.shields.io/badge/chat%20on-discord-7289da.svg)](https://discord.com/invite/MxpaCfKSqF)
[![CodSpeed Badge](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/CodSpeedHQ/codspeed-go)

</div>

This repo contains the integration libraries for using CodSpeed in Go:

- [`go-runner`](./go-runner/): Golang benchmark builder and runner
- [`compat/testing`](./compat/testing/): Compatibility layer for the `testing` package.

## Usage

Integrating CodSpeed into your Go codebase requires **no modification**. You can continue using `go test` and the `testing` package as you normally would. When running your benchmarks in CI with CodSpeed, we will manually build and run the benchmarks and report the results to CodSpeed.

For information on how to integrate it, see the [CodSpeed documentation](https://codspeed.io/docs/benchmarks/golang). If you need further information to integrate CodSpeed to your project, please feel free to open an issue or ask for help on our discord server.


## Manual Usage

To run the benchmarks with CodSpeed locally, you need to install the `go-runner` crate which is used to build and execute the benchmarks with instrumentation:
```bash
$ cd go-runner
$ cargo install --path .
```

Then you can run the benchmarks with (the syntax is equivalent to `go test` but supports fewer flags). This will print all the benchmarks that can be run with CodSpeed and warnings if some benchmarks are not supported.
```bash
$ cd example
$ export CODSPEED_PROFILE_FOLDER=/tmp/codspeed
$ go-runner test -bench=.
[INFO  go_runner] Discovered 1 package
[INFO  go_runner] Total benchmarks discovered: 2
[INFO  go_runner] Found BenchmarkFibonacci10           in "fib_test.go"
[INFO  go_runner] Found BenchmarkFibonacci20_Loop      in "fib_test.go"
[INFO  go_runner] Generating custom runner for package: example
[INFO  go_runner] Running benchmarks for package: example
Running with CodSpeed (mode: walltime)
goos: linux
goarch: amd64
cpu: 12th Gen Intel(R) Core(TM) i7-1260P @ 1672.130MHz
BenchmarkFibonacci10/fibonacci(10)/fibonacci(10)-16                    1              1523 ns/op
BenchmarkFibonacci20_Loop-16                                           1             31373 ns/op
PASS
[INFO  go_runner] Parsed 2 raw results
[INFO  go_runner] Results written to "/tmp/codspeed/results/177951.json"
```
