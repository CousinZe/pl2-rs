use std::ffi::c_void;
use std::os::raw::c_char;

pub use crate::sys_types::*;

extern "C" {
    pub(crate) fn pl2b_getLocaleName() -> *const c_char;
    pub(crate) fn pl2b_errorBuffer(buffer_size: u16) -> *mut Error;
    pub(crate) fn pl2b_errPrintf(
        error: *mut Error,
        error_code: u16,
        source_info: SourceInfo,
        extra_data: *const c_void,
        fmt: *const c_char,
        ...
    );
    pub(crate) fn pl2b_dropError(error: *mut Error);
    pub(crate) fn pl2b_isError(error: *mut Error) -> bool;
    pub(crate) fn pl2b_argsLen(cmd: *const Command) -> u16;
    pub(crate) fn pl2b_initProgram(program: *mut Program);
    pub(crate) fn pl2b_parse(
        source: *const c_char,
        parse_bufsiz: u16,
        error: *mut Error
    ) -> Program;
    pub(crate) fn pl2b_dropProgram(program: *mut Program);
    pub(crate) fn pl2b_debugPrintProgram(program: *const Program);
    pub(crate) fn pl2b_parseSemVer(src: *const c_char, err: *mut Error);
}

