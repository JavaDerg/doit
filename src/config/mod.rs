use crate::config::file::FileBoundConfig;
use std::collections::HashMap;
use std::ffi::{CStr, CString};

mod cli;
pub mod file;

pub use file::*;
use std::io::Error;

const DEFAULT_CONFIG: &'static str = include_str!("../default.ron");

pub struct Config {
    rt_cfg: RuntimeConfig,
    fb_cfg: Option<FileBoundConfig>,
}

pub enum RuntimeConfig {
    Normal {
        action: ExecutionAction,
        target_user: *mut libc::passwd,
    },
}

pub enum ExecutionAction {
    Command(String),
    /// flag for clean environment
    Shell(bool),
}

pub enum ConfigError {
    Io(std::io::Error),
    InvalidCliArgs,
    InvalidUserId(u32),
    InvalidUserName(String),
}

pub fn load_config() -> Result<Config, ConfigError> {
    use structopt::StructOpt;

    let config: cli::CliConfig = cli::CliConfig::from_args();
    if !verify_cli_config(&config) {
        return Err(ConfigError::InvalidCliArgs);
    }
    let rt_config = gen_rt_config(config)?;

    let file_cfg = if cfg!(debug_assertions) {
        Some(String::from(DEFAULT_CONFIG))
    } else {
        match std::fs::read_to_string("/etc/doit.ron") {
            Ok(str) => Some(str),
            Err(err) => {
                todo!("{:?}", err);
                None
            }
        }
    };
    let fb_cfg = match file_cfg {
        Some(str) => match ron::from_str(&str) {
            Ok(fbc) => Some(fbc),
            Err(err) => {
                todo!("{:?}", err);
                None
            }
        },
        None => None,
    };

    Ok(Config {
        rt_cfg: rt_config,
        fb_cfg
    })
}

pub fn gen_rt_config(config: cli::CliConfig) -> Result<RuntimeConfig, ConfigError> {
    Ok(RuntimeConfig::Normal {
        target_user: if let Some(uid) = config.target_id {
            let pwd = unsafe { libc::getpwuid(uid) };
            if pwd.is_null() {
                todo!("print error: invalid user id");
                return Err(ConfigError::InvalidUserId(uid));
            }
            pwd
        } else {
            let name = config.target_name.as_ref().unwrap();
            let cname =
                CString::new(name.as_bytes()).expect("Unable to convert string to CString?");
            let pwd = unsafe { libc::getpwnam(cname.as_ptr()) };
            if pwd.is_null() {
                todo!("print error: invalid user name");
                return Err(ConfigError::InvalidUserName(name.clone()));
            }
            pwd
        },
        action: if !config.command.is_empty() {
            ExecutionAction::Command(config.command.join(" "))
        } else {
            ExecutionAction::Shell(config.shell > 1)
        },
    })
}

pub fn verify_cli_config(config: &cli::CliConfig) -> bool {
    let mut present_modes = 0u32;
    let mut present_selector = 0u32;

    // Check for mods
    if !config.command.is_empty() {
        present_modes += 1;
    }
    if config.shell > 0 {
        present_modes += 1;
    }

    // Check for selectors
    if config.target_id.is_some() {
        present_selector += 1;
    }
    if config.target_name.is_some() {
        present_selector += 1;
    }

    // Sanity check
    [
        match present_modes {
            0 => {
                todo!("Print error");
                false
            }
            1 => true,
            _ => {
                todo!("Print advanced error");
                false
            }
        },
        match present_selector {
            0 | 1 => true,
            _ => {
                todo!("Print advanced error");
                false
            }
        },
    ]
    .iter()
    .all(|y| *y)
}
