use std::collections::HashMap;
use std::io::{self, BufRead};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;
use procfs::process::Process;
use termtree::Tree;

use crate::affinity;
use crate::log;
use crate::nice;

// ----------------------------------------------------------------------------
// CLI runner
// ----------------------------------------------------------------------------

/// Runs **tree** subcommand.
pub fn run(args: &ArgMatches) -> Result<()> {
    match args.subcommand() {
        Some(("show", args)) => run_show(args),
        Some(("modify", args)) => run_modify(args),
        _ => unreachable!("{}", crate::cli::SUBCOMMAND_REQUIRED),
    }
}

/// Runs **tree modify** subcommand.
fn run_modify(args: &ArgMatches) -> Result<()> {
    match args.subcommand() {
        Some(("affinity", args)) => run_modify_affinity(args),
        Some(("nice", args)) => run_modify_nice(args),
        _ => unreachable!("{}", crate::cli::SUBCOMMAND_REQUIRED),
    }
}

/// Runs **tree show** subcommand.
fn run_show(args: &ArgMatches) -> Result<()> {
    match args.subcommand() {
        Some(("affinity", args)) => run_show_affinity(args),
        Some(("backtrace", args)) => run_show_backtrace(args),
        Some(("nice", args)) => run_show_nice(args),
        Some(("plain", args)) => run_show_plain(args),
        _ => unreachable!("{}", crate::cli::SUBCOMMAND_REQUIRED),
    }
}

/// Runs **tree show plain** subcommand.
fn run_show_plain(args: &ArgMatches) -> Result<()> {
    let arguments = args.get_flag("arguments");
    let threads = args.get_flag("threads");

    let payload = |process: &Process| {
        let command = if arguments {
            process.cmdline().ok().map(|cmd| cmd.join(" "))
        } else {
            None
        };

        Ok(format!(
            "{} {}",
            process.pid,
            command.as_ref().unwrap_or(&process.stat()?.comm)
        ))
    };

    if let Some(pids) = args.get_many("pid") {
        for pid in pids {
            let tree = ProcessTree::new(*pid, threads)?;
            let tree = tree.to_termtree(&payload);
            println!("{tree}");
        }
    } else {
        for pid in piderator(io::stdin()) {
            let tree = ProcessTree::new(pid, threads)?;
            let tree = tree.to_termtree(&payload);
            println!("{tree}");
        }
    }

    Ok(())
}

/// Runs **tree show affinity** subcommand.
fn run_show_affinity(args: &ArgMatches) -> Result<()> {
    let threads = args.get_flag("threads");

    let payload = |process: &Process| {
        let command = &process.stat()?.comm;

        affinity::get(process.pid)
            .map(|affinity| format!("{} {command} {affinity:?}", process.pid))
    };

    if let Some(pids) = args.get_many("pid") {
        for pid in pids {
            let tree = ProcessTree::new(*pid, threads)?;
            let tree = tree.to_termtree(&payload);
            println!("{tree}");
        }
    } else {
        for pid in piderator(io::stdin()) {
            let tree = ProcessTree::new(pid, threads)?;
            let tree = tree.to_termtree(&payload);
            println!("{tree}");
        }
    }

    Ok(())
}

/// Runs **tree show backtrace** subcommand.
fn run_show_backtrace(args: &ArgMatches) -> Result<()> {
    let threads = args.get_flag("threads");
    let verbose = args.get_flag("verbose");

    let payload = |process: &Process| {
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

                        payload.push(format!("{pid} {comm} {line}"));
                    }

                    Ok(payload.join("\n"))
                } else {
                    let error = String::from_utf8_lossy(&gdb.stderr)
                        .lines()
                        .fold(String::default(), |acc, line| acc + " " + line);

                    Err(anyhow!("{} {} {}", pid, comm, error))
                }
            }

            Err(error) => {
                Err(anyhow!("{} {} failed to run gdb: {}", pid, comm, error))
            }
        }
    };

    if let Some(pids) = args.get_many("pid") {
        for pid in pids {
            let tree = ProcessTree::new(*pid, threads)?;
            let tree = tree.to_termtree(&payload);
            println!("{tree}");
        }
    } else {
        for pid in piderator(io::stdin()) {
            let tree = ProcessTree::new(pid, threads)?;
            let tree = tree.to_termtree(&payload);
            println!("{tree}");
        }
    }

    Ok(())
}

/// Runs **tree show nice** subcommand.
fn run_show_nice(args: &ArgMatches) -> Result<()> {
    let threads = args.get_flag("threads");

    let payload = |process: &Process| {
        let command = &process.stat()?.comm;
        let pid = process.pid;

        // need to convert into u32 as required by libc::getpriority
        pid.try_into().map_or_else(
            |_| Ok(format!("invalid process id: {pid}")),
            |pid| {
                nice::get(pid).map(|value| format!("{pid} {command} {value}"))
            },
        )
    };

    if let Some(pids) = args.get_many("pid") {
        for pid in pids {
            let tree = ProcessTree::new(*pid, threads)?;
            let tree = tree.to_termtree(&payload);
            println!("{tree}");
        }
    } else {
        for pid in piderator(io::stdin()) {
            let tree = ProcessTree::new(pid, threads)?;
            let tree = tree.to_termtree(&payload);
            println!("{tree}");
        }
    }

    Ok(())
}

/// Runs **tree modify affinity** subcommand.
fn run_modify_affinity(args: &ArgMatches) -> Result<()> {
    let threads = args.get_flag("threads");
    let verbose = args.get_flag("verbose");

    let cpuset: Vec<usize> = match args
        .get_one::<String>("cpuset")
        .map(String::as_str)
        .unwrap()
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

    if let Some(pids) = args.get_many("pid") {
        for pid in pids {
            let tree = ProcessTree::new(*pid, threads)?;
            tree.modify(&f);
        }
    } else {
        for pid in piderator(io::stdin()) {
            let tree = ProcessTree::new(pid, threads)?;
            tree.modify(&f);
        }
    }

    Ok(())
}

/// Runs **tree modify nice** subcommand.
fn run_modify_nice(args: &ArgMatches) -> Result<()> {
    let threads = args.get_flag("threads");
    let verbose = args.get_flag("verbose");

    let niceness = args.get_one::<i32>("niceness").copied().unwrap_or(10);

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

    if let Some(pids) = args.get_many("pid") {
        for pid in pids {
            let tree = ProcessTree::new(*pid, threads)?;
            tree.modify(&f);
        }
    } else {
        for pid in piderator(io::stdin()) {
            let tree = ProcessTree::new(pid, threads)?;
            tree.modify(&f);
        }
    }

    Ok(())
}

// ----------------------------------------------------------------------------
// process tree data structure
// ----------------------------------------------------------------------------

/// A process tree.
#[derive(Debug)]
struct ProcessTree {
    /// The root process of this tree.
    root: Process,

    /// The children of this tree.
    children: Vec<ProcessTree>,
}

impl ProcessTree {
    /// Returns a new process tree with parent `pid` as its root.
    fn new(pid: i32, threads: bool) -> Result<Self> {
        let root = Process::new(pid)
            .with_context(|| format!("reading process {pid} failed"))?;

        let mut tree = Self::from(root);

        let mut procs: HashMap<i32, Vec<Process>> = HashMap::new();

        for process in procfs::process::all_processes()
            .context("reading all processes failed")?
        {
            let process = process?;

            let children = procs.entry(process.stat()?.ppid).or_default();

            children.push(process);
        }

        convert(&mut procs, &mut tree);

        if threads {
            add_threads(&mut tree).context("adding threads to tree failed")?;
        }

        Ok(tree)
    }

    /// Recursively modify the process tree.
    fn modify<F>(&self, f: &F)
    where
        F: Fn(&Process) -> Result<()>,
    {
        if let Err(e) = f(&self.root) {
            log::error(format!("{e}"));
        }

        for child in &self.children {
            child.modify(f);
        }
    }

    fn to_termtree<F>(&self, payload: &F) -> Tree<String>
    where
        F: Fn(&Process) -> Result<String>,
    {
        let p = match payload(&self.root) {
            Ok(payload) => payload,
            Err(e) => format!("{e}"),
        };

        let mut tree = Tree::new(p);
        tree.set_multiline(true);

        for child in &self.children {
            tree.push(child.to_termtree(payload));
        }

        tree
    }
}

impl From<Process> for ProcessTree {
    fn from(process: Process) -> Self {
        Self {
            root: process,
            children: vec![],
        }
    }
}

// ----------------------------------------------------------------------------
// tree recursion helpers
// ----------------------------------------------------------------------------

/// Recursively moves children from procs into tree.
fn convert(procs: &mut HashMap<i32, Vec<Process>>, tree: &mut ProcessTree) {
    if let Some(children) = procs.remove(&tree.root.pid) {
        tree.children = children.into_iter().map(ProcessTree::from).collect();

        for child in &mut tree.children {
            convert(procs, child);
        }
    }
}

/// Recursively adds threads to the children of their respective parent
/// processes in the tree.
fn add_threads(tree: &mut ProcessTree) -> Result<()> {
    for child in &mut tree.children {
        add_threads(child).with_context(|| {
            format!(
                "adding threads for child process {} failed",
                child.root.pid
            )
        })?;
    }

    let path = format!("/proc/{}/task", tree.root.pid);

    if let Ok(entries) = std::fs::read_dir(path) {
        for tid in entries {
            let tid = tid?
                .file_name()
                .into_string()
                .unwrap()
                .parse::<i32>()
                .unwrap();

            if tid != tree.root.pid {
                let task = Process::new(tid)
                    .with_context(|| format!("reading thread {tid} failed"))?;

                let task = ProcessTree::from(task);

                tree.children.push(task);
            }
        }
    }

    Ok(())
}

// ----------------------------------------------------------------------------
// utility Iterator for reading PIDs from STDIN
// ----------------------------------------------------------------------------

fn piderator(stdin: io::Stdin) -> impl Iterator<Item = i32> {
    PIDerator::from(stdin).flatten()
}

struct PIDerator<B> {
    underlying: io::Lines<B>,
}

impl<B> From<io::Lines<B>> for PIDerator<B> {
    fn from(underlying: io::Lines<B>) -> Self {
        Self { underlying }
    }
}

impl From<io::Stdin> for PIDerator<io::StdinLock<'_>> {
    fn from(stdin: io::Stdin) -> Self {
        Self::from(stdin.lines())
    }
}

impl<B: BufRead> Iterator for PIDerator<B> {
    type Item = Option<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.underlying.next() {
            Some(Ok(line)) if line.trim().is_empty() => Some(None),

            Some(Ok(line)) => match crate::pid::validate(line) {
                Ok(pid) => Some(Some(pid)),
                Err(e) => {
                    log::error(e);
                    Some(None)
                }
            },

            Some(Err(e)) => {
                log::error(format!("broken line: {e}"));
                Some(None)
            }

            None => None,
        }
    }
}
