use std::ffi::c_void;
use std::os::raw::c_char;

#[repr(C)]
pub struct SourceInfo {
    file_name: *const c_char,
    line: u16
}

#[repr(C)]
pub struct Error {
    extra_data: *const c_void,
    source_info: SourceInfo,
    error_code: u16,
    error_buffer_size: u16,
    reason: c_char
}

extern "C" {
    fn pl2b_getLocaleName() -> *const c_char;
    fn pl2b_errorBuffer(buffer_size: u16) -> *mut Error;
    fn pl2b_errPrintf(
        error: *mut Error,
        error_code: u16,
        source_info: SourceInfo,
        extra_data: *const c_void,
        fmt: *const c_char,
        ...
    );
    fn pl2b_dropError(error: *mut Error);
    fn pl2b_isError(error: *mut Error) -> bool;
}
