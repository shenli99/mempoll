use std::{fs::File, io::IoSliceMut, os::fd::AsFd};

use nix::sys::uio::preadv;

fn main() {
    let path = format!("/proc/{}/mem", 3703u32);
    println!("{path}");
    let fd = File::open(path).unwrap();
    let mut buff = Box::new([0u8;4096]);
    let mut iov = [ IoSliceMut::new(buff.as_mut_slice()) ];
    let len = preadv(fd.as_fd(), iov.as_mut_slice(), 0x5602b0994eb0usize as i64).unwrap();
    println!("{len}");
    println!("{:?}", buff);
}