use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod commands;
mod object;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize an empty Git repository
    Init {
        /// Force the creation of a new Git repository (will overwrite the current one)
        #[arg(short, long)]
        force: bool,
    },
    /// Inspect a Git object
    CatFile {
        /// Pretty-print the object
        #[arg(short = 'p', long)]
        pretty_print: bool,

        /// Indicates a Git object's hash
        object_hash: String,
    },
    /// Create a Git object
    HashObject {
        /// Write the object into the object database
        #[arg(short = 'w', long)]
        write: bool,

        /// Path to the file to hash
        file: PathBuf,
    },
    /// List the contents of a Git tree object
    LsTree {
        /// Indicates a Git tree's hash
        tree_hash: String,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init { force } => commands::init::invoke(force),
        Command::CatFile {
            object_hash,
            pretty_print,
        } => commands::cat_file::invoke(pretty_print, &object_hash),
        Command::HashObject { write, file } => commands::hash_object::invoke(write, &file),
        Command::LsTree { tree_hash } => commands::ls_tree::invoke(&tree_hash),
    }?;

    Ok(())
}
