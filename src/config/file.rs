use serde::Deserializer;
use std::collections::HashMap;
use std::iter::FromIterator;

pub type InnerFBConfig = HashMap<String, HashMap<Command, ExecRight>>;

pub struct FileBoundConfig(pub InnerFBConfig);

pub type Command = Selector;

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
}

impl<'de> serde::Deserialize<'de> for FileBoundConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        let inner = InnerFBConfig::deserialize(deserializer)?;
        Ok(Self(inner))
    }
}

impl PartialEq<String> for Selector {
    fn eq(&self, other: &String) -> bool {
        match self {
            Selector::Match(dc) => dc == other,
            Selector::Word(word) => {
                String::from_iter(other.chars().take_while(|c| !c.is_whitespace())) == *word
            }
            Selector::Regex(regex) => fancy_regex::Regex::new(regex)
                .unwrap()
                .is_match(other)
                .expect("Regex failed to run"),
        }
    }
}
