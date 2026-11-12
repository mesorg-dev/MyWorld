use macroquad::prelude::*;

use crate::input::InputState;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Player {
    pub position: Vec2,
    pub radius: f32,
    pub speed: f32,
    pub health: f32,
    pub max_health: f32,
    pub time_charge: f32,
    pub max_time_charge: f32,
    pub attack_cooldown: f32,
    pub attack_fired: bool,
    pub attack_direction: Vec2,
    pub attack_flash: f32,
    pub dash_remaining: f32,
}

#[allow(dead_code)]
impl Player {
    pub fn new() -> Self {
        Self {
            position: vec2(240.0, 135.0),
            radius: 8.0,
            speed: 95.0,
            health: 100.0,
            max_health: 100.0,
            time_charge: 100.0,
            max_time_charge: 100.0,
            attack_cooldown: 0.0,
            attack_fired: false,
            attack_direction: Vec2::X,
            attack_flash: 0.0,
            dash_remaining: 0.0,
        }
    }

    pub fn update(&mut self, input: &InputState, dt: f32) {
        let direction = input.move_dir.normalize_or_zero();
        if input.aim_dir.length_squared() > 0.0 {
            self.attack_direction = input.aim_dir;
        }
        if input.dash_pressed && self.dash_remaining <= 0.0 {
            self.dash_remaining = 0.16;
        }
        let movement_direction = if self.dash_remaining > 0.0 {
            if direction.length_squared() > 0.0 {
                direction
            } else {
                input.aim_dir
            }
        } else {
            direction
        };
        let speed = if self.dash_remaining > 0.0 {
            300.0
        } else {
            self.speed
        };
        self.position += movement_direction * speed * dt;
        self.dash_remaining = (self.dash_remaining - dt).max(0.0);

        let arena_min = 18.0;
        let arena_max_x = 462.0;
        let arena_max_y = 252.0;
        self.position.x = self.position.x.clamp(arena_min, arena_max_x);
        self.position.y = self.position.y.clamp(arena_min, arena_max_y);

        self.attack_fired = input.attack_pressed && self.attack_cooldown <= 0.0;
        if self.attack_fired {
            self.attack_cooldown = 0.2;
            self.attack_direction = input.aim_dir;
            self.attack_flash = 0.12;
        }

        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }
        self.attack_flash = (self.attack_flash - dt).max(0.0);
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
    }
}
