
use std::{
    fs::File,
    io::{
        prelude::*, BufReader, IoSlice, IoSliceMut
    }
};

use crate::cverror::MemResult;

use nix::{unistd::Pid};
use  nix::sys::uio::{
    RemoteIoVec,
    process_vm_readv,
    process_vm_writev
};

pub type Addr = usize;

/// Struct used to describe each line item in linux /proc/<pid>/maps file
#[derive(Debug)]
pub struct MemMap {
    pub start: Addr,
    pub end: Addr,
    pub perms: String,
    pub offset: u64,
    pub dev: String,
    pub inode: u64,
    pub path: String
}


impl MemMap {

    pub fn parse_maps(fpath: String) -> MemResult<Vec<MemMap>> {
        let file = File::open(fpath)
        .expect("Failed to read process mmap file");

        let reader = BufReader::new(file);

        let mut maps = Vec::<MemMap>::new();

        for line in reader.lines() {
            let lref = line?;
    
            let mm = MemMap::parse_line(lref)?;
    
            maps.push(mm);
        }

        Ok(maps)
    }

    pub fn parse_line(line: String) -> MemResult<Self> {
        let minfo = line.split_whitespace().collect::<Vec<&str>>();
        println!("{:?}", minfo);
        let (arng, perm, off, dv, inod) = 
                                        (minfo[0],
                                         String::from(minfo[1]),
                                         u64::from_str_radix(minfo[2], 16)?,
                                         String::from(minfo[3]),
                                         u64::from_str_radix(minfo[4], 16)?);

        // Paths are optional
        let mname = if minfo.len() == 6 {
            String::from(minfo[5])
        } else {
            String::from("")
        };

        let m = arng.split('-').collect::<Vec<&str>>();
        let (lwr, upr) = (usize::from_str_radix(m[0], 16)?,
                                        usize::from_str_radix(m[1], 16)?);

        Ok(
            Self {
                start: lwr,
                end: upr,
                perms: perm,
                offset: off,
                dev: dv,
                inode: inod,
                path: mname
        })
    }
}

/// Find a byte sequence within a memory buffer
/// this can likely be heavily optimized, but works for now
pub fn memmem(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|win| win == needle)
}

/// Read in the memory maps for a process
pub fn get_proc_maps(pid: u32) -> MemResult<Vec<MemMap>> {

    let maps = MemMap::parse_maps(format!("/proc/{}/maps", pid))?;
    Ok(maps)
}

/// Read remote memory from a process
pub fn proc_read_vm(rpid: i32, addr: Addr, buf: &mut[u8], size: usize) -> MemResult<usize>{

    let mut lmeml = [IoSliceMut::new(buf), ];

    let rmem = RemoteIoVec {
        base: addr,
        len: size
    };

    let rmeml = [rmem, ];
    let p = Pid::from_raw(rpid);
    let rb = process_vm_readv(p, &mut lmeml, &rmeml).unwrap();

    Ok(rb)
}

/// Modify memory in a remote process
pub fn proc_write_vm(rpid: i32, addr: Addr, buf: &[u8], size: usize) -> MemResult<usize>{

    let lmem = [IoSlice::new(buf), ];

    let rmem = RemoteIoVec {
        base: addr,
        len: size
    };

    let rmem = [rmem, ];
    let p = Pid::from_raw(rpid);
    let rb = process_vm_writev(p, &lmem, &rmem).unwrap();

    Ok(rb)
}