mod util;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;

#[test]
fn pid() -> Result<(), Box<dyn Error>> {
    let mut cmd = util::bin(&["tree", "show", "plain", "x"])?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not a process ID"));

    let mut cmd = util::bin(&["tree", "show", "plain", "1"])?;
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("not a process ID").not());

    let pid_max = std::fs::read_to_string("/proc/sys/kernel/pid_max")?
        .trim()
        .parse::<u64>()?;

    let too_high = format!("{}", pid_max + 1);

    let mut cmd = util::bin(&["tree", "show", "plain", &too_high])?;
    cmd.assert()
        .stderr(predicate::str::contains("higher than pid_max"));

    Ok(())
}
