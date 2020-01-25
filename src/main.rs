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

use commands::{Mapping, default_mappings};
use window::Window;

struct RVim {
    terminal: Stdout,
    size: (u16, u16),
    window: Window,

    keymaps: Mapping,
}

impl RVim {
    fn new() -> Result<Self> {
        let (width, height) = size()?;

        Ok(Self { 
            terminal: stdout(),
            size: (width, height),
            window: Window::new(width, height - 1),
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

        let mut lines = self.window.visible_lines();
        for y in 0..self.size.1 - 1 {
            let line = lines.next();

            self.terminal.queue(cursor::MoveTo(0, y))?.queue(Clear(ClearType::CurrentLine))?;
            match line {
                Some(slice) => self.terminal.queue(style::Print(slice))?,
                None => self.terminal.queue(style::Print("~"))?,
            };
        }
        let (x, y) = self.window.rel_cursor_pos();
        self.terminal.execute(cursor::MoveTo(x, y))?;

        Ok(())
    }

    pub fn current_window_mut(&mut self) -> &mut Window {
        &mut self.window
    }

    fn mainloop(&mut self) -> Result<()> {
        loop {
            self.draw()?;

            let ev = event::read()?;
            if let event::Event::Key(event::KeyEvent { code: event::KeyCode::Char('q'), modifiers: _ }) = ev {
                break;
            }
            if let event::Event::Key(key_event) = ev {
                if let Some(func) = self.keymaps.get_mapping(&key_event) {
                    func(self);
                }
            }
            if let event::Event::Resize(width, height) = ev {
                self.size = (width, height);
                self.draw()?;
            }
        }

        Ok(())
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

    disable_raw_mode()?;

    Ok(())
}
