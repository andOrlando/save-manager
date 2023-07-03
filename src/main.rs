mod functions;
mod types;
mod data;
mod cli;

use std::process::exit;

use cli::{Cli, Command, UpdateCommand};
use functions::{create, delete, update_name, switch, list, save, load, overwrite, remove};

use clap::Parser;
use colored::Colorize;

fn main() {

    let cli = Cli::parse();
    let res = match &cli.command {
        Command::Create { name, paths } => create(name, paths),
        Command::Delete { name } => delete(name),
        Command::Update { command } => match command {
            UpdateCommand::Name { name } => update_name(name),
        },
        Command::Switch { name } => switch(name),
        Command::List { category } => list(category),
        Command::Save { name } => save(name),
        Command::Load { name } => load(name),
        Command::Overwrite { name } => overwrite(name),
        Command::Remove { name } => remove(name),
    };
    
    if res.is_ok() { exit(0) }
    println!("{} {}", "error:".bold().bright_red(), res.unwrap_err()); exit(1)
}


