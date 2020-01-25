use crossterm::event::{KeyEvent, KeyCode};
use ropey::Rope;
use std::fmt::Write;

pub struct ExBar {
    buffer: Rope,
    cursor_index: usize,
}

pub enum ExResult {
    StillEditing,
    Aborted,
    Finished(String),
}

impl ExBar {
    pub fn new() -> Self {
        Self {
            buffer: Rope::new(),
            cursor_index: 0,
        }
    }

    pub fn buffer(&self) -> &Rope {
        &self.buffer
    }

    pub fn cursor_index(&self) -> usize {
        self.cursor_index
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> ExResult {
        if !key.modifiers.is_empty() {
            return ExResult::StillEditing;
        }

        match key.code {
            KeyCode::Char(chr) => {
                self.buffer.insert_char(self.cursor_index, chr);
                self.cursor_index += 1;
                ExResult::StillEditing
            },
            KeyCode::Enter => {
                let mut result = String::new();
                write!(result, "{}", self.buffer).unwrap();
                self.clear();
                ExResult::Finished(result)
            },
            KeyCode::Backspace => {
                match (self.buffer.len_chars() == 0, self.cursor_index == 0) {
                    (true, _) => return ExResult::Aborted,
                    (false, true) => {},
                    (false, false) => {
                        self.buffer.remove(self.cursor_index - 1..self.cursor_index);
                        self.cursor_index -= 1;
                    },
                }

                ExResult::StillEditing
            },
            KeyCode::Delete => {
                let len = self.buffer.len_chars();
                if self.cursor_index < len {
                    self.buffer.remove(self.cursor_index..self.cursor_index + 1);
                }

                ExResult::StillEditing
            },
            KeyCode::Left => {
                if self.cursor_index > 0 {
                    self.cursor_index -= 1;
                }

                ExResult::StillEditing
            },
            KeyCode::Right => {
                if self.cursor_index < self.buffer.len_chars() {
                    self.cursor_index += 1;
                }

                ExResult::StillEditing
            },
            KeyCode::Esc => ExResult::Aborted,
            _ => ExResult::StillEditing,
        }
    }

    pub fn clear(&mut self) {
        self.buffer = Rope::new();
        self.cursor_index = 0;
    }
}

