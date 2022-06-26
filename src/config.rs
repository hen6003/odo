#![allow(dead_code)]

use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub rule: Vec<Rule>,
}

#[derive(Deserialize, Debug)]
pub struct Rule {
    pub password: Option<bool>,
    pub persist: Option<bool>,

    pub identity: String,

    pub r#as: Option<String>,

    pub commands: Option<Vec<String>>,
}