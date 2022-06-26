use std::{
    borrow::Cow::{self, Borrowed, Owned},
    ffi::{CStr, CString},
};

use rustyline::{config::Configurer, highlight::Highlighter, ColorMode, Editor};
use rustyline_derive::{Completer, Helper, Hinter, Validator};

#[derive(Completer, Helper, Hinter, Validator)]
struct MaskingHighlighter {
    masking: bool,
}

impl Highlighter for MaskingHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        use unicode_width::UnicodeWidthStr;
        if self.masking {
            Owned("*".repeat(line.width()))
        } else {
            Borrowed(line)
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        self.masking
    }
}

pub struct ConvHandler {
    username: String,
    rl: Editor<MaskingHighlighter>,
}

impl ConvHandler {
    pub fn new(username: &str) -> Self {
        let mut rl = Editor::new();
        let h = MaskingHighlighter { masking: true };

        rl.set_helper(Some(h));
        rl.set_color_mode(ColorMode::Forced); // force masking
        rl.set_auto_add_history(false); // make sure password is not added to history

        Self {
            username: username.to_string(),
            rl,
        }
    }
}

impl pam::Converse for ConvHandler {
    fn prompt_echo(&mut self, _msg: &CStr) -> Result<CString, ()> {
        CString::new(self.username.clone()).map_err(|_| ())
    }

    fn prompt_blind(&mut self, msg: &CStr) -> Result<CString, ()> {
        let readline = self.rl.readline(msg.to_str().unwrap()).unwrap();

        CString::new(readline).map_err(|_| ())
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
