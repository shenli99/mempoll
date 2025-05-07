pub mod proc_memory;
pub mod process_vm_memory;
pub mod ptrace_memory;

pub trait MemoryReader {
    fn read(&self, address: u64, length: usize) -> Result<Vec<u8>, MemoryError>;
}

pub trait MemoryWriter {
    fn write(&self, address: u64, data: &[u8]) -> Result<(), MemoryError>;
}

#[derive(Debug)]
pub enum MemoryError {
    IoError(String),

    PtraceError(String),
    PtraceReadError(String),
    PtraceWriteError(String),

    ProcessVmError(String),
    ProcessVmReadError(String),
    ProcessVmWriteError(String),

    ProcMemError(String),
    ProcReadError(String),
    ProcUninitError(String),
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

