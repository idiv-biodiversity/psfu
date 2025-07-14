use anyhow::Result;
use procfs::process::Process;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProcessID(pub i32);

impl ProcessID {
    pub fn into_process(self) -> Result<Process> {
        let process = Process::new(self.0)?;
        Ok(process)
    }
}
