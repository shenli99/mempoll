use std::fs::File;

use crate::process::Process;

use super::{MemoryError, MemoryReader, MemoryWriter};

pub struct ProcessVmMemory {
    process: Process,
    file: Option<File>
}

impl ProcessVmMemory {
    pub fn new(pid: u32) -> Self {
        ProcessVmMemory {
            process: Process::new(pid),
            file: None
        }
    }

    pub fn open(&mut self) -> Result<(), MemoryError> {
        if self.file.is_none() {
            let path = format!("/proc/{}/mem", self.process.pid);
            self.file = Some(File::open(path).map_err(|e| MemoryError::ProcMemError(e.to_string()))?)
        }

        Ok(())
    }
}

impl MemoryReader for ProcessVmMemory {
    fn read<T: Sized + Copy>(&self, address: usize) -> Result<T, MemoryError> {
        unimplemented!()
    }

    fn readbuf(&self, address: usize, buf: &mut [u8]) -> Result<usize, MemoryError> {
        unimplemented!()
    }
}

impl MemoryWriter for ProcessVmMemory {
    fn write<T: Sized + Copy>(&self, address: usize, value: &T) -> Result<(), MemoryError> {
        unimplemented!()
    }

    fn writebuf(&self, address: usize, buf: &[u8]) -> Result<usize, MemoryError> {
        unimplemented!()
    }
}
