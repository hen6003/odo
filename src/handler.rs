use std::{
    borrow::Cow::{self, Borrowed, Owned},
    ffi::{CStr, CString},
    io::{stdout, Write},
};

pub struct ConvHandler {
    password_mask: Option<char>,
    username: String,
}

impl ConvHandler {
    pub fn new(username: &str, password_mask: Option<char>) -> Self {
        Self {
            username: username.to_string(),
            password_mask,
        }
    }
}

impl pam::Converse for ConvHandler {
    fn prompt_echo(&mut self, _msg: &CStr) -> Result<CString, ()> {
        CString::new(self.username.clone()).map_err(|_| ())
    }

    fn prompt_blind(&mut self, msg: &CStr) -> Result<CString, ()> {
        print!("{}", msg.to_str().map_err(|_| ())?);
        stdout().flush().map_err(|_| ())?;

        let password = crate::password::read_password(self.password_mask).map_err(|_| ())?;

        println!();

        CString::new(password).map_err(|_| ())
    }

    fn info(&mut self, msg: &CStr) {
        println!("{}", msg.to_str().unwrap());
    }

    fn error(&mut self, msg: &CStr) {
        println!("\x1b[31m{}\x1b[0m", msg.to_str().unwrap());
    }

    fn username(&self) -> &str {
        &self.username
    }
}
