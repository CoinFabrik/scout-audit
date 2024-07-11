use cargo_scout_audit::{
    startup::{run_scout, CargoSubCommand, Cli},
    utils::{print::print_error, telemetry},
};
use clap::Parser;

fn main() {
    let subscriber = telemetry::get_subscriber("scout".into(), "warn".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

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
