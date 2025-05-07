use std::{fmt::Debug, io::{BufRead, BufReader}};

#[derive(Debug)]
pub enum ProcessError {
    IoError(String),

    MapsOpenError(String),

    MapParseError,
    MapParseConvertError(String),
}

pub mod permissions {
    pub const READABLE: u8 = 0b0001;
    pub const WRITABLTE: u8 = 0b0010;
    pub const EXECUTABLE: u8 = 0b0100;
    pub const SHARED: u8 = 0b100;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemeryType {
    Heap,
    Stack,
    Vdso,
    Vvar,
    Vsyscall,
    Anonymous,
    Ashmem(String),
    MemFd(String),
    File(String),
    Other(String)
}

type Permission = u8;

#[derive(Debug, Clone)]
pub struct MapRange {
    pub address: (usize, usize),
    perms: Permission,
    pub offset: usize,
    #[cfg(feature = "detail")]
    pub dev: (u8, u8),
    #[cfg(feature = "detail")]
    pub inode: u64,
    pub pathname: MemeryType,
}

#[derive(Debug)]
pub struct Process {
    pub pid: u32,
    pub maps: Vec<MapRange>
}

impl MemeryType {
    pub fn from_str(pathname: Option<&str>) -> Self {
        match pathname {
            None => MemeryType::Anonymous,
            Some(p) if p.starts_with("[stack") => MemeryType::Stack,
            Some(p) if p == "[heap]" => MemeryType::Heap,
            Some(p) if p == "[vdso]" => MemeryType::Vdso,
            Some(p) if p == "[vvar]" => MemeryType::Vvar,
            Some(p) if p == "[vsyscall]" => MemeryType::Vsyscall,
            Some(p) if p.starts_with("/dev/ashmem/") => MemeryType::Ashmem(p.to_string()),
            Some(p) if p.starts_with("/memfd:") => MemeryType::MemFd(p.to_string()),
            Some(p) if p.starts_with("/") => MemeryType::File(p.to_string()),
            Some(p) if p.starts_with("[") => MemeryType::Other(p.to_string()),
            Some(_) => MemeryType::Anonymous,
        }
    }
}

impl MapRange {
    fn new(s: &str) -> Result<MapRange, ProcessError> {
        let mut parts = s.splitn(6, ' ');
        let addr = parts.next().unwrap().split_once('-').ok_or(ProcessError::MapParseError)?;
        let addr_s = usize::from_str_radix(addr.0, 16).map_err(|e|ProcessError::MapParseConvertError(e.to_string()))?;
        let addr_e = usize::from_str_radix(addr.1, 16).map_err(|e|ProcessError::MapParseConvertError(e.to_string()))?;

        let perms_raw = parts.next().unwrap_or("----");
        let mut perms = 0u8;
        let mut perms_iter = perms_raw.chars();
        if perms_iter.next() == Some('r') { perms |= permissions::READABLE; }
        if perms_iter.next() == Some('w') { perms |= permissions::WRITABLTE; }
        if perms_iter.next() == Some('x') { perms |= permissions::EXECUTABLE; }
        if perms_iter.next() == Some('s') { perms |= permissions::SHARED; }

        let offset = usize::from_str_radix(parts.next().unwrap_or("0"), 16).unwrap();

        #[cfg(feature = "detail")]
        {
            let dev = parts.next().unwrap_or("0:0").split_once(":").unwrap();
            let dev_0 = u8::from_str_radix(dev.0, 16).unwrap();
            let dev_1 = u8::from_str_radix(dev.1, 10).unwrap();

            let inode = parts.next().unwrap_or("0").parse::<u64>().map_err(|e|ProcessError::MapParseConvertError(e.to_string()))?;
        }

        #[cfg(not(feature = "detail"))]
        {
            parts.next();
            parts.next();
        }

        let path_raw = parts.next().unwrap_or("").trim_start().trim_end();

        #[cfg(feature = "detail")]
        {
        Ok( MapRange{
            address: (addr_s, addr_e),
            perms: perms,
            offset: offset,
            dev: (dev_0, dev_1),
            inode: inode,
            pathname: MemeryType::from_str(if path_raw.len() == 0 {None} else {Some(path_raw)})
        } )
        }
        
        #[cfg(not(feature = "detail"))]
        {
        Ok( MapRange{
            address: (addr_s, addr_e),
            perms: perms,
            offset: offset,
            pathname: MemeryType::from_str(if path_raw.len() == 0 {None} else {Some(path_raw)})
        } )
        }
    }

    #[inline]
    pub fn readable(&self) -> bool {
        self.perms & permissions::READABLE != 0
    }

    #[inline]
    pub fn writable(&self) -> bool {
        self.perms & permissions::WRITABLTE != 0
    }

    #[inline]
    pub fn executable(&self) -> bool {
        self.perms & permissions::EXECUTABLE != 0
    }

    #[inline]
    pub fn shared(&self) -> bool {
        self.perms & permissions::SHARED != 0
    }
}

impl Process {
    pub fn new(pid: u32) -> Self {
        Process { pid, maps: Vec::new() }
    }

    pub fn maps(&mut self) -> Result<(), ProcessError> {
        if self.maps.is_empty() {
            let maps_file = std::fs::File::open(format!("/proc/{}/maps", self.pid)).map_err(|e|ProcessError::MapsOpenError(e.to_string()))?;

            let reader = BufReader::new(maps_file);
            for line in reader.lines() {
                match line {
                    Err(e) => return Err(ProcessError::IoError(e.to_string())),
                    Ok(s) => self.maps.push(MapRange::new(&s)?),
                }
            }
            
            Ok(())
        } else {
            Ok(())
        }
    }
}
