use cargo_scout_audit::{
    cli_args,
    run::run_scout,
    util::{
        logger::{get_subscriber, init_subscriber},
        print::print_full_error,
    },
};
use clap::Parser;
use tracing::level_filters::LevelFilter;

fn main() {
    let subscriber = get_subscriber(
        "cargo-scout-audit".to_string(),
        LevelFilter::INFO,
        std::io::stdout,
    );
    init_subscriber(subscriber);

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
