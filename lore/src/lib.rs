pub mod input {
    pub struct InputManager {
        keybinds: std::collections::HashMap<String, lore_render::VirtualKeyCode>,
    }

    impl InputManager {
        pub fn new() -> InputManager {
            InputManager {
                keybinds: std::collections::HashMap::new(),
            }
        }

        pub fn set_key(&mut self, name: String, keycode: lore_render::VirtualKeyCode) {
            self.keybinds.insert(name, keycode);
        }
    }
}
