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
mod ex;

use commands::{Mapping, Mode, default_mappings};
use window::Window;
use ex::{ExBar, ExResult};

struct RVim {
    terminal: Stdout,
    size: (u16, u16),
    window: Window,
    bar: ExBar,

    mode: Mode,
    keymaps: Mapping,

    error_message: Option<String>,
    running: bool,
}

impl RVim {
    fn new() -> Result<Self> {
        let (width, height) = size()?;

        Ok(Self { 
            terminal: stdout(),
            size: (width, height),
            window: Window::new(width, height - 1),
            bar: ExBar::new(),
            mode: Mode::Normal,
            keymaps: default_mappings(),
            error_message: None,
            running: true,
        })
    }

    fn read_file<P: AsRef<std::path::Path>>(&mut self, path: P) -> IOResult<()> {
        self.window.set_buffer(Rope::from_reader(std::fs::File::open(path)?)?);
        Ok(())
    }

    fn write_file<P: AsRef<std::path::Path>>(&mut self, path: P) {
        let file = match std::fs::File::create(path) {
            Ok(file) => file,
            Err(err) => {
                self.error_message = Some(format!("Couldn't open file: {}", err));
                return;
            }
        };

        match self.window.buffer().write_to(file) {
            Ok(()) => {},
            Err(err) => {
                self.error_message = Some(format!("Couldn't write file: {}", err));
            }
        };
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
        if let Mode::Ex = self.mode {
            self.terminal.queue(style::Print(":"))?
                         .queue(style::Print(self.bar.buffer()))?
                         .queue(cursor::Show)?
                         .execute(cursor::MoveTo(1 + self.bar.cursor_index() as u16, self.size.1 - 1))?;
        } else {
            self.terminal.queue(style::Print(&self.mode))?.queue(style::Print(" "))?;
            if let Some(msg) = self.error_message.as_ref() {
                self.terminal.queue(style::Print(msg))?;
            }

            let (x, y) = self.window.rel_cursor_pos();
            self.terminal.queue(cursor::Show)?.execute(cursor::MoveTo(x, y))?;
        }

        Ok(())
    }

    pub fn current_window_mut(&mut self) -> &mut Window {
        &mut self.window
    }

    pub fn set_mode(&mut self, mode: Mode) {
        if let Mode::Ex = mode {
            self.error_message = None;
        }

        self.mode = mode;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    fn mainloop(&mut self) -> Result<()> {
        while self.running {
            self.draw()?;

            let ev = event::read()?;
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
            Mode::Ex => {
                match self.bar.handle_key(key) {
                    ExResult::Aborted => self.set_mode(Mode::Normal),
                    ExResult::StillEditing => {},
                    ExResult::Finished(cmd) => {
                        self.perform_ex_cmd(cmd);
                        self.set_mode(Mode::Normal);
                    },
                }
            },
            _ => {}
        }
    }

    fn perform_ex_cmd(&mut self, cmd: String) {
        let mut tokens = cmd.split_whitespace();
        let c = match tokens.next() {
            Some(c) => c,
            None => return,
        };

        match c {
            "q" => self.stop(),
            "w" => {
                let filename = match tokens.next() {
                    Some(name) => name,
                    None => {
                        self.error_message = Some(format!("Expected filename"));
                        return;
                    },
                };

                self.write_file(filename);
            },
            c @ _ => self.error_message = Some(format!("Unknown command: {}", c)),
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
