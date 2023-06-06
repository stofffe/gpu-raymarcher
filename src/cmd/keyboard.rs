use crate::context::Context;
use crate::input::{KeyCode, KeyModifier};

/// Returns true if KeyCode is pressed
/// Accepts repeating
pub fn key_pressed(ctx: &Context, keycode: KeyCode) -> bool {
    ctx.input.keyboard.key_pressed(keycode)
}

/// Returns true if KeyCode was pressed this frame
pub fn key_just_pressed(ctx: &Context, keycode: KeyCode) -> bool {
    ctx.input.keyboard.key_just_pressed(keycode)
}

/// Returns true is KeyCode was released this frame
pub fn key_released(ctx: &Context, keycode: KeyCode) -> bool {
    ctx.input.keyboard.key_released(keycode)
}

/// Returns true if KeyModifer is pressed
/// Accepts repeating
pub fn modifier_pressed(ctx: &Context, key_modifier: KeyModifier) -> bool {
    ctx.input.keyboard.modifier_pressed(key_modifier)
}

/// Returns true if KeyModifer was pressed this frame
pub fn modifer_just_pressed(ctx: &Context, key_modifier: KeyModifier) -> bool {
    ctx.input.keyboard.modifier_just_pressed(key_modifier)
}

/// Returns true if KeyModifier was released this frame
pub fn modifer_released(ctx: &Context, key_modifier: KeyModifier) -> bool {
    ctx.input.keyboard.modifier_released(key_modifier)
}
