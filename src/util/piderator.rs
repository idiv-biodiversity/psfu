use std::io::{self, BufRead};

use clap::ArgMatches;

use crate::log;

/// Returns an iterator reading process IDs from `ArgMatches` if there are any
/// or from `STDIN` otherwise.
pub fn args_or_stdin(args: &ArgMatches) -> Box<dyn Iterator<Item = i32> + '_> {
    if let Some(pids) = args.get_many("pid") {
        Box::new(pids.copied())
    } else {
        Box::new(stdin())
    }
}

/// Returns an iterator reading process IDs from `STDIN`.
fn stdin() -> impl Iterator<Item = i32> {
    PIDerator::from(io::stdin()).flatten()
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
