use std::mem::MaybeUninit;

use nix::{libc, sys};
use nix::unistd::Pid;
use crate::process::Process;

use super::{MemoryError, MemoryReader, MemoryWriter};

pub struct PtraceMemory {
    process: Process,
    is_attach: bool,
}

impl PtraceMemory {
    pub fn new(pid: u32) -> Self {
        PtraceMemory { 
            process: Process::new(pid),
            is_attach: false,
        }
    }

    pub fn attach(&self) -> Result<(), MemoryError> {
        if self.is_attach {
            Ok(())
        } else {    
            let pid = Pid::from_raw(self.process.pid as i32);
            sys::ptrace::attach(pid).map_err(|e|MemoryError::PtraceError(e.to_string()))?;
            sys::wait::waitpid(pid, None).map_err(|e|MemoryError::PtraceAttachError(e.to_string()))?;
            Ok(())
        }
    }

    pub fn dettach(&self) -> Result<(), MemoryError> {
        if self.is_attach {
            let pid = Pid::from_raw(self.process.pid as i32);
            sys::ptrace::detach(pid, None).map_err(|e|MemoryError::PtraceDettachError(e.to_string()))
        }else{
            Ok(())
        }
    }
}

impl MemoryReader for PtraceMemory {
    fn read<T: Sized + Copy>(&self, address: usize) -> Result<T, MemoryError> {
        self.attach()?;

        let pid = Pid::from_raw(self.process.pid as i32);
        let size = std::mem::size_of::<T>();
        let word_size = std::mem::size_of::<libc::c_long>();
        let mut res = MaybeUninit::<T>::uninit();
        let ptr = res.as_mut_ptr() as *mut u8;

        let mut offset = 0;

        while offset < size {
            let curr_addr = address + offset;
            let aligned_addr = curr_addr - (curr_addr % word_size);
            let offset1 = curr_addr % word_size;
            let word = sys::ptrace::read(pid, (aligned_addr) as sys::ptrace::AddressType)
                .map_err(|e|MemoryError::PtraceReadError(e.to_string()))?;
            let bytes = word.to_ne_bytes();

            if aligned_addr < address {
                //in the begin of T
                let bytes_to_copy = std::cmp::min(word_size - offset1, size);
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        bytes.as_ptr().add(offset1),
                        ptr,
                        bytes_to_copy,
                    );
                }
                offset += bytes_to_copy;
            }else{
                let bytes_to_copy = std::cmp::min(word_size, size - offset);
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        bytes.as_ptr(),
                        ptr.add(offset),
                        bytes_to_copy,
                    );
                }
                offset += bytes_to_copy;
            }
        }
        if offset == size {
            Ok(unsafe {
                res.assume_init()
            })
        }else{
            Err(MemoryError::ProcReadError(format!("Short read, result: {offset}").to_string()))
        }
    }

    fn readbuf(&self, address: usize, buf: &mut [u8]) -> Result<usize, MemoryError> {
        self.attach()?;

        let word_size = std::mem::size_of::<libc::c_long>();
        let mut buff_offset: usize = 0;
        let pid = Pid::from_raw(self.process.pid as i32);

        while buff_offset < buf.len() {
            let curr_addr = address + buff_offset;
            let aligned_addr = curr_addr - (curr_addr % word_size);
            let data = sys::ptrace::read(pid, aligned_addr as sys::ptrace::AddressType)
                .map_err(|e| MemoryError::PtraceReadError(e.to_string()))?;
            let data_bytes = data.to_ne_bytes();

            let offset = curr_addr % word_size;

            if aligned_addr < address {
                let bytes_to_copy = std::cmp::min(word_size - offset, buf.len());
                //buf[buff_offset..buff_offset + bytes_to_copy]
                //.copy_from_slice(&data_bytes[offset..offset + bytes_to_copy]);
                unsafe {
                    std::ptr::copy_nonoverlapping(data_bytes.as_ptr().add(offset), buf.as_mut_ptr(), bytes_to_copy);
                }
                buff_offset += bytes_to_copy;
            }else{
                let bytes_to_copy = std::cmp::min(word_size, buf.len() - buff_offset);
                unsafe {
                    std::ptr::copy_nonoverlapping(data_bytes.as_ptr(), buf.as_mut_ptr().add(buff_offset), bytes_to_copy);
                }
                buff_offset += bytes_to_copy;
            }
        }

        if buff_offset == buf.len() {
            Ok(buff_offset)
        }else{
            Err(MemoryError::ProcReadError(format!("Short read, result: {buff_offset}").to_string()))
        }
    }
}

impl MemoryWriter for PtraceMemory {
    fn write<T: Sized + Copy>(&self, address: usize, value: &T) -> Result<(), MemoryError> {
        self.attach()?;

        let pid = Pid::from_raw(self.process.pid as i32);
        let size = std::mem::size_of::<T>();
        let word_size = std::mem::size_of::<libc::c_long>();
        let ptr = value as *const T as *const u8;
        let mut offset: usize =0;
        let mut word_bytes = [0u8; std::mem::size_of::<libc::c_long>()];

        while offset < size {
            let curr_addr = address + offset;
            let offset1 = curr_addr % word_size;

            if offset1 == 0 {
                let left = size - offset;
                if left >= word_size {
                    let raw_data = unsafe {
                        std::slice::from_raw_parts(ptr.add(offset), word_size)
                    };
                    word_bytes.copy_from_slice(raw_data);
                    let data = libc::c_long::from_ne_bytes(word_bytes);
                    sys::ptrace::write(pid, curr_addr as sys::ptrace::AddressType, data).map_err(|e|MemoryError::PtraceWriteError(e.to_string()))?;

                    offset += word_size;
                }else{
                    let word = sys::ptrace::read(pid, curr_addr as sys::ptrace::AddressType).map_err(|e|MemoryError::ProcWriteError(e.to_string()))?;
                    let bytes = word.to_ne_bytes();
                    unsafe {
                        std::ptr::copy_nonoverlapping(bytes.as_ptr(), word_bytes.as_mut_ptr(), word_size);
                        std::ptr::copy_nonoverlapping(ptr, word_bytes.as_mut_ptr(), left);
                    }
                    let data = libc::c_long::from_ne_bytes(bytes);
                    sys::ptrace::write(pid, curr_addr as sys::ptrace::AddressType, data).map_err(|e|MemoryError::PtraceWriteError(e.to_string()))?;
                    offset += left;
                }
            } else {
                let aligned_addr = curr_addr - offset1;
                let word = sys::ptrace::read(pid, aligned_addr as sys::ptrace::AddressType).map_err(|e|MemoryError::ProcWriteError(e.to_string()))?;
                let bytes = word.to_ne_bytes();
                let bytes_to_copy = std::cmp::min(word_size - offset1, size);
                unsafe {
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), word_bytes.as_mut_ptr(), word_size);
                    std::ptr::copy_nonoverlapping(ptr, word_bytes.as_mut_ptr().add(offset1), bytes_to_copy);
                }
                let data = libc::c_long::from_ne_bytes(bytes);
                sys::ptrace::write(pid, aligned_addr as sys::ptrace::AddressType, data).map_err(|e|MemoryError::PtraceWriteError(e.to_string()))?;
                offset += bytes_to_copy;
            }
        }

        if offset == size {
            Ok(())
        } else {
            Err(MemoryError::PtraceWriteError(format!("Short written, result: {offset}").to_string()))
        }
    }

    fn writebuf(&self, address: usize, buf: &[u8]) -> Result<usize, MemoryError> {
        self.attach()?;

        let pid = Pid::from_raw(self.process.pid as i32);
        let size = buf.len();
        let word_size = std::mem::size_of::<libc::c_long>();
        let ptr = buf.as_ptr();
        let mut offset: usize =0;
        let mut word_bytes = [0u8; std::mem::size_of::<libc::c_long>()];

        while offset < size {
            let curr_addr = address + offset;
            let offset1 = curr_addr % word_size;

            if offset1 == 0 {
                let left = size - offset;
                if left >= word_size {
                    let raw_data = unsafe {
                        std::slice::from_raw_parts(ptr.add(offset), word_size)
                    };
                    word_bytes.copy_from_slice(raw_data);
                    let data = libc::c_long::from_ne_bytes(word_bytes);
                    sys::ptrace::write(pid, curr_addr as sys::ptrace::AddressType, data).map_err(|e|MemoryError::PtraceWriteError(e.to_string()))?;

                    offset += word_size;
                }else{
                    let word = sys::ptrace::read(pid, curr_addr as sys::ptrace::AddressType).map_err(|e|MemoryError::ProcWriteError(e.to_string()))?;
                    let bytes = word.to_ne_bytes();
                    unsafe {
                        std::ptr::copy_nonoverlapping(bytes.as_ptr(), word_bytes.as_mut_ptr(), word_size);
                        std::ptr::copy_nonoverlapping(ptr, word_bytes.as_mut_ptr(), left);
                    }
                    let data = libc::c_long::from_ne_bytes(bytes);
                    sys::ptrace::write(pid, curr_addr as sys::ptrace::AddressType, data).map_err(|e|MemoryError::PtraceWriteError(e.to_string()))?;
                    offset += left;
                }
            } else {
                let aligned_addr = curr_addr - offset1;
                let word = sys::ptrace::read(pid, aligned_addr as sys::ptrace::AddressType).map_err(|e|MemoryError::ProcWriteError(e.to_string()))?;
                let bytes = word.to_ne_bytes();
                let bytes_to_copy = std::cmp::min(word_size - offset1, size);
                unsafe {
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), word_bytes.as_mut_ptr(), word_size);
                    std::ptr::copy_nonoverlapping(ptr, word_bytes.as_mut_ptr().add(offset1), bytes_to_copy);
                }
                let data = libc::c_long::from_ne_bytes(bytes);
                sys::ptrace::write(pid, aligned_addr as sys::ptrace::AddressType, data).map_err(|e|MemoryError::PtraceWriteError(e.to_string()))?;
                offset += bytes_to_copy;
            }
        }

        if offset == size {
            Ok(offset)
        } else {
            Err(MemoryError::PtraceWriteError(format!("Short written, result: {offset}").to_string()))
        }
    }
}
