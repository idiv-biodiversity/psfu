//! Getting and setting process niceness.

use anyhow::{Result, anyhow};
use errno::Errno;

/// Returns the niceness of the given process.
pub fn get(pid: u32) -> Result<i32> {
    // because C APIs suck: might return -1 both as indicatior of failure and
    // legitimate return value, so we have to explicitly reset errno to check
    errno::set_errno(Errno(0));

    let nice = unsafe { libc::getpriority(libc::PRIO_PROCESS, pid) };
    let err = errno::errno();

    if nice == -1 && err != Errno(0) {
        return Err(anyhow!("libc::getpriority: {err}"));
    }

    Ok(nice)
}

/// Sets the niceness of the given process.
pub fn set(pid: u32, niceness: i32) -> Result<()> {
    // because C APIs suck: might return -1 both as indicatior of failure and
    // legitimate return value, so we have to explicitly reset errno to check
    errno::set_errno(Errno(0));

    let nice = unsafe { libc::setpriority(libc::PRIO_PROCESS, pid, niceness) };
    let err = errno::errno();

    if nice == -1 && err != Errno(0) {
        return Err(anyhow!("for pid {pid} libc::setpriority: {err}"));
    }

    Ok(())
}
