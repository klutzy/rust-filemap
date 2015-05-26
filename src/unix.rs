use std::os::unix::prelude::*;
use std::slice;
use std::fs::File;
use libc;

use super::{FileMapError, FileMapResult};

pub type OsError = libc::c_int;

#[inline(always)]
fn errno() -> OsError {
    extern {
        fn __errno_location() -> *mut libc::c_int;
    }

    unsafe {
        *__errno_location()
    }
}

#[inline(always)]
fn err<T>() -> FileMapResult<T> {
    Err(FileMapError::OsError(errno()))
}

#[inline(always)]
pub fn page_size() -> usize {
    unsafe {
        libc::sysconf(libc::_SC_PAGESIZE) as usize
    }
}

pub struct FileMapInner {
    pa: *mut libc::c_void,
    len: usize,
}

impl FileMapInner {
    pub fn new_immut(file: &File, offset: usize, length: usize) -> FileMapResult<FileMapInner> {
        let addr = 0 as *mut libc::c_void;
        let len = length as libc::size_t;
        let prot = libc::PROT_READ;
        let flags = libc::MAP_PRIVATE | libc::MAP_FILE;
        let fd = file.as_raw_fd();

        let offset = offset as libc::off_t;

        let pa = unsafe {
            libc::mmap(addr, len, prot, flags, fd, offset)
        };

        if pa == libc::MAP_FAILED {
            return err();
        }

        let inner = FileMapInner {
            pa: pa,
            len: length,
        };
        Ok(inner)
    }

    pub fn new_mut(file: &File,
                   offset: usize,
                   length: usize,
                   shared: bool) -> FileMapResult<FileMapInner> {
        let addr = 0 as *mut libc::c_void;
        let len = length as libc::size_t;
        let prot = libc::PROT_READ | libc::PROT_WRITE;
        let flags = libc::MAP_FILE;
        let flags = if shared {
            flags | libc::MAP_SHARED
        } else {
            flags | libc::MAP_PRIVATE
        };
        let fd = file.as_raw_fd();

        let offset = offset as libc::off_t;

        let pa = unsafe {
            libc::mmap(addr, len, prot, flags, fd, offset)
        };

        if pa == libc::MAP_FAILED {
            return err();
        }

        let inner = FileMapInner {
            pa: pa,
            len: length,
        };
        Ok(inner)
    }

    #[inline(always)]
    pub fn as_slice<'s>(&'s self) -> &'s [u8] {
        unsafe { slice::from_raw_parts(self.pa as *const u8, self.len) }
    }

    #[inline(always)]
    pub fn as_slice_mut<'s>(&'s mut self) -> &'s mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.pa as *mut u8, self.len) }
    }
}

impl Drop for FileMapInner {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.pa, self.len as libc::size_t);
        }
    }
}
