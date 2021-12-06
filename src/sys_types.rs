use std::ffi::c_void;
use std::os::raw::c_char;

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct SourceInfo {
    pub(crate) file_name: *const c_char,
    pub(crate) line: u16
}

#[repr(C)]
pub(crate) struct Error {
    pub(crate) extra_data: *const c_void,
    pub(crate) source_info: crate::sys_types::SourceInfo,
    pub(crate) error_code: u16,
    pub(crate) error_buffer_size: u16,
    pub(crate) reason: c_char
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct CmdPart {
    pub(crate) text: *const c_char,
    pub(crate) is_string: bool
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Command {
    pub(crate) prev: *const crate::sys_types::Command,
    pub(crate) next: *const crate::sys_types::Command,
    
    pub(crate) extra_data: *const c_void,
    pub(crate) resolve_cache: *const c_void,
    pub(crate) source_info: crate::sys_types::SourceInfo,
    pub(crate) command: crate::sys_types::CmdPart
}

#[repr(C)]
pub(crate) struct Program {
    commands: *const Command
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct SemVer {
    major: u16,
    minor: u16,
    patch: u16,
    postfix: [c_char; 15],
    exact: bool
}

pub(crate) type PCallCommandStub = unsafe extern "C" fn(
  program: *mut Program,
  context: *mut c_void,
  command: *const Command,
  error: *mut Error
);

pub(crate) type CommandRouterStub = unsafe extern "C" fn(command: *const Command) -> bool;

pub(crate) type InitStub = unsafe extern "C" fn(error: *mut Error) -> *mut c_void;

pub(crate) type AtExitStub = unsafe extern "C" fn(context: *mut c_void);

pub(crate) type CleanupStub = unsafe extern "C" fn(cmd_extra: *mut c_void);

#[repr(C)]
pub(crate) struct PCallCommand {
  cmd_name: *const c_char,
  router_stub: CommandRouterStub
}

pub(crate) type LoadLanguage = unsafe extern "C" fn(semver: SemVer, error: * mut Error);

