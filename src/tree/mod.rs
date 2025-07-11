mod modify;
mod show;

use std::collections::HashMap;

use anyhow::{Context, Result};
use clap::ArgMatches;
use procfs::process::Process;
use termtree::Tree;

use crate::log;

// ----------------------------------------------------------------------------
// CLI runner
// ----------------------------------------------------------------------------

/// Runs `tree` subcommand.
pub fn run(args: &ArgMatches) -> Result<()> {
    match args.subcommand() {
        Some(("modify", args)) => modify::run(args),
        Some(("show", args)) => show::run(args),
        _ => unreachable!("{}", crate::cli::SUBCOMMAND_REQUIRED),
    }
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

        tree.convert(&mut procs);

        if threads {
            tree.add_threads()
                .context("adding threads to tree failed")?;
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

    /// Recursively adds threads to the children of their respective parent
    /// processes in the tree.
    fn add_threads(&mut self) -> Result<()> {
        for child in &mut self.children {
            child.add_threads().with_context(|| {
                format!(
                    "adding threads for child process {} failed",
                    child.root.pid
                )
            })?;
        }

        let path = format!("/proc/{}/task", self.root.pid);

        if let Ok(entries) = std::fs::read_dir(path) {
            for tid in entries {
                let tid = tid?
                    .file_name()
                    .into_string()
                    .unwrap()
                    .parse::<i32>()
                    .unwrap();

                if tid != self.root.pid {
                    let task = Process::new(tid).with_context(|| {
                        format!("reading thread {tid} failed")
                    })?;

                    let task = Self::from(task);

                    self.children.push(task);
                }
            }
        }

        Ok(())
    }

    /// Recursively moves children from procs into tree.
    fn convert(&mut self, procs: &mut HashMap<i32, Vec<Process>>) {
        if let Some(children) = procs.remove(&self.root.pid) {
            self.children = children.into_iter().map(Self::from).collect();

            for child in &mut self.children {
                child.convert(procs);
            }
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
