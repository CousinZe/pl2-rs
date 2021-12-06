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

pub fn get_locale_name() -> &'static str {
    unsafe { cstr2str(pl2b_getLocaleName()) }
}

#[derive(Clone, Copy)]
pub struct SourceInfo<'p> {
    inner: crate::sys::SourceInfo,
    _phantom: PhantomData<&'p ()>
}

impl<'p> SourceInfo<'p> {
    pub unsafe fn new_unchecked(inner: crate::sys::SourceInfo) -> Self {
        Self {
            inner,
            _phantom: PhantomData::default()
        }
    }

    pub fn unknown() -> Self {
        Self {
            inner: crate::sys::SourceInfo {
                file_name: "".as_ptr() as _,
                line: 0
            },
            _phantom: PhantomData::default()
        }
    }

    pub unsafe fn into_inner(self) -> crate::sys::SourceInfo {
        self.inner
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
    pub unsafe fn new_unchecked(inner: *mut crate::sys::Error) -> Self {
        Self {
            inner,
            _phantom: PhantomData::default()
        }
    }

    pub unsafe fn into_inner(self) -> *mut crate::sys::Error {
        self.inner
    }
}

impl<'p> Error<'p> {
    pub fn source_info(&self) -> SourceInfo<'p> {
        unsafe {
            SourceInfo::new_unchecked((*self.inner).source_info)
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
        self.inner.text == null()
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
    pub unsafe fn new_unchecked(inner: *const crate::sys_types::Command) -> Self {
        Self {
            inner,
            cached_size: Cell::new(None),
            _phantom: PhantomData::default()
        }
    }

    pub unsafe fn into_inner(self) -> *const crate::sys_types::Command {
        self.inner
    }

    pub fn size(&self) -> usize {
        if let Some(size) = self.cached_size.get() {
            return size;
        }
        
        let size = unsafe { pl2b_argsLen(self.inner) };
        self.cached_size.set(Some(size as usize));
        size as usize
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

pub struct Program {
    #[allow(unused)]
    buf: String,
    inner: crate::sys_types::Program
}

impl Program {
    pub unsafe fn new_unchecked(inner: crate::sys_types::Program) -> Self {
        Self {
            buf: "".into(),
            inner
        }
    }

    pub fn parse(input: impl ToString) -> Result<Self, Error<'static>> {
        let mut buf = input.to_string();
        buf.push('\x00');
        
        unsafe {
            let ptr = buf.as_mut_str().as_mut_ptr();
            let err = pl2b_errorBuffer(512);
            let inner = pl2b_parse(ptr as *mut c_char, 64, err);

            if pl2b_isError(err) {
                Err(Error::new_unchecked(err))
            } else {
                pl2b_dropError(err);
                Ok(Self { buf, inner })
            }
        }
    }

    pub fn run(&self) -> Result<(), Error> {
        unsafe {
            let err = pl2b_errorBuffer(512);
            pl2b_run(&self.inner as *const _, err);
            if pl2b_isError(err) {
                Err(Error::new_unchecked(err))
            } else {
                pl2b_dropError(err);
                Ok(())
            }
        }
    }

    pub fn debug_print(&self) {
        unsafe {
            pl2b_debugPrintProgram(&self.inner);
        }
    }

    pub fn first_command<'a>(&'a self) -> Option<Command<'a>> {
        unsafe {
            if self.inner.commands == null() {
                None
            } else {
                Some(Command::new_unchecked(self.inner.commands))
            }
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            pl2b_dropProgram(&mut self.inner);
        }
    }
}

#[inline(always)]
pub fn ensure_pcall_command_stub_interface(
    _f: for<'a> fn(&'a Program, *mut (), Command<'a>) 
            -> Result<Option<Command<'a>>, Box<dyn std::error::Error>>
) {}

#[inline(always)]
pub fn ensure_pcall_command_router_stub_interface(
    _f: for<'a> fn(Command<'a>) -> bool
) {}

#[inline(always)]
pub fn ensure_init_stub_interface(
    _f: fn() -> Result<*mut (), Box<dyn std::error::Error>> 
) {}

#[macro_export] macro_rules! make_pcall_command_stub {
    ($fn_name:ident, $output_name:ident) => {
        pub(crate) unsafe extern "C" fn $output_name(
            program: *mut $crate::sys_types::Program,
            ctx: *mut std::ffi::c_void,
            command: *const $crate::sys_types::Command,
            error: *mut $crate::sys_types::Error
        ) {
            $crate::ensure_pcall_command_stub_interface($fn_name);
            let program = $crate::Program::new_unchecked(program);
            let r = $fn_name(
                program,
                ctx as _,
                $crate::Command::new_unchecked(command)
            );
            std::mem::forget(program);
            match r {
                Ok(Some(command)) => command.into_inner(),
                Ok(None) => std::ptr::null(),
                Err(e) => {
                    let mut reason = e.to_string();
                    reason.push('\x00');
                    let source_info = command.source_info;

                    $crate::sys::pl2b_errPrintf(
                        error,
                        -1,
                        source_info,
                        std::ptr::null(),
                        reason.as_str().as_ptr() as _
                    );
                    std::ptr::null()
                }
            }
        }
    }
}

#[macro_export] macro_rules! make_pcall_router_stub {
    ($fn_name:ident, $output_name:ident) => {
        pub(crate) unsafe extern "C" fn $output_name(
            command: *const $crate::sys_types::Command
        ) -> bool {
            $crate::ensure_pcall_command_router_stub_interface($fn_name);
            $fn_name($crate::Command::new_unchecked(command))
        }
    }
}

#[macro_export] macro_rules! make_init_stub {
    ($fn_name:ident, $output_name:ident) => {
        pub(crate) unsafe extern "C" fn $output_name(error: *mut Error) -> *mut c_void {
            match $fn_name() {
                Ok(data) => Box::into_raw(Box::new(data)) as _,
                Err(e) => {
                    let mut reason = e.to_string();
                    reason.push('\x00');

                    $crate::sys::pl2b_errPrintf(
                        error,
                        -1,
                        $crate::SourceInfo::unknown().into_inner(),
                        std::ptr::null(),
                        reason.as_str().as_ptr() as _
                    );
                    std::ptr::null()
                }
            }
        }
    }
}

#[macro_export] macro_rules! make_atexit_stub {
    ($fn_name:ident, $output_name:ident) => {
        pub(crate) unsafe extern "C" fn $output_name(context: *mut c_void) {
            $fn_name(Box::into_inner(Box::from_raw(context)))
        }
    }
}

pub fn make_pcall_cmd(
    cmd_name: &'static str,
    stub: Option<crate::sys::PCallCommandStub>,
    deprecated: bool,
    removed: bool
) -> crate::sys::PCallCommand {
    crate::sys::PCallCommand {
        cmd_name: cmd_name.as_ptr() as _,
        router_stub: None,
        stub,
        deprecated,
        removed
    }
}

pub fn make_pcall_cmd_custom_router(
    router_stub: Option<crate::sys::CommandRouterStub>,
    stub: Option<crate::sys::PCallCommandStub>,
    deprecated: bool,
    removed: bool
) -> crate::sys::PCallCommand {
    crate::sys::PCallCommand {
        cmd_name: null(),
        router_stub,
        stub,
        deprecated,
        removed
    }
}

pub fn make_empty_pcall_cmd() -> crate::sys::PCallCommand {
    crate::sys::PCallCommand {
        cmd_name: std::ptr::null(),
        router_stub: None,
        stub: None,
        deprecated: false,
        removed: false
    }
}

pub fn make_language(
    lang_name: Option<&'static str>,
    lang_info: Option<&'static str>,
    
    init: Option<crate::sys::InitStub>,
    atexit: Option<crate::sys::AtExitStub>,
    pcall_cmds: &'static [crate::sys::PCallCommand],
    fallback: Option<crate::sys::PCallCommandStub>
) -> crate::sys::Language {
    crate::sys::Language {
        lang_name: if let Some(name) = lang_name { name.as_ptr() as _ } else { null() },
        lang_info: if let Some(info) = lang_info { info.as_ptr() as _ } else { null() },
        init,
        atexit,
        cmd_cleanup: None,
        pcall_cmds: pcall_cmds.as_ptr(),
        fallback
    }
}

