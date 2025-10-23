use std::process::Command;

use anyhow::{Result, anyhow};
use clap::ArgMatches;
use procfs::process::Process;

use crate::affinity;
use crate::nice;
use crate::tree::{ProcessTree, Threads};
use crate::util::pid::ProcessID;
use crate::util::piderator;

/// Runs `tree show` subcommand.
pub fn run(args: &ArgMatches) -> Result<()> {
    match args.subcommand() {
        Some(("affinity", args)) => run_affinity(args),
        Some(("backtrace", args)) => run_backtrace(args),
        Some(("nice", args)) => run_nice(args),
        Some(("oom_score", args)) => run_oom_score(args),
        Some(("oom_score_adj", args)) => run_oom_score_adj(args),
        Some(("plain", args)) => run_plain(args),
        _ => unreachable!("{}", crate::cli::SUBCOMMAND_REQUIRED),
    }
}

/// Runs `tree show affinity` subcommand.
fn run_affinity(args: &ArgMatches) -> Result<()> {
    let payload = |process: Process| {
        affinity::get(process.pid).map(|affinity| format!("{affinity:?}"))
    };

    print_tree(args, payload)
}

/// Runs `tree show backtrace` subcommand.
fn run_backtrace(args: &ArgMatches) -> Result<()> {
    let verbose = args.get_flag("verbose");

    let payload = |process: Process| {
        let pid = process.pid;
        let comm = &process.stat()?.comm;

        let mut gdb_cmd = Command::new("gdb");
        gdb_cmd
            // no ~/.gdbinit and no .gdbinit
            .args(["-nh", "-nx"])
            // run backtrace in batch mode
            .args(["-batch", "-ex", "bt"])
            // use this pid
            .arg("-p")
            .arg(process.pid.to_string());

        match gdb_cmd.output() {
            Ok(gdb) => {
                if gdb.status.success() {
                    let output = String::from_utf8_lossy(&gdb.stdout);

                    let mut payload = vec![];

                    for line in output.lines() {
                        if !line.starts_with('#') && !verbose {
                            continue;
                        }

                        payload.push(line);
                    }

                    Ok(payload.join("\n"))
                } else {
                    let error = String::from_utf8_lossy(&gdb.stderr)
                        .lines()
                        .fold(String::default(), |acc, line| acc + " " + line);

                    Err(anyhow!("{pid} {comm} {error}"))
                }
            }

            Err(error) => {
                Err(anyhow!("{pid} {comm} failed to run gdb: {error}"))
            }
        }
    };

    print_tree(args, payload)
}

/// Runs `tree show nice` subcommand.
fn run_nice(args: &ArgMatches) -> Result<()> {
    let payload = |process: Process| {
        let pid = process.pid;

        // need to convert into u32 as required by libc::getpriority
        pid.try_into().map_or_else(
            |_| Ok(format!("invalid process id: {pid}")),
            |pid| nice::get(pid).map(|value| format!("{value}")),
        )
    };

    print_tree(args, payload)
}

/// Runs `tree show oom_score` subcommand.
fn run_oom_score(args: &ArgMatches) -> Result<()> {
    let payload = |process: Process| {
        process
            .oom_score()
            .map(|value| format!("{value}"))
            .map_err(From::from)
    };

    print_tree(args, payload)
}

/// Runs `tree show oom_score_adj` subcommand.
fn run_oom_score_adj(args: &ArgMatches) -> Result<()> {
    let payload = |process: Process| {
        process
            .oom_score_adj()
            .map(|value| format!("{value}"))
            .map_err(From::from)
    };

    print_tree(args, payload)
}

/// Runs `tree show plain` subcommand.
fn run_plain(args: &ArgMatches) -> Result<()> {
    let payload = |_: Process| Ok(String::new());

    print_tree(args, payload)
}

// ----------------------------------------------------------------------------
// helper
// ----------------------------------------------------------------------------

/// Print process tree from arguments or STDIN with content from payload
/// function.
fn print_tree<F>(args: &ArgMatches, payload: F) -> Result<()>
where
    F: Fn(Process) -> Result<String>,
{
    let arguments = args.get_flag("arguments");

    let payload = |pid: ProcessID| {
        let pid = pid.0;
        let process = Process::new(pid)?;
        let comm = &process.stat()?.comm;

        let command = if arguments {
            process.cmdline().ok().map(|cmd| cmd.join(" "))
        } else {
            None
        };
        let command = command.as_ref().unwrap_or(comm);

        let mut output = vec![];

        let p = payload(process)?;

        if p.trim().is_empty() {
            output.push(format!("{pid} {command}"));
        } else {
            for line in p.lines() {
                output.push(format!("{pid} {command} {line}"));
            }
        }

        Ok(output.join("\n"))
    };

    let threads = args.get_flag("threads");

    for pid in piderator::args_or_stdin(args) {
        let tree = ProcessTree::new(pid, Threads(threads))?;
        let tree = tree.to_termtree(&payload);
        println!("{tree}");
    }

    Ok(())
}
