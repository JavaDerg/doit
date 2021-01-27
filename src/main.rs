mod config;
mod user;
mod linux;

use std::collections::{HashMap, VecDeque};
use std::ffi::CStr;
use crate::config::{ConfigError, Config, RuntimeConfig, ExecutionAction};

#[derive(Debug)]
pub enum DoitError {

}

fn main() {
    /*let uid = linux::get_user_id();
    let pwd = linux::get_user(uid);
    println!("{:#?}", pwd);
    return;*/

    if let Err(err) = run() {
        todo!("{:?}", err)
    }
}

fn run() -> Result<(), DoitError> {
    let mut term = term::stderr().expect("Unable to obtain stderr");

    let config = match config::load_config() {
        Ok(cfg) => cfg,
        Err(err) => {
            term.fg(term::color::RED);
            match err {
                ConfigError::Io(err) => writeln!(term, "[DoIt] Io Error '{}'", err).unwrap(),
                ConfigError::InvalidUserId(id) => writeln!(term, "[DoIt] User #{} does not exist", id).unwrap(),
                ConfigError::InvalidUserName(name) => writeln!(term, "[DoIt] User {} does not exist", name).unwrap(),
                ConfigError::InvalidCliArgs => (),
            }
            std::process::exit(1);
        }
    };

    match config.rt_cfg {
        RuntimeConfig::Normal {
            action,
            target_user,
        } => {}
    }

    Ok(())

    /*
    let target_uid = match matches.value_of_lossy("target_id").unwrap().parse::<u32>() {
        Ok(uid) => uid,
        Err(err) => {
            eprintln!("[DoIt] Provided uid is invalid '{:?}'", err);
            std::process::exit(1);
        }
    };

    if unsafe { libc::getpwuid(target_uid) }.is_null() {
        eprintln!("[DoIt] Unknown target uid {}", target_uid);
        std::process::exit(1);
    }

    /* Check password */
    if !authenticate(target_uid) {
        eprintln!("[DoIt] Failed to authenticate");
        std::process::exit(1);
    }

    /* Check group */
    let gc = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    let mut groups = (0..gc).map(|_| u32::MAX).collect::<Vec<_>>();
    if unsafe { libc::getgroups(groups.len() as i32, groups.as_mut_ptr()) } == -1 {
        eprintln!("[DoIt] Failed to query groups of user");
        std::process::exit(1);
    }
    eprintln!("[DoIt] {:?}", groups);

    /* Elevate process */
    if unsafe { libc::setuid(target_uid) } == -1 {
        eprintln!("[DoIt] Unable to obtain access, you must set this binary to chmod 4755 and transfer ownership to root");
        std::process::exit(1);
    }

    /* Run command */
    let mut process = match std::process::Command::new(args.pop_front().unwrap())
        .args(args)
        .spawn() {
        Ok(proc) => proc,
        Err(err) => {
            eprintln!("[DoIt] Unable to start process '{}'", err);
            std::process::exit(1);
        }
    };
    eprintln!("[DoIt] {:?}", process.wait());
    /*
    std::process::exit(

            .wait()
            .unwrap()
            .code()
            .unwrap_or(0),
    );*/
    */
}

fn authenticate(target_uid: u32) -> bool {
    let uid = unsafe { libc::getuid() };
    if uid == target_uid {
        return true;
    }
    check_password(uid)
}

fn check_password(uid: u32) -> bool {
    let passwd = unsafe { libc::getpwuid(uid) };
    if passwd.is_null() {
        eprintln!("[DoIt] Unknown uid {}", uid);
        return false;
    }

    let name = unsafe { CStr::from_ptr((*passwd).pw_name) };
    let name = name.to_string_lossy().to_string();

    for _ in 0..3 {
        let password = read_password(&name);
        let mut auth = pam::Authenticator::with_password("su").unwrap();
        auth.get_handler().set_credentials(&name, &password);
        if let Err(code) = auth.authenticate() {
            eprintln!("[DoIt] Invalid password {}", code);
            continue;
        }
        return true;
    }

    false
}

fn read_password(name: &str) -> String {
    eprint!("[DoIt] Password for {} (visible): ", name);
    let mut password = String::with_capacity(128);
    std::io::stdin().read_line(&mut password).unwrap();
    let _ = password.pop();
    password
}
