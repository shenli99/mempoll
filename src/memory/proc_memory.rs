use crate::process::Process;
use std::fs::File;
use std::io::{Read, Seek, Write};
use super::{MemoryError, MemoryReader, MemoryWriter};

pub struct ProcMemory {
    process: Process,
    file: Option<File>
}

impl ProcMemory {
    pub fn new(pid: u32) -> Self {
        ProcMemory {
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

impl MemoryReader for ProcMemory {
    fn read(&self, address: u64, length: usize) -> Result<Vec<u8>, MemoryError> {
        match self.file {
            Some(_) => {
                let mut file = self.file.as_ref().unwrap();
                let mut buffer = vec![0u8; length];
                file.seek(std::io::SeekFrom::Start(address)).map_err(|e| MemoryError::ProcReadError(e.to_string()))?;
                file.read_exact(&mut buffer).map_err(|e| MemoryError::ProcReadError(e.to_string()))?;
                Ok(buffer)
            }
            None => Err(MemoryError::ProcUninitError("uninit file".to_string()))
        }
    }
}

impl MemoryWriter for ProcMemory {
    fn write(&self, address: u64, data: &[u8]) -> Result<(), MemoryError> {
        match self.file {
            Some(_) => {
                let mut file = self.file.as_ref().unwrap();
                file.seek(std::io::SeekFrom::Start(address)).map_err(|e| MemoryError::ProcReadError(e.to_string()))?;
                file.write_all(data).map_err(|e| MemoryError::ProcReadError(e.to_string()))?;
                Ok(())
            }
            None => Err(MemoryError::ProcUninitError("uninit file".to_string()))
        }
    }
}
