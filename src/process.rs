use core::fmt;
use std::{fmt::Debug, io::{BufRead, BufReader}};

#[derive(Debug)]
pub enum ProcessError {
    IoError(String),

    MapsOpenError(String),
    MapsReadError(String),

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
pub enum MemoryType {
    ///Bad
    Bad,
    ///Video
    V,
    ///C++ alloc
    Ca,
    ///C++ .bss
    Cb,
    ///C++ .data
    Cd,
    ///C++ heap
    Ch,
    //Java heap
    Jh,
    //Java
    J,
    ///Anonymous
    A,
    ///Code system
    Xs,
    ///Stack
    S,
    ///Ashmem
    As,
    ///Other
    Other,
    ///Code app
    Xa,
    ///PPSSPP
    Ps,
}

type Permission = u8;

#[derive(Clone)]
pub struct MapRange {
    pub address: (usize, usize),
    perms: Permission,
    pub offset: usize,
    pub dev: (u8, u8),
    pub inode: u32,
    pub pathname: String,
    pub memory_type: MemoryType
}

impl fmt::Debug for MapRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MapRange ( addr: {:X}-{:X}, type:{:?}, pathname:{})", self.address.0, self.address.1, self.memory_type, self.pathname)
    }
}

#[derive(Debug)]
pub struct Process {
    pub pid: u32,
    pub maps: Vec<MapRange>
}

impl MemoryType {
    pub fn new(pathname: Option<&str>, perms: Permission, offset: i64, last_is_cd: bool) -> Self {
        if perms & permissions::EXECUTABLE != 0 {
            if let Some(name) = pathname {
                if name.contains("/data/app")
                    || name.contains("/data/user") {
                    return MemoryType::Xa;
                } else {
                    return MemoryType::Xs;
                }
            } else {
                return MemoryType::Xa;
            }
        }

        if let Some(name) = pathname {
            if name.starts_with("/dev") {
                if name.starts_with("/dev/mali")
                    || name.contains("/dev/kgsl")
                    || name.contains("/dev/nv")
                    || name.contains("/dev/tegra")
                    || name.contains("/dev/ion")
                    || name.contains("/dev/pvr")
                    || name.contains("/dev/render")
                    || name.contains("/dev/galcore")
                    || name.contains("/dev/fimg2d")
                    || name.contains("/dev/quadd")
                    || name.contains("/dev/graphics")
                    || name.contains("/dev/mm_")
                    || name.contains("/dev/dri/")
                {
                    return MemoryType::V;
                } else if name.contains("/dev/xLog") {
                    return MemoryType::Bad;
                }
            } else if name.starts_with("/system/fonts/")
                || name.starts_with("anon_inode:dmabuf") {
                return MemoryType::Bad;
            } else if name.contains("[anon:.bss]") {
                return if last_is_cd { MemoryType::Cd } else { MemoryType::Other };
            } else if name.starts_with("/system/") {
                return MemoryType::Other;
            } else if name.contains("/dev/zero/") {
                return MemoryType::Ca;
            } else if name.contains("PPSSPP_RAM") {
                return MemoryType::Ps;
            } else if !name.contains("system@")
                && !name.contains("gralloc")
                && !name.starts_with("[vdso]")
                && !name.starts_with("[vectors]")
                && (!name.starts_with("/dev/") || name.starts_with("/dev/ashmem")) {
                if name.contains("dalvik") {
                    if (name.contains("exp")
                        || name.contains("dalvik-alloc")
                        || name.contains("dalvik-main")
                        || name.contains("dalvik-large")
                        || name.contains("dalvik-free"))
                        && !name.contains("itmap")
                        && !name.contains("ygote")
                        && !name.contains("ard")
                        && !name.contains("jit")
                        && !name.contains("inear") {
                        return MemoryType::Jh;
                    } else {
                        return MemoryType::J;
                    }
                } else if name.contains("/lib") && name.contains(".so") {
                    if name.contains("/data/") || name.contains("/mnt/") {
                        return MemoryType::Cd;
                    }
                } else if name.contains("malloc") {
                    return MemoryType::Ca;
                } else if name.contains("[heap]") {
                    return MemoryType::Ch;
                } else if name.contains("[stack") {
                    return MemoryType::S;
                } else if name.starts_with("[anon") {
                    if name.contains("scudo") 
                        || name.contains("libc_malloc")
                        || name.contains("bionic_alloc_small_object") {
                        return MemoryType::Ca;
                    } else if name.contains("stack") {
                        return MemoryType::S;
                    } else if name.contains("ashmem") {
                        return MemoryType::As;
                    } else if name.contains("gfx")
                        || name.contains("gralloc")
                        || name.contains("dmabuf")
                        || name.contains("GD") {
                        return MemoryType::V;
                    }
                } else if name.starts_with("/dev/ashmen") && !name.contains("MemoryHeapBase") {
                    return MemoryType::As;
                }
            }
            return MemoryType::Other;
        } else if (perms & permissions::READABLE != 0) && (perms & permissions::WRITABLTE != 0) && (perms & permissions::EXECUTABLE == 0) && (offset == 0)
        {
            return MemoryType::A;
        }
        return MemoryType::Other;
    }
}

impl MapRange {
    fn new(s: &str, last_is_cd: bool) -> Result<MapRange, ProcessError> {
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

        let dev = parts.next().unwrap_or("0:0").split_once(":").unwrap();
        let dev_0 = u8::from_str_radix(dev.0, 16).unwrap();
        let dev_1 = u8::from_str_radix(dev.1, 10).unwrap();

        let inode = parts.next().unwrap_or("0").parse::<u32>().map_err(|e|ProcessError::MapParseConvertError(e.to_string()))?;

        let path_raw = parts.next().unwrap_or("").trim_start().trim_end();

        let memory_type = MemoryType::new(Some(path_raw), perms, offset as i64, last_is_cd);
        Ok( MapRange{
            address: (addr_s, addr_e),
            perms: perms,
            offset: offset,
            dev: (dev_0, dev_1),
            inode: inode,
            pathname: path_raw.to_string(),
            memory_type: memory_type
        } )
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

            let mut last_is_cd = false;
            for line in reader.lines() {
                match line {
                    Err(e) => return Err(ProcessError::IoError(e.to_string())),
                    Ok(s) => {
                        self.maps.push(MapRange::new(&s, last_is_cd)?);
                        last_is_cd = self.maps.last().unwrap().memory_type == MemoryType::Cd;
                    }
                }
            }
            
            Ok(())
        } else {
            Ok(())
        }
    }
}
