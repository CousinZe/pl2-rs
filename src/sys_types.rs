use std::ffi::c_void;
use std::os::raw::c_char;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SourceInfo {
    pub(crate) file_name: *const c_char,
    pub(crate) line: u16
}

#[repr(C)]
pub struct Error {
    pub(crate) extra_data: *const c_void,
    pub(crate) source_info: SourceInfo,
    pub(crate) error_code: u16,
    pub(crate) error_buffer_size: u16,
    pub(crate) reason: c_char
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CmdPart {
    pub(crate) text: *const c_char,
    pub(crate) is_string: bool
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Command {
    pub(crate) prev: *const Command,
    pub(crate) next: *const Command,
    
    pub(crate) extra_data: *const c_void,
    pub(crate) resolve_cache: *const c_void,
    pub(crate) source_info: SourceInfo,
    pub(crate) command: CmdPart
}

#[repr(C)]
pub struct Program {
    pub(crate) commands: *const Command
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SemVer {
    major: u16,
    minor: u16,
    patch: u16,
    postfix: [c_char; 15],
    exact: bool
}

pub type PCallCommandStub = unsafe extern "C" fn(
    program: *mut Program,
    context: *mut c_void,
    command: *const Command,
    error: *mut Error
) -> *const Command;

pub type CommandRouterStub = unsafe extern "C" fn(command: *const Command) -> bool;

pub type InitStub = unsafe extern "C" fn(error: *mut Error) -> *mut c_void;

pub type AtExitStub = unsafe extern "C" fn(context: *mut c_void);

pub type CmdCleanupStub = unsafe extern "C" fn(cmd_extra: *mut c_void);

#[repr(C)]
pub struct PCallCommand {
    pub(crate) cmd_name: *const c_char,
    pub router_stub: CommandRouterStub,
    pub stub: PCallCommandStub,
    pub deprecated: bool,
    pub removed: bool
}

#[repr(C)]
pub struct Language {
    pub(crate) lang_name: *const c_char,
    pub(crate) lang_info: *const c_char,

    pub(crate) init: InitStub,
    pub(crate) atexit: AtExitStub,
    pub(crate) cmd_cleanup: CmdCleanupStub,
    pub(crate) pcall_cmds: *const PCallCommand,
    pub(crate) fallback: *const PCallCommandStub
}

pub type LoadLanguage = unsafe extern "C" fn(semver: SemVer, error: * mut Error);

