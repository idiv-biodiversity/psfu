use std::io::{stdin, IsTerminal};

use clap::{crate_description, crate_name, crate_version};
use clap::{Arg, ArgAction, Command};

pub const SUBCOMMAND_REQUIRED: &str =
    "CLI argument parser should have been set up to require a subcommand";

pub fn build() -> Command {
    Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg_required_else_help(true)
        .args_conflicts_with_subcommands(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .infer_subcommands(true)
        .subcommand_required(true)
        .subcommand(cmd_tree())
        .arg(arg_help())
        .arg(arg_version())
}

// ----------------------------------------------------------------------------
// commands
// ----------------------------------------------------------------------------

fn cmd_tree() -> Command {
    Command::new("tree")
        .about("process tree commands")
        .arg_required_else_help(true)
        .disable_help_flag(true)
        .subcommand_required(true)
        .subcommand(cmd_tree_modify())
        .subcommand(cmd_tree_show())
}

fn cmd_tree_modify() -> Command {
    Command::new("modify")
        .about("modify processes")
        .arg_required_else_help(true)
        .disable_help_flag(true)
        .subcommand_required(true)
        .subcommand(cmd_modify_affinity())
}

fn cmd_tree_show() -> Command {
    Command::new("show")
        .about("show processes")
        .arg_required_else_help(true)
        .disable_help_flag(true)
        .subcommand_required(true)
        .subcommand(cmd_show_affinity())
        .subcommand(cmd_show_backtrace())
        .subcommand(cmd_show_plain())
}

// ----------------------------------------------------------------------------
// leaf commands
// ----------------------------------------------------------------------------

fn cmd_modify_affinity() -> Command {
    Command::new("affinity")
        .arg(arg_cpuset())
        .arg(arg_help())
        .arg(arg_pid())
        .arg(arg_threads())
        .arg(arg_verbose())
        .about("modify process tree affinity (cpuset)")
}

fn cmd_show_affinity() -> Command {
    Command::new("affinity")
        .arg(arg_help())
        .arg(arg_pid())
        .arg(arg_threads())
        .about("show process tree with affinity (cpuset)")
}

fn cmd_show_backtrace() -> Command {
    Command::new("backtrace")
        .alias("bt")
        .arg(arg_help())
        .arg(arg_pid())
        .arg(arg_threads())
        .arg(arg_verbose())
        .about("show process tree with backtrace")
}

fn cmd_show_plain() -> Command {
    Command::new("plain")
        .arg(arg_help())
        .arg(arg_pid())
        .arg(arg_show_arguments())
        .arg(arg_threads())
        .about("show process tree")
}

// ----------------------------------------------------------------------------
// arguments
// ----------------------------------------------------------------------------

fn arg_cpuset() -> Arg {
    Arg::new("cpuset")
        .help("single integer or 'free' for all")
        .required(true)
        .action(ArgAction::Set)
        .value_parser(is_cpuset)
}

fn arg_help() -> Arg {
    Arg::new("help")
        .short('?')
        .long("help")
        .action(ArgAction::Help)
        .help("print help (use --help to see all options)")
        .long_help("Print help.")
}

fn arg_pid() -> Arg {
    Arg::new("pid")
        .help("process IDs")
        .action(ArgAction::Append)
        .required(stdin().is_terminal())
        .value_parser(is_pid)
}

fn arg_show_arguments() -> Arg {
    Arg::new("arguments")
        .long("arguments")
        .short('a')
        .action(ArgAction::SetTrue)
        .help("show arguments")
}

fn arg_threads() -> Arg {
    Arg::new("threads")
        .long("threads")
        .short('t')
        .action(ArgAction::SetTrue)
        .help("include threads")
}

fn arg_verbose() -> Arg {
    Arg::new("verbose")
        .long("verbose")
        .action(ArgAction::SetTrue)
        .long_help("Adds verbose output.")
        .hide_short_help(true)
}

fn arg_version() -> Arg {
    Arg::new("version")
        .long("version")
        .help("print version")
        .long_help("Print version.")
        .action(ArgAction::Version)
}

// ----------------------------------------------------------------------------
// value parsers
// ----------------------------------------------------------------------------

fn is_cpuset(s: &str) -> Result<String, String> {
    if s == "free" || s.parse::<u64>().is_ok() {
        Ok(String::from(s))
    } else {
        Err(format!("invalid cpuset: {s:?}"))
    }
}

fn is_pid(s: &str) -> Result<i32, String> {
    crate::pid::validate(s)
}

// ----------------------------------------------------------------------------
// tests
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    #[test]
    fn verify_cli() {
        super::build().debug_assert();
    }
}
