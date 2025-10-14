# Changelog


<sub>The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).</sub>



## [0.4.1] - 2025-10-14

### <!-- 0 -->🚀 Features
- Add test with .go folder by @not-matthias

### <!-- 1 -->🐛 Bug Fixes
- Dont try to patch folders ending with .go by @not-matthias in [#21](https://github.com/CodSpeedHQ/codspeed-go/pull/21)


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


[0.4.1]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.4.0..v0.4.1
[0.4.0]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.3.0..v0.4.0
[0.3.0]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.2.0..v0.3.0
[0.2.0]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.1.2..v0.2.0
[0.1.2]: https://github.com/CodSpeedHQ/codspeed-go/compare/v0.1.1..v0.1.2

<!-- generated by git-cliff -->
