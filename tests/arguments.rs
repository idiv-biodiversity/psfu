mod util;

use std::error::Error;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn pid() -> Result<(), Box<dyn Error>> {
    let mut cmd = util::bin(&["tree", "show", "plain", "x"])?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error: Invalid value"));

    let mut cmd = util::bin(&["tree", "show", "plain", "0"])?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error: Invalid value"));

    let mut cmd = util::bin(&["tree", "show", "plain", "1"])?;
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("error: Invalid value").not());

    let pid_max = std::fs::read_to_string("/proc/sys/kernel/pid_max")?
        .trim()
        .parse::<u64>()?;

    let too_high = format!("{}", pid_max + 1);

    let mut cmd = util::bin(&["tree", "show", "plain", &too_high])?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error: Invalid value"));

    Ok(())
}
