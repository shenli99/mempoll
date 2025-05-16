#![feature(portable_simd)]

pub mod process;
pub mod memory;
pub mod searcher;

/*
/// 读取进程的内存
pub fn read_memory(pid: u32, address: u64, length: usize, method: MemoryMethod) -> Result<Vec<u8>, String> {
    let reader = match method {
        MemoryMethod::Proc => Box::new(ProcMemory::new(pid)),
        MemoryMethod::Ptrace => Box::new(ptrace_memory::new(pid)),
        MemoryMethod::ProcessVmRead => Box::new(process_vm_memory::new(pid)),
    };
    reader.read(address, length)
}

/// 写入进程的内存
pub fn write_memory(pid: u32, address: u64, data: &[u8], method: MemoryMethod) -> Result<(), String> {
    let writer = match method {
        MemoryMethod::Proc => Box::new(ProcMemory::new(pid)),
        MemoryMethod::Ptrace => Box::new(ptrace_memory::new(pid)),
        MemoryMethod::ProcessVmRead => Box::new(process_vm_memory::new(pid)),
    };
    writer.write(address, data)
}
*/
/// 支持的内存读写方式
#[derive(Debug)]
pub enum MemoryMethod {
    Proc,
    Ptrace,
    ProcessVmRead,
}
