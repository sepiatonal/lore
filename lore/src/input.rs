use std::sync::mpsc::Receiver;
use std::collections::HashMap;
use lore_render::{KeyboardInput, ElementState, VirtualKeyCode, MouseButton};
use lore_render::api::MouseEvent;

/**
* Lore_render provides two Receiver objects for keyboard and mouse input. These are clunky to work with.
* An InputManager provides an easier-to-use API for managing game input, given a Receiver for KeyboardInput and MouseEvents.
**/
pub struct InputManager {
    keybinds: HashMap<String, VirtualKeyCode>,
    keystates: HashMap<VirtualKeyCode, bool>,
    key_input: Receiver<KeyboardInput>,
    mouse_input: Receiver<MouseEvent>,

    mouse_position: (f64, f64),
    // how much the mouse position has changed since last update
    mouse_change: (f64, f64),
    // These booleans are reset every update. They are only true on the update where the click is received.
    mouse_left_clicked: bool,
    mouse_middle_clicked: bool,
    mouse_right_clicked: bool,
}
// TODO function to check if mouse is currently down (must store that)
// TODO function to check if a key was pressed/released this particular frame
// TODO mode where clicks/presses are not reset until polled?

impl InputManager {
    pub fn new(key_input: Receiver<KeyboardInput>, mouse_input: Receiver<MouseEvent>) -> InputManager {
        InputManager {
            keybinds: HashMap::new(),
            keystates: HashMap::new(),
            key_input,
            mouse_input,
            mouse_position: (0.0, 0.0),
            mouse_change: (0.0, 0.0),
            mouse_left_clicked: false,
            mouse_middle_clicked: false,
            mouse_right_clicked: false,
        }
    }

    pub fn update(&mut self) {
        self.update_keys();
        self.update_mouse();
    }

    fn update_keys(&mut self) {
        for k in self.key_input.try_iter() {
            if let Some(vk) = k.virtual_keycode {
                let state = k.state == ElementState::Pressed;
                self.keystates.insert(vk, state);
            }
        }
    }

    pub fn is_key_down(&mut self, keybind: &str) -> bool {
        let maybe_keycode = self.keybinds.get(keybind);
        if maybe_keycode.is_none() {
            return false;
        }
        let keycode = maybe_keycode.unwrap();
        let maybe_state = self.keystates.get(&keycode);
        match maybe_state {
            Some(state) => {
                *state
            },
            None => {
                false
            }
        }
    }

    fn update_mouse(&mut self) {
        self.mouse_change = (0.0, 0.0);
        self.mouse_left_clicked = false;
        self.mouse_middle_clicked = false;
        self.mouse_right_clicked = false;

        for m in self.mouse_input.try_iter() {
            // pos is the current position of the mouse, not the change
            match m {
                MouseEvent::Move(_, pos, _) => {
                    // old position
                    let (ox, oy) = self.mouse_position;
                    // delta position
                    let (dx, dy) = (pos.x - ox, pos.y - oy);
                    // old delta position
                    let (odx, ody) = self.mouse_change;
                    self.mouse_change = (odx + dx, ody + dy);
                    self.mouse_position = (pos.x, pos.y);
                },
                MouseEvent::Button(_, state, button, _mods) => {
                    if state == ElementState::Pressed {
                        match button {
                            MouseButton::Left => {
                                self.mouse_left_clicked = true;
                            },
                            MouseButton::Right => {
                                self.mouse_right_clicked = true;
                            },
                            MouseButton::Middle => {
                                self.mouse_middle_clicked = true;
                            },
                            _ => {}
                        }
                    }
                },
                // TODO mouse wheel
                MouseEvent::Wheel(_, _, _, _) => {}
            }
        }
    }

    pub fn is_left_mouse_clicked(&self) -> bool {
        self.mouse_left_clicked
    }

    pub fn is_right_mouse_clicked(&self) -> bool {
        self.mouse_right_clicked
    }

    pub fn is_middle_mouse_clicked(&self) -> bool {
        self.mouse_middle_clicked
    }

    pub fn get_mouse_delta(&self) -> (f64, f64) {
        self.mouse_change
    }

    pub fn get_mouse_position(&self) -> (f64, f64) {
        self.mouse_position
    }

    pub fn set_key(&mut self, name: &str, keycode: VirtualKeyCode) {
        self.keybinds.insert(String::from(name), keycode);
    }
}
