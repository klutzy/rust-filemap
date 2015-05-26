use std::os::windows::prelude::*;
use std::mem;
use std::slice;
use std::fs::File;
use std::ptr;
use libc;

use super::{FileMapError, FileMapResult};

pub type OsError = libc::DWORD;

#[inline(always)]
fn errno() -> OsError {
    extern "system" {
        fn GetLastError() -> libc::DWORD;
    }

    unsafe {
        GetLastError()
    }
}

#[inline(always)]
fn err<T>() -> FileMapResult<T> {
    Err(FileMapError::OsError(errno()))
}

#[inline(always)]
pub fn page_size() -> usize {
    unsafe {
        let mut info = mem::zeroed();
        libc::GetSystemInfo(&mut info);
        info.dwPageSize as usize
    }
}

pub struct FileMapInner {
    map_obj: libc::HANDLE,
    pa: *mut libc::c_void,
    len: usize,
}

impl FileMapInner {
    pub fn new_immut(file: &File, offset: usize, length: usize) -> FileMapResult<FileMapInner> {
        let handle = file.as_raw_handle() as libc::HANDLE;
        let attr = ptr::null_mut();
        let protect = libc::PAGE_READONLY;
        let len_high = ((length >> 32) & 0xffff_ffff) as libc::DWORD;
        let len_low = (length & 0xffff_ffff) as libc::DWORD;

        let map_obj = unsafe {
            libc::CreateFileMappingW(handle, attr, protect, len_high, len_low, ptr::null())
        };
        if map_obj == ptr::null_mut() {
            return err();
        }

        let access = libc::FILE_MAP_READ;
        let off_high = ((offset >> 32) & 0xffff_ffff) as libc::DWORD;
        let off_low = (offset & 0xffff_ffff) as libc::DWORD;
        let len = length as libc::SIZE_T;
        let pa = unsafe {
            libc::MapViewOfFile(map_obj, access, off_high, off_low, len)
        };

        let inner = FileMapInner {
            map_obj: map_obj,
            pa: pa,
            len: length,
        };
        Ok(inner)
    }

    pub fn new_mut(file: &File,
                   offset: usize,
                   length: usize,
                   shared: bool) -> FileMapResult<FileMapInner> {
        let handle = file.as_raw_handle() as libc::HANDLE;
        let attr = ptr::null_mut();
        let protect = if shared {
            libc::PAGE_READWRITE
        } else {
            libc::PAGE_READONLY
        };
        let len_high = ((length >> 32) & 0xffff_ffff) as libc::DWORD;
        let len_low = (length & 0xffff_ffff) as libc::DWORD;

        let map_obj = unsafe {
            libc::CreateFileMappingW(handle, attr, protect, len_high, len_low, ptr::null())
        };
        if map_obj == ptr::null_mut() {
            return err();
        }

        let access = if shared {
            libc::FILE_MAP_WRITE
        } else {
            libc::FILE_MAP_COPY
        };
        let off_high = ((offset >> 32) & 0xffff_ffff) as libc::DWORD;
        let off_low = (offset & 0xffff_ffff) as libc::DWORD;
        let len = length as libc::SIZE_T;
        let pa = unsafe {
            libc::MapViewOfFile(map_obj, access, off_high, off_low, len)
        };

        let inner = FileMapInner {
            map_obj: map_obj,
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
            libc::UnmapViewOfFile(self.pa);
            libc::CloseHandle(self.map_obj);
        }
    }
}
