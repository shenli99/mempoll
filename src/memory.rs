pub mod proc_memory;
pub mod process_vm_memory;
pub mod ptrace_memory;

pub trait MemoryReader {
    fn read<T: Sized + Copy>(&self, address: usize) -> Result<T, MemoryError>;
    fn readbuf(&self, address: usize, buf: &mut [u8]) -> Result<usize, MemoryError>;
}

pub trait MemoryWriter {
    fn write<T: Sized + Copy>(&self, address: usize, value: &T) -> Result<(), MemoryError>;
    fn writebuf(&self, address: usize, buf: &[u8]) -> Result<usize, MemoryError>;
}

#[derive(Debug)]
pub enum MemoryError {
    IoError(String),

    PreadError(String),
    PwriteError(String),

    PtraceError(String),
    PtraceAttachError(String),
    PtraceDettachError(String),
    PtraceReadError(String),
    PtraceWriteError(String),

    ProcessVmError(String),
    ProcessVmReadError(String),
    ProcessVmWriteError(String),

    ProcMemError(String),
    ProcReadError(String),
    ProcWriteError(String),
    ProcUninitError(String),
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

