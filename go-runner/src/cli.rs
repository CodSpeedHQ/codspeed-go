use std::iter::Peekable;

#[derive(Debug)]
pub enum CliExit {
    Help,
    Version,
    MissingArgument,
    UnknownFlag,
}

/// A `go test` flag, classified by whether it takes a value.
///
/// Flag names follow cmd/go/internal/{base/flag.go, work/build.go, test/testflag.go}.
enum Flag<'a> {
    /// Takes no value.
    Boolean,
    /// Takes one value, attached (`-p=1`) or as the next argument (`-p 1`).
    Value(Option<&'a str>),
    /// Not a `go test` flag, e.g. a custom flag of the test binary.
    Unknown(Option<&'a str>),
}

impl<'a> Flag<'a> {
    /// Classifies `arg`, or returns `None` if it is not a flag.
    fn from_arg(arg: &'a str) -> Option<Self> {
        if Self::parse_bool_flag(arg).is_some() {
            return Some(Self::Boolean);
        }
        if let Some(value) = Self::parse_value_flag(arg) {
            return Some(Self::Value(value));
        }
        let (_, value) = Self::split(arg)?;
        Some(Self::Unknown(value))
    }

    /// Parses a `go test` flag that takes a value and returns its attached value (`-p=1`), or
    /// `Some(None)` when the value is the next argument (`-p 1`). Returns `None` if `arg` is
    /// not a known value flag. Like [`Self::parse_bool_flag`], it accepts `--` and `test.`.
    fn parse_value_flag(arg: &str) -> Option<Option<&str>> {
        const VALUE_FLAGS: &[&str] = &[
            "asmflags",
            "bench",
            "benchtime",
            "blockprofile",
            "blockprofilerate",
            "buildmode",
            "C",
            "compiler",
            "count",
            "covermode",
            "coverpkg",
            "coverprofile",
            "cpu",
            "cpuprofile",
            "debug-actiongraph",
            "debug-runtime-trace",
            "debug-trace",
            "exec",
            "fuzz",
            "fuzzminimizetime",
            "fuzztime",
            "gccgoflags",
            "gcflags",
            "installsuffix",
            "ldflags",
            "list",
            "memprofile",
            "memprofilerate",
            "mod",
            "modfile",
            "mutexprofile",
            "mutexprofilefraction",
            "o",
            "outputdir",
            "overlay",
            "p",
            "parallel",
            "pgo",
            "pkgdir",
            "run",
            "shuffle",
            "skip",
            "tags",
            "timeout",
            "toolexec",
            "trace",
            "vet",
        ];
        let (name, value) = Self::split(arg)?;
        VALUE_FLAGS.contains(&name).then_some(value)
    }

    /// Parses a boolean `go test` flag and returns its value, or `None` if `arg` is not a
    /// known boolean flag or has an invalid value. Accepted spellings:
    ///
    /// - `-v`, `--v`: bare, means true
    /// - `-v=true`, `--v=true`: also `1`, `t`, `T`, `TRUE`, `True`
    /// - `-v=false`, `--v=false`: also `0`, `f`, `F`, `FALSE`, `False`
    /// - `-test.v`, `--test.v`, `-test.v=true`: the same flag under the test binary's name
    ///
    /// `-v true` is not a form: Go reads `true` as a separate argument, not as the value.
    fn parse_bool_flag(arg: &str) -> Option<bool> {
        const BOOL_FLAGS: &[&str] = &[
            "a",
            "artifacts",
            "asan",
            "benchmem",
            "buildvcs",
            "c",
            "cover",
            "failfast",
            "fullpath",
            "json",
            "linkshared",
            "modcacherw",
            "msan",
            "n",
            "race",
            "short",
            "trimpath",
            "v",
            "work",
            "x",
        ];
        // Spellings accepted by Go's `strconv.ParseBool`.
        const TRUE_VALUES: &[&str] = &["1", "t", "T", "TRUE", "true", "True"];
        const FALSE_VALUES: &[&str] = &["0", "f", "F", "FALSE", "false", "False"];

        let (name, value) = Self::split(arg)?;
        if !BOOL_FLAGS.contains(&name) {
            return None;
        }
        match value {
            None => Some(true),
            Some(value) if TRUE_VALUES.contains(&value) => Some(true),
            Some(value) if FALSE_VALUES.contains(&value) => Some(false),
            Some(_) => None,
        }
    }

    /// Consumes the flag's value from `args` when it is a separate argument.
    fn consume_value(self, args: &mut Peekable<impl Iterator<Item = String>>) -> Option<String> {
        match self {
            Self::Boolean | Self::Value(Some(_)) | Self::Unknown(Some(_)) => None,
            Self::Value(None) => args.next(),
            // Like Go, treat a dash-leading argument after an unknown flag as a flag.
            Self::Unknown(None) => args.next_if(|arg| !arg.starts_with('-')),
        }
    }

    /// Splits `-name`, `--name` or `-test.name`, with an optional `=value`, into name and value.
    fn split(arg: &str) -> Option<(&str, Option<&str>)> {
        let name = arg.strip_prefix('-')?;
        let name = name.strip_prefix('-').unwrap_or(name);
        let (name, value) = match name.split_once('=') {
            Some((name, value)) => (name, Some(value)),
            None => (name, None),
        };
        let name = name.strip_prefix("test.").unwrap_or(name);
        Some((name, value))
    }
}

#[derive(Debug)]
pub struct Cli {
    /// Run only benchmarks matching regexp
    pub bench: String,

    /// Run each benchmark for duration d (e.g., '3s')
    pub benchtime: String,

    /// Package patterns to run benchmarks for
    pub packages: Vec<String>,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            bench: ".".into(),
            benchtime: "3s".into(),
            packages: vec!["./...".into()],
        }
    }
}

impl Cli {
    pub fn parse() -> Self {
        match Self::parse_args(std::env::args().skip(1)) {
            Ok(cli) => cli,
            Err(CliExit::Help) => std::process::exit(0),
            Err(CliExit::Version) => std::process::exit(0),
            Err(CliExit::MissingArgument) => std::process::exit(2),
            Err(CliExit::UnknownFlag) => std::process::exit(1),
        }
    }

    /// Parses the command-line arguments into a Cli instance.
    ///
    /// # Why not use clap or structopt?
    ///
    /// Unfortunately `go test` supports arguments with a different syntax than those crates. For
    /// example, `-bench` and `-benchtime` use single dashes instead of double dashes.
    ///
    /// We can't do this with clap/structopt, because they only support single dashes for single-letter
    /// flags (e.g., `-h`), and double dashes for multi-letter flags (e.g., `--help`).
    fn parse_args(args: impl Iterator<Item = String>) -> Result<Self, CliExit> {
        let mut args = args.peekable();
        let mut instance = Self::default();

        // We currently only support the `test` subcommand.
        let cmd = args.next();
        assert!(
            cmd == Some("test".to_string()),
            "Expected 'test' as the first argument, got {cmd:?}",
        );

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!(
                        "\
The Codspeed Go Benchmark Runner

USAGE:
    go-runner test [OPTIONS] [PACKAGES...]

OPTIONS:
    -bench <pattern>     Run only benchmarks matching regexp (defaults to '.')
    -benchtime <duration> Run each benchmark for duration d (defaults to '3s')
    --dry-run            Build benchmarks but don't execute them
    -h, --help           Print help information
    -V, --version        Print version information

SUPPORTED FLAGS:
    -bench, -benchtime, --dry-run

UNSUPPORTED FLAGS (will be warned about):
    -benchmem, -count, -cpu, -cpuprofile, -memprofile, -trace, etc."
                    );
                    return Err(CliExit::Help);
                }
                "-V" | "--version" => {
                    println!("{}", env!("CARGO_PKG_VERSION"));
                    return Err(CliExit::Version);
                }
                "-bench" => {
                    instance.bench = args.next().ok_or_else(|| {
                        eprintln!("error: `-bench` requires a pattern");
                        CliExit::MissingArgument
                    })?;
                }
                s if s.starts_with("-bench=") => {
                    instance.bench = s.split_once('=').unwrap().1.to_string();
                }
                "-benchtime" => {
                    instance.benchtime = args.next().ok_or_else(|| {
                        eprintln!("error: `-benchtime` requires a duration");
                        CliExit::MissingArgument
                    })?;
                }
                s if s.starts_with("-benchtime=") => {
                    instance.benchtime = s.split_once('=').unwrap().1.to_string();
                }
                s if s.starts_with('-') => {
                    log::warn!("flag '{s}' is not supported by CodSpeed Go runner, ignoring");
                    let value = Flag::from_arg(s).and_then(|flag| flag.consume_value(&mut args));
                    if let Some(value) = value {
                        log::warn!("ignoring value '{value}' of unsupported flag '{s}'");
                    }
                }
                _ => {
                    // Collect package arguments for filtering
                    instance.packages = {
                        let mut packages = vec![arg];
                        packages.extend(args);
                        packages
                    };
                    break;
                }
            }
        }
        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn str_to_iter(cmd: &str) -> Result<Cli, CliExit> {
        let args: Vec<String> = if cmd.trim().is_empty() {
            Vec::new()
        } else {
            cmd.split_whitespace()
                .map(|s| s.to_string())
                .skip(1)
                .collect()
        };
        Cli::parse_args(args.into_iter())
    }

    #[test]
    fn test_cli_parse_defaults() {
        let cli = str_to_iter("go-runner test").unwrap();
        assert_eq!(cli.bench, ".");
        assert_eq!(cli.benchtime, Cli::default().benchtime);
        assert_eq!(cli.packages, Cli::default().packages);
    }

    #[test]
    fn test_cli_parse_with_bench_flag() {
        let cli = str_to_iter("go-runner test -bench Test").unwrap();
        assert_eq!(cli.bench, "Test");

        let cli = str_to_iter("go-runner test -bench=BenchmarkFoo").unwrap();
        assert_eq!(cli.bench, "BenchmarkFoo");
    }

    #[test]
    fn test_cli_parse_with_benchtime_flag() {
        let cli = str_to_iter("go-runner test -benchtime 3s").unwrap();
        assert_eq!(cli.benchtime, "3s".to_string());

        let cli = str_to_iter("go-runner test -benchtime=10x").unwrap();
        assert_eq!(cli.benchtime, "10x".to_string());
    }

    #[test]
    fn test_cli_parse_with_packages() {
        let cli = str_to_iter("go-runner test package1 package2").unwrap();
        assert_eq!(cli.bench, ".");
        assert_eq!(
            cli.packages,
            vec!["package1".to_string(), "package2".to_string()]
        );
    }

    #[test]
    fn test_cli_parse_combined_flags() {
        let cli = str_to_iter("go-runner test -bench=BenchmarkFoo -benchtime 5s ./pkg").unwrap();
        assert_eq!(cli.bench, "BenchmarkFoo");
        assert_eq!(cli.benchtime, "5s".to_string());
        assert_eq!(cli.packages, vec!["./pkg".to_string()]);
    }

    #[test]
    fn test_cli_parse_help_flag() {
        let result = str_to_iter("go-runner test -h");
        assert!(matches!(result, Err(CliExit::Help)));

        let result = str_to_iter("go-runner test --help");
        assert!(matches!(result, Err(CliExit::Help)));
    }

    #[test]
    fn test_cli_parse_version_flag() {
        let result = str_to_iter("go-runner test -V");
        assert!(matches!(result, Err(CliExit::Version)));

        let result = str_to_iter("go-runner test --version");
        assert!(matches!(result, Err(CliExit::Version)));
    }

    #[test]
    fn test_cli_parse_invalid() {
        let result = str_to_iter("go-runner test -bench");
        assert!(matches!(result, Err(CliExit::MissingArgument)));

        let result = str_to_iter("go-runner test -benchtime");
        assert!(matches!(result, Err(CliExit::MissingArgument)));

        // Unknown flags now generate warnings but don't cause errors
        let result = str_to_iter("go-runner test -unknown");
        assert!(result.is_ok());
    }

    #[rstest]
    #[case::separate_p("-p 1 ./... -bench Foo", &["./...", "-bench", "Foo"])]
    #[case::attached_p("-p=1 ./...", &["./..."])]
    #[case::path_value("-cpuprofile ./cpu.out ./...", &["./..."])]
    #[case::subtest_value("-run TestX/sub ./...", &["./..."])]
    #[case::bool_then_package("-v fmt", &["fmt"])]
    #[case::test_prefixed_bool("-test.v fmt", &["fmt"])]
    #[case::bool_then_flag("-v -bench Foo", &["./..."])]
    #[case::trailing_flag("-p", &["./..."])]
    #[case::unknown_then_flag("-custom -timeout 5m ./pkg", &["./pkg"])]
    fn test_cli_drops_unsupported_flags(#[case] args: &str, #[case] packages: &[&str]) {
        let cli = str_to_iter(&format!("go-runner test {args}")).unwrap();
        assert_eq!(cli.packages, packages, "{args}");
    }

    #[rstest]
    #[case::bare("-v", Some(true))]
    #[case::double_dash("--v", Some(true))]
    #[case::true_value("-v=true", Some(true))]
    #[case::one_value("-v=1", Some(true))]
    #[case::upper_true_value("-v=TRUE", Some(true))]
    #[case::false_value("--v=false", Some(false))]
    #[case::short_false_value("-short=f", Some(false))]
    #[case::test_prefix("-test.v", Some(true))]
    #[case::test_prefix_double_dash("--test.v", Some(true))]
    #[case::test_prefix_value("-test.short=false", Some(false))]
    #[case::invalid_value("-v=maybe", None)]
    #[case::empty_value("-v=", None)]
    #[case::triple_dash("---v", None)]
    #[case::value_flag("-p", None)]
    #[case::value_flag_with_value("-p=1", None)]
    #[case::unknown("-integration", None)]
    #[case::not_a_flag("v", None)]
    fn test_parse_bool_flag(#[case] arg: &str, #[case] expected: Option<bool>) {
        assert_eq!(Flag::parse_bool_flag(arg), expected, "{arg}");
    }

    // Whole-token arguments: `str_to_iter` would split "-s -w" in two.
    #[rstest]
    #[case::ldflags("-ldflags", "-s -w")]
    #[case::double_dash_gcflags("--gcflags", "-N")]
    #[case::run("-run", "-x")]
    #[case::test_prefixed_run("-test.run", "-bench")]
    fn test_cli_drops_dash_leading_value(#[case] flag: &str, #[case] value: &str) {
        let args = ["test", flag, value, "./pkg"].map(String::from);
        let cli = Cli::parse_args(args.into_iter()).unwrap();
        assert_eq!(cli.packages, ["./pkg"], "{flag} {value}");
    }
}
