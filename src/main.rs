mod cli;
mod commands;
mod error;
mod git;
mod meta;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::List(args) => commands::list::run(args),
        Commands::New(args) => commands::new::run(args),
        Commands::Rm(args) => commands::rm::run(args),
        Commands::Clean(args) => commands::clean::run(args),
        Commands::Go(args) => commands::go::run(args),
        Commands::Status(args) => commands::status::run(args),
        Commands::Tag(args) => commands::tag::run(args),
    };

    if let Err(err) = result {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}
