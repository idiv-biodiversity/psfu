use std::io::{stdin, IsTerminal};

use clap::{crate_description, crate_name, crate_version};
use clap::{Arg, ArgAction, Command};

pub fn build() -> Command {
    // ------------------------------------------------------------------------
    // arguments
    // ------------------------------------------------------------------------

    let pid = Arg::new("pid")
        .help("process IDs")
        .action(ArgAction::Append)
        .required(stdin().is_terminal())
        .value_parser(is_pid);

    let arguments = Arg::new("arguments")
        .long("arguments")
        .short('a')
        .action(ArgAction::SetTrue)
        .help("show arguments");

    let cpuset = Arg::new("cpuset")
        .help("single integer or 'free' for all")
        .required(true)
        .action(ArgAction::Set)
        .value_parser(is_cpuset);

    let threads = Arg::new("threads")
        .long("threads")
        .short('t')
        .action(ArgAction::SetTrue)
        .help("include threads");

    let verbose = Arg::new("verbose")
        .long("verbose")
        .action(ArgAction::SetTrue)
        .long_help("Adds verbose output.")
        .hide_short_help(true);

    // ------------------------------------------------------------------------
    // help/version modifications
    // ------------------------------------------------------------------------

    let help = Arg::new("help")
        .short('?')
        .long("help")
        .action(ArgAction::Help)
        .help("print help (use --help to see all options)")
        .long_help("Print help.");

    let version = Arg::new("version")
        .long("version")
        .help("print version")
        .long_help("Print version.")
        .action(ArgAction::Version);

    // ------------------------------------------------------------------------
    // show commands
    // ------------------------------------------------------------------------

    let plain = Command::new("plain")
        .arg(&help)
        .arg(&pid)
        .arg(&arguments)
        .arg(&threads)
        .about("show process tree");

    let affinity_s = Command::new("affinity")
        .arg(&help)
        .arg(&pid)
        .arg(&threads)
        .about("show process tree with affinity (cpuset)");

    let backtrace = Command::new("backtrace")
        .alias("bt")
        .arg(&help)
        .arg(&pid)
        .arg(&threads)
        .arg(&verbose)
        .about("show process tree with backtrace");

    // ------------------------------------------------------------------------
    // modify commands
    // ------------------------------------------------------------------------

    let affinity_m = Command::new("affinity")
        .arg(&cpuset)
        .arg(&help)
        .arg(&pid)
        .arg(&threads)
        .arg(&verbose)
        .about("modify process tree affinity (cpuset)");

    // ------------------------------------------------------------------------
    // tree commands
    // ------------------------------------------------------------------------

    let modify = Command::new("modify")
        .about("modify processes")
        .arg_required_else_help(true)
        .disable_help_flag(true)
        .subcommand_required(true)
        .subcommand(affinity_m);

    let show = Command::new("show")
        .about("show processes")
        .arg_required_else_help(true)
        .disable_help_flag(true)
        .subcommand_required(true)
        .subcommand(affinity_s)
        .subcommand(backtrace)
        .subcommand(plain);

    // ------------------------------------------------------------------------
    // top-level commands
    // ------------------------------------------------------------------------

    let tree = Command::new("tree")
        .about("process tree commands")
        .arg_required_else_help(true)
        .disable_help_flag(true)
        .subcommand_required(true)
        .subcommand(modify)
        .subcommand(show);

    // ------------------------------------------------------------------------
    // put it all together
    // ------------------------------------------------------------------------

    Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg_required_else_help(true)
        .args_conflicts_with_subcommands(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .infer_subcommands(true)
        .subcommand_required(true)
        .subcommand(tree)
        .arg(help)
        .arg(version)
}

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

pub const SUBCOMMAND_REQUIRED: &str =
    "CLI argument parser should have been set up to require a subcommand";

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
