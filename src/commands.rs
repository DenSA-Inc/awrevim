use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::RVim;

type RVimFunction = fn(&mut RVim);
type KeyMap = HashMap<KeyEvent, RVimFunction>;

pub enum Mode {
    Normal,
    Insert,
    Operator,
    Ex,
}

impl Display for Mode {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        use Mode::*;

        write!(fmt, "--- {} ---", match self {
            Normal => "NORMAL",
            Insert => "INSERT",
            Operator => "OPERATOR",
            Ex => "EX",
        })
    }
}

pub struct Mapping {
    nmaps: KeyMap,
    imaps: KeyMap,
    omaps: KeyMap,
    xmaps: KeyMap,
}

impl Mapping {
    pub fn new() -> Self {
        Self {
            nmaps: HashMap::new(),
            imaps: HashMap::new(),
            omaps: HashMap::new(),
            xmaps: HashMap::new(),
        }
    }

    fn get_map(&self, mode: &Mode) -> &KeyMap {
        use Mode::*;
        match mode {
            Normal => &self.nmaps,
            Insert => &self.imaps,
            Operator => &self.omaps,
            Ex => &self.xmaps,
        }
    }

    fn get_map_mut(&mut self, mode: &Mode) -> &mut KeyMap {
        use Mode::*;
        match mode {
            Normal => &mut self.nmaps,
            Insert => &mut self.imaps,
            Operator => &mut self.omaps,
            Ex => &mut self.xmaps,
        }
    }

    pub fn get_mapping(&self, mode: &Mode, event: &KeyEvent) -> Option<&fn(&mut RVim)> {
        self.get_map(mode).get(event)
    }

    pub fn insert_mapping(&mut self, mode: &Mode, event: KeyEvent, func: fn(&mut RVim)) {
        self.get_map_mut(mode).insert(event, func);
    }

    pub fn insert_nomod_mapping(&mut self, mode: &Mode, code: KeyCode, func: fn(&mut RVim)) {
        self.get_map_mut(mode).insert(KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
        }, func);
    }
}

pub fn default_mappings() -> Mapping {
    use Mode::*;
    let mut map = Mapping::new();

    map.insert_nomod_mapping(&Normal, KeyCode::Char('h'), move_left);
    map.insert_nomod_mapping(&Normal, KeyCode::Char('j'), move_down);
    map.insert_nomod_mapping(&Normal, KeyCode::Char('k'), move_up);
    map.insert_nomod_mapping(&Normal, KeyCode::Char('l'), move_right);
    map.insert_nomod_mapping(&Normal, KeyCode::Left, move_left);
    map.insert_nomod_mapping(&Normal, KeyCode::Down, move_down);
    map.insert_nomod_mapping(&Normal, KeyCode::Up, move_up);
    map.insert_nomod_mapping(&Normal, KeyCode::Right, move_right);

    map.insert_mapping(&Normal, KeyEvent { code: KeyCode::Char('d'), modifiers: KeyModifiers::CONTROL }, move_half_down);
    map.insert_mapping(&Normal, KeyEvent { code: KeyCode::Char('u'), modifiers: KeyModifiers::CONTROL }, move_half_up);

    map.insert_nomod_mapping(&Normal, KeyCode::Char('i'), start_insert);
    map.insert_nomod_mapping(&Normal, KeyCode::Char(':'), start_ex);
    map.insert_nomod_mapping(&Insert, KeyCode::Esc, back_normal);

    map
}

fn move_down(editor: &mut RVim) {
    editor.current_window_mut().move_cursor_down(1);
}

fn move_up(editor: &mut RVim) {
    editor.current_window_mut().move_cursor_up(1);
}

fn move_right(editor: &mut RVim) {
    editor.current_window_mut().move_cursor_right(1);
}

fn move_left(editor:  &mut RVim) {
    editor.current_window_mut().move_cursor_left(1);
}

fn move_half_down(editor: &mut RVim) {
    let window = editor.current_window_mut();
    let half = (window.size().1 + 1) / 2;
    window.move_cursor_down(half as usize);
}

fn move_half_up(editor: &mut RVim) {
    let window = editor.current_window_mut();
    let half = (window.size().1 + 1) / 2;
    window.move_cursor_up(half as usize);
}

fn start_insert(editor: &mut RVim) {
    editor.set_mode(Mode::Insert);
}

fn back_normal(editor: &mut RVim) {
    editor.set_mode(Mode::Normal);
}

fn start_ex(editor: &mut RVim) {
    editor.set_mode(Mode::Ex);
}
