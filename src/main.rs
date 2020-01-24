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

use commands::{Mapping, default_mappings};

struct RVim {
    terminal: Stdout,
    pub buffer: Rope,
    pub size: (u16, u16),
    pub cursor: (usize, usize),

    keymaps: Mapping,
}

impl RVim {
    fn new() -> Result<Self> {
        Ok(Self { 
            terminal: stdout(),
            buffer: Rope::new(),
            size: size()?,
            cursor: (0, 0),
            keymaps: default_mappings(),
        })
    }

    fn read_file<P: AsRef<std::path::Path>>(&mut self, path: P) -> IOResult<()> {
        self.buffer = Rope::from_reader(std::fs::File::open(path)?)?;
        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        if self.size.0 < 2 || self.size.1 < 2 {
            return Ok(());
        }

        let mut lines = self.buffer.lines();
        for y in 0..self.size.1 - 1 {
            let line = lines.next();

            self.terminal.queue(cursor::MoveTo(0, y))?;
            match line {
                Some(slice) => self.terminal.queue(style::Print(slice))?,
                None => self.terminal.queue(style::Print("~"))?,
            };
        }
        self.terminal.execute(cursor::MoveTo(self.cursor.0 as u16, self.cursor.1 as u16))?;

        Ok(())
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
