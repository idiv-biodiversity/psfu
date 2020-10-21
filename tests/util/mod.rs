use std::error::Error;
use std::process::Command;

use assert_cmd::prelude::*;

pub fn bin(args: &[&str]) -> Result<Command, Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.args(args);
    Ok(cmd)
}
