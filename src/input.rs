use macroquad::prelude::*;

pub const STICK_CENTER: Vec2 = Vec2::new(58.0, 210.0);
pub const STICK_RADIUS: f32 = 34.0;
pub const AIM_STICK_CENTER: Vec2 = Vec2::new(335.0, 207.0);
pub const AIM_STICK_RADIUS: f32 = 34.0;
pub const KNIFE_BUTTON: Vec2 = Vec2::new(435.0, 70.0);
pub const DASH_BUTTON: Vec2 = Vec2::new(435.0, 130.0);
pub const TIMESTOP_BUTTON: Vec2 = Vec2::new(435.0, 190.0);
pub const ACTION_BUTTON_RADIUS: f32 = 21.0;

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
    pub touch_controls_visible: bool,
    pub move_stick_position: Vec2,
    pub move_stick_active: bool,
    pub aim_stick_position: Vec2,
    pub aim_stick_active: bool,
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

    let screen_size = vec2(screen_width(), screen_height());
    let mouse = mouse_position();
    let mut aim_position = screen_to_virtual(vec2(mouse.0, mouse.1), screen_size);
    let mut move_dir = move_dir.normalize_or_zero();
    let mut attack_pressed = is_mouse_button_pressed(MouseButton::Left);
    let mut attack_held = is_mouse_button_down(MouseButton::Left);
    let mut throw_pressed = is_mouse_button_pressed(MouseButton::Right);
    let mut dash_pressed = is_key_pressed(KeyCode::LeftShift);
    let mut timestop_pressed = is_key_pressed(KeyCode::Space);
    let touches = touches();
    let mut move_stick_position = STICK_CENTER;
    let mut move_stick_active = false;
    let mut aim_stick_position = AIM_STICK_CENTER;
    let mut aim_stick_active = false;
    let mut touch_aim_offset = None;

    for touch in &touches {
        if matches!(touch.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
            continue;
        }

        let position = screen_to_virtual(touch.position, screen_size);
        if position.x < 240.0 {
            let offset = position - STICK_CENTER;
            move_stick_position = STICK_CENTER + offset.clamp_length_max(STICK_RADIUS);
            move_stick_active = true;
            move_dir += joystick_direction(offset);
            continue;
        }

        let started = touch.phase == TouchPhase::Started;
        if position.distance(KNIFE_BUTTON) <= ACTION_BUTTON_RADIUS {
            throw_pressed |= started;
        } else if position.distance(DASH_BUTTON) <= ACTION_BUTTON_RADIUS {
            dash_pressed |= started;
        } else if position.distance(TIMESTOP_BUTTON) <= ACTION_BUTTON_RADIUS {
            timestop_pressed |= started;
        } else {
            let offset = position - AIM_STICK_CENTER;
            aim_stick_position = AIM_STICK_CENTER + offset.clamp_length_max(AIM_STICK_RADIUS);
            aim_stick_active = true;
            touch_aim_offset = Some((offset, started));
        }
    }

    move_dir = move_dir.clamp_length_max(1.0);
    let mut aim_dir = (aim_position - player_position).normalize_or_zero();
    if let Some((offset, pressed)) = touch_aim_offset {
        aim_dir = aim_stick_direction(offset);
        aim_position = player_position + aim_dir * 110.0;
        attack_pressed |= pressed && aim_dir.length_squared() > 0.0;
        attack_held = aim_dir.length_squared() > 0.0;
    }

    InputState {
        move_dir,
        aim_dir,
        aim_position,
        attack_pressed,
        attack_held,
        throw_pressed,
        dash_pressed,
        timestop_pressed,
        ult_pressed: is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::R),
        touch_controls_visible: cfg!(target_os = "android") || !touches.is_empty(),
        move_stick_position,
        move_stick_active,
        aim_stick_position,
        aim_stick_active,
    }
}

fn joystick_direction(offset: Vec2) -> Vec2 {
    let distance = offset.length();
    let deadzone = 5.0;
    if distance <= deadzone {
        Vec2::ZERO
    } else {
        offset.normalize() * ((distance - deadzone) / (STICK_RADIUS - deadzone)).clamp(0.0, 1.0)
    }
}

fn aim_stick_direction(offset: Vec2) -> Vec2 {
    if offset.length_squared() <= 6.0_f32.powi(2) {
        Vec2::ZERO
    } else {
        offset.normalize()
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

    #[test]
    fn joystick_has_a_deadzone_and_respects_direction() {
        assert_eq!(joystick_direction(Vec2::new(2.0, 0.0)), Vec2::ZERO);
        let up = joystick_direction(Vec2::new(0.0, -STICK_RADIUS));
        assert!(up.y < -0.99);
        assert_eq!(up.x, 0.0);
    }

    #[test]
    fn aim_stick_has_a_deadzone_and_aims_in_the_pushed_direction() {
        assert_eq!(aim_stick_direction(Vec2::new(3.0, 0.0)), Vec2::ZERO);
        let up = aim_stick_direction(Vec2::new(0.0, -AIM_STICK_RADIUS));
        assert!(up.y < -0.99);
        assert_eq!(up.x, 0.0);
    }
}
