use procfs::sys::kernel::pid_max;

pub fn validate<S: AsRef<str>>(s: S) -> Result<i32, String> {
    let s = s.as_ref();

    s.parse::<i32>()
        .map_err(|e| format!("{:?}: {}", s, e))
        .and_then(check_range)
}

fn check_range(pid: i32) -> Result<i32, String> {
    let pid_max = pid_max().map_err(|e| format!("{}", e))?;

    if pid < 1 || pid > pid_max {
        Err(format!("invalid PID: {}", pid))
    } else {
        Ok(pid)
    }
}
