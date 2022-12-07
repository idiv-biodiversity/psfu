//! Getting and setting processor affinity aka cpuset.

use anyhow::{anyhow, Result};
use errno::errno;

// For the joy of using a C interface.
static CPU_SET_SIZE: usize = std::mem::size_of::<libc::cpu_set_t>();

/// Returns the processor affinity aka cpuset.
pub fn get(pid: libc::pid_t) -> Result<Vec<usize>> {
    let mask = unsafe {
        let mut mask = std::mem::zeroed();
        libc::CPU_ZERO(&mut mask);

        if 0 != libc::sched_getaffinity(pid, CPU_SET_SIZE, &mut mask) {
            return Err(anyhow!("sched_getaffinity: {}", errno()));
        }

        mask
    };

    let mut affinity = Vec::new();

    for i in 0..libc::CPU_SETSIZE as usize {
        if unsafe { libc::CPU_ISSET(i, &mask) } {
            affinity.push(i);
        }
    }

    Ok(affinity)
}

/// Sets processor affinity aka cpuset.
pub fn set(pid: libc::pid_t, cpuset: &[usize]) -> Result<()> {
    unsafe {
        let mut mask = std::mem::zeroed();
        libc::CPU_ZERO(&mut mask);

        for i in cpuset {
            libc::CPU_SET(*i, &mut mask);
        }

        if 0 != libc::sched_setaffinity(pid, CPU_SET_SIZE, &mask) {
            return Err(anyhow!("sched_setaffinity: {}", errno()));
        }
    };

    Ok(())
}
