mod util;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;

#[test]
fn pid() -> Result<(), Box<dyn Error>> {
    let mut cmd = util::bin(&["tree", "show", "x"])?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not a process ID"));

    let mut cmd = util::bin(&["tree", "show", "1"])?;
    cmd.assert()
        .stderr(predicate::str::contains("not a process ID").not());

    let pid_max = std::fs::read_to_string("/proc/sys/kernel/pid_max")?
        .trim()
        .parse::<u64>()?;

    let mut cmd = util::bin(&["tree", "show", &format!("{}", pid_max + 1)])?;
    cmd.assert()
        .stderr(predicate::str::contains("higher than pid_max"));

    Ok(())
}
