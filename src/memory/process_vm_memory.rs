use super::{MemoryError, MemoryReader, MemoryWriter};

pub struct ProcessVmMemory {
    pid: u32,
}

impl ProcessVmMemory {
    pub fn new(pid: u32) -> Self {
        ProcessVmMemory { pid }
    }
}

impl MemoryReader for ProcessVmMemory {
    fn read(&self, address: u64, length: usize) -> Result<Vec<u8>, MemoryError> {
        // 使用 `process_vm_readv` 系统调用来读取内存
        // 这里的实现需要通过 libc 调用进行封装，具体可参考相关文档或代码示例
        unimplemented!()
    }
}

impl MemoryWriter for ProcessVmMemory {
    fn write(&self, address: u64, data: &[u8]) -> Result<(), MemoryError> {
        // 使用 `process_vm_writev` 系统调用来写入内存
        // 这里的实现需要通过 libc 调用进行封装，具体可参考相关文档或代码示例
        unimplemented!()
    }
}
