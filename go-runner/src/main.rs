use codspeed_go_runner::cli::Cli;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .parse_env("CODSPEED_LOG")
        .filter_module("handlebars", log::LevelFilter::Off)
        .format_timestamp(None)
        .init();

    let cli = Cli::parse();
    codspeed_go_runner::run_benchmarks(Path::new("."), &cli)?;

    Ok(())
}
