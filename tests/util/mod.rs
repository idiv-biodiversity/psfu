use assert_cmd::prelude::*;
use std::error::Error;
use std::process::Command;

pub fn bin(args: &[&str]) -> Result<Command, Box<dyn Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.args(args);
    Ok(cmd)
}
