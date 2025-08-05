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
        let mut bench = ".".to_string();

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
    go-runner test [OPTIONS]

OPTIONS:
    -bench <pattern>     Run only benchmarks matching regexp (defaults to '.')
    -h, --help           Print help information
    -V, --version        Print version information"
                    );
                    return Err(CliExit::Help);
                }
                "-V" | "--version" => {
                    println!("{}", env!("CARGO_PKG_VERSION"));
                    return Err(CliExit::Version);
                }
                "-bench" => {
                    bench = args.next().ok_or_else(|| {
                        eprintln!("error: `-bench` requires a pattern");
                        CliExit::MissingArgument
                    })?;
                }
                s if s.starts_with("-bench=") => {
                    bench = s.split_once('=').unwrap().1.to_string();
                }

                s if s.starts_with('-') => {
                    eprintln!("Unknown flag: {s}");
                    return Err(CliExit::UnknownFlag);
                }

                _ => {
                    eprintln!(
                        "warning: package arguments are not currently supported, ignoring '{arg}'"
                    );
                    // Consume and ignore all remaining arguments
                    for remaining_arg in args {
                        eprintln!(
                            "warning: package arguments are not currently supported, ignoring '{remaining_arg}'"
                        );
                    }
                    break;
                }
            }
        }
        Ok(Self { bench })
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
    }

    #[test]
    fn test_cli_parse_with_bench_flag() {
        let cli = str_to_iter("go-runner test -bench Test").unwrap();
        assert_eq!(cli.bench, "Test");

        let cli = str_to_iter("go-runner test -bench=BenchmarkFoo").unwrap();
        assert_eq!(cli.bench, "BenchmarkFoo");
    }

    #[test]
    fn test_cli_parse_ignores_packages() {
        let cli = str_to_iter("go-runner test package1 package2").unwrap();
        assert_eq!(cli.bench, ".");
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

        let result = str_to_iter("go-runner test -unknown");
        assert!(matches!(result, Err(CliExit::UnknownFlag)));
    }
}
