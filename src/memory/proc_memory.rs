use crate::process::Process;
use crate::searcher::{MemorySearcher, SearchError, SearchRule};
use std::io::IoSlice;
use std::{fs::File, io::IoSliceMut};
use std::mem::MaybeUninit;
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

                let len = nix::sys::uio::preadv(fd, &mut bufs, address as i64).map_err(|e|MemoryError::PreadError(e.to_string()))?;

                if len != buf.len() {
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

// impl MemorySearcher for ProcMemory {
//     fn search<T: Eq + Sized , const N: usize>(&self, rule: SearchRule<T>) -> Result<Vec<usize>, SearchError> {
//         let mut buff = Box::new([0u8;N]);
//         let res = Vec::<usize>::new();
//         todo!()
//     }
// }