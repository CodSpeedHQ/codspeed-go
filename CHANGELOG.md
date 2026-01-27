# Changelog


<sub>The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).</sub>



## [1.0.1] - 2026-01-27

### <!-- 1 -->🐛 Bug Fixes
- Disable stripping of symbols and debug information by @not-matthias in [#52](https://github.com/CodSpeedHQ/codspeed-go/pull/52)
- Ensure test binary exists after execution by @not-matthias
- Reduce fib computation workload by @not-matthias

### <!-- 7 -->⚙️ Internals
- Fix Cargo.lock after release by @adriencaccia


## [1.0.0] - 2026-01-20

### <!-- 0 -->🚀 Features
- Rewrite crate to use overlay files by @not-matthias

### <!-- 1 -->🐛 Bug Fixes
- Limit maximum b.N benchmark time by @not-matthias
- Limit maximum `b.Loop()` execution time by @not-matthias
- Check Go version compatibility before running tests by @not-matthias in [#49](https://github.com/CodSpeedHQ/codspeed-go/pull/49)
- Bump minimum go version due to synctest usage by @not-matthias
- Run tests using `go` executable by @not-matthias
- Add support for go1.24 by @not-matthias
- Sort snapshots using URI to have deterministic ordering by @not-matthias
- Only execute benchmarks, exclude tests by @not-matthias

### <!-- 2 -->🏗️ Refactor
- Move shared logic to external file by @not-matthias

### <!-- 7 -->⚙️ Internals
- Release v1.0.0 by @adriencaccia
- Update patch files by @not-matthias in [#50](https://github.com/CodSpeedHQ/codspeed-go/pull/50)
- Remove caddy test by @not-matthias
- Remove hugo test by @not-matthias
- Remove dry-run by @not-matthias
- Remove testing by @not-matthias
- Remove example-codspeed by @not-matthias
- Remove the forked pkgs by @not-matthias


## [0.6.2] - 2025-12-15

### <!-- 1 -->🐛 Bug Fixes
- Remove codspeed folder used for integration tests by @not-matthias in [#45](https://github.com/CodSpeedHQ/codspeed-go/pull/45)

### <!-- 7 -->⚙️ Internals
- Release v0.6.2 by @adriencaccia
- Add example using integration test by @not-matthias


## [0.6.1] - 2025-12-04

### <!-- 1 -->🐛 Bug Fixes
- Incomplete reading of benchmark time in b.N loops by @not-matthias in [#43](https://github.com/CodSpeedHQ/codspeed-go/pull/43)

### <!-- 7 -->⚙️ Internals
- Release v0.6.1 by @adriencaccia


## [0.6.0] - 2025-11-28

### <!-- 1 -->🐛 Bug Fixes
- Emit markers for benchmarks using b.Loop() by @not-matthias in [#42](https://github.com/CodSpeedHQ/codspeed-go/pull/42)
- Save measurements only in the benchmark framework by @not-matthias in [#41](https://github.com/CodSpeedHQ/codspeed-go/pull/41)
- Use precise version in fork script by @not-matthias

### <!-- 7 -->⚙️ Internals
- Release v0.6.0 by @adriencaccia
- Add edge case where timer is stopped at end of benchmark by @not-matthias
- Add more test cases to handle setup scenarios by @not-matthias


## [0.5.1] - 2025-11-19

### <!-- 0 -->🚀 Features
- Reduce warmup time to avoid long benchmark times by @not-matthias
- Allow benchmarks with RunParallel by @not-matthias
- Process raw results while running benchmarks by @not-matthias in [#38](https://github.com/CodSpeedHQ/codspeed-go/pull/38)
- Switch to mimalloc by @not-matthias
- Use shared temp directory for all packages by @not-matthias

### <!-- 1 -->🐛 Bug Fixes
- Emit StopBenchmark command even when benches failed by @not-matthias in [#40](https://github.com/CodSpeedHQ/codspeed-go/pull/40)
- Local package not used due to incorrect env var by @not-matthias in [#39](https://github.com/CodSpeedHQ/codspeed-go/pull/39)
- Clear raw_results folder after processing by @not-matthias
- Incorrect first iteration time by @not-matthias

### <!-- 2 -->🏗️ Refactor
- Revert patches on drop by @not-matthias

### <!-- 7 -->⚙️ Internals
- Release v0.5.1 by @adriencaccia
- Add examples with RunParallel by @not-matthias
- Enable log warnings for external modules by @not-matthias in [#36](https://github.com/CodSpeedHQ/codspeed-go/pull/36)


## [0.5.0] - 2025-11-13

### <!-- 0 -->🚀 Features
- Avoid parsing source code multiple times when patching imports by @not-matthias in [#33](https://github.com/CodSpeedHQ/codspeed-go/pull/33)
- Parallelize imports patching by @not-matthias
- Add rayon to parallelize result processing by @not-matthias
- Remove intermediate walltime result allocations by @not-matthias
- Keep temporary build directory to speedup execution by @not-matthias
- Optimize parsing by checking for benchmark functions and import paths by @not-matthias
- Add codspeed benchmarks by @not-matthias

### <!-- 1 -->🐛 Bug Fixes
- Do not delete the profile folder (#34) by @not-matthias in [#34](https://github.com/CodSpeedHQ/codspeed-go/pull/34)
- Cleanup temporary files during tests by @not-matthias
- Copy whole git repository to avoid missing files or packages by @not-matthias
- Ignore corrupted or template go files which can't be parsed by @not-matthias

### <!-- 3 -->📚 Documentation
- Clarify reason for not using clap or structopt in argument parsing by @not-matthias in [#32](https://github.com/CodSpeedHQ/codspeed-go/pull/32)
- Fix comment numbering by @not-matthias

### <!-- 7 -->⚙️ Internals
- Release v0.5.0 by @adriencaccia
- Add file count benchmark parameter by @not-matthias
- Use dircpy crate to copy folders by @not-matthias
- Add example with mod replace by @not-matthias
- Add RELEASE.md with the release process by @adriencaccia


## [0.4.2] - 2025-11-07

### <!-- 0 -->🚀 Features
- Add step to verify up-to-date fork scripts by @not-matthias in [#29](https://github.com/CodSpeedHQ/codspeed-go/pull/29)
- Speedup tests using `cargo nextest` by @not-matthias
- Check for version requirements in tests by @not-matthias
- Add quic-go project by @not-matthias
- Patch import paths for testing and sub-packages by @not-matthias
- Test multiple go versions by @not-matthias
- Add example with `_test` package by @not-matthias
- Add support for external test packages by @not-matthias
- Discover external test packages by @not-matthias
- Add discovery tests for all projects by @not-matthias

### <!-- 1 -->🐛 Bug Fixes
- Add compatibility for Go versions < 1.25 in fstest package by @not-matthias in [#30](https://github.com/CodSpeedHQ/codspeed-go/pull/30)
- Ensure build works with go workspaces by @not-matthias
- Panic on build error to find bugs during tests by @not-matthias
- Use testing fork imports by @not-matthias
- Remove abi fork to avoid duplicate symbol error; patch synctest to allow build by @not-matthias
- Remove double stop bug by @not-matthias
- Dont run vendor benches by @not-matthias in [#25](https://github.com/CodSpeedHQ/codspeed-go/pull/25)

### <!-- 3 -->📚 Documentation
- Add documentation about test execution by @not-matthias in [#26](https://github.com/CodSpeedHQ/codspeed-go/pull/26)

### <!-- 7 -->⚙️ Internals
- Release v0.4.2 by @adriencaccia
- Add testing sub-package benches and tests by @not-matthias
- Replace compat package with direct testing import by @not-matthias
- Bump fork to 1.25 by @not-matthias
- Update fork script and patches by @not-matthias
- Add example using `go mod vendor` by @not-matthias


## [0.4.1] - 2025-10-14

### <!-- 0 -->🚀 Features
- Add test with .go folder by @not-matthias

### <!-- 1 -->🐛 Bug Fixes
- Dont try to patch folders ending with .go by @not-matthias in [#21](https://github.com/CodSpeedHQ/codspeed-go/pull/21)

### <!-- 7 -->⚙️ Internals
- Release v0.4.1 by @art049


## [0.4.0] - 2025-10-14

### <!-- 0 -->🚀 Features
- Add benchmark markers by @not-matthias in [#18](https://github.com/CodSpeedHQ/codspeed-go/pull/18)
- Add bench with setup by @not-matthias
- Add support for `package main` benches by @not-matthias in [#19](https://github.com/CodSpeedHQ/codspeed-go/pull/19)
- Add test for example with `package main` by @not-matthias

### <!-- 1 -->🐛 Bug Fixes
- Dont append to list before stopping testing timer by @not-matthias
- Remove wrong reset timer by @not-matthias
- BenchTime is already in seconds by @not-matthias
- Move sleep tests to own package by @not-matthias
- Use sharded walltime runs by @not-matthias
- Run benchmarks of all packages by @not-matthias
- Run go-runner in release mode by @not-matthias

### <!-- 7 -->⚙️ Internals
- Add a changelog by @art049
- Pin rust version by @art049
- Bump instrument-hooks by @not-matthias in [#20](https://github.com/CodSpeedHQ/codspeed-go/pull/20)
- Bump instrument-hooks by @not-matthias


## [0.3.0] - 2025-09-19

### <!-- 0 -->🚀 Features
- Add caddy submodule and test by @not-matthias

### <!-- 1 -->🐛 Bug Fixes
- Remove old comment by @not-matthias in [#16](https://github.com/CodSpeedHQ/codspeed-go/pull/16)
- Use `info` log filter by default by @not-matthias
- Patch all imports by @not-matthias
- Use correct number of rounds for b.Loop() benchmarks by @not-matthias

### <!-- 7 -->⚙️ Internals
- Release v0.3.0 by @art049


## [0.2.0] - 2025-09-12

### <!-- 0 -->🚀 Features
- Add quicktest support + examples by @not-matthias in [#14](https://github.com/CodSpeedHQ/codspeed-go/pull/14)
- Fork quicktest by @not-matthias
- Add slogtest support + examples by @not-matthias
- Fork slogassert by @not-matthias
- Allow testify in benchmarks by @not-matthias

### <!-- 1 -->🐛 Bug Fixes
- Add env to override go pkg version by @not-matthias in [#15](https://github.com/CodSpeedHQ/codspeed-go/pull/15)
- Remove verifier by @not-matthias
- Ensure tests are deterministic and use single snapshot per test by @not-matthias
- Forward `Benchmark` function by @not-matthias

### <!-- 7 -->⚙️ Internals
- Release v0.2.0 by @art049


## [0.1.2] - 2025-09-08

### <!-- 1 -->🐛 Bug Fixes
- Ignore version in snapshots by @not-matthias in [#13](https://github.com/CodSpeedHQ/codspeed-go/pull/13)
- Only use local pkg in tests by @not-matthias
- Warn when running RunParallel by @not-matthias

### <!-- 7 -->⚙️ Internals
- Release v0.1.2 by @art049


## [0.1.1] - 2025-09-03

### <!-- 0 -->🚀 Features
- Resolve git-relative path inside benchmark by @not-matthias
- Add nested example by @not-matthias
- Support benchtime and pkg args by @not-matthias
- Specify codspeed mode by @not-matthias
- Use local codspeed-go pkg in CI and for tests by @not-matthias
- Run latest version of codspeed-go in PR by @not-matthias
- Add codspeed root frame by @not-matthias
- Add perf support by @not-matthias
- Add instrument-hooks artifacts by @not-matthias
- Add release workflow by @not-matthias in [#6](https://github.com/CodSpeedHQ/codspeed-go/pull/6)
- Add integration tests by @not-matthias in [#4](https://github.com/CodSpeedHQ/codspeed-go/pull/4)
- Add go-compatible CLI by @not-matthias
- Parse and convert raw results into walltime results by @not-matthias
- Run codspeed_runner.go with tags by @not-matthias
- Generate codspeed_runner.go to call benchmarks by @not-matthias
- Patch imports of benchmarks by @not-matthias
- Discover benchmarks in go project by @not-matthias
- Verify benchmarks with AST parser by @not-matthias
- Add test projects as submodule by @not-matthias
- Init crate by @not-matthias
- Add pre-commit and ci by @not-matthias
- Add readme by @not-matthias in [#2](https://github.com/CodSpeedHQ/codspeed-go/pull/2)
- Add ci by @not-matthias
- Add instrumentation to extract round times by @not-matthias
- Add codspeed compat layer by @not-matthias
- Add examples by @not-matthias

### <!-- 1 -->🐛 Bug Fixes
- Remove profile env var mutex in test by @not-matthias in [#7](https://github.com/CodSpeedHQ/codspeed-go/pull/7)
- Use git-relative file paths by @not-matthias
- Run tests in real project dir by @not-matthias
- Hide _codspeed.go filenames by @not-matthias
- Build binary, then run it by @not-matthias
- Implement `MatchString` testdep by @not-matthias in [#8](https://github.com/CodSpeedHQ/codspeed-go/pull/8)
- Change crate name by @not-matthias in [#11](https://github.com/CodSpeedHQ/codspeed-go/pull/11)
- Resolve path using first URI part by @not-matthias in [#9](https://github.com/CodSpeedHQ/codspeed-go/pull/9)

### <!-- 2 -->🏗️ Refactor
- Running with codspeed message by @not-matthias

### <!-- 7 -->⚙️ Internals
- Release v0.1.1 by @art049
- Define the dist profile within the workspace to fix cargo dist by @art049
- Use resolver v3 for workspace by @not-matthias
- Update benchmarks by @not-matthias
- Add licenses by @not-matthias in [#5](https://github.com/CodSpeedHQ/codspeed-go/pull/5)
- Fork testing by @not-matthias
- Add pre-commit hook by @not-matthias


[1.0.1]: https://github.com/CodSpeedHQ/codspeed-go/compare/v1.0.0..v1.0.1
[1.0.0]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.6.2..v1.0.0
[0.6.2]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.6.1..v0.6.2
[0.6.1]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.6.0..v0.6.1
[0.6.0]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.5.1..v0.6.0
[0.5.1]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.5.0..v0.5.1
[0.5.0]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.4.2..v0.5.0
[0.4.2]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.4.1..v0.4.2
[0.4.1]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.4.0..v0.4.1
[0.4.0]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.3.0..v0.4.0
[0.3.0]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.2.0..v0.3.0
[0.2.0]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.1.2..v0.2.0
[0.1.2]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.1.1..v0.1.2

<!-- generated by git-cliff -->
