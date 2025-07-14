mod modify;
mod show;

use std::collections::HashMap;

use anyhow::{Context, Result};
use clap::ArgMatches;
use procfs::process::Process;
use termtree::Tree;

use crate::log;
use crate::util::pid::ProcessID;

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

#[derive(Clone, Copy)]
struct Threads(bool);

/// A process tree.
#[derive(Debug)]
struct ProcessTree {
    /// The root process of this tree.
    root: ProcessID,

    /// The children of this tree.
    children: Vec<ProcessTree>,
}

impl ProcessTree {
    /// Returns a new process tree with parent `pid` as its root.
    fn new(pid: i32, threads: Threads) -> Result<Self> {
        let root = ProcessID(pid);

        let mut tree = Self::from(root);

        let mut procs: HashMap<ProcessID, Vec<ProcessID>> = HashMap::new();

        for process in procfs::process::all_processes()
            .context("reading all processes failed")?
        {
            let process = process?;

            let children =
                procs.entry(ProcessID(process.stat()?.ppid)).or_default();

            children.push(ProcessID(process.pid));
        }

        tree.convert(&mut procs);

        if threads.0 {
            tree.add_threads()
                .context("adding threads to tree failed")?;
        }

        Ok(tree)
    }

    /// Recursively modify the process tree.
    fn modify<F>(&self, f: &F)
    where
        F: Fn(Process) -> Result<()>,
    {
        if let Err(e) = self.root.into_process().and_then(f) {
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
                    child.root.0
                )
            })?;
        }

        let path = format!("/proc/{}/task", self.root.0);

        if let Ok(entries) = std::fs::read_dir(path) {
            for tid in entries {
                let tid = tid?
                    .file_name()
                    .into_string()
                    .unwrap()
                    .parse::<i32>()
                    .unwrap();

                if tid != self.root.0 {
                    let task = Self::from(ProcessID(tid));

                    self.children.push(task);
                }
            }
        }

        Ok(())
    }

    /// Recursively moves children from procs into tree.
    fn convert(&mut self, procs: &mut HashMap<ProcessID, Vec<ProcessID>>) {
        if let Some(children) = procs.remove(&self.root) {
            self.children = children.into_iter().map(Self::from).collect();

            for child in &mut self.children {
                child.convert(procs);
            }
        }
    }

    fn to_termtree<F>(&self, payload: &F) -> Tree<String>
    where
        F: Fn(ProcessID) -> Result<String>,
    {
        let p = match payload(self.root) {
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

impl From<ProcessID> for ProcessTree {
    fn from(root: ProcessID) -> Self {
        Self {
            root,
            children: vec![],
        }
    }
}
