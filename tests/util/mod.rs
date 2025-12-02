use std::error::Error;

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;

pub fn bin(args: &[&str]) -> Result<Command, Box<dyn Error>> {
    let mut cmd = cargo_bin_cmd!();
    cmd.args(args);
    Ok(cmd)
}
