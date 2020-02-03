use anyhow::{Context, Result};
use clap::ArgMatches;
use procfs::process::Process;
use std::collections::HashMap;
use std::process::{Command, Stdio};

use crate::config::Config;

/// Runs **tree** subcommand.
pub fn run(args: &ArgMatches) -> Result<()> {
    match args.subcommand() {
        ("backtrace", Some(args)) => {
            let config = Config::from_args(&args);
            let pid: i32 = args.value_of("pid").unwrap().parse().unwrap();
            let tree = ProcessTree::new(pid, &config)?;

            tree.print_backtrace(config.verbose)
        }

        ("show", Some(args)) => {
            let config = Config::from_args(&args);
            let pid: i32 = args.value_of("pid").unwrap().parse().unwrap();
            let tree = ProcessTree::new(pid, &config)?;

            tree.print();

            Ok(())
        }

        // unreachable because subcommand is required
        _ => unreachable!(),
    }
}

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
    fn new(parent: i32, config: &Config) -> Result<ProcessTree> {
        ProcessTree::from_parent(parent, config.threads).with_context(|| {
            format!("generating tree for root process {} failed", parent)
        })
    }

    /// Returns a new process tree with parent `pid` as its root.
    fn from_parent(pid: i32, threads: bool) -> Result<ProcessTree> {
        let root = Process::new(pid)
            .with_context(|| format!("reading process {} failed", pid))?;

        let mut tree = ProcessTree::leaf(root);

        let mut procs: HashMap<i32, Vec<Process>> = HashMap::new();

        for process in procfs::process::all_processes()
            .context("reading all processes failed")?
        {
            let children =
                procs.entry(process.stat.ppid).or_insert_with(|| vec![]);

            children.push(process);
        }

        convert_rec(&mut procs, &mut tree);

        if threads {
            add_threads_rec(&mut tree)
                .context("adding threads to tree failed")?
        }

        Ok(tree)
    }

    /// Returns a new process tree without children.
    fn leaf(process: Process) -> ProcessTree {
        ProcessTree {
            root: process,
            children: vec![],
        }
    }

    /// Prints this tree.
    fn print(&self) {
        print_rec(self, 0)
    }

    /// Prints the tree of backtraces.
    fn print_backtrace(&self, verbose: bool) -> Result<()> {
        backtrace_rec(self, 0, verbose).context("tracing failed")
    }
}

/// Recursively moves children from procs into tree.
fn convert_rec(
    procs: &mut HashMap<i32, Vec<Process>>,
    tree: &mut ProcessTree,
) {
    if let Some(children) = procs.remove(&tree.root.pid) {
        tree.children = children.into_iter().map(ProcessTree::leaf).collect();

        for child in tree.children.iter_mut() {
            convert_rec(procs, child)
        }
    }
}

/// Recursively adds threads to the children of their respective parent
/// processes in the tree.
fn add_threads_rec(tree: &mut ProcessTree) -> Result<()> {
    for child in tree.children.iter_mut() {
        add_threads_rec(child).with_context(|| {
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

/// Prints the tree.
fn print_rec(tree: &ProcessTree, indent: usize) {
    let prefix = " ".repeat(indent);

    println!("{}{} {}", prefix, tree.root.pid, tree.root.stat.comm);

    for child in &tree.children {
        print_rec(child, indent + 2);
    }
}

/// Prints the tree including backtraces.
fn backtrace_rec(
    tree: &ProcessTree,
    indent: usize,
    verbose: bool,
) -> Result<()> {
    let prefix = " ".repeat(indent);

    let stderr = if verbose {
        Stdio::inherit()
    } else {
        Stdio::null()
    };

    let mut command = Command::new("gdb");
    command
        .args(&["-nh", "-nx"])
        .args(&["-batch", "-ex", "bt"])
        .arg("-p")
        .arg(format!("{}", tree.root.pid));

    let gdb = command
        .stdout(Stdio::piped())
        .stderr(stderr)
        .output()
        .with_context(|| format!("running {:?} failed", command))?;

    let output = String::from_utf8_lossy(&gdb.stdout);

    for line in output.lines() {
        if !line.starts_with('#') && !verbose {
            continue;
        }

        println!(
            "{}{} {} {}",
            prefix, tree.root.pid, tree.root.stat.comm, line
        );
    }

    for child in &tree.children {
        let result = backtrace_rec(child, indent + 2, verbose);

        if let Err(e) = result {
            let prefix = " ".repeat(indent + 2);

            println!(
                "{}{} {} [error] {:#}",
                prefix, child.root.pid, child.root.stat.comm, e
            );
        }
    }

    Ok(())
}
