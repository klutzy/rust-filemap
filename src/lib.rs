extern crate libc;

use std::ops;
use std::marker;
use std::fs::File;

#[cfg(unix)] #[path="unix.rs"] mod imp;
#[cfg(windows)] #[path="windows.rs"] mod imp;

#[derive(Clone, Copy, Debug)]
pub enum FileMapError {
    InvalidLength,
    OsError(imp::OsError),
}

pub type FileMapResult<T> = Result<T, FileMapError>;

fn round_offset(offset: usize, length: usize) -> FileMapResult<(usize, usize, usize)> {
    if length == 0 {
        return Err(FileMapError::InvalidLength);
    }

    let page_size = imp::page_size();

    let padding = offset % page_size;
    let new_offset = offset - padding;
    let length = length + padding;

    Ok((new_offset, padding, length))
}

/// read-only file mapping
pub struct FileMap<'a> {
    _marker: marker::PhantomData<&'a u8>,
    inner: imp::FileMapInner,
    length: usize,
    padding: usize,
}

impl<'a> FileMap<'a> {
    pub fn new(file: &'a File, offset: usize, length: usize) -> FileMapResult<FileMap<'a>> {
        let (new_offset, padding, new_length) = try!(round_offset(offset, length));
        let inner = try!(imp::FileMapInner::new_immut(file, new_offset, new_length));

        let result = FileMap {
            _marker: marker::PhantomData,
            inner: inner,
            length: length,
            padding: padding,
        };
        Ok(result)
    }
}

/// read-write file mapping
pub struct FileMapMut<'a> {
    _marker: marker::PhantomData<&'a u8>,
    inner: imp::FileMapInner,
    length: usize,
    padding: usize,
}

impl<'a> FileMapMut<'a> {
    /// NOTE: this function does not require mutable `file`.
    pub fn new(file: &'a File,
               offset: usize,
               length: usize,
               shared: bool) -> FileMapResult<FileMapMut<'a>> {
        let (new_offset, padding, new_length) = try!(round_offset(offset, length));
        let inner = try!(imp::FileMapInner::new_mut(file, new_offset, new_length, shared));

        let result = FileMapMut {
            _marker: marker::PhantomData,
            inner: inner,
            length: length,
            padding: padding,
        };
        Ok(result)
    }
}

macro_rules! filemap_impl {
    ($t:ident) => (
        impl<'a> $t<'a> {
            fn as_slice<'b>(&'b self) -> &'b [u8] {
                &self.inner.as_slice()[self.padding..]
            }

            pub fn len(&self) -> usize {
                self.length
            }
        }

        impl<'a> ops::Index<ops::RangeFull> for $t<'a> {
            type Output = [u8];

            #[inline(always)]
            fn index<'b>(&'b self, _range: ops::RangeFull) -> &'b [u8] {
                &self.as_slice()
            }
        }

        impl<'a> ops::Index<ops::Range<usize>> for $t<'a> {
            type Output = [u8];

            #[inline(always)]
            fn index<'b>(&'b self, range: ops::Range<usize>) -> &'b [u8] {
                &self.as_slice()[range.start..range.end]
            }
        }

        impl<'a> ops::Index<ops::RangeFrom<usize>> for $t<'a> {
            type Output = [u8];

            #[inline(always)]
            fn index<'b>(&'b self, range: ops::RangeFrom<usize>) -> &'b [u8] {
                &self.as_slice()[range.start..]
            }
        }

        impl<'a> ops::Index<ops::RangeTo<usize>> for $t<'a> {
            type Output = [u8];

            #[inline(always)]
            fn index<'b>(&'b self, range: ops::RangeTo<usize>) -> &'b [u8] {
                &self.as_slice()[..range.end]
            }
        }
    )
}

filemap_impl!(FileMap);
filemap_impl!(FileMapMut);

impl<'a> FileMapMut<'a> {
    fn as_slice_mut<'b>(&'b mut self) -> &'b mut [u8] {
        &mut self.inner.as_slice_mut()[self.padding..]
    }
}

impl<'a> ops::IndexMut<ops::RangeFull> for FileMapMut<'a> {
    #[inline(always)]
    fn index_mut<'b>(&'b mut self, _range: ops::RangeFull) -> &'b mut [u8] {
        self.as_slice_mut()
    }
}

impl<'a> ops::IndexMut<ops::Range<usize>> for FileMapMut<'a> {
    #[inline(always)]
    fn index_mut<'b>(&'b mut self, range: ops::Range<usize>) -> &'b mut [u8] {
        &mut self.as_slice_mut()[range.start..range.end]
    }
}

impl<'a> ops::IndexMut<ops::RangeFrom<usize>> for FileMapMut<'a> {
    #[inline(always)]
    fn index_mut<'b>(&'b mut self, range: ops::RangeFrom<usize>) -> &'b mut [u8] {
        &mut self.as_slice_mut()[range.start..]
    }
}

impl<'a> ops::IndexMut<ops::RangeTo<usize>> for FileMapMut<'a> {
    #[inline(always)]
    fn index_mut<'b>(&'b mut self, range: ops::RangeTo<usize>) -> &'b mut [u8] {
        &mut self.as_slice_mut()[..range.end]
    }
}

#[cfg(test)] mod test;
