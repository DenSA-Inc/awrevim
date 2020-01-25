use ropey::{Rope, RopeSlice};
use std::cmp::min;

pub struct Window {
    buffer: Rope,
    size: (u16, u16),
    scroll_offset: (usize, usize),
    cursor: (usize, usize),
    cursor_x_saved: usize,
}

impl Window {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            buffer: Rope::new(),
            size: (width, height),
            scroll_offset: (0, 0),
            cursor: (0, 0),
            cursor_x_saved: 0,
        }
    }

    pub fn buffer(&self) -> &Rope {
        &self.buffer
    }

    pub fn size(&self) -> (u16, u16) {
        self.size
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
        self.cursor_x_saved = 0;
    }

    pub fn move_cursor_down(&mut self, lines: usize) {
        let limit = self.buffer.len_lines() - 1;
        let target = min(self.cursor.1 + lines, limit);
        let diff = target - self.cursor.1;

        let max = self.scroll_offset.1 + self.size.1 as usize;
        if target >= max {
            self.scroll_offset.1 += target - max + 1;
        }

        self.cursor.0 = self.cursor_x_saved;
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

        self.cursor = (self.cursor_x_saved, target);
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
        if line_len > 0 {
            line_len -= 1;
        }
        self.cursor.0 = min(line_len, self.cursor.0);
    }

    pub fn move_cursor_left(&mut self, cols: usize) {
        self.cursor.0 = if cols > self.cursor.0 { 0 } else { self.cursor.0 - cols };
        self.cursor_x_saved = self.cursor.0;
    }

    pub fn move_cursor_right(&mut self, cols: usize) {
        self.cursor.0 += cols;
        self.adjust_cursor_x();
        self.cursor_x_saved = self.cursor.0;
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        let (relx, rely) = self.rel_cursor_pos();
        if rely >= height {
            self.scroll_offset.1 += (rely - height + 1) as usize;
        }
        if relx >= width {
            self.scroll_offset.0 += (relx - width + 1) as usize;
        }
        self.size = (width, height);
    }

    pub fn insert_char(&mut self, chr: char) {
        let line_index = self.buffer.line_to_char(self.cursor.1);
        self.buffer.insert_char(line_index + self.cursor.0, chr);
        self.cursor.0 += 1;
    }

    pub fn insert_enter(&mut self) {
        self.insert_char('\n');
        self.move_cursor_down(1);
        self.cursor.0 = 0;
    }

    pub fn backspace(&mut self) {
        if self.cursor == (0, 0) {
            return;
        }

        let line_index = self.buffer.line_to_char(self.cursor.1);
        let index = line_index + self.cursor.0 - 1;
        self.buffer.remove(index..index + 1);
        
        let new_line = self.buffer.char_to_line(index);
        if new_line != self.cursor.1 {
            self.move_cursor_up(1);
        }
        self.cursor.0 = index - self.buffer.line_to_char(new_line);
    }

    pub fn delete(&mut self) {
        let index = self.buffer.line_to_char(self.cursor.1) + self.cursor.0;
        if index < self.buffer.len_chars() {
            self.buffer.remove(index..index + 1);
        }
        self.adjust_cursor_x();
    }
}

