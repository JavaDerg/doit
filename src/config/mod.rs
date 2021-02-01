use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::io::Error;
use std::path::Path;

pub use file::*;

use crate::config::file::FileBoundConfig;

mod cli;
pub mod file;

const DEFAULT_CONFIG: &str = include_str!("../default.ron");
const CONFIG_PATH: &Path = Path::new("/etc/doit.ron");

pub struct Config {
    pub rt_cfg: RuntimeConfig,
    pub fb_cfg: Result<FileBoundConfig, ConfigError>,
}

pub enum RuntimeConfig {
    Normal {
        action: ExecutionAction,
        target_user: *mut libc::passwd,
    },
}

pub enum ExecutionAction {
    Command(String),
    /// false: command `su {USERNAME}` will be executed
    /// true: command `su {USERNAME} -` will be executed
    Shell(bool),
}

pub enum ConfigError {
    Io(std::io::Error),
    ParseErr(ron::Error),
    InvalidConfigPerm,
    InvalidCliArgs,
    InvalidUserId(u32),
    InvalidUserName(String),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Io(err)
    }
}

pub fn load_config() -> Result<Config, ConfigError> {
    use structopt::StructOpt;

    let config: cli::CliConfig = cli::CliConfig::from_args();
    if !verify_cli_config(&config) {
        return Err(ConfigError::InvalidCliArgs);
    }
    let rt_config = gen_rt_config(config)?;

    let file_cfg: Result<String, ConfigError> = if cfg!(debug_assertions) {
        Ok(String::from(DEFAULT_CONFIG))
    } else {
        if !crate::linux::is_readonly(CONFIG_PATH) {
            return Err(ConfigError::InvalidConfigPerm);
        }
        std::fs::read_to_string(CONFIG_PATH).into()
    };
    let fb_cfg = match file_cfg {
        Ok(cfg) => ron::from_str::<FileBoundConfig>(&str).map_err(ConfigError::ParseErr),
        x => x,
    };

    Ok(Config {
        rt_cfg: rt_config,
        fb_cfg,
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
