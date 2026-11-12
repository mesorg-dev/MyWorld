use macroquad::prelude::*;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
pub struct InputState {
    pub move_dir: Vec2,
    pub aim_dir: Vec2,
    pub aim_position: Vec2,
    pub attack_pressed: bool,
    pub attack_held: bool,
    pub throw_pressed: bool,
    pub dash_pressed: bool,
    pub timestop_pressed: bool,
    pub ult_pressed: bool,
}

pub fn read_input(player_position: Vec2) -> InputState {
    let mut move_dir = Vec2::ZERO;

    if is_key_down(KeyCode::W) {
        move_dir.y -= 1.0;
    }
    if is_key_down(KeyCode::S) {
        move_dir.y += 1.0;
    }
    if is_key_down(KeyCode::A) {
        move_dir.x -= 1.0;
    }
    if is_key_down(KeyCode::D) {
        move_dir.x += 1.0;
    }

    let mouse = mouse_position();
    let aim_position = screen_to_virtual(
        vec2(mouse.0, mouse.1),
        vec2(screen_width(), screen_height()),
    );
    let aim_dir = (aim_position - player_position).normalize_or_zero();

    InputState {
        move_dir,
        aim_dir,
        aim_position,
        attack_pressed: is_mouse_button_pressed(MouseButton::Left),
        attack_held: is_mouse_button_down(MouseButton::Left),
        throw_pressed: is_mouse_button_pressed(MouseButton::Right),
        dash_pressed: is_key_pressed(KeyCode::LeftShift),
        timestop_pressed: is_key_pressed(KeyCode::Space),
        ult_pressed: is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::R),
    }
}

fn screen_to_virtual(mouse: Vec2, screen_size: Vec2) -> Vec2 {
    let scale = (screen_size.x / 480.0).min(screen_size.y / 270.0);
    let viewport_size = vec2(480.0, 270.0) * scale;
    let viewport_origin = (screen_size - viewport_size) * 0.5;
    (mouse - viewport_origin) / scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_coordinates_map_to_virtual_arena_center() {
        let mapped = screen_to_virtual(vec2(640.0, 360.0), vec2(1280.0, 720.0));
        assert!((mapped - vec2(240.0, 135.0)).length() < 0.01);
    }

    #[test]
    fn mouse_mapping_accounts_for_letterboxed_window() {
        let mapped = screen_to_virtual(vec2(500.0, 118.75), vec2(1000.0, 800.0));
        assert!((mapped - vec2(240.0, 0.0)).length() < 0.01);
    }
}
