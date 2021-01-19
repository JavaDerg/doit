guse std::collections::VecDeque;
use std::ffi::CStr;

fn main() {
    let matches = clap::App::new("doit")
        .args(&[
            clap::Arg::with_name("install")
                .long("install")
                .conflicts_with("args"),
            clap::Arg::with_name("target_id")
                .short("i")
                .takes_value(true)
                .default_value("0"),
            clap::Arg::with_name("args")
                .takes_value(true)
                .required_unless("install")
                .multiple(true)
                .last(true)
        ])
        .get_matches();

    let mut args = matches.values_of_lossy("args").unwrap().into_iter().collect::<VecDeque<_>>();
    if args.len() == 0 {
        eprintln!("[DoIt] No command provided");
        std::process::exit(1);
    }

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
