use cargo_scout_audit::{
    cli::{CargoSubCommand, Cli},
    startup::run_scout,
    utils::{print::print_error, telemetry},
};
use clap::Parser;
use tracing::level_filters::LevelFilter;

fn main() {
    let subscriber = telemetry::get_subscriber("scout".into(), LevelFilter::OFF, std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let cli = Cli::parse();

    match cli.subcmd {
        CargoSubCommand::ScoutAudit(opts) => {
            match run_scout(opts) {
                Ok(_) => std::process::exit(0),
                Err(e) => {
                    print_error(e.to_string().trim());
                    std::process::exit(1);
                }
            }
        }
    }
}
