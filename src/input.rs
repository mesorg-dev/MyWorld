use macroquad::prelude::*;

pub const STICK_CENTER: Vec2 = Vec2::new(58.0, 210.0);
pub const STICK_RADIUS: f32 = 34.0;
pub const AIM_STICK_CENTER: Vec2 = Vec2::new(335.0, 207.0);
pub const AIM_STICK_RADIUS: f32 = 34.0;
pub const KNIFE_BUTTON: Vec2 = Vec2::new(435.0, 50.0);
pub const DASH_BUTTON: Vec2 = Vec2::new(435.0, 105.0);
pub const TIMESTOP_BUTTON: Vec2 = Vec2::new(435.0, 160.0);
pub const STRIKE_BUTTON: Vec2 = Vec2::new(435.0, 215.0);
pub const ACTION_BUTTON_RADIUS: f32 = 21.0;

#[derive(Clone, Copy, Debug)]
pub struct TouchLayout {
    pub move_stick: Vec2,
    pub aim_stick: Vec2,
    pub knife: Vec2,
    pub dash: Vec2,
    pub timestop: Vec2,
    pub strike: Vec2,
}

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

pub fn read_input(player_position: Vec2, last_aim_dir: Vec2) -> InputState {
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
    let controls = touch_layout(screen_size);
    let mouse = mouse_position();
    let mouse_aim_position = screen_to_virtual(vec2(mouse.0, mouse.1), screen_size);
    let mut aim_position = mouse_aim_position;
    let mut move_dir = move_dir.normalize_or_zero();
    let mut aim_dir = if cfg!(target_os = "android") {
        last_aim_dir.normalize_or_zero()
    } else {
        (mouse_aim_position - player_position).normalize_or_zero()
    };
    let mut attack_pressed = is_mouse_button_pressed(MouseButton::Left);
    let mut attack_held = is_mouse_button_down(MouseButton::Left);
    let mut throw_pressed = is_mouse_button_pressed(MouseButton::Right);
    let mut dash_pressed = is_key_pressed(KeyCode::LeftShift);
    let mut timestop_pressed = is_key_pressed(KeyCode::Space);
    let touches = touches();
    let mut move_stick_position = controls.move_stick;
    let mut move_stick_active = false;
    let mut aim_stick_position = controls.aim_stick;
    let mut aim_stick_active = false;
    let mut touch_aim_offset = None;

    for touch in &touches {
        if matches!(touch.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
            continue;
        }

        let position = screen_to_virtual(touch.position, screen_size);
        if position.x < 240.0 {
            let offset = position - controls.move_stick;
            move_stick_position = controls.move_stick + offset.clamp_length_max(STICK_RADIUS);
            move_stick_active = true;
            move_dir += joystick_direction(offset);
            continue;
        }

        let started = touch.phase == TouchPhase::Started;
        if position.distance(controls.knife) <= ACTION_BUTTON_RADIUS {
            throw_pressed |= started;
        } else if position.distance(controls.dash) <= ACTION_BUTTON_RADIUS {
            dash_pressed |= started;
        } else if position.distance(controls.timestop) <= ACTION_BUTTON_RADIUS {
            timestop_pressed |= started;
        } else if position.distance(controls.strike) <= ACTION_BUTTON_RADIUS {
            attack_pressed |= started;
            attack_held = true;
        } else {
            let offset = position - controls.aim_stick;
            aim_stick_position = controls.aim_stick + offset.clamp_length_max(AIM_STICK_RADIUS);
            aim_stick_active = true;
            touch_aim_offset = Some(offset);
        }
    }

    move_dir = move_dir.clamp_length_max(1.0);
    if cfg!(target_os = "android") {
        aim_position = player_position + aim_dir * 110.0;
    }
    if let Some(offset) = touch_aim_offset {
        aim_dir = touch_aim_direction(offset, last_aim_dir);
        aim_position = player_position + aim_dir * 110.0;
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

fn touch_aim_direction(offset: Vec2, last_aim_dir: Vec2) -> Vec2 {
    let direction = aim_stick_direction(offset);
    if direction.length_squared() > 0.0 {
        direction
    } else {
        last_aim_dir.normalize_or_zero()
    }
}

fn screen_to_virtual(mouse: Vec2, screen_size: Vec2) -> Vec2 {
    let view_size = virtual_view_size(screen_size);
    let view_origin = (vec2(480.0, 270.0) - view_size) * 0.5;
    mouse / screen_size * view_size + view_origin
}

pub fn virtual_view_size(screen_size: Vec2) -> Vec2 {
    let aspect = screen_size.x / screen_size.y;
    if aspect >= 480.0 / 270.0 {
        vec2(270.0 * aspect, 270.0)
    } else {
        vec2(480.0, 480.0 / aspect)
    }
}

pub fn touch_layout(screen_size: Vec2) -> TouchLayout {
    let view_size = virtual_view_size(screen_size);
    let view_origin = (vec2(480.0, 270.0) - view_size) * 0.5;
    let button_top = view_origin.y + KNIFE_BUTTON.y;
    let button_bottom = view_origin.y + view_size.y - (270.0 - STRIKE_BUTTON.y);
    let button_scale = (button_bottom - button_top) / (STRIKE_BUTTON.y - KNIFE_BUTTON.y);
    let button_x = view_origin.x + view_size.x - (480.0 - KNIFE_BUTTON.x);

    TouchLayout {
        move_stick: vec2(
            view_origin.x + STICK_CENTER.x,
            view_origin.y + view_size.y - (270.0 - STICK_CENTER.y),
        ),
        aim_stick: vec2(
            view_origin.x + view_size.x - (480.0 - AIM_STICK_CENTER.x),
            view_origin.y + view_size.y - (270.0 - AIM_STICK_CENTER.y),
        ),
        knife: vec2(button_x, button_top),
        dash: vec2(
            button_x,
            button_top + (DASH_BUTTON.y - KNIFE_BUTTON.y) * button_scale,
        ),
        timestop: vec2(
            button_x,
            button_top + (TIMESTOP_BUTTON.y - KNIFE_BUTTON.y) * button_scale,
        ),
        strike: vec2(button_x, button_bottom),
    }
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
    fn mouse_mapping_accounts_for_tall_window_view() {
        let mapped = screen_to_virtual(vec2(500.0, 118.75), vec2(1000.0, 800.0));
        assert!((mapped - vec2(240.0, 0.0)).length() < 0.01);
    }

    #[test]
    fn widescreen_view_keeps_controls_near_the_screen_edges() {
        let screen = vec2(2400.0, 1080.0);
        assert_eq!(virtual_view_size(screen), vec2(600.0, 270.0));
        assert_eq!(screen_to_virtual(screen / 2.0, screen), vec2(240.0, 135.0));

        let controls = touch_layout(screen);
        assert!((controls.knife.x - 495.0).abs() < 0.01);
        assert!((controls.move_stick.x + 2.0).abs() < 0.01);
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

    #[test]
    fn aim_stick_deadzone_preserves_the_last_aim_direction() {
        let last_direction = vec2(0.0, -1.0);
        assert_eq!(
            touch_aim_direction(Vec2::new(2.0, 0.0), last_direction),
            last_direction
        );
    }
}
