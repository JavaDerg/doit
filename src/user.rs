use std::collections::HashMap;
use crate::config::{Command, ExecRight, UserMode};

pub struct UserModel(HashMap<Command, (ExecRight, UserMode)>);
