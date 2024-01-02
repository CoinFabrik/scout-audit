use cargo_scout_audit_soroban::startup::{run_scout, CargoSubCommand, Cli};
use clap::Parser;

fn main() {
    env_logger::init();

    let cli = Cli::parse();
    match cli.subcmd {
        CargoSubCommand::ScoutAuditSoroban(opts) => run_scout(opts).unwrap(),
    }
}
