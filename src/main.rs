#![deny(clippy::all)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]

mod affinity;
mod cli;
mod log;
mod pid;
mod tree;

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::build().get_matches();

    match args.subcommand() {
        Some(("tree", args)) => tree::run(args),
        _ => unreachable!(crate::cli::SUBCOMMAND_REQUIRED),
    }
}
