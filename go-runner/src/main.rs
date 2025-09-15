use codspeed_go_runner::cli::Cli;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .parse_env(env_logger::Env::new().filter_or("CODSPEED_LOG", "info"))
        .filter_module("handlebars", log::LevelFilter::Off)
        .format_timestamp(None)
        .init();

    let cli = Cli::parse();
    let profile_dir = std::env::var("CODSPEED_PROFILE_FOLDER").unwrap_or("/tmp".into());
    codspeed_go_runner::run_benchmarks(profile_dir, Path::new("."), &cli)?;

    Ok(())
}
