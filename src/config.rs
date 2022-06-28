#![allow(dead_code)]

use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub mask: Option<char>,
    pub rule: Vec<Rule>,
}

#[derive(Deserialize, Debug)]
pub struct Rule {
    pub auth: Option<bool>,
    pub persist: Option<bool>,

    pub identity: String,

    pub r#as: Option<String>,

    pub commands: Option<Vec<String>>,

    pub keepenv: Option<bool>,
    //pub setenv = Option<Vec
}
