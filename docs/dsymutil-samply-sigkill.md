# macOS: `dsymutil` SIGKILL under `samply` (Go benchmark symbolication)

## TL;DR

On macOS, running the Go benchmark runner under `codspeed run` (i.e. inside
`samply record`) intermittently failed the build with:

```
link: running dsymutil failed: signal: killed
```

This is **not** an out-of-memory kill. It is caused by **samply**: samply injects
`DYLD_INSERT_LIBRARIES` into the *entire* process subtree it launches. Go's linker
spawns `dsymutil` to flatten DWARF; `dsymutil` inherits the injection, hands its
mach task port to samply, and is then killed with `SIGKILL` (signal 9).

**Fix in this repo:** build the Go test binary with `-ldflags=-s=false -w` on
macOS. `-w` drops DWARF, so the linker never invokes `dsymutil`, while `-s=false`
keeps the Mach-O symbol table so samply can still symbolicate benchmark frames
(`example.fibonacci`, etc.). See `go-runner/src/runner/mod.rs`.

## How we got here (symptom chain)

1. **Symbols missing in the profile.** Benchmark frames showed up as
   `fun_129280` instead of `example.fibonacci`. The test binary's Mach-O symbol
   table had been stripped: Go's macOS linker defaults to `-s -w` for cgo
   (externally-linked) binaries, and the runner only passed `-w`, leaving `-s`
   active. Forcing `-s=false` restored the symbols.
2. **Keeping DWARF (`-w=false`) then failed the build** under `codspeed run` with
   `running dsymutil failed: signal: killed`. That is the subject of this doc.

## Root cause (proven)

### It is samply, not OOM

- The kill is **100% reproducible under `samply record`** wrapping the build, and
  **never reproduces** running the runner directly. (Bare `dsymutil` on the
  example uses ~57 MB — nowhere near OOM on a 16 GB+ machine.)
- The macOS system log shows **no jetsam/`memorystatus` kill and no AMFI entry**
  for `dsymutil`. The kernel OOM killer always logs; its silence means the
  `SIGKILL` came from **userspace** (samply), not the kernel.

### The mechanism

samply profiles a launched process by injecting a preload library:

- `samply/src/mac/process_launcher.rs` builds the child environment from
  `std::env::vars_os()` and adds `DYLD_INSERT_LIBRARIES=libsamply_mac_preload.dylib`
  and `SAMPLY_BOOTSTRAP_SERVER_NAME=<mach server>`.
- Because these are ordinary environment variables, **every descendant inherits
  them**, not just the direct target.
- `samply-mac-preload/src/lib.rs` runs as a global constructor in each process
  that loads it. It connects to samply's bootstrap server, **sends its own mach
  task port** (`mach_task_self()`) — comment: *"Then the parent can control us
  completely"* — and blocks waiting for a `"Proceed"` reply.

Under `codspeed run`, the whole command runs inside `samply record`:

```
samply record -- cargo run … codspeed-go-runner … go test …
                                  └─ go (linker) ─ dsymutil   ← inherits the injection
```

`dsymutil` (spawned by Go's linker) loads the preload, hands its task to samply,
and is subsequently **SIGKILLed**. Go's linker reports the dead child as
`running dsymutil failed: signal: killed` and the build fails.

### Why `dsymutil` specifically

We isolated it with a minimal spawner (see "Minimal reproduction"):

| Process (as a samply grandchild)         | Result            |
|------------------------------------------|-------------------|
| locally-built CPU-loop worker            | exit 0 ✅          |
| worker doing 200k `open()`/`close()`     | exit 0 ✅          |
| worker doing 50× `fork()`/`waitpid()`    | exit 0 ✅          |
| `/bin/sleep`, `/usr/bin/true`, `nm`, …   | exit 0 ✅          |
| **`dsymutil`**                           | **SIGKILL (9)** ❌ |
| `dsymutil` with the samply env *cleared* before exec | exit 0 ✅ |

So:

- It is **not** a generic "samply kills all children" issue (workers survive).
- It is **not** sampling frequency (reproduces at `--rate 1`).
- It is **not** file-I/O interposition or `fork`.
- It is triggered specifically by **samply taking over `dsymutil`'s task** —
  clearing `DYLD_INSERT_LIBRARIES` / `SAMPLY_BOOTSTRAP_SERVER_NAME` for the
  `dsymutil` child makes it survive.

`dsymutil` is heavily multithreaded and links `posix_spawn`/`fork`/`execve`; the
exact interaction between its runtime and samply holding its task control port is
what turns the takeover into a `SIGKILL`. The other binaries either don't load
the preload (platform binaries like `/bin/sh` have `DYLD_*` stripped by dyld, so
*their* children don't inherit it either) or tolerate the takeover.

Note `/bin/sh -c 'dsymutil …'` does **not** reproduce: `sh` is a restricted
platform binary, so dyld strips `DYLD_*` from its environment and its `dsymutil`
child never loads the preload. A non-restricted intermediary (Homebrew Go linker,
a self-built spawner) is required — which is exactly the real build.

## Minimal reproduction

A self-contained reproduction lives in the samply fork at
`../samply-codspeed/repro-dsymutil-sigkill/` (`repro.sh`). In essence:

```sh
# A locally-built parent that execs dsymutil and reports how it died.
cc -o spawner spawner.c            # forks + execs `dsymutil -f <macho> -o out.dwarf`

./spawner                          # WITHOUT samply  -> "dsymutil exit 0"
samply record --save-only -o /tmp/p.json.gz -- ./spawner   # -> "dsymutil killed by signal 9"
```

The kill is deterministic under samply and absent without it.

## Workarounds evaluated

| # | Workaround | Keeps line info? | Notes |
|---|------------|------------------|-------|
| 1 | **`-ldflags=-s=false -w` on macOS** *(chosen)* | No (function names only) | Drops DWARF → linker never runs `dsymutil`. Symbol table kept → samply still resolves function names. Simple, fully within the runner. |
| 2 | Strip `DYLD_INSERT_LIBRARIES` / `SAMPLY_BOOTSTRAP_SERVER_NAME` from build subprocesses | Yes | Requires the runner to scrub samply's env before invoking `go test`, but then the build's own children also aren't profiled (fine). Fragile: env var names are a samply implementation detail. |
| 3 | Don't profile the build at all — only wrap benchmark *execution* in samply | Yes | The correct long-term fix, but it lives in `codspeed run` / samply, not in this runner. `codspeed run` currently wraps the entire `cargo run … go test …`, build included. |
| 4 | Pre-build the binary outside samply, then run it under samply | Yes | Not possible from inside the runner because `codspeed run` already wraps the whole runner invocation. |
| 5 | Fix samply so its task takeover doesn't SIGKILL `dsymutil` | Yes | Upstream samply fix; tracked via the repro in the samply fork. |

For the Go runner today, **#1** is the pragmatic, robust choice: it removes the
`dsymutil` step entirely and preserves the only thing walltime profiling needs —
function-level symbols. The only thing lost on macOS is source-line / inlined
frame info in flamegraphs.

## Verification

After the fix (`-ldflags=-s=false -w`):

- `nm` shows `_example.fibonacci` / `_example.BenchmarkFibonacciDarwin` in the
  binary; no `.dSYM` is produced and `dsymutil` never runs.
- `codspeed run … -bench=BenchmarkFibonacciDarwin` completes and reports
  `BenchmarkFibonacciDarwin: best …µs` instead of `build failed`.
- A fresh samply profile attributes the hot leaf to `example.fibonacci` rather
  than `fun_129280`.

## References

- Go issue: [cmd/link: allow configuration of dsymutil and strip for darwin/mach-o linking #47316](https://github.com/golang/go/issues/47316)
- samply: `DYLD_INSERT_LIBRARIES` + mach task-port siphoning (see the "system
  executables block DYLD_INSERT_LIBRARIES" error message in samply's source).
- samply preload handshake: `samply-mac-preload/src/lib.rs`
  (`set_up_samply_connection`), `samply/src/mac/process_launcher.rs`
  (`TaskAccepter`, `AcceptedTask::start_execution`).
