use ropey::{Rope, RopeSlice};
use std::cmp::min;

pub struct Window {
    buffer: Rope,
    size: (u16, u16),
    scroll_offset: (usize, usize),
    cursor: (usize, usize),
}

impl Window {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            buffer: Rope::new(),
            size: (width, height),
            scroll_offset: (0, 0),
            cursor: (0, 0),
        }
    }

    pub fn visible_lines(&self) -> impl Iterator<Item = RopeSlice> {
        self.buffer.lines_at(self.scroll_offset.1).take(self.size.1 as usize)
    }

    pub fn visible_lines_from(&self, line: u16) -> impl Iterator<Item = RopeSlice> {
        self.buffer.lines_at(self.scroll_offset.1 + line as usize).take((self.size.1 - line) as usize)
    }

    pub fn rel_cursor_pos(&self) -> (u16, u16) {
        ((self.cursor.0 - self.scroll_offset.0) as u16, (self.cursor.1 - self.scroll_offset.1) as u16)
    }

    pub fn set_buffer(&mut self, buffer: Rope) {
        self.buffer = buffer;
        self.scroll_offset = (0, 0);
        self.cursor = (0, 0);
    }

    pub fn move_cursor_down(&mut self, lines: usize) {
        let limit = self.buffer.len_lines() - 1;
        let target = min(self.cursor.1 + lines, limit);
        let diff = target - self.cursor.1;

        let max = self.scroll_offset.1 + self.size.1 as usize;
        if target >= max {
            self.scroll_offset.1 += target - max + 1;
        }

        self.cursor.1 += diff;
        self.adjust_cursor_x();
    }

    pub fn move_cursor_up(&mut self, lines: usize) {
        let target = if lines > self.cursor.1 { 0 } else {
            self.cursor.1 - lines
        };

        if target < self.scroll_offset.1 {
            self.scroll_offset.1 = target;
        }

        self.cursor.1 = target;
        self.adjust_cursor_x();
    }

    fn adjust_cursor_x(&mut self) {
        fn is_newline(chr: char) -> bool {
            match chr {
                '\x0a' | '\x0b' | '\x0c' | '\x0d' | '\u{85}' | '\u{2028}' | '\u{2029}' => true,
                _ => false,
            }
        }

        let line = self.buffer.line(self.cursor.1);
        let mut line_len = line.len_chars();
        if line_len > 0 && is_newline(line.char(line_len - 1)) {
            line_len -= 1;
        }
        if line_len > 0 {
            line_len -= 1;
        }
        self.cursor.0 = min(line_len, self.cursor.0);
    }

    pub fn move_cursor_left(&mut self, cols: usize) {
        self.cursor.0 = if cols > self.cursor.0 { 0 } else { self.cursor.0 - cols };
    }

    pub fn move_cursor_right(&mut self, cols: usize) {
        self.cursor.0 += cols;
        self.adjust_cursor_x();
    }
}

