use crate::combat::{CombatEffects, Knife, RoadRoller};
use crate::input::{
    touch_layout, virtual_view_size, TouchLayout, ACTION_BUTTON_RADIUS, AIM_STICK_RADIUS,
    STICK_RADIUS,
};
use macroquad::prelude::*;

use crate::enemies::{Enemy, EnemyKind, EnemyProjectile};
use crate::player::Player;
use crate::room::{ExitSide, Room, SpikeTrap};

const VIRTUAL_WIDTH: f32 = 480.0;
const VIRTUAL_HEIGHT: f32 = 270.0;

#[derive(Clone, Debug)]
pub struct ArenaRenderer;

impl ArenaRenderer {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_frame(
        &self,
        player: &Player,
        room: &Room,
        enemies: &[Enemy],
        enemy_projectiles: &[EnemyProjectile],
        knives: &[Knife],
        road_roller: &Option<RoadRoller>,
        effects: &CombatEffects,
        aim_position: Vec2,
        touch_controls_visible: bool,
        move_stick_position: Vec2,
        move_stick_active: bool,
        aim_stick_position: Vec2,
        aim_stick_active: bool,
        room_number: u32,
        time_charge: f32,
        time_stop_active: bool,
        time_stop_remaining: f32,
        time_stop_cooldown_remaining: f32,
        knife_cooldown_remaining: f32,
        road_roller_cooldown_remaining: f32,
        road_roller_unlocked: bool,
        left_handed: bool,
    ) {
        set_default_camera();
        clear_background(Color::new(0.015, 0.012, 0.035, 1.0));

        let screen_size = vec2(screen_width(), screen_height());
        let view_size = virtual_view_size(screen_size);
        let camera = Camera2D {
            target: vec2(VIRTUAL_WIDTH / 2.0, VIRTUAL_HEIGHT / 2.0) + effects.camera_offset(),
            zoom: vec2(2.0 / view_size.x, 2.0 / view_size.y),
            ..Default::default()
        };
        set_camera(&camera);

        self.draw_background(view_size);
        self.draw_arena(room.number);
        self.draw_impact_marks(effects);
        self.draw_room(room);
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
        if let Some(roller) = road_roller {
            self.draw_road_roller(roller);
        }
        self.draw_player(player, time_stop_active);
        self.draw_combat_particles(effects);

        let ui_camera = Camera2D {
            target: vec2(VIRTUAL_WIDTH / 2.0, VIRTUAL_HEIGHT / 2.0),
            zoom: vec2(2.0 / view_size.x, 2.0 / view_size.y),
            ..Default::default()
        };
        set_camera(&ui_camera);
        self.draw_aim_cursor(aim_position);
        self.draw_hud(
            player,
            enemies.len(),
            room_number,
            time_charge,
            time_stop_active,
            time_stop_remaining,
            player.dash_cooldown_remaining,
            knife_cooldown_remaining,
            time_stop_cooldown_remaining,
            road_roller_cooldown_remaining,
            road_roller_unlocked,
        );
        if touch_controls_visible {
            let layout = touch_layout(screen_size, left_handed);
            self.draw_touch_controls(
                layout,
                move_stick_position,
                move_stick_active,
                aim_stick_position,
                aim_stick_active,
                player.dash_cooldown_remaining,
                knife_cooldown_remaining,
                time_stop_cooldown_remaining,
                road_roller_cooldown_remaining,
                road_roller_unlocked,
                player.block_held,
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

    #[allow(clippy::too_many_arguments)]
    fn draw_touch_controls(
        &self,
        layout: TouchLayout,
        move_stick_position: Vec2,
        move_stick_active: bool,
        aim_stick_position: Vec2,
        aim_stick_active: bool,
        dash_cooldown_remaining: f32,
        knife_cooldown_remaining: f32,
        time_stop_cooldown_remaining: f32,
        road_roller_cooldown_remaining: f32,
        road_roller_unlocked: bool,
        block_held: bool,
    ) {
        let glass = Color::new(0.06, 0.1, 0.18, 0.52);
        let outline = Color::new(0.35, 0.78, 0.92, 0.8);
        draw_circle(
            layout.move_stick.x,
            layout.move_stick.y,
            STICK_RADIUS,
            glass,
        );
        draw_circle_lines(
            layout.move_stick.x,
            layout.move_stick.y,
            STICK_RADIUS,
            2.0,
            outline,
        );
        let move_knob = if move_stick_active {
            move_stick_position
        } else {
            layout.move_stick
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
            layout.aim_stick
        };
        draw_circle(
            layout.aim_stick.x,
            layout.aim_stick.y,
            AIM_STICK_RADIUS,
            Color::new(0.16, 0.055, 0.16, 0.48),
        );
        draw_circle_lines(
            layout.aim_stick.x,
            layout.aim_stick.y,
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

        self.draw_touch_button(
            layout.knife,
            "KNIFE",
            Color::new(0.68, 0.25, 0.55, 0.82),
            knife_cooldown_remaining,
        );
        self.draw_touch_button(
            layout.dash,
            "DASH",
            Color::new(0.1, 0.55, 0.78, 0.82),
            dash_cooldown_remaining,
        );
        self.draw_touch_button(
            layout.timestop,
            "TIME",
            Color::new(0.35, 0.3, 0.78, 0.88),
            time_stop_cooldown_remaining,
        );
        self.draw_touch_button(
            layout.strike,
            "STRIKE",
            Color::new(0.85, 0.2, 0.28, 0.88),
            0.0,
        );
        self.draw_touch_button(
            layout.ult,
            if road_roller_unlocked {
                "ROLLER"
            } else {
                "LOCK"
            },
            Color::new(
                0.9,
                0.48,
                0.12,
                if road_roller_unlocked { 0.9 } else { 0.45 },
            ),
            if road_roller_unlocked {
                road_roller_cooldown_remaining
            } else {
                0.0
            },
        );
        self.draw_touch_button(
            layout.block,
            "BLOCK",
            Color::new(
                if block_held { 0.16 } else { 0.12 },
                if block_held { 0.85 } else { 0.52 },
                0.88,
                0.82,
            ),
            0.0,
        );
        self.draw_touch_button(layout.pause, "II", Color::new(0.2, 0.25, 0.38, 0.8), 0.0);
    }

    pub fn draw_menu_overlay(&self, title: &str, entries: &[String], selected: usize) {
        set_default_camera();
        let screen_size = vec2(screen_width(), screen_height());
        let view_size = virtual_view_size(screen_size);
        let camera = Camera2D {
            target: vec2(VIRTUAL_WIDTH / 2.0, VIRTUAL_HEIGHT / 2.0),
            zoom: vec2(2.0 / view_size.x, 2.0 / view_size.y),
            ..Default::default()
        };
        set_camera(&camera);

        let origin = (vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT) - view_size) * 0.5;
        let compact = entries.len() >= 4;
        let tall = entries.len() >= 5;
        let panel_y = if tall {
            38.0
        } else if compact {
            50.0
        } else {
            68.0
        };
        let panel_height = if tall {
            192.0
        } else if compact {
            160.0
        } else {
            134.0
        };
        let title_y = if tall {
            68.0
        } else if compact {
            80.0
        } else {
            96.0
        };
        draw_rectangle(
            origin.x,
            origin.y,
            view_size.x,
            view_size.y,
            Color::new(0.005, 0.005, 0.015, 0.78),
        );
        draw_rectangle(
            135.0,
            panel_y,
            210.0,
            panel_height,
            Color::new(0.045, 0.025, 0.08, 0.98),
        );
        draw_rectangle_lines(
            135.0,
            panel_y,
            210.0,
            panel_height,
            2.0,
            Color::new(0.92, 0.16, 0.46, 0.9),
        );

        let title_width = measure_text(title, None, 18, 1.0).width;
        draw_text(
            title,
            240.0 - title_width / 2.0,
            title_y,
            18.0,
            Color::new(0.15, 0.88, 1.0, 1.0),
        );

        for (index, entry) in entries.iter().enumerate() {
            let y = Self::menu_entry_y(index, entries.len());
            let active = index == selected;
            if active {
                draw_rectangle(
                    153.0,
                    y - if tall {
                        9.0
                    } else if compact {
                        12.0
                    } else {
                        16.0
                    },
                    174.0,
                    if tall {
                        18.0
                    } else if compact {
                        21.0
                    } else {
                        25.0
                    },
                    Color::new(0.35, 0.08, 0.2, 0.9),
                );
            }

            let font_size = if tall { 10 } else { 11 };
            let text_width = measure_text(entry, None, font_size, 1.0).width;
            draw_text(
                entry,
                240.0 - text_width / 2.0,
                y,
                font_size as f32,
                if active {
                    WHITE
                } else {
                    Color::new(0.7, 0.68, 0.8, 1.0)
                },
            );
        }

        set_default_camera();
    }

    pub fn menu_entry_y(index: usize, count: usize) -> f32 {
        if count >= 5 {
            96.0 + index as f32 * 24.0
        } else if count >= 4 {
            109.0 + index as f32 * 26.0
        } else {
            124.0 + index as f32 * 34.0
        }
    }

    fn draw_touch_button(&self, center: Vec2, label: &str, fill: Color, cooldown: f32) {
        let label_text = if cooldown > 0.0 {
            format!("{cooldown:.1}")
        } else {
            label.to_owned()
        };
        let fill = if cooldown > 0.0 {
            Color::new(fill.r * 0.4, fill.g * 0.4, fill.b * 0.4, fill.a)
        } else {
            fill
        };
        draw_circle(center.x, center.y, ACTION_BUTTON_RADIUS, fill);
        draw_circle_lines(
            center.x,
            center.y,
            ACTION_BUTTON_RADIUS,
            2.0,
            Color::new(0.78, 0.84, 1.0, 0.9),
        );
        let text_width = measure_text(&label_text, None, 7, 1.0).width;
        draw_text(
            &label_text,
            center.x - text_width / 2.0,
            center.y + 2.5,
            7.0,
            WHITE,
        );
    }

    fn draw_background(&self, view_size: Vec2) {
        let origin = (vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT) - view_size) * 0.5;
        draw_rectangle(
            origin.x,
            origin.y,
            view_size.x,
            view_size.y,
            Color::new(0.025, 0.018, 0.045, 1.0),
        );

        let grid = Color::new(0.08, 0.045, 0.11, 0.55);
        let mut x = (origin.x / 24.0).floor() * 24.0;
        while x <= origin.x + view_size.x {
            draw_line(x, origin.y, x, origin.y + view_size.y, 0.5, grid);
            x += 24.0;
        }
        let mut y = (origin.y / 24.0).floor() * 24.0;
        while y <= origin.y + view_size.y {
            draw_line(origin.x, y, origin.x + view_size.x, y, 0.5, grid);
            y += 24.0;
        }
    }

    fn draw_arena(&self, room_number: u32) {
        let (border, floor_a, floor_b) = match room_number % 3 {
            1 => (
                Color::new(0.08, 0.055, 0.12, 1.0),
                Color::new(0.14, 0.105, 0.17, 1.0),
                Color::new(0.105, 0.08, 0.145, 1.0),
            ),
            2 => (
                Color::new(0.025, 0.085, 0.11, 1.0),
                Color::new(0.08, 0.15, 0.17, 1.0),
                Color::new(0.055, 0.115, 0.14, 1.0),
            ),
            _ => (
                Color::new(0.12, 0.055, 0.035, 1.0),
                Color::new(0.18, 0.11, 0.075, 1.0),
                Color::new(0.14, 0.08, 0.055, 1.0),
            ),
        };
        draw_rectangle(12.0, 12.0, 456.0, 246.0, border);
        draw_rectangle(20.0, 28.0, 440.0, 222.0, floor_b);

        for y in (30..248).step_by(16) {
            for x in (22..458).step_by(16) {
                let alternate = ((x / 16) + (y / 16)) % 2 == 0;
                let color = if alternate { floor_a } else { floor_b };
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

    fn draw_impact_marks(&self, effects: &CombatEffects) {
        for mark in &effects.marks {
            draw_circle(mark.position.x, mark.position.y, mark.radius, mark.color);
            draw_circle(
                mark.position.x + mark.radius * 0.6,
                mark.position.y - mark.radius * 0.35,
                mark.radius * 0.35,
                Color::new(
                    mark.color.r * 0.7,
                    mark.color.g * 0.7,
                    mark.color.b * 0.7,
                    0.6,
                ),
            );
        }
    }

    fn draw_combat_particles(&self, effects: &CombatEffects) {
        for particle in &effects.particles {
            let alpha = (particle.remaining / particle.lifetime).clamp(0.0, 1.0);
            draw_circle(
                particle.position.x,
                particle.position.y,
                particle.radius * alpha,
                Color::new(
                    particle.color.r,
                    particle.color.g,
                    particle.color.b,
                    particle.color.a * alpha,
                ),
            );
        }
    }

    fn draw_road_roller(&self, roller: &RoadRoller) {
        let p = roller.position;
        if !roller.impacted {
            draw_circle(
                p.x,
                p.y,
                40.0,
                Color::new(1.0, 0.48, 0.12, 0.12 + roller.windup * 0.1),
            );
            draw_circle_lines(
                p.x,
                p.y,
                34.0 + roller.windup.sin() * 2.0,
                2.0,
                Color::new(1.0, 0.62, 0.18, 0.9),
            );
            draw_line(p.x, p.y - 72.0, p.x, p.y - 28.0, 2.0, ORANGE);
            draw_rectangle(
                p.x - 12.0,
                p.y - 64.0,
                24.0,
                18.0,
                Color::new(0.32, 0.3, 0.34, 1.0),
            );
            draw_rectangle(
                p.x - 15.0,
                p.y - 51.0,
                30.0,
                10.0,
                Color::new(0.72, 0.35, 0.08, 1.0),
            );
        } else {
            let fade = (roller.impact_remaining / 0.32).clamp(0.0, 1.0);
            draw_circle(
                p.x,
                p.y,
                50.0 * (1.0 - fade * 0.25),
                Color::new(1.0, 0.48, 0.12, fade * 0.32),
            );
            draw_circle_lines(
                p.x,
                p.y,
                50.0 * (1.0 - fade * 0.25),
                3.0,
                Color::new(1.0, 0.8, 0.36, fade),
            );
        }
    }

    fn draw_room(&self, room: &Room) {
        let doors = [
            (ExitSide::Top, Rect::new(228.0, 12.0, 24.0, 13.0)),
            (ExitSide::Right, Rect::new(455.0, 123.0, 13.0, 24.0)),
            (ExitSide::Bottom, Rect::new(228.0, 245.0, 24.0, 13.0)),
            (ExitSide::Left, Rect::new(12.0, 123.0, 13.0, 24.0)),
        ];
        for (side, rect) in doors {
            let is_exit = side == room.exit;
            let open = is_exit && room.exit_open;
            let color = if open {
                Color::new(0.08, 0.9, 0.88, 1.0)
            } else if is_exit {
                Color::new(0.92, 0.4, 0.16, 1.0)
            } else {
                Color::new(0.17, 0.12, 0.2, 1.0)
            };
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
            draw_rectangle_lines(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                1.5,
                Color::new(0.06, 0.025, 0.08, 1.0),
            );
            if open {
                let center = side.exit_position();
                draw_circle(center.x, center.y, 3.0, WHITE);
            }
        }

        for trap in &room.traps {
            self.draw_spike_trap(trap);
        }
    }

    fn draw_spike_trap(&self, trap: &SpikeTrap) {
        let position = trap.position;
        let warning = trap.is_warning();
        let active = trap.is_active();
        let height = trap.height();
        let glow = if active {
            Color::new(1.0, 0.12, 0.28, 0.7)
        } else if warning {
            Color::new(1.0, 0.58, 0.12, 0.62)
        } else {
            Color::new(0.2, 0.14, 0.24, 0.45)
        };
        draw_circle(
            position.x,
            position.y,
            16.0,
            Color::new(0.04, 0.025, 0.07, 0.8),
        );
        draw_circle_lines(position.x, position.y, 16.0, 1.5, glow);

        if height <= 0.0 {
            return;
        }

        let spike_color = if active {
            Color::new(1.0, 0.34, 0.4, 1.0)
        } else {
            Color::new(1.0, 0.72, 0.25, 0.95)
        };
        for index in 0..8 {
            let angle = index as f32 * std::f32::consts::TAU / 8.0;
            let direction = vec2(angle.cos(), angle.sin());
            let side = vec2(-direction.y, direction.x) * 2.7;
            let base = position + direction * 5.0;
            let tip = position + direction * (5.0 + height * 11.0);
            draw_triangle(base + side, base - side, tip, spike_color);
        }
    }

    fn draw_player(&self, player: &Player, time_stop_active: bool) {
        let p = player.position;
        let facing = if player.attack_direction.length_squared() > 0.0 {
            player.attack_direction
        } else {
            Vec2::X
        };
        let hand = p + facing * 8.0;
        let stand = p - facing * 15.0;
        draw_circle(p.x, p.y + 7.0, 8.0, Color::new(0.015, 0.01, 0.025, 0.58));
        self.draw_stand(stand, facing, player.attack_flash > 0.0);
        if time_stop_active {
            draw_circle_lines(p.x, p.y, 16.0, 1.5, Color::new(0.08, 0.88, 1.0, 0.6));
        }

        if player.block_held {
            let forward = p + facing * 14.0;
            let side = vec2(-facing.y, facing.x) * 11.0;
            draw_line(
                (forward - side).x,
                (forward - side).y,
                (forward + side).x,
                (forward + side).y,
                4.0,
                Color::new(0.12, 0.78, 1.0, 0.88),
            );
            draw_line(
                (forward - side).x,
                (forward - side).y,
                (forward + side).x,
                (forward + side).y,
                1.0,
                WHITE,
            );
        }

        if player.attack_flash > 0.0 {
            let side = vec2(-facing.y, facing.x) * 12.0;
            let start = p + facing * 8.0;
            let end = p + facing * 37.0;
            draw_line(
                (start - side).x,
                (start - side).y,
                end.x,
                end.y,
                5.0,
                Color::new(0.1, 0.9, 1.0, 0.22),
            );
            draw_line(
                (start - side * 0.7).x,
                (start - side * 0.7).y,
                end.x,
                end.y,
                2.5,
                Color::new(0.1, 0.9, 1.0, 0.95),
            );
            draw_line((start + side).x, (start + side).y, end.x, end.y, 1.0, WHITE);
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

    fn draw_stand(&self, position: Vec2, facing: Vec2, attacking: bool) {
        let size = if attacking { 1.0 } else { 0.9 };
        let side = vec2(-facing.y, facing.x);
        let shoulder = position + facing * if attacking { 8.0 } else { 3.0 };
        let fist = shoulder + facing * if attacking { 16.0 } else { 7.0 };
        let armor = Color::new(0.49, 0.2, 0.78, 0.94);
        let highlight = Color::new(0.24, 0.89, 1.0, 1.0);

        draw_circle(
            position.x,
            position.y + 7.0,
            7.5,
            Color::new(0.03, 0.015, 0.06, 0.52),
        );
        draw_rectangle(
            position.x - 5.0 * size,
            position.y - 3.0,
            10.0 * size,
            10.0,
            armor,
        );
        draw_rectangle(
            position.x - 4.0,
            position.y - 11.0,
            8.0,
            8.0,
            Color::new(0.69, 0.45, 0.94, 1.0),
        );
        draw_rectangle(position.x - 5.0, position.y - 12.0, 10.0, 3.0, highlight);
        draw_rectangle(
            shoulder.x + side.x * 3.0 - 2.0,
            shoulder.y + side.y * 3.0 - 2.0,
            4.0,
            7.0,
            highlight,
        );
        draw_rectangle(fist.x - 2.5, fist.y - 2.5, 5.0, 5.0, WHITE);
        if attacking {
            draw_line(
                shoulder.x,
                shoulder.y,
                fist.x,
                fist.y,
                3.0,
                Color::new(0.23, 0.88, 1.0, 0.85),
            );
        }
    }

    fn cooldown_label(remaining: f32) -> String {
        if remaining > 0.0 {
            format!("{remaining:.1}")
        } else {
            "READY".to_owned()
        }
    }

    fn draw_enemy(&self, enemy: &Enemy) {
        let p = enemy.position;
        if enemy.telegraph_remaining > 0.0 {
            let range = if enemy.kind == EnemyKind::Sniper {
                270.0
            } else {
                190.0
            };
            let intensity = if enemy.kind == EnemyKind::Sniper {
                0.75
            } else {
                0.5
            };
            let end = p + enemy.telegraph_direction * range;
            draw_line(
                p.x,
                p.y,
                end.x,
                end.y,
                if enemy.kind == EnemyKind::Sniper {
                    2.0
                } else {
                    1.0
                },
                Color::new(1.0, 0.12, 0.28, intensity),
            );
            draw_circle(
                end.x,
                end.y,
                if enemy.kind == EnemyKind::Sniper {
                    3.0
                } else {
                    2.0
                },
                Color::new(1.0, 0.25, 0.38, intensity),
            );
        }
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
        draw_circle(
            p.x,
            p.y + size * 0.75,
            size * 1.4,
            Color::new(0.015, 0.01, 0.025, 0.52),
        );
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
        let eye = p + enemy.facing_direction * size * 0.5;
        draw_rectangle(eye.x - 1.5, eye.y - 2.0, 3.0, 2.0, WHITE);
        if enemy.kind == EnemyKind::Brute {
            let side = vec2(-enemy.facing_direction.y, enemy.facing_direction.x);
            let shield_center = p + enemy.facing_direction * size * 0.95;
            draw_line(
                (shield_center - side * 6.0).x,
                (shield_center - side * 6.0).y,
                (shield_center + side * 6.0).x,
                (shield_center + side * 6.0).y,
                3.0,
                Color::new(0.32, 0.9, 1.0, 0.95),
            );
        }

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

    #[allow(clippy::too_many_arguments)]
    fn draw_hud(
        &self,
        player: &Player,
        enemy_count: usize,
        room_number: u32,
        time_charge: f32,
        time_stop_active: bool,
        time_stop_remaining: f32,
        dash_cooldown_remaining: f32,
        knife_cooldown_remaining: f32,
        time_stop_cooldown_remaining: f32,
        road_roller_cooldown_remaining: f32,
        road_roller_unlocked: bool,
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
        draw_text(
            format!(
                "DASH {}  KNIFE {}",
                Self::cooldown_label(dash_cooldown_remaining),
                Self::cooldown_label(knife_cooldown_remaining)
            ),
            20.0,
            23.0,
            7.0,
            Color::new(0.82, 0.8, 0.92, 1.0),
        );

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
            if !road_roller_unlocked {
                "ULT LOCKED"
            } else if road_roller_cooldown_remaining > 0.0 {
                "ULT COOLING"
            } else {
                "ULT READY"
            },
            235.0,
            13.0,
            8.0,
            Color::new(1.0, 0.62, 0.2, 1.0),
        );

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
                format!("{:.1}", time_stop_remaining.max(0.0)),
                440.0,
                13.0,
                10.0,
                WHITE,
            );
        } else if time_stop_cooldown_remaining > 0.0 {
            draw_text(
                format!("{time_stop_cooldown_remaining:.1}"),
                440.0,
                13.0,
                10.0,
                WHITE,
            );
        }
        draw_text(
            format!("ROOM {room_number}  HOSTILES {enemy_count}"),
            20.0,
            265.0,
            10.0,
            WHITE,
        );
        draw_text(
            "WASD MOVE  LMB STRIKE  RMB KNIFE  SHIFT DASH  SPACE TIME  E BLOCK  Q ULT",
            166.0,
            265.0,
            8.0,
            Color::new(0.7, 0.68, 0.78, 1.0),
        );
    }
}
