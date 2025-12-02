mod util;

use std::error::Error;

use predicates::prelude::*;

#[test]
fn pid() -> Result<(), Box<dyn Error>> {
    let mut cmd = util::bin(&["tree", "show", "plain", "x"])?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::is_match("error: [Ii]nvalid value").unwrap());

    let mut cmd = util::bin(&["tree", "show", "plain", "0"])?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::is_match("error: [Ii]nvalid value").unwrap());

    let mut cmd = util::bin(&["tree", "show", "plain", "1"])?;
    cmd.assert().success().stderr(
        predicate::str::is_match("error: [Ii]nvalid value")
            .unwrap()
            .not(),
    );

    let pid_max = procfs::sys::kernel::pid_max()?;
    let too_high = format!("{}", pid_max + 1);

    let mut cmd = util::bin(&["tree", "show", "plain", &too_high])?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::is_match("error: [Ii]nvalid value").unwrap());

    Ok(())
}
