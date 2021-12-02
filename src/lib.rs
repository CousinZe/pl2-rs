pub mod sys;
pub mod sys_types;

use std::ffi::CStr;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ptr::addr_of;

use crate::sys::*;

#[derive(Clone, Copy)]
pub struct SourceInfo<'p> {
    inner: crate::sys::SourceInfo,
    _phantom: PhantomData<&'p ()>
}

impl<'p> SourceInfo<'p> {
    fn new(inner: crate::sys::SourceInfo) -> Self {
        Self {
            inner,
            _phantom: PhantomData::default()
        }
    }
}

impl<'p> SourceInfo<'p> {
    pub fn file_name(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.inner.file_name).to_str().unwrap()
        }
    }

    pub fn line(&self) -> u16 {
        self.inner.line
    }
}

pub struct Error<'p> {
    inner: *mut crate::sys::Error,
    _phantom: PhantomData<&'p ()>
}

impl<'p> Error<'p> {
    fn new(inner: *mut crate::sys::Error) -> Self {
        Self {
            inner,
            _phantom: PhantomData::default()
        }
    }

    unsafe fn into_inner(self) -> *mut crate::sys::Error {
        self.inner
    }
}

impl<'p> Error<'p> {
    pub fn source_info(&self) -> SourceInfo<'p> {
        unsafe {
            SourceInfo::new((*self.inner).source_info)
        }
    }

    pub fn error_code(&self) -> u16 {
        unsafe {
            (*self.inner).error_code
        }
    }

    pub fn reason(&self) -> &'p str {
        unsafe {
            CStr::from_ptr(addr_of!((*self.inner).reason)).to_str().unwrap()
        }
    }
}

impl<'p> Debug for Error<'p> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self)
    }
}

impl<'p> Display for Error<'p> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        let source_info = self.source_info();
        let error_code = self.error_code();
        let reason = self.reason();
        write!(
            fmt,
            "in file {}:{}: error[{}]: {}",
            source_info.file_name(),
            source_info.line(),
            error_code,
            reason
        )
    }
}

impl<'p> Drop for Error<'p> {
    fn drop(&mut self) {
        unsafe {
            pl2b_dropError(self.inner);
        }
    }
}

impl<'p> std::error::Error for Error<'p> {}

pub struct CmdPart<'p> {
    inner: crate::sys_types::CmdPart,
    _phantom: PhantomData<&'p ()>
}





