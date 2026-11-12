use crate::combat::Knife;
use crate::input::{
    ACTION_BUTTON_RADIUS, AIM_STICK_CENTER, AIM_STICK_RADIUS, DASH_BUTTON, KNIFE_BUTTON,
    STICK_CENTER, STICK_RADIUS, TIMESTOP_BUTTON,
};
use macroquad::prelude::*;

use crate::enemies::{Enemy, EnemyKind, EnemyProjectile};
use crate::player::Player;

const VIRTUAL_WIDTH: f32 = 480.0;
const VIRTUAL_HEIGHT: f32 = 270.0;

#[derive(Clone, Debug)]
pub struct ArenaRenderer;

impl ArenaRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn draw_frame(
        &self,
        player: &Player,
        enemies: &[Enemy],
        enemy_projectiles: &[EnemyProjectile],
        knives: &[Knife],
        aim_position: Vec2,
        touch_controls_visible: bool,
        move_stick_position: Vec2,
        move_stick_active: bool,
        aim_stick_position: Vec2,
        aim_stick_active: bool,
        wave: u32,
        time_charge: f32,
        time_stop_active: bool,
        time_stop_remaining: f32,
    ) {
        set_default_camera();
        clear_background(Color::new(0.015, 0.012, 0.035, 1.0));

        let scale = (screen_width() / VIRTUAL_WIDTH).min(screen_height() / VIRTUAL_HEIGHT);
        let viewport_width = (VIRTUAL_WIDTH * scale).round() as i32;
        let viewport_height = (VIRTUAL_HEIGHT * scale).round() as i32;
        let camera = Camera2D {
            target: vec2(VIRTUAL_WIDTH / 2.0, VIRTUAL_HEIGHT / 2.0),
            zoom: vec2(2.0 / VIRTUAL_WIDTH, 2.0 / VIRTUAL_HEIGHT),
            viewport: Some((
                (screen_width() as i32 - viewport_width) / 2,
                (screen_height() as i32 - viewport_height) / 2,
                viewport_width,
                viewport_height,
            )),
            ..Default::default()
        };
        set_camera(&camera);

        self.draw_arena();
        for enemy in enemies {
            self.draw_enemy(enemy);
        }
        for projectile in enemy_projectiles {
            let color = match projectile.kind {
                EnemyKind::Sniper => Color::new(0.55, 0.42, 1.0, 1.0),
                _ => Color::new(1.0, 0.2, 0.22, 1.0),
            };
            draw_circle(
                projectile.position.x,
                projectile.position.y,
                projectile.radius + 2.0,
                Color::new(color.r, color.g, color.b, 0.3),
            );
            draw_circle(
                projectile.position.x,
                projectile.position.y,
                projectile.radius,
                color,
            );
        }
        for knife in knives {
            draw_line(
                knife.position.x,
                knife.position.y,
                knife.position.x - knife.velocity.x.signum() * 5.0,
                knife.position.y - knife.velocity.y.signum() * 5.0,
                2.0,
                Color::new(0.75, 0.9, 1.0, 1.0),
            );
        }
        self.draw_player(player);
        self.draw_aim_cursor(aim_position);
        self.draw_hud(
            player,
            enemies.len(),
            wave,
            time_charge,
            time_stop_active,
            time_stop_remaining,
        );
        if touch_controls_visible {
            self.draw_touch_controls(
                move_stick_position,
                move_stick_active,
                aim_stick_position,
                aim_stick_active,
            );
        }

        set_default_camera();
    }

    fn draw_aim_cursor(&self, position: Vec2) {
        if !(0.0..VIRTUAL_WIDTH).contains(&position.x)
            || !(0.0..VIRTUAL_HEIGHT).contains(&position.y)
        {
            return;
        }

        let color = Color::new(1.0, 0.35, 0.65, 1.0);
        draw_line(
            position.x - 4.0,
            position.y,
            position.x - 1.5,
            position.y,
            1.0,
            color,
        );
        draw_line(
            position.x + 1.5,
            position.y,
            position.x + 4.0,
            position.y,
            1.0,
            color,
        );
        draw_line(
            position.x,
            position.y - 4.0,
            position.x,
            position.y - 1.5,
            1.0,
            color,
        );
        draw_line(
            position.x,
            position.y + 1.5,
            position.x,
            position.y + 4.0,
            1.0,
            color,
        );
    }

    fn draw_touch_controls(
        &self,
        move_stick_position: Vec2,
        move_stick_active: bool,
        aim_stick_position: Vec2,
        aim_stick_active: bool,
    ) {
        let glass = Color::new(0.06, 0.1, 0.18, 0.52);
        let outline = Color::new(0.35, 0.78, 0.92, 0.8);
        draw_circle(STICK_CENTER.x, STICK_CENTER.y, STICK_RADIUS, glass);
        draw_circle_lines(STICK_CENTER.x, STICK_CENTER.y, STICK_RADIUS, 2.0, outline);
        let move_knob = if move_stick_active {
            move_stick_position
        } else {
            STICK_CENTER
        };
        draw_circle(
            move_knob.x,
            move_knob.y,
            13.0,
            Color::new(0.12, 0.82, 0.9, if move_stick_active { 0.72 } else { 0.38 }),
        );

        let aim_knob = if aim_stick_active {
            aim_stick_position
        } else {
            AIM_STICK_CENTER
        };
        draw_circle(
            AIM_STICK_CENTER.x,
            AIM_STICK_CENTER.y,
            AIM_STICK_RADIUS,
            Color::new(0.16, 0.055, 0.16, 0.48),
        );
        draw_circle_lines(
            AIM_STICK_CENTER.x,
            AIM_STICK_CENTER.y,
            AIM_STICK_RADIUS,
            2.0,
            Color::new(0.95, 0.3, 0.65, 0.85),
        );
        draw_circle(
            aim_knob.x,
            aim_knob.y,
            13.0,
            Color::new(0.92, 0.2, 0.58, if aim_stick_active { 0.78 } else { 0.4 }),
        );

        self.draw_touch_button(KNIFE_BUTTON, "KNIFE", Color::new(0.68, 0.25, 0.55, 0.82));
        self.draw_touch_button(DASH_BUTTON, "DASH", Color::new(0.1, 0.55, 0.78, 0.82));
        self.draw_touch_button(TIMESTOP_BUTTON, "TIME", Color::new(0.35, 0.3, 0.78, 0.88));
    }

    fn draw_touch_button(&self, center: Vec2, label: &str, fill: Color) {
        draw_circle(center.x, center.y, ACTION_BUTTON_RADIUS, fill);
        draw_circle_lines(
            center.x,
            center.y,
            ACTION_BUTTON_RADIUS,
            2.0,
            Color::new(0.78, 0.84, 1.0, 0.9),
        );
        let text_width = measure_text(label, None, 7, 1.0).width;
        draw_text(
            label,
            center.x - text_width / 2.0,
            center.y + 2.5,
            7.0,
            WHITE,
        );
    }

    fn draw_arena(&self) {
        draw_rectangle(12.0, 12.0, 456.0, 246.0, Color::new(0.08, 0.055, 0.12, 1.0));
        draw_rectangle(20.0, 28.0, 440.0, 222.0, Color::new(0.12, 0.09, 0.16, 1.0));

        for y in (30..248).step_by(16) {
            for x in (22..458).step_by(16) {
                let alternate = ((x / 16) + (y / 16)) % 2 == 0;
                let color = if alternate {
                    Color::new(0.14, 0.105, 0.17, 1.0)
                } else {
                    Color::new(0.105, 0.08, 0.145, 1.0)
                };
                draw_rectangle(x as f32, y as f32, 15.0, 15.0, color);
            }
        }

        draw_rectangle(20.0, 20.0, 440.0, 5.0, Color::new(0.95, 0.15, 0.42, 1.0));
        draw_rectangle(20.0, 253.0, 440.0, 5.0, Color::new(0.95, 0.15, 0.42, 1.0));
        draw_rectangle(15.0, 25.0, 5.0, 228.0, Color::new(0.95, 0.15, 0.42, 1.0));
        draw_rectangle(460.0, 25.0, 5.0, 228.0, Color::new(0.95, 0.15, 0.42, 1.0));

        for x in [46.0, 434.0] {
            draw_rectangle(x - 6.0, 32.0, 12.0, 3.0, Color::new(0.05, 0.02, 0.08, 1.0));
            draw_rectangle(x - 4.0, 35.0, 8.0, 2.0, Color::new(0.2, 0.85, 1.0, 1.0));
            draw_rectangle(x - 6.0, 239.0, 12.0, 3.0, Color::new(0.05, 0.02, 0.08, 1.0));
            draw_rectangle(x - 4.0, 237.0, 8.0, 2.0, Color::new(0.95, 0.15, 0.42, 1.0));
        }

        draw_rectangle_lines(
            12.0,
            12.0,
            456.0,
            246.0,
            2.0,
            Color::new(0.3, 0.18, 0.38, 1.0),
        );
    }

    fn draw_player(&self, player: &Player) {
        let p = player.position;
        let facing = if player.attack_direction.length_squared() > 0.0 {
            player.attack_direction
        } else {
            Vec2::X
        };
        let hand = p + facing * 8.0;

        if player.attack_flash > 0.0 {
            draw_line(
                p.x,
                p.y - 2.0,
                p.x + facing.x * 23.0,
                p.y + facing.y * 23.0 - 2.0,
                3.0,
                Color::new(0.1, 0.9, 1.0, 1.0),
            );
            draw_rectangle(hand.x - 2.0, hand.y - 2.0, 5.0, 5.0, WHITE);
        }

        draw_rectangle(
            p.x - 5.0,
            p.y + 3.0,
            4.0,
            5.0,
            Color::new(0.05, 0.08, 0.16, 1.0),
        );
        draw_rectangle(
            p.x + 1.0,
            p.y + 3.0,
            4.0,
            5.0,
            Color::new(0.05, 0.08, 0.16, 1.0),
        );
        draw_rectangle(
            p.x - 7.0,
            p.y - 3.0,
            14.0,
            10.0,
            Color::new(0.05, 0.72, 0.78, 1.0),
        );
        draw_rectangle(
            p.x - 5.0,
            p.y - 10.0,
            10.0,
            8.0,
            Color::new(0.9, 0.64, 0.45, 1.0),
        );
        draw_rectangle(
            p.x - 6.0,
            p.y - 12.0,
            12.0,
            4.0,
            Color::new(0.08, 0.04, 0.1, 1.0),
        );
        draw_rectangle(
            p.x + facing.x * 5.0 - 2.0,
            p.y + facing.y * 5.0 - 1.0,
            5.0,
            5.0,
            WHITE,
        );
        draw_rectangle(
            p.x - 1.0,
            p.y - 8.0,
            2.0,
            2.0,
            Color::new(0.08, 0.04, 0.1, 1.0),
        );

        if player.dash_remaining > 0.0 {
            draw_rectangle(
                p.x - 10.0,
                p.y + 7.0,
                20.0,
                2.0,
                Color::new(0.1, 0.85, 1.0, 0.65),
            );
        }
    }

    fn draw_enemy(&self, enemy: &Enemy) {
        let p = enemy.position;
        let (coat, accent) = match enemy.kind {
            EnemyKind::Shooter => (
                Color::new(0.78, 0.14, 0.34, 1.0),
                Color::new(1.0, 0.55, 0.2, 1.0),
            ),
            EnemyKind::Rusher => (
                Color::new(0.92, 0.28, 0.12, 1.0),
                Color::new(1.0, 0.8, 0.2, 1.0),
            ),
            EnemyKind::Brute => (
                Color::new(0.42, 0.24, 0.6, 1.0),
                Color::new(0.25, 0.85, 1.0, 1.0),
            ),
            EnemyKind::Sniper => (
                Color::new(0.22, 0.38, 0.8, 1.0),
                Color::new(1.0, 0.25, 0.62, 1.0),
            ),
        };

        let size = enemy.radius;
        draw_rectangle(
            p.x - size,
            p.y + size * 0.35,
            size * 0.65,
            size * 0.65,
            Color::new(0.04, 0.025, 0.06, 1.0),
        );
        draw_rectangle(
            p.x + size * 0.35,
            p.y + size * 0.35,
            size * 0.65,
            size * 0.65,
            Color::new(0.04, 0.025, 0.06, 1.0),
        );
        draw_rectangle(p.x - size, p.y - size * 0.45, size * 2.0, size * 1.25, coat);
        draw_rectangle(
            p.x - size * 0.7,
            p.y - size * 1.3,
            size * 1.4,
            size * 0.8,
            accent,
        );
        draw_rectangle(
            p.x - size * 0.85,
            p.y - size * 0.1,
            size * 1.7,
            size * 0.3,
            Color::new(0.05, 0.025, 0.06, 1.0),
        );

        if enemy.health < enemy.max_health {
            let width = size * 2.0;
            draw_rectangle(
                p.x - size,
                p.y - size * 1.8,
                width,
                2.0,
                Color::new(0.08, 0.025, 0.06, 1.0),
            );
            draw_rectangle(
                p.x - size,
                p.y - size * 1.8,
                width * (enemy.health / enemy.max_health).clamp(0.0, 1.0),
                2.0,
                Color::new(1.0, 0.2, 0.32, 1.0),
            );
        }
    }

    fn draw_hud(
        &self,
        player: &Player,
        enemy_count: usize,
        wave: u32,
        time_charge: f32,
        time_stop_active: bool,
        time_stop_remaining: f32,
    ) {
        draw_rectangle(20.0, 4.0, 98.0, 11.0, Color::new(0.025, 0.02, 0.05, 1.0));
        draw_rectangle(
            21.0,
            5.0,
            96.0 * (player.health / player.max_health).clamp(0.0, 1.0),
            9.0,
            Color::new(0.94, 0.12, 0.28, 1.0),
        );
        draw_text("HP", 22.0, 13.0, 8.0, WHITE);

        draw_rectangle(125.0, 4.0, 98.0, 11.0, Color::new(0.025, 0.02, 0.05, 1.0));
        draw_rectangle(
            126.0,
            5.0,
            96.0 * (time_charge / 100.0).clamp(0.0, 1.0),
            9.0,
            Color::new(0.08, 0.78, 0.92, 1.0),
        );
        draw_text("TIME", 128.0, 13.0, 8.0, WHITE);

        draw_text(
            if time_stop_active {
                "TIME STOP"
            } else {
                "WORLD TIME"
            },
            335.0,
            13.0,
            11.0,
            if time_stop_active {
                Color::new(0.15, 0.9, 1.0, 1.0)
            } else {
                Color::new(1.0, 0.3, 0.55, 1.0)
            },
        );
        if time_stop_active {
            draw_text(
                &format!("{:.1}", time_stop_remaining.max(0.0)),
                440.0,
                13.0,
                10.0,
                WHITE,
            );
        }
        draw_text(
            &format!("WAVE {wave}  HOSTILES {enemy_count}"),
            20.0,
            265.0,
            10.0,
            WHITE,
        );
        draw_text(
            "WASD MOVE   LMB STRIKE   RMB KNIFE   SHIFT DASH   SPACE TIME",
            166.0,
            265.0,
            8.0,
            Color::new(0.7, 0.68, 0.78, 1.0),
        );
    }
}
