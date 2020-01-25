use std::io::{stdout, Write, Stdout};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, size};
use crossterm::{Result, queue};
use crossterm::terminal::{ClearType, Clear};
use crossterm::cursor;
use crossterm::event;
use crossterm::style;
use crossterm::{ExecutableCommand, QueueableCommand};

use ropey::Rope;

use std::io::Result as IOResult;

mod commands;
mod window;

use commands::{Mapping, Mode, default_mappings};
use window::Window;

struct RVim {
    terminal: Stdout,
    size: (u16, u16),
    window: Window,

    mode: Mode,
    keymaps: Mapping,
}

impl RVim {
    fn new() -> Result<Self> {
        let (width, height) = size()?;

        Ok(Self { 
            terminal: stdout(),
            size: (width, height),
            window: Window::new(width, height - 1),
            mode: Mode::Normal,
            keymaps: default_mappings(),
        })
    }

    fn read_file<P: AsRef<std::path::Path>>(&mut self, path: P) -> IOResult<()> {
        self.window.set_buffer(Rope::from_reader(std::fs::File::open(path)?)?);
        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        if self.size.0 < 2 || self.size.1 < 2 {
            return Ok(());
        }

        self.terminal.queue(cursor::Hide)?;
        let mut lines = self.window.visible_lines();
        for y in 0..self.size.1 - 1 {
            let line = lines.next();

            self.terminal.queue(cursor::MoveTo(0, y))?.queue(Clear(ClearType::CurrentLine))?;
            match line {
                Some(slice) => self.terminal.queue(style::Print(slice))?,
                None => self.terminal.queue(style::Print("~"))?,
            };
        }
        self.terminal.queue(cursor::MoveTo(0, self.size.1 - 1))?.queue(Clear(ClearType::CurrentLine))?;
        let (x, y) = self.window.rel_cursor_pos();
        self.terminal.queue(cursor::Show)?.execute(cursor::MoveTo(x, y))?;

        Ok(())
    }

    pub fn current_window_mut(&mut self) -> &mut Window {
        &mut self.window
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn mainloop(&mut self) -> Result<()> {
        loop {
            self.draw()?;

            let ev = event::read()?;
            if let event::Event::Key(event::KeyEvent { code: event::KeyCode::Char('q'), modifiers: _ }) = ev {
                break;
            }
            if let event::Event::Key(key_event) = ev {
                if let Some(func) = self.keymaps.get_mapping(&self.mode, &key_event) {
                    func(self);
                } else {
                    self.default_action(key_event);
                }
            }
            if let event::Event::Resize(width, height) = ev {
                self.size = (width, height);
                self.window.resize(width, height - 1);
                self.draw()?;
            }
        }

        Ok(())
    }

    fn default_action(&mut self, key: event::KeyEvent) {
        match self.mode {
            Mode::Insert => {
                if key.modifiers.is_empty() {
                    use event::KeyCode::*;

                    match key.code {
                        Char(chr) => self.window.insert_char(chr),
                        Enter => self.window.insert_enter(),
                        Backspace => self.window.backspace(),
                        Delete => self.window.delete(),
                        _ => {},
                    }
                }
            },
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let mut terminal = stdout();
    enable_raw_mode()?;
    queue!(terminal, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    terminal.flush()?;

    let mut vim = RVim::new()?;
    if let Some(file) = std::env::args().nth(1) {
        vim.read_file(file)?;
    }
    vim.mainloop()?;

    queue!(terminal, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    disable_raw_mode()?;
    terminal.flush()?;

    Ok(())
}
