use libc::{__errno_location, getpwuid, getuid, passwd, setuid, EAGAIN, ENOENT, EPERM};
use std::alloc::Layout;
use std::ffi::{CStr, CString, OsStr};
use std::os::raw::c_char;
use std::path::Path;

pub type UserId = u32;

#[derive(Debug)]
pub enum LinuxError {
    MissingPermission,
    NotFound,
    InvalidString,
    Other(i32),
}

#[derive(Debug)]
pub struct User {
    name: String,
    passwd: String,
    user_id: UserId,
    group_id: u32,
    comment: String,
    home: String,
    shell: String,
}

pub fn get_user_id() -> UserId {
    unsafe { getuid() }
}

pub fn set_user(user: UserId) -> Result<(), LinuxError> {
    let ret = unsafe { setuid(user) };
    if ret != 0 {
        return Err(unsafe { get_err() });
    }
    Ok(())
}

pub fn get_user(user: UserId) -> Result<User, LinuxError> {
    let pwd = unsafe { getpwuid(user) };
    if pwd.is_null() {
        return Err(unsafe { get_err() });
    }
    Ok(unsafe {
        let pwd = &*pwd;
        User {
            name: deref_const_str_or_empty(pwd.pw_name)?,
            passwd: deref_const_str_or_empty(pwd.pw_passwd)?,
            user_id: pwd.pw_uid,
            group_id: pwd.pw_gid,
            comment: deref_const_str_or_empty(pwd.pw_gecos)?,
            home: deref_const_str_or_empty(pwd.pw_dir)?,
            shell: deref_const_str_or_empty(pwd.pw_shell)?,
        }
    })
}

pub fn is_readonly(path: &Path) -> Result<bool, LinuxError> {
    let flags = unsafe { read_stat(path.as_os_str()) }?;
    Ok(flags & 0o22 == 0)
}

unsafe fn deref_const_str_or_empty(s: *const c_char) -> Result<String, LinuxError> {
    if s.is_null() {
        return Ok(String::default());
    }
    let cstr = CStr::from_ptr(s);
    String::from_utf8(Vec::from(cstr.to_bytes())).map_err(|_| LinuxError::InvalidString)
}

unsafe fn read_stat(path: &OsStr) -> Result<u32, LinuxError> {
    let layout = Layout::new::<libc::stat>();
    let mut stat: *mut libc::stat = std::alloc::alloc_zeroed(layout) as *mut libc::stat;
    if stat.is_null() {
        panic!("Unable to allocate memory");
    }
    match libc::stat(path.as_bytes().as_ptr() as *const c_char, stat) {
        0 => (), // All is fine,
        -1 => return Err(get_err()),
        _ => unreachable!(),
    }
    let perms = stat.st_mode;
    std::alloc::dealloc(stat as *mut u8, layout);
    Ok(perms)
}

unsafe fn get_err() -> LinuxError {
    match *(__errno_location()) {
        EPERM => LinuxError::MissingPermission,
        ENOENT => LinuxError::NotFound,
        code => LinuxError::Other(code),
    }
}
