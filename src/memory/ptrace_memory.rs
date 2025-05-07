use super::{MemoryError, MemoryReader, MemoryWriter};
use std::ptr;
use libc::{self};

pub struct PtraceMemory {
    pid: u32,
}

impl PtraceMemory {
    pub fn new(pid: u32) -> Self {
        PtraceMemory { pid }
    }
}

impl MemoryReader for PtraceMemory {
    fn read(&self, address: u64, length: usize) -> Result<Vec<u8>, MemoryError> {
        let mut buffer = vec![0u8; length];
        unsafe {
            let pid = self.pid as libc::pid_t;
            let mut i = 0;
            while i < length {
                let data = libc::ptrace(libc::PTRACE_PEEKDATA, pid, (address + i as u64) as *mut libc::c_void, ptr::null_mut::<libc::c_void>());
                if data == -1 {
                    return Err(MemoryError::ProcReadError("ptrace failed".into()));
                }
                buffer[i] = data as u8;
                i += 1;
            }
        }
        Ok(buffer)
    }
}

impl MemoryWriter for PtraceMemory {
    fn write(&self, address: u64, data: &[u8]) -> Result<(), MemoryError> {
        unsafe {
            let pid = self.pid as libc::pid_t;
            for (i, &byte) in data.iter().enumerate() {
                let result = libc::ptrace(libc::PTRACE_POKEDATA, pid, (address + i as u64) as *mut libc::c_void, byte as *mut libc::c_char);
                if result == -1 {
                    return Err(MemoryError::PtraceWriteError("ptrace failed".into()));
                }
            }
        }
        Ok(())
    }
}
