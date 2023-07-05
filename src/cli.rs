use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use clap::builder::Styles;
use anstyle::{Style, Color, AnsiColor};

#[derive(Parser)]
#[command(author, version, about, styles=Styles::styled()
    .header(Style::new())
    .error(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::BrightRed))))
    .usage(Style::new())
    .literal(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::BrightGreen))))
    .placeholder(Style::new())
    .valid(Style::new())
    .invalid(Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightRed))))
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Creates save
    Create {
        /// Name of new save
        #[arg(value_name="name")]
        name: String,
        /// Path to new save
        #[arg(value_name="path")]
        paths: Vec<PathBuf>
    },
    /// Deletes save
    Delete {
        /// Name of file to delete
        #[arg(value_name="name")]
        name: String
    },
    /// Changes properties of save
    Update {
        
        #[command(subcommand)]
        command: UpdateCommand

    },
    /// Switches active save
    Switch {
        /// Name of save to switch to
        #[arg(value_name="name")]
        name: String
    },
    /// Lists saves and versions
    List {
        /// Category of things you want to list
        #[arg(value_name="category")]
        category: Option<ListCategory>
    },
    /// Saves version
    Save {
        /// Name of version
        #[arg(value_name="name")]
        name: Option<String>
    },
    /// Loads version
    Load {
        /// Either name or index of version to load. `auto` to load autosave
        #[arg(value_name="name|index")]
        name: Option<String>
    },
    /// Overwrites version
    Overwrite {
        /// Name or index of version to overwrite
        #[arg(value_name="name|index")]
        name: String
    },
    /// Removes version
    Remove {
        /// List of names or indices to remove
        #[arg(value_name="name|index")]
        names: Vec<String>
    }
}

#[derive(Clone, ValueEnum)]
pub enum ListCategory {
    /// List all saves
    Saves,
    /// List all revisions of current save
    Versions
}

#[derive(Subcommand)]
pub enum UpdateCommand {
    /// Updates name of save
    Name {
        /// New name for save
        #[arg(value_name="name")]
        name: String
    }
}
