#![deny(clippy::all)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]

mod affinity;
mod cli;
mod config;
mod log;
mod pid;
mod tree;

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::build().get_matches();

    match args.subcommand() {
        ("tree", Some(args)) => tree::run(args),

        // unreachable because subcommand is required
        _ => unreachable!(),
    }
}
