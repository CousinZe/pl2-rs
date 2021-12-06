pub mod sys;
pub mod sys_types;

use std::cell::Cell;
use std::ffi::CStr;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ptr::{addr_of, null};

use crate::sys::*;

unsafe fn cstr2str<'a>(src: *const c_char) -> &'a str {
    CStr::from_ptr(src).to_str().unwrap()
}

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
            cstr2str(self.inner.file_name)
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
            cstr2str(addr_of!((*self.inner).reason))
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

impl<'p> CmdPart<'p> {
    pub fn is_empty(&self) -> bool {
        unsafe { self.inner.text == null() }
    }

    pub fn is_string(&self) -> bool {
        self.inner.is_string
    }

    pub fn text(&self) -> &str {
        unsafe {
            if self.inner.text != null() {
                cstr2str(self.inner.text)
            } else {
                ""
            }
        }
    }
}

impl<'p> Display for CmdPart<'p> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            Ok(())
        } else if self.is_string() {
            write!(fmt, "{:?}", self.text())
        } else {
            write!(fmt, "{}", self.text())
        }
    }
}

pub struct Command<'p> {
    inner: *const crate::sys_types::Command,
    cached_size: Cell<Option<usize>>,
    _phantom: PhantomData<&'p ()>
}

impl<'p> Command<'p> {
    pub fn size(&self) -> usize {
        if let Some(size) = self.cached_size.get() {
            return size;
        }
        
        let mut counter = 0;
        unsafe {
            let mut iter: *const crate::sys_types::CmdPart = addr_of!((*self.inner).command);
            while (*iter).text != null() {
                counter += 1;
                iter = iter.add(1);
            }
        }
        self.cached_size.set(Some(counter));
        counter
    }

    pub fn part(&self, idx: usize) -> CmdPart<'p> {
        assert!(idx < self.size());
        unsafe {
            CmdPart {
                inner: *addr_of!((*self.inner).command).add(idx),
                _phantom: PhantomData::default()
            }
        }
    }

    pub fn prev_command(&self) -> Option<Self> {
        unsafe {
            if (*self.inner).prev == null() {
                None
            } else {
                Some(Self {
                    inner: (*self.inner).prev,
                    cached_size: Cell::new(None),
                    _phantom: PhantomData::default()
                })
            }
        }
    }

    pub fn next_command(&self) -> Option<Self> {
        unsafe {
            if (*self.inner).next == null() {
                None
            } else {
                Some(Self {
                    inner: (*self.inner).next,
                    cached_size: Cell::new(None),
                    _phantom: PhantomData::default()
                })
            }
        }
    }
}

impl<'p> Display for Command<'p> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.size() {
            if i == 0 {
                write!(fmt, "{}", self.part(0))?;
            } else {
                write!(fmt, " {}", self.part(i))?;
            }
        }
        Ok(())
    }
}



