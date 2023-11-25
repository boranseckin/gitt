use clap::{Parser, Subcommand, Args};

use gitt::{init_git_dir, parse_object_hash};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create an empty Git repository
    Init,

    /// Inspect a Git object
    CatFile {
        #[command(flatten)]
        flags: CatFileFlags,

        /// Indicates a Git object's hash
        object: String,
    }
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct CatFileFlags {
    /// Pretty print <object> content
    #[arg(short, long)]
    pretty_print: bool,

    /// Show <object> type
    #[arg(short = 't', long = "type")]
    kind: bool,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Command::Init => { init_git_dir() },
        Command::CatFile { object, flags: _ } => {
            let _ = dbg!(parse_object_hash(object).unwrap());
        },
    }
}

