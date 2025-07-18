#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]

mod affinity;
mod cli;
mod log;
mod nice;
mod pid;
mod tree;
mod util;

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::build().get_matches();

    match args.subcommand() {
        Some(("tree", args)) => tree::run(args),
        _ => unreachable!("{}", crate::cli::SUBCOMMAND_REQUIRED),
    }
}
