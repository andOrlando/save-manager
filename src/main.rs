mod functions;
mod types;
mod data;
mod cli;

use std::process::exit;

use cli::{Cli, Command};
use functions::{create, delete, switch, list, save, load, remove};

use clap::Parser;
use colored::Colorize;

fn main() {

    let cli = Cli::parse();
    let res = match &cli.command {
        Command::Create { name, paths } => create(name, paths),
        Command::Delete { name } => delete(name),
        Command::Switch { name } => switch(name),
        Command::List { category } => list(category),
        Command::Save { name } => save(name),
        Command::Load { name } => load(name),
        Command::Remove { name } => remove(name),
    };
    
    if res.is_ok() { exit(0) }
    println!("{} {}", "error:".bold().bright_red(), res.unwrap_err()); exit(1)
}


