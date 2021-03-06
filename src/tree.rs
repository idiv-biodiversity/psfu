use std::collections::HashMap;
use std::io::{self, BufRead};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;
use procfs::process::Process;

use crate::affinity;
use crate::config::Config;
use crate::log;

// ----------------------------------------------------------------------------
// CLI runner
// ----------------------------------------------------------------------------

/// Runs **tree** subcommand.
pub fn run(args: &ArgMatches) -> Result<()> {
    match args.subcommand() {
        ("show", Some(args)) => run_show(args),
        ("modify", Some(args)) => run_modify(args),

        // unreachable because subcommand is required
        _ => unreachable!(),
    }
}

/// Runs **tree modify** subcommand.
fn run_modify(args: &ArgMatches) -> Result<()> {
    match args.subcommand() {
        ("affinity", Some(args)) => run_modify_affinity(args),

        // unreachable because subcommand is required
        _ => unreachable!(),
    }
}

/// Runs **tree show** subcommand.
fn run_show(args: &ArgMatches) -> Result<()> {
    match args.subcommand() {
        ("affinity", Some(args)) => run_show_affinity(args),
        ("backtrace", Some(args)) => run_show_backtrace(args),
        ("plain", Some(args)) => run_show_plain(args),

        // unreachable because subcommand is required
        _ => unreachable!(),
    }
}

/// Runs **tree show plain** subcommand.
fn run_show_plain(args: &ArgMatches) -> Result<()> {
    let config = Config::from_args(args);

    let payload = |process: &Process| {
        let command = if config.arguments {
            process.cmdline().ok().map(|cmd| cmd.join(" "))
        } else {
            None
        };

        let command = command.as_ref().unwrap_or(&process.stat.comm);

        Ok(vec![format!("{} {}", process.pid, command)])
    };

    match args.values_of("pid") {
        Some(pids) => {
            for pid in pids {
                let pid: i32 = pid.parse().unwrap();
                let tree = ProcessTree::new(pid, &config)?;
                tree.show(&payload, 0);
            }
        }

        None => {
            for pid in piderator(io::stdin().lock()) {
                let tree = ProcessTree::new(pid, &config)?;
                tree.show(&payload, 0);
            }
        }
    }

    Ok(())
}

/// Runs **tree show affinity** subcommand.
fn run_show_affinity(args: &ArgMatches) -> Result<()> {
    let config = Config::from_args(args);

    let payload = |process: &Process| {
        let command = &process.stat.comm;

        affinity::get(process.pid).map(|affinity| {
            vec![format!("{} {} {:?}", process.pid, command, affinity)]
        })
    };

    match args.values_of("pid") {
        Some(pids) => {
            for pid in pids {
                let pid: i32 = pid.parse().unwrap();
                let tree = ProcessTree::new(pid, &config)?;
                tree.show(&payload, 0);
            }
        }

        None => {
            for pid in piderator(io::stdin().lock()) {
                let tree = ProcessTree::new(pid, &config)?;
                tree.show(&payload, 0);
            }
        }
    }

    Ok(())
}

/// Runs **tree show backtrace** subcommand.
fn run_show_backtrace(args: &ArgMatches) -> Result<()> {
    let config = Config::from_args(args);

    let payload = |process: &Process| {
        let pid = process.pid;
        let comm = &process.stat.comm;

        let mut gdb_cmd = Command::new("gdb");
        gdb_cmd
            // no ~/.gdbinit and no .gdbinit
            .args(&["-nh", "-nx"])
            // run backtrace in batch mode
            .args(&["-batch", "-ex", "bt"])
            // use this pid
            .arg("-p")
            .arg(process.pid.to_string());

        match gdb_cmd.output() {
            Ok(gdb) => {
                if gdb.status.success() {
                    let output = String::from_utf8_lossy(&gdb.stdout);

                    let mut payload = vec![];

                    for line in output.lines() {
                        if !line.starts_with('#') && !config.verbose {
                            continue;
                        }

                        payload.push(format!("{} {} {}", pid, comm, line));
                    }

                    Ok(payload)
                } else {
                    let error = String::from_utf8_lossy(&gdb.stderr)
                        .lines()
                        .fold(String::from(""), |acc, line| acc + " " + line);

                    Err(anyhow!("{} {} {}", pid, comm, error))
                }
            }

            Err(error) => {
                Err(anyhow!("{} {} failed to run gdb: {}", pid, comm, error))
            }
        }
    };

    match args.values_of("pid") {
        Some(pids) => {
            for pid in pids {
                let pid: i32 = pid.parse().unwrap();
                let tree = ProcessTree::new(pid, &config)?;
                tree.show(&payload, 0);
            }
        }

        None => {
            for pid in piderator(io::stdin().lock()) {
                let tree = ProcessTree::new(pid, &config)?;
                tree.show(&payload, 0);
            }
        }
    }

    Ok(())
}

/// Runs **tree modify affinity** subcommand.
fn run_modify_affinity(args: &ArgMatches) -> Result<()> {
    let config = Config::from_args(args);

    let cpuset: Vec<usize> = match args.value_of("cpuset").unwrap() {
        "free" => (0..libc::CPU_SETSIZE as usize).collect(),
        cpuset => vec![cpuset.parse().unwrap()],
    };

    let f = |process: &Process| {
        if config.verbose {
            let pid = &process.pid;
            let cmd = &process.stat.comm;
            eprintln!("modifying process {} {}", pid, cmd);
        }

        affinity::set(process.pid, &cpuset)
    };

    match args.values_of("pid") {
        Some(pids) => {
            for pid in pids {
                let pid: i32 = pid.parse().unwrap();
                let tree = ProcessTree::new(pid, &config)?;
                tree.modify(&f);
            }
        }

        None => {
            for pid in piderator(io::stdin().lock()) {
                let tree = ProcessTree::new(pid, &config)?;
                tree.modify(&f);
            }
        }
    }

    Ok(())
}

// ----------------------------------------------------------------------------
// process tree data structure
// ----------------------------------------------------------------------------

/// A process tree.
#[derive(Clone, Debug)]
struct ProcessTree {
    /// The root process of this tree.
    root: Process,

    /// The children of this tree.
    children: Vec<ProcessTree>,
}

impl ProcessTree {
    /// Returns a new process tree with parent `pid` as its root.
    fn new(parent: i32, config: &Config) -> Result<Self> {
        Self::from_parent(parent, config.threads).with_context(|| {
            format!("generating tree for root process {} failed", parent)
        })
    }

    /// Returns a new process tree with parent `pid` as its root.
    fn from_parent(pid: i32, threads: bool) -> Result<Self> {
        let root = Process::new(pid)
            .with_context(|| format!("reading process {} failed", pid))?;

        let mut tree = Self::leaf(root);

        let mut procs: HashMap<i32, Vec<Process>> = HashMap::new();

        for process in procfs::process::all_processes()
            .context("reading all processes failed")?
        {
            let children =
                procs.entry(process.stat.ppid).or_insert_with(Vec::new);

            children.push(process);
        }

        convert(&mut procs, &mut tree);

        if threads {
            add_threads(&mut tree).context("adding threads to tree failed")?
        }

        Ok(tree)
    }

    /// Returns a new process tree without children.
    const fn leaf(process: Process) -> Self {
        Self {
            root: process,
            children: vec![],
        }
    }

    /// Recursively modify the process tree.
    fn modify<F>(&self, f: &F)
    where
        F: Fn(&Process) -> Result<()>,
    {
        if let Err(e) = f(&self.root) {
            log::error(format!("{}", e));
        }

        for child in &self.children {
            child.modify(f)
        }
    }

    /// Recursively show the process tree.
    fn show<F>(&self, payload: &F, indent: usize)
    where
        F: Fn(&Process) -> Result<Vec<String>>,
    {
        let prefix = " ".repeat(indent);

        match payload(&self.root) {
            Ok(payload) => {
                for p in payload {
                    println!("{}{}", prefix, p);
                }
            }

            Err(e) => {
                eprintln!("{}{}", prefix, e);
            }
        }

        for child in &self.children {
            child.show(payload, indent + 2);
        }
    }
}

// ----------------------------------------------------------------------------
// tree recursion helpers
// ----------------------------------------------------------------------------

/// Recursively moves children from procs into tree.
fn convert(procs: &mut HashMap<i32, Vec<Process>>, tree: &mut ProcessTree) {
    if let Some(children) = procs.remove(&tree.root.pid) {
        tree.children = children.into_iter().map(ProcessTree::leaf).collect();

        for child in &mut tree.children {
            convert(procs, child)
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
        })?
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
                let task = Process::new(tid).with_context(|| {
                    format!("reading thread {} failed", tid)
                })?;

                let task = ProcessTree::leaf(task);

                tree.children.push(task);
            }
        }
    }

    Ok(())
}

// ----------------------------------------------------------------------------
// utility Iterator for reading PIDs from STDIN
// ----------------------------------------------------------------------------

fn piderator<'a>(s: io::StdinLock<'a>) -> impl Iterator<Item = i32> + 'a {
    PIDerator::new(s.lines()).flatten()
}

struct PIDerator<B> {
    underlying: io::Lines<B>,
}

impl<B: BufRead> PIDerator<B> {
    fn new(b: io::Lines<B>) -> Self {
        Self { underlying: b }
    }
}

impl<B: BufRead> Iterator for PIDerator<B> {
    type Item = Option<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.underlying.next() {
            Some(Ok(line)) if line.trim().is_empty() => Some(None),

            Some(Ok(line)) => match crate::pid::validate(&line) {
                Ok(pid) => Some(Some(pid)),
                Err(e) => {
                    log::error(e);
                    Some(None)
                }
            },

            Some(Err(e)) => {
                log::error(format!("broken line: {}", e));
                Some(None)
            }

            None => None,
        }
    }
}
