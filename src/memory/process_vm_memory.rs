use std::{io::{IoSlice, IoSliceMut}, mem::{size_of, MaybeUninit}};
use nix::{sys::uio::{process_vm_readv, process_vm_writev, RemoteIoVec, }, unistd::Pid};
use crate::{process::{MapRange, Process}, searcher::{MemorySearcher, SearchError, SearchRule}};

use super::{MemoryError, MemoryReader, MemoryWriter};

pub struct ProcessVmMemory {
    process: Process,
}

impl ProcessVmMemory {
    pub fn new(pid: u32) -> Self {
        ProcessVmMemory {
            process: Process::new(pid),
        }
    }
}

impl MemoryReader for ProcessVmMemory {
    fn read<T: Sized + Copy>(&self, address: usize) -> Result<T, MemoryError> {
        let mut res = MaybeUninit::<T>::uninit();
        let size = size_of::<T>();

        let mut local_iov = [ IoSliceMut::new( unsafe {
            std::slice::from_raw_parts_mut(res.as_mut_ptr() as *mut u8, size)
        }) ];

        let remote_iov = [ RemoteIoVec{
            base: address,
            len: size
        } ];

        let len = process_vm_readv(Pid::from_raw(self.process.pid as i32), &mut local_iov, &remote_iov)
            .map_err(|e|MemoryError::ProcessVmReadError(e.to_string()))?;

        if len == 0 {
            Err(MemoryError::ProcessVmError(format!("result: {len}").to_string()))
        }else if len != size {
            Err(MemoryError::ProcessVmReadError("Short read".to_string()))
        }else{
            Ok(unsafe {
                res.assume_init()
            })
        }
    }

    fn readbuf(&self, address: usize, buf: &mut [u8]) -> Result<usize, MemoryError> {
        let size = buf.len();
        let mut local_iov = [ IoSliceMut::new(buf) ];
        let remote_iov = [ RemoteIoVec{
            base: address,
            len: size
        }];

        let len = process_vm_readv(Pid::from_raw(self.process.pid as i32), &mut local_iov, &remote_iov)
            .map_err(|e|MemoryError::ProcessVmReadError(e.to_string()))?;

        if len == 0 {
            Err(MemoryError::ProcessVmError(format!("result: {len}").to_string()))
        }else if size != len {
            Err(MemoryError::ProcessVmReadError("Short read".to_string()))
        }else{
            Ok(len)
        }
    }
}

impl MemoryWriter for ProcessVmMemory {
    fn write<T: Sized + Copy>(&self, address: usize, value: &T) -> Result<(), MemoryError> {
        let size = size_of::<T>();
        let local_iov = [ IoSlice::new(unsafe {
            std::slice::from_raw_parts(value as *const T as *const u8, size)
        }) ]; 
        let remote_iov = [ RemoteIoVec{
            base: address,
            len: size
        } ];

        let len = process_vm_writev(Pid::from_raw(self.process.pid as i32), &local_iov, &remote_iov)
            .map_err(|e|MemoryError::ProcessVmWriteError(e.to_string()))?;

        if len == 0 {
            Err(MemoryError::ProcessVmError(format!("result: {len}").to_string()))
        }else if len != size {
            Err(MemoryError::ProcessVmWriteError("Short written".to_string()))
        }else{
            Ok(())
        }
    }

    fn writebuf(&self, address: usize, buf: &[u8]) -> Result<usize, MemoryError> {
        let size = buf.len();
        let local_iov = [ IoSlice::new(buf)];
        let remote_iov = [ RemoteIoVec{
            base: address,
            len: size
        }];

        let len = process_vm_writev(Pid::from_raw(self.process.pid as i32), &local_iov, &remote_iov)
            .map_err(|e|MemoryError::ProcessVmWriteError(e.to_string()))?;

        if len == 0 {
            Err(MemoryError::ProcessVmError(format!("result: {len}").to_string()))
        }else if len != size {
            Err(MemoryError::ProcessVmWriteError("Short written".to_string()))
        }else{
            Ok(size)
        }
    }
}

impl MemorySearcher for ProcessVmMemory {
    fn search<T: SearchRule, const N: usize>(&self, rule: T, filter: Option<impl Fn(&MapRange) -> bool>) -> Result<Vec<usize>, SearchError>
    {
        let mut buff = Box::new([0u8;N]);
        let mut res = Vec::<usize>::new();
        match filter {
            Some(f) => {
                for i in 0..self.process.maps.len() {
                    if f(&self.process.maps[i]) {
                        let addr = self.process.maps[i].address;
                        let len = addr.1 - addr.0 + 1;
                        let mut offset = 0;
                        while offset < len {
                            let read_bytes = self.readbuf(addr.0 + offset, buff.as_mut_slice())
                                .map_err(|e|SearchError::ReadError(e.to_string()))?;
                            offset += read_bytes;
                            res.extend(rule.search(buff.as_slice(), read_bytes));
                        }
                    }
                }
            },
            None => {
                for i in 0..self.process.maps.len() {
                    let addr = self.process.maps[i].address;
                    let len = addr.1 - addr.0 + 1;
                    let mut offset = 0;
                    while offset < len {
                        let read_bytes = self.readbuf(addr.0 + offset, buff.as_mut_slice())
                            .map_err(|e|SearchError::ReadError(e.to_string()))?;
                        offset += read_bytes;
                        res.extend(rule.search(buff.as_slice(), read_bytes));
                    }
                }
            }
        }

        Ok(res)
    }
}