use cargo_scout_audit::run::run_scout;
use clap::Parser;
use util::print::print_full_error;

fn main() {
    let cli = cli_args::Cli::parse();

    match cli.subcmd {
        cli_args::CargoSubCommand::ScoutAudit(opts) => match run_scout(opts) {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                print_full_error(&e);
                std::process::exit(1);
            }
        },
    }
}
