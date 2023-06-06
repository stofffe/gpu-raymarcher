pub use winit::event::VirtualKeyCode as KeyCode;
use winit::event::{ModifiersState, MouseButton};

use std::collections::HashSet;

use crate::render::RenderContext;

#[derive(Default)]
pub struct InputContext {
    pub keyboard: KeyboardContext,
    pub mouse: MouseContext,
}

#[derive(Default)]
pub struct MouseContext {
    on_screen: bool,
    pos: (f64, f64),
    mouse_delta: (f64, f64),
    pressed: HashSet<MouseButton>,
    previous_pressed: HashSet<MouseButton>,
    scroll_delta: (f64, f64),
}

impl MouseContext {
    /// Returns true if Button is down
    /// Accepts repeating
    pub fn button_pressed(&self, keycode: MouseButton) -> bool {
        self.pressed.contains(&keycode)
    }

    /// Returns true if Button was pressed this frame
    /// Does not accept repeating
    pub fn button_just_pressed(&self, keycode: MouseButton) -> bool {
        self.pressed.contains(&keycode) && !self.previous_pressed.contains(&keycode)
    }

    /// Returns true is MouseButton was released this frame
    pub fn button_released(&self, keycode: MouseButton) -> bool {
        !self.pressed.contains(&keycode) && self.previous_pressed.contains(&keycode)
    }

    /// Returns if mouse is on screen or not
    pub fn on_screen(&self) -> bool {
        self.on_screen
    }

    /// Returns the current physical coordinates for the mouse
    pub fn mouse_pos_physical(&self) -> (f64, f64) {
        self.pos
    }

    /// Returns the current pixel under the mouse
    pub fn mouse_pos_pixel(&self, ctx: &RenderContext) -> (u32, u32) {
        // When holding the mouse button down pos can get bigger than physical size
        // So clamp to avoid out of bounds
        let relative_x = self.pos.0 / ctx.window_size.width as f64;
        let relative_y = self.pos.1 / ctx.window_size.height as f64;
        let pixel_x = relative_x * ctx.resolution.0 as f64;
        let pixel_y = relative_y * ctx.resolution.1 as f64;
        (pixel_x as u32, pixel_y as u32)
    }

    /// Returns the (dx, dy) change in mouse position
    pub fn mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    pub fn scroll_delta(&self) -> (f64, f64) {
        self.scroll_delta
    }

    /// Sets mouse off screen
    pub(crate) fn set_on_screen(&mut self, on_screen: bool) {
        self.on_screen = on_screen;
    }

    // Sets the current position of the mouse
    pub(crate) fn set_pos(&mut self, x: f64, y: f64, ctx: &RenderContext) {
        self.pos = (x, y);

        // Check if mouse is on screen
        // When holding mouse button CursorLeft event will not be called so need check here
        if x >= 0.0
            && x < ctx.window_size.width as f64
            && y >= 0.0
            && y < ctx.window_size.height as f64
        {
            self.on_screen = true;
        } else {
            self.on_screen = false;
        }
    }

    /// Sets the (dx, dy) change in mouse position
    pub(crate) fn set_mouse_delta(&mut self, change: (f64, f64)) {
        self.mouse_delta = change;
    }

    pub(crate) fn set_scroll_delta(&mut self, change: (f64, f64)) {
        self.scroll_delta = change;
    }

    /// Sets button for current frame
    pub(crate) fn press_button(&mut self, keycode: MouseButton) {
        self.pressed.insert(keycode);
    }

    /// Release button
    pub(crate) fn release_button(&mut self, keycode: MouseButton) {
        self.pressed.remove(&keycode);
    }

    /// Save current buttons in previous
    /// Should be called each frame
    pub(crate) fn save_buttons(&mut self) {
        self.previous_pressed = self.pressed.clone()
    }
}

#[derive(Default)]
pub struct KeyboardContext {
    pressed: HashSet<KeyCode>,
    previous_pressed: HashSet<KeyCode>,
    pressed_modifiers: HashSet<KeyModifier>,
    previous_pressed_modifiers: HashSet<KeyModifier>,
}

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum KeyModifier {
    Shift,
    Ctrl,
    Alt,
    Logo,
}

// Getting keys
impl KeyboardContext {
    /// Returns true if KeyCode is down
    /// Accepts repeating
    pub fn key_pressed(&self, keycode: KeyCode) -> bool {
        self.pressed.contains(&keycode)
    }

    /// Returns true if KeyCode was pressed this frame
    /// Does not accepts repeating
    pub fn key_just_pressed(&self, keycode: KeyCode) -> bool {
        self.pressed.contains(&keycode) && !self.previous_pressed.contains(&keycode)
    }

    /// Returns true is KeyCode was released this frame
    pub fn key_released(&self, keycode: KeyCode) -> bool {
        !self.pressed.contains(&keycode) && self.previous_pressed.contains(&keycode)
    }

    pub fn modifier_pressed(&self, modifier: KeyModifier) -> bool {
        self.pressed_modifiers.contains(&modifier)
    }

    pub fn modifier_just_pressed(&self, modifier: KeyModifier) -> bool {
        self.pressed_modifiers.contains(&modifier)
            && !self.previous_pressed_modifiers.contains(&modifier)
    }

    pub fn modifier_released(&self, modifier: KeyModifier) -> bool {
        !self.pressed_modifiers.contains(&modifier)
            && self.previous_pressed_modifiers.contains(&modifier)
    }
}

impl KeyboardContext {
    /// Sets key for current frame
    pub(crate) fn set_key(&mut self, keycode: KeyCode) {
        self.pressed.insert(keycode);
    }

    /// Release key
    pub(crate) fn release_key(&mut self, keycode: KeyCode) {
        self.pressed.remove(&keycode);
    }

    pub fn modifiers_changed(&mut self, state: ModifiersState) {
        self.pressed_modifiers.clear();
        if state.shift() {
            self.pressed_modifiers.insert(KeyModifier::Shift);
        }
        if state.ctrl() {
            self.pressed_modifiers.insert(KeyModifier::Ctrl);
        }
        if state.alt() {
            self.pressed_modifiers.insert(KeyModifier::Alt);
        }
        if state.logo() {
            self.pressed_modifiers.insert(KeyModifier::Logo);
        }
    }

    /// Save current keys in previous
    /// Should be called each frame
    pub(crate) fn save_keys(&mut self) {
        self.previous_pressed = self.pressed.clone();
    }

    pub(crate) fn save_modifiers(&mut self) {
        self.previous_pressed_modifiers = self.pressed_modifiers.clone();
    }
}

#[cfg(test)]
mod tests {
    use winit::event::ModifiersState;

    use crate::input::KeyCode;
    use crate::input::KeyModifier;
    use crate::input::KeyboardContext;

    #[test]
    fn key_pressed_test() {
        let mut kc = KeyboardContext::default();

        kc.set_key(KeyCode::A);

        assert!(kc.key_pressed(KeyCode::A));
        assert!(!kc.key_pressed(KeyCode::B));

        kc.save_keys();
        kc.set_key(KeyCode::B);

        assert!(kc.key_pressed(KeyCode::A));
        assert!(kc.key_pressed(KeyCode::B));

        kc.save_keys();
        kc.release_key(KeyCode::A);

        assert!(!kc.key_pressed(KeyCode::A));
        assert!(kc.key_pressed(KeyCode::B));
    }

    #[test]
    fn key_just_pressed_test() {
        let mut kc = KeyboardContext::default();
        kc.set_key(KeyCode::A);

        assert!(kc.key_just_pressed(KeyCode::A));

        kc.save_keys();
        kc.set_key(KeyCode::A);

        assert!(!kc.key_just_pressed(KeyCode::A));
    }

    #[test]
    fn key_released_test() {
        let mut kc = KeyboardContext::default();
        kc.set_key(KeyCode::A);

        assert!(!kc.key_released(KeyCode::A));

        kc.save_keys();
        kc.release_key(KeyCode::A);

        assert!(kc.key_released(KeyCode::A));
    }

    #[test]
    fn modifer_pressed_test() {
        let mut kc = KeyboardContext::default();

        // Press Shift
        kc.modifiers_changed(ModifiersState::SHIFT);

        assert!(kc.modifier_pressed(KeyModifier::Shift));
        assert!(!kc.modifier_pressed(KeyModifier::Ctrl));

        kc.save_modifiers();

        // Press Shift and Ctrl
        kc.modifiers_changed(ModifiersState::SHIFT | ModifiersState::CTRL);

        assert!(kc.modifier_pressed(KeyModifier::Shift));
        assert!(kc.modifier_pressed(KeyModifier::Ctrl));

        kc.save_modifiers();

        // Release Shift
        kc.modifiers_changed(ModifiersState::CTRL);

        assert!(!kc.modifier_pressed(KeyModifier::Shift));
        assert!(kc.modifier_pressed(KeyModifier::Ctrl));
    }

    #[test]
    fn modifier_just_pressed_test() {
        let mut kc = KeyboardContext::default();
        // Press shift
        kc.modifiers_changed(ModifiersState::SHIFT);

        assert!(kc.modifier_just_pressed(KeyModifier::Shift));

        kc.save_modifiers();

        // Release shift
        kc.modifiers_changed(ModifiersState::from_bits(0).unwrap());

        assert!(!kc.modifier_just_pressed(KeyModifier::Shift));
    }

    #[test]
    fn modifier_released_test() {
        let mut kc = KeyboardContext::default();

        // Press shift
        kc.modifiers_changed(ModifiersState::SHIFT);

        assert!(!kc.modifier_released(KeyModifier::Shift));
        assert!(!kc.modifier_released(KeyModifier::Ctrl));

        kc.save_modifiers();

        // Release shift
        kc.modifiers_changed(ModifiersState::from_bits(0).unwrap());

        assert!(kc.modifier_released(KeyModifier::Shift));
        assert!(!kc.modifier_released(KeyModifier::Ctrl));
    }
}
