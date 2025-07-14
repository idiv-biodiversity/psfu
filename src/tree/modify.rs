use anyhow::{Result, anyhow};
use clap::ArgMatches;
use procfs::process::Process;

use crate::affinity;
use crate::nice;
use crate::tree::{ProcessTree, Threads};
use crate::util::piderator;

/// Runs `tree modify` subcommand.
pub fn run(args: &ArgMatches) -> Result<()> {
    match args.subcommand() {
        Some(("affinity", args)) => run_affinity(args),
        Some(("nice", args)) => run_nice(args),
        Some(("oom_score_adj", args)) => run_oom_score_adj(args),
        _ => unreachable!("{}", crate::cli::SUBCOMMAND_REQUIRED),
    }
}

/// Runs `tree modify affinity` subcommand.
fn run_affinity(args: &ArgMatches) -> Result<()> {
    let verbose = args.get_flag("verbose");

    let cpuset: Vec<usize> = match args
        .get_one::<String>("cpuset")
        .map(String::as_str)
        .expect("cpuset is a required argument")
    {
        "free" => (0..libc::CPU_SETSIZE as usize).collect(),
        cpuset => vec![cpuset.parse().unwrap()],
    };

    let f = |process: &Process| {
        if verbose {
            let pid = &process.pid;
            let cmd = &process.stat()?.comm;
            eprintln!("modifying process {pid} {cmd}");
        }

        affinity::set(process.pid, &cpuset)
    };

    modify_tree(args, f)
}

/// Runs `tree modify nice` subcommand.
fn run_nice(args: &ArgMatches) -> Result<()> {
    let verbose = args.get_flag("verbose");

    let niceness = args
        .get_one::<i32>("niceness")
        .copied()
        .expect("niceness is a required argument");

    let f = |process: &Process| {
        if verbose {
            let pid = &process.pid;
            let cmd = &process.stat()?.comm;
            eprintln!("modifying process {pid} {cmd}");
        }

        // need to convert into u32 as required by libc::getpriority
        process.pid.try_into().map_or_else(
            |_| Err(anyhow!("invalid process id: {}", process.pid)),
            |pid| nice::set(pid, niceness),
        )
    };

    modify_tree(args, f)
}

/// Runs `tree modify oom_score_adj` subcommand.
fn run_oom_score_adj(args: &ArgMatches) -> Result<()> {
    let verbose = args.get_flag("verbose");

    let adjustment = args
        .get_one::<i16>("oom_score_adj")
        .copied()
        .expect("oom score adjustment is a required argument");

    let f = |process: &Process| {
        if verbose {
            let pid = &process.pid;
            let cmd = &process.stat()?.comm;
            eprintln!("modifying process {pid} {cmd}");
        }

        process.set_oom_score_adj(adjustment)?;

        Ok(())
    };

    modify_tree(args, f)
}

// ----------------------------------------------------------------------------
// helper
// ----------------------------------------------------------------------------

/// Modify process tree from arguments or STDIN with changes from `f`.
fn modify_tree<F>(args: &ArgMatches, f: F) -> Result<()>
where
    F: Fn(&Process) -> Result<()>,
{
    for pid in piderator::args_or_stdin(args) {
        let tree = ProcessTree::new(pid, Threads(true))?;
        tree.modify(&f);
    }

    Ok(())
}
