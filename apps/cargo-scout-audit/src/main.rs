use cargo_scout_audit::{
    cli::{CargoSubCommand, Cli},
    startup::run_scout,
    utils::{logger, print::print_error},
};
use clap::Parser;
use tracing::level_filters::LevelFilter;

fn main() {
    let subscriber = logger::get_subscriber("scout".into(), LevelFilter::OFF, std::io::stdout);
    logger::init_subscriber(subscriber);

    let cli = Cli::parse();

    match cli.subcmd {
        CargoSubCommand::ScoutAudit(opts) => {
            if let Err(e) = run_scout(opts) {
                print_error(e.to_string().trim());
                std::process::exit(1);
            }
        }
    }
}
