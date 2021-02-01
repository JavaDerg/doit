use std::collections::HashMap;
use std::iter::FromIterator;

pub type InnerFBConfig = HashMap<UserPair, HashMap<Command, ExecRight>>;

#[derive(serde::Deserialize)]
pub struct FileBoundConfig {
    pub users: InnerFBConfig,
    pub settings: InnerFBSettings,
}

#[derive(Hash, Eq, PartialEq, serde::Deserialize)]
pub struct InnerFBSettings {
    login_timeout: u32,
}

#[derive(Hash, Eq, PartialEq, serde::Deserialize)]
pub enum Command {
    Sel(Selector),
    Shell,
    All,
}

#[derive(Hash, Eq, PartialEq, serde::Deserialize)]
pub struct UserPair(Selector, UserMode);

#[derive(Hash, Eq, PartialEq, serde::Deserialize)]
pub enum UserMode {
    Default,
    NoPw
}

#[derive(Hash, Eq, PartialEq, serde::Deserialize)]
pub enum ExecRight {
    Root,
    Other(Vec<String>),
}

#[derive(Hash, Eq, PartialEq, serde::Deserialize)]
pub enum Selector {
    /// Matches all
    Match(String),
    /// Matches the first word
    Word(String),
    /// Regex match
    Regex(String),
    /// Format string with dynamic values
    // TODO Format(String),
}

impl Selector {
    fn check(&self, other: &String) -> bool {
        match self {
            Selector::Match(dc) => dc == other,
            Selector::Word(word) => {
                String::from_iter(other.chars().take_while(|c| !c.is_whitespace())) == *word
            }
            Selector::Regex(regex) => fancy_regex::Regex::new(regex)
                .unwrap()
                .is_match(other)
                .expect("Regex failed to run"),
            // TODO Selector::Format(_) => {}
        }
    }
}
