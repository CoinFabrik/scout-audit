use cargo_scout_audit::{
    startup::{run_scout, CargoSubCommand, Cli},
    utils::telemetry::{self},
};
use clap::Parser;

fn main() {
    let subscriber =
        telemetry::get_subscriber("scout".into(), "warn".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let cli = Cli::parse();
    match cli.subcmd {
        CargoSubCommand::ScoutAudit(opts) => run_scout(opts).unwrap(),
    }
}
