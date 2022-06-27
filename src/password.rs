use console::Key;
use std::{
    fs::File,
    io::{stdout, Error, ErrorKind, Result, Write},
};

fn read_data(tty: console::Term, mask: Option<char>) -> Result<String> {
    let mut password = String::new();

    loop {
        let key = tty.read_key()?;

        match key {
            Key::Char(char) => {
                if let Some(mask) = mask {
                    print!("{}", mask);
                    stdout().flush()?;
                }

                password.push(char);
            }

            Key::Enter => {
                break Ok(password);
            }

            Key::Backspace => {
                if password.pop() != None && mask.is_some() {
                    tty.move_cursor_left(0)?;
                    tty.clear_chars(1)?;
                }
            }

            _ => (),
        }
    }
}

pub fn read_password(mask: Option<char>) -> Result<String> {
    let tty = console::Term::stdout();

    if tty.is_term() {
        read_data(tty, mask)
    } else {
        Err(Error::new(
            ErrorKind::Other,
            "Password cannot be read from non-tty",
        ))
    }
}
