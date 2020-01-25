use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::RVim;

pub struct Mapping {
    maps: HashMap<KeyEvent, fn(&mut RVim)>,
}

impl Mapping {
    pub fn new() -> Self {
        Self {
            maps: HashMap::new(),
        }
    }

    pub fn get_mapping(&self, event: &KeyEvent) -> Option<&fn(&mut RVim)> {
        self.maps.get(event)
    }

    pub fn insert_mapping(&mut self, event: KeyEvent, func: fn(&mut RVim)) {
        self.maps.insert(event, func);
    }

    pub fn insert_nomod_mapping(&mut self, code: KeyCode, func: fn(&mut RVim)) {
        self.maps.insert(KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
        }, func);
    }
}

pub fn default_mappings() -> Mapping {
    let mut mode = Mapping::new();

    mode.insert_nomod_mapping(KeyCode::Char('h'), move_left);
    mode.insert_nomod_mapping(KeyCode::Char('j'), move_down);
    mode.insert_nomod_mapping(KeyCode::Char('k'), move_up);
    mode.insert_nomod_mapping(KeyCode::Char('l'), move_right);
    mode.insert_nomod_mapping(KeyCode::Left, move_left);
    mode.insert_nomod_mapping(KeyCode::Down, move_down);
    mode.insert_nomod_mapping(KeyCode::Up, move_up);
    mode.insert_nomod_mapping(KeyCode::Right, move_right);

    mode
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

