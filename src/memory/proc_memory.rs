use crate::process::{MapRange, Process};
use crate::searcher::{MemorySearcher, SearchError, SearchRule};
use std::io::IoSlice;
use std::os::fd::AsFd;
use std::{fs::File, io::IoSliceMut};
use std::mem::MaybeUninit;
use super::{MemoryError, MemoryReader, MemoryWriter};

pub struct ProcMemory {
    pub process: Process,
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
    fn read<T: Sized + Copy>(&self, address: usize) -> Result<T, MemoryError> {
        match self.file {
            Some(_) => {
                let fd = self.file.as_ref().unwrap();
                let mut res = MaybeUninit::<T>::uninit();
                let size = std::mem::size_of::<T>();

                let len = nix::sys::uio::pread(fd, unsafe {
                    std::slice::from_raw_parts_mut(res.as_mut_ptr() as *mut u8, size)
                }, address as i64).map_err(|e|MemoryError::PreadError(e.to_string()))?;

                if len != size {
                    Err(MemoryError::ProcReadError(format!("Short pread, result: {len}").to_string()))
                } else {
                    Ok(unsafe {
                        res.assume_init()
                    })
                }
            },
            None => Err(MemoryError::ProcUninitError("Uninit file".to_string()))
        }
    }

    fn readbuf(&self, address: usize, buf: &mut [u8]) -> Result<usize, MemoryError> {
        match self.file {
            Some(_) => {
                let fd = self.file.as_ref().unwrap();
                let mut bufs = [ IoSliceMut::new(buf) ];
                let len = nix::sys::uio::preadv(fd.as_fd(), bufs.as_mut_slice(), address as i64).unwrap();//.map_err(|e|MemoryError::PreadError(e.to_string()))?;

                if len <= 0 {
                    Err(MemoryError::ProcReadError(format!("Short read, result: {len}").to_string()))
                }else{
                    Ok(len)
                }
            },
            None => Err(MemoryError::ProcUninitError("Uninit file".to_string()))
        }
    }
}

impl MemoryWriter for ProcMemory {
    fn write<T: Sized>(&self, address: usize, value: &T) -> Result<(), MemoryError> {
        match self.file {
            Some(_) => {
                let fd = self.file.as_ref().unwrap();
                let size = std::mem::size_of::<T>();

                let len = nix::sys::uio::pwrite(fd, unsafe {
                    std::slice::from_raw_parts(value as *const T as *const u8, size)
                }, address as i64).map_err(|e|MemoryError::PwriteError(e.to_string()))?;

                if len != size {
                    Err(MemoryError::ProcWriteError("Short pwrite".to_string()))
                } else {
                    Ok(())
                }
            },
            None => Err(MemoryError::ProcUninitError("Uninit file".to_string()))
        }
    }

    fn writebuf(&self, address: usize, buf: &[u8]) -> Result<usize, MemoryError> {
        match self.file {
            Some(_) => {
                let fd = self.file.as_ref().unwrap();
                let bufs = [ IoSlice::new(buf) ];
                nix::sys::uio::pwritev(fd, &bufs, address as i64).map_err(|e|MemoryError::PreadError(e.to_string()))
            },
            None => Err(MemoryError::ProcUninitError("Uninit file".to_string()))
        }
    }
}

impl MemorySearcher for ProcMemory {
    fn search<T: SearchRule, const N: usize>(&self, rule: T, filter: Option<impl Fn(&MapRange) -> bool>) -> Result<Vec<usize>, SearchError>
    {
        let mut buff = Box::new([0u8;N]);
        let mut res = Vec::<usize>::new();
        match filter {
            Some(f) => {
                for i in 0..self.process.maps.len() {
                    if f(&self.process.maps[i]) {
                        let addr = self.process.maps[i].address;
                        let mut offset = 0;
                        while offset + addr.0 < addr.1 {
                            let read_bytes = self.readbuf(addr.0 + offset, buff.as_mut_slice())
                                .map_err(|e|SearchError::ReadError(e.to_string()))?;
                            res.extend(rule.search(buff.as_slice(), read_bytes).map(|v|v+addr.0+offset));
                            offset += read_bytes;
                        }
                    }
                }
            },
            None => {
                for i in 0..self.process.maps.len() {
                    let addr = self.process.maps[i].address;
                    let mut offset = 0;
                    while offset + addr.0 < addr.1 {
                        let read_bytes = self.readbuf(addr.0 + offset, buff.as_mut_slice())
                            .map_err(|e|SearchError::ReadError(e.to_string()))?;
                        res.extend(rule.search(buff.as_slice(), read_bytes).map(|v|v+addr.0+offset));
                        offset += read_bytes;
                    }
                }
            }
        }

        Ok(res)
    }
}