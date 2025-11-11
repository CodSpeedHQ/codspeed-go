#[derive(Debug)]
pub enum CliExit {
    Help,
    Version,
    MissingArgument,
    UnknownFlag,
}

#[derive(Debug)]
pub struct Cli {
    /// Run only benchmarks matching regexp
    pub bench: String,

    /// Run each benchmark for duration d (e.g., '3s')
    pub benchtime: String,

    /// Package patterns to run benchmarks for
    pub packages: Vec<String>,

    /// Build benchmarks but don't execute them
    pub dry_run: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            bench: ".".into(),
            benchtime: "3s".into(),
            packages: vec!["./...".into()],
            dry_run: false,
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

    fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Self, CliExit> {
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
                "--dry-run" => {
                    instance.dry_run = true;
                }
                s if s.starts_with('-') => {
                    eprintln!(
                        "warning: flag '{s}' is not supported by CodSpeed Go runner, ignoring"
                    );
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

    #[test]
    fn test_cli_parse_dry_run_flag() {
        let cli = str_to_iter("go-runner test --dry-run").unwrap();
        assert!(cli.dry_run);

        let cli = str_to_iter("go-runner test").unwrap();
        assert!(!cli.dry_run);
    }

    #[test]
    fn test_cli_parse_dry_run_with_other_flags() {
        let cli =
            str_to_iter("go-runner test --dry-run -bench=BenchmarkFoo -benchtime 5s").unwrap();
        assert!(cli.dry_run);
        assert_eq!(cli.bench, "BenchmarkFoo");
        assert_eq!(cli.benchtime, "5s");
    }

    #[test]
    fn test_cli_parse_dry_run_with_packages() {
        let cli = str_to_iter("go-runner test --dry-run ./pkg1 ./pkg2").unwrap();
        assert!(cli.dry_run);
        assert_eq!(
            cli.packages,
            vec!["./pkg1".to_string(), "./pkg2".to_string()]
        );
    }
}
