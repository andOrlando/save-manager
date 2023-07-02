mod functions;
mod types;
mod data;
mod cli;

use cli::{Cli, Command};
use functions::{create, delete, switch, list, save, load, remove};

use clap::Parser;
use types::AppError;
use colored::Colorize;

fn main() -> Result<(), u8> {
    let res = cli();
    if res.is_ok() { return Ok(()); }
    
    let message = res.unwrap_err();
    println!("{} {}", "error:".bold().bright_red(), message);
    Err(1)
    
}

fn cli() -> AppError {
    let cli = Cli::parse();
    match &cli.command {
        Command::Create { name, path } => create(name, path)?,
        Command::Delete { name } => delete(name)?,
        Command::Switch { name } => switch(name)?,
        Command::List { category } => list(category)?,
        Command::Save { name } => save(name)?,
        Command::Load { name } => load(name)?,
        Command::Remove { name } => remove(name)?,
    }
    
    Ok(())
}


