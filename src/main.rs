mod combat;
mod enemies;
mod input;
mod player;
mod render;
mod room;
mod skills;
mod time;

use macroquad::prelude::*;

use crate::combat::Knife;
use crate::enemies::{
    spawn_room, update_enemies, update_enemy_projectiles, Enemy, EnemyProjectile,
};
use crate::input::{pointer_pressed_position, read_input, InputState};
use crate::player::Player;
use crate::render::ArenaRenderer;
use crate::room::Room;
use crate::time::{DelayedHit, FinisherKind, TimeState};

const KNIFE_COOLDOWN: f32 = 0.55;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppScreen {
    MainMenu,
    Playing,
    Paused,
    Settings,
}

struct App {
    time: TimeState,
    input: InputState,
    player: Player,
    room: Room,
    enemies: Vec<Enemy>,
    enemy_projectiles: Vec<EnemyProjectile>,
    knives: Vec<Knife>,
    knife_cooldown_remaining: f32,
    renderer: ArenaRenderer,
    fullscreen: bool,
    screen: AppScreen,
    settings_return_screen: AppScreen,
    menu_selection: usize,
    left_handed: bool,
    run_started: bool,
}

impl App {
    fn new() -> Self {
        Self {
            time: TimeState::new(),
            input: InputState::default(),
            player: Player::new(),
            room: Room::new(1),
            enemies: spawn_room(1),
            enemy_projectiles: Vec::new(),
            knives: Vec::new(),
            knife_cooldown_remaining: 0.0,
            renderer: ArenaRenderer::new(),
            fullscreen: false,
            screen: AppScreen::MainMenu,
            settings_return_screen: AppScreen::MainMenu,
            menu_selection: 0,
            left_handed: false,
            run_started: false,
        }
    }

    fn update(&mut self) {
        if is_key_pressed(KeyCode::F11) {
            self.fullscreen = !self.fullscreen;
            set_fullscreen(self.fullscreen);
        }

        if self.screen != AppScreen::Playing {
            self.update_menu();
            return;
        }

        self.input = read_input(
            self.player.position,
            self.player.attack_direction,
            self.left_handed,
        );
        if self.input.pause_pressed {
            self.screen = AppScreen::Paused;
            self.menu_selection = 0;
            return;
        }

        self.update_gameplay();
    }

    fn update_menu(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            match self.screen {
                AppScreen::Paused => self.screen = AppScreen::Playing,
                AppScreen::Settings => {
                    self.screen = self.settings_return_screen;
                    self.menu_selection = 0;
                }
                AppScreen::MainMenu | AppScreen::Playing => {}
            }
            return;
        }

        let entries = self.menu_entries();
        if entries.is_empty() {
            return;
        }

        if is_key_pressed(KeyCode::Down) {
            self.menu_selection = (self.menu_selection + 1) % entries.len();
        } else if is_key_pressed(KeyCode::Up) {
            self.menu_selection = (self.menu_selection + entries.len() - 1) % entries.len();
        }

        let clicked = pointer_pressed_position().and_then(|position| {
            if !(135.0..345.0).contains(&position.x) {
                return None;
            }
            entries.iter().enumerate().find_map(|(index, _)| {
                let center_y = ArenaRenderer::menu_entry_y(index, entries.len());
                (position.y >= center_y - 14.0 && position.y <= center_y + 12.0).then_some(index)
            })
        });
        if let Some(index) = clicked {
            self.menu_selection = index;
        }

        let activate = clicked.is_some() || is_key_pressed(KeyCode::Enter);
        if !activate {
            return;
        }

        match self.screen {
            AppScreen::MainMenu => match self.menu_selection {
                0 => {
                    self.run_started = true;
                    self.screen = AppScreen::Playing;
                }
                1 => {
                    self.settings_return_screen = AppScreen::MainMenu;
                    self.screen = AppScreen::Settings;
                    self.menu_selection = 0;
                }
                _ => {}
            },
            AppScreen::Paused => match self.menu_selection {
                0 => self.screen = AppScreen::Playing,
                1 => {
                    self.settings_return_screen = AppScreen::Paused;
                    self.screen = AppScreen::Settings;
                    self.menu_selection = 0;
                }
                2 => {
                    self.restart_run();
                    self.screen = AppScreen::Playing;
                }
                3 => {
                    self.screen = AppScreen::MainMenu;
                    self.menu_selection = 0;
                }
                _ => {}
            },
            AppScreen::Settings => match self.menu_selection {
                0 => self.left_handed = !self.left_handed,
                1 => {
                    self.screen = self.settings_return_screen;
                    self.menu_selection = 0;
                }
                _ => {}
            },
            AppScreen::Playing => {}
        }
    }

    fn menu_entries(&self) -> Vec<String> {
        match self.screen {
            AppScreen::MainMenu => vec![
                if self.run_started {
                    "CONTINUE".to_owned()
                } else {
                    "START".to_owned()
                },
                "SETTINGS".to_owned(),
            ],
            AppScreen::Paused => vec![
                "RESUME".to_owned(),
                "SETTINGS".to_owned(),
                "RESTART RUN".to_owned(),
                "MAIN MENU".to_owned(),
            ],
            AppScreen::Settings => vec![
                format!(
                    "CONTROL SIDE: {}",
                    if self.left_handed { "LEFT" } else { "RIGHT" }
                ),
                "BACK".to_owned(),
            ],
            AppScreen::Playing => Vec::new(),
        }
    }

    fn update_gameplay(&mut self) {
        let dt = get_frame_time().min(0.033);
        self.knife_cooldown_remaining = (self.knife_cooldown_remaining - dt).max(0.0);
        let time_stop_expired = self.time.update(dt);
        if time_stop_expired {
            self.release_delayed_hits();
        }

        if self.input.timestop_pressed {
            if self.time.time_stop_active {
                self.time.release_time_stop();
                self.release_delayed_hits();
            } else {
                self.time.activate_time_stop();
            }
        }

        self.player.update(&self.input, dt);

        if self.player.attack_fired {
            self.stand_attack();
        }
        if self.input.throw_pressed {
            self.throw_knife();
        }

        if self.time.world_dt > 0.0 {
            let enemy_update = update_enemies(&mut self.enemies, &self.player, self.time.world_dt);
            self.player.take_damage(enemy_update.contact_damage);
            self.enemy_projectiles.extend(enemy_update.projectiles);
            let projectile_damage = update_enemy_projectiles(
                &mut self.enemy_projectiles,
                &self.player,
                self.time.world_dt,
            );
            self.player.take_damage(projectile_damage);
            self.player
                .take_damage(self.room.update_traps(&self.player, self.time.world_dt));
            self.update_knives(self.time.world_dt);
        }

        let defeated = self
            .enemies
            .iter()
            .filter(|enemy| enemy.health <= 0.0)
            .count();
        if defeated > 0 {
            self.time.time_charge = (self.time.time_charge + defeated as f32 * 12.0).min(100.0);
            self.enemies.retain(|enemy| enemy.health > 0.0);
        }

        if self.enemies.is_empty() {
            self.room.open_exit();
        }

        if self.room.can_leave(self.player.position) {
            self.advance_room();
        }

        if self.player.health <= 0.0 {
            self.player.health = self.player.max_health;
            self.player.position = vec2(240.0, 135.0);
            self.room = Room::new(self.room.number);
            self.enemies = spawn_room(self.room.number);
            self.enemy_projectiles.clear();
            self.knives.clear();
        }
    }

    fn draw(&self) {
        self.renderer.draw_frame(
            &self.player,
            &self.room,
            &self.enemies,
            &self.enemy_projectiles,
            &self.knives,
            self.input.aim_position,
            self.input.touch_controls_visible,
            self.input.move_stick_position,
            self.input.move_stick_active,
            self.input.aim_stick_position,
            self.input.aim_stick_active,
            self.room.number,
            self.time.time_charge,
            self.time.time_stop_active,
            self.time.time_stop_remaining,
            self.time.time_stop_cooldown_remaining,
            self.knife_cooldown_remaining,
            self.left_handed,
        );
        if self.screen != AppScreen::Playing {
            let (title, entries) = match self.screen {
                AppScreen::MainMenu => ("MY WORLD", self.menu_entries()),
                AppScreen::Paused => ("PAUSED", self.menu_entries()),
                AppScreen::Settings => ("SETTINGS", self.menu_entries()),
                AppScreen::Playing => return,
            };
            self.renderer
                .draw_menu_overlay(title, &entries, self.menu_selection);
        }
    }

    fn stand_attack(&mut self) {
        let direction = self.input.aim_dir;
        if direction.length_squared() == 0.0 {
            return;
        }

        let target_id = self
            .enemies
            .iter()
            .find(|enemy| {
                let offset = enemy.position - self.player.position;
                let distance = offset.length();
                distance <= 34.0 + enemy.radius
                    && distance > 0.0
                    && offset.normalize().dot(direction) > 0.35
            })
            .map(|enemy| enemy.id);
        if let Some(target_id) = target_id {
            let impulse = direction * 42.0;
            if self.time.time_stop_active {
                self.time
                    .queue_hit(target_id, 18.0, impulse, FinisherKind::Stand);
            } else if let Some(target) = self
                .enemies
                .iter_mut()
                .find(|target| target.id == target_id)
            {
                target.health -= 18.0;
                target.position += impulse * 0.03;
            }
        }
    }

    fn advance_room(&mut self) {
        let entry_position = self.room.exit.entry_position();
        let next_number = self.room.number + 1;
        self.room = Room::new(next_number);
        self.player.position = entry_position;
        self.enemies = spawn_room(next_number);
        self.enemy_projectiles.clear();
        self.knives.clear();
        self.time.time_charge = (self.time.time_charge + 15.0).min(100.0);
    }

    fn restart_run(&mut self) {
        self.time = TimeState::new();
        self.input = InputState::default();
        self.player = Player::new();
        self.room = Room::new(1);
        self.enemies = spawn_room(1);
        self.enemy_projectiles.clear();
        self.knives.clear();
        self.knife_cooldown_remaining = 0.0;
        self.run_started = true;
        self.menu_selection = 0;
    }

    fn throw_knife(&mut self) {
        if self.knife_cooldown_remaining > 0.0 {
            return;
        }
        self.knife_cooldown_remaining = KNIFE_COOLDOWN;
        let direction = if self.input.aim_dir.length_squared() > 0.0 {
            self.input.aim_dir
        } else {
            Vec2::X
        };
        let start = self.player.position
            + direction
                * if self.time.time_stop_active {
                    25.0
                } else {
                    8.0
                };
        self.knives.push(Knife {
            position: start,
            velocity: direction * 260.0,
            remaining: 1.25,
        });

        if self.time.time_stop_active {
            if let Some(enemy) = self
                .enemies
                .iter()
                .find(|enemy| enemy.position.distance(start) <= enemy.radius + 5.0)
            {
                self.time
                    .queue_hit(enemy.id, 24.0, direction * 55.0, FinisherKind::Knife);
                self.knives.pop();
            }
        }
    }

    fn update_knives(&mut self, dt: f32) {
        for knife in &mut self.knives {
            knife.position += knife.velocity * dt;
            knife.remaining -= dt;
        }

        let mut hits = Vec::new();
        self.knives.retain(|knife| {
            if knife.remaining <= 0.0
                || !(18.0..462.0).contains(&knife.position.x)
                || !(20.0..250.0).contains(&knife.position.y)
            {
                return false;
            }

            if let Some(enemy) = self
                .enemies
                .iter()
                .find(|enemy| enemy.position.distance(knife.position) <= enemy.radius + 3.0)
            {
                hits.push((enemy.id, knife.velocity.normalize_or_zero() * 45.0));
                false
            } else {
                true
            }
        });

        for (target_id, impulse) in hits {
            if let Some(enemy) = self.enemies.iter_mut().find(|enemy| enemy.id == target_id) {
                enemy.health -= 24.0;
                enemy.position += impulse * 0.03;
            }
        }
    }

    fn release_delayed_hits(&mut self) {
        for DelayedHit {
            target_id,
            damage,
            impulse_vector,
            finisher_type: _,
        } in self.time.take_delayed_hits()
        {
            if let Some(enemy) = self.enemies.iter_mut().find(|enemy| enemy.id == target_id) {
                enemy.health -= damage;
                enemy.position += impulse_vector * 0.03;
            }
        }
    }
}

#[macroquad::main("MY WORLD")]
async fn main() {
    simulate_mouse_with_touch(false);
    let mut app = App::new();

    loop {
        app.update();
        app.draw();
        next_frame().await;
    }
}
