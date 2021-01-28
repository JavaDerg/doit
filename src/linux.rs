use libc::{getuid, setuid, getpwuid, passwd, __errno_location, EAGAIN, EPERM};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub type UserId = u32;

#[derive(Debug)]
pub enum LinuxError {
    MissingPermission,
    NotFound,
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
        let pwd = *pwd;
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

unsafe fn deref_const_str_or_empty(s: *const c_char) -> Result<String, LinuxError> {
    if s.is_null() {
        return String::from("");
    }
    let cstr = CStr::from_ptr(s);
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}

unsafe fn get_err() -> LinuxError {
    match unsafe { *(__errno_location()) } {
        EPERM => LinuxError::MissingPermission,
        code => LinuxError::Other(code),
    }
}