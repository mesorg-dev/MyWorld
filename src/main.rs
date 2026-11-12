mod audio;
mod combat;
mod enemies;
mod input;
mod player;
mod render;
mod room;
mod skills;
mod time;

use macroquad::prelude::*;

use crate::audio::SoundBank;
use crate::combat::{CombatEffects, Knife, RoadRoller};
use crate::enemies::{
    spawn_room, update_enemies, update_enemy_projectiles, Enemy, EnemyProjectile,
};
use crate::input::{pointer_pressed_position, read_input, InputState};
use crate::player::{Player, DASH_COOLDOWN};
use crate::render::ArenaRenderer;
use crate::room::Room;
use crate::skills::{SkillTree, SkillUpgrade};
use crate::time::{DelayedHit, FinisherKind, TimeState, DEFAULT_TIME_STOP_DURATION};

const KNIFE_COOLDOWN: f32 = 0.55;
const KNIFE_COOLDOWN_REDUCTION: f32 = 0.08;
const MIN_KNIFE_COOLDOWN: f32 = 0.31;
const TIME_STOP_DURATION_BONUS: f32 = 0.4;
const MAX_TIME_STOP_DURATION: f32 = 2.7;
const DASH_COOLDOWN_REDUCTION: f32 = 0.12;
const MIN_DASH_COOLDOWN: f32 = 0.54;
const ROAD_ROLLER_COST: f32 = 50.0;
const ROAD_ROLLER_COOLDOWN: f32 = 9.0;
const ROAD_ROLLER_DAMAGE: f32 = 48.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppScreen {
    MainMenu,
    Playing,
    Paused,
    Settings,
    Upgrade,
}

struct App {
    time: TimeState,
    input: InputState,
    player: Player,
    room: Room,
    enemies: Vec<Enemy>,
    enemy_projectiles: Vec<EnemyProjectile>,
    knives: Vec<Knife>,
    road_roller: Option<RoadRoller>,
    road_roller_cooldown_remaining: f32,
    effects: CombatEffects,
    sounds: SoundBank,
    sound_enabled: bool,
    knife_cooldown_remaining: f32,
    knife_cooldown_duration: f32,
    skills: SkillTree,
    renderer: ArenaRenderer,
    fullscreen: bool,
    screen: AppScreen,
    settings_return_screen: AppScreen,
    menu_selection: usize,
    left_handed: bool,
    run_started: bool,
}

impl App {
    fn new(sounds: SoundBank) -> Self {
        Self {
            time: TimeState::new(),
            input: InputState::default(),
            player: Player::new(),
            room: Room::new(1),
            enemies: spawn_room(1),
            enemy_projectiles: Vec::new(),
            knives: Vec::new(),
            road_roller: None,
            road_roller_cooldown_remaining: 0.0,
            effects: CombatEffects::default(),
            sounds,
            sound_enabled: true,
            knife_cooldown_remaining: 0.0,
            knife_cooldown_duration: KNIFE_COOLDOWN,
            skills: SkillTree::new(),
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
                AppScreen::Upgrade => {
                    self.screen = AppScreen::Playing;
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
                let hit_radius = if entries.len() >= 5 { 10.0 } else { 14.0 };
                (position.y >= center_y - hit_radius && position.y <= center_y + hit_radius)
                    .then_some(index)
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
                1 => self.sound_enabled = !self.sound_enabled,
                2 => {
                    self.screen = self.settings_return_screen;
                    self.menu_selection = 0;
                }
                _ => {}
            },
            AppScreen::Upgrade => {
                let upgrades = self.skills.available_upgrades();
                if self.menu_selection < upgrades.len() {
                    self.apply_upgrade(upgrades[self.menu_selection]);
                }
                self.screen = AppScreen::Playing;
                self.menu_selection = 0;
            }
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
                format!("SOUND: {}", if self.sound_enabled { "ON" } else { "OFF" }),
                "BACK".to_owned(),
            ],
            AppScreen::Upgrade => self
                .skills
                .available_upgrades()
                .into_iter()
                .map(|upgrade| {
                    let level = self.skills.level(upgrade);
                    let effect = match upgrade {
                        SkillUpgrade::TimeStop => "+0.4S",
                        SkillUpgrade::KnifeCooldown => "CD -0.08S",
                        SkillUpgrade::DashCooldown => "CD -0.12S",
                        SkillUpgrade::RoadRoller => "UNLOCK ULT",
                    };
                    format!(
                        "{} {effect}  {}/{}",
                        upgrade.label(),
                        level + 1,
                        if upgrade == SkillUpgrade::RoadRoller {
                            1
                        } else {
                            SkillTree::MAX_LEVEL
                        }
                    )
                })
                .chain(std::iter::once("CONTINUE".to_owned()))
                .collect(),
            AppScreen::Playing => Vec::new(),
        }
    }

    fn update_gameplay(&mut self) {
        let dt = get_frame_time().min(0.033);
        self.effects.update(dt);
        self.knife_cooldown_remaining = (self.knife_cooldown_remaining - dt).max(0.0);
        self.road_roller_cooldown_remaining = (self.road_roller_cooldown_remaining - dt).max(0.0);
        let time_stop_expired = self.time.update(dt);
        if time_stop_expired {
            self.release_delayed_hits();
            self.sounds.time_stop(self.sound_enabled);
        }

        if self.input.timestop_pressed {
            if self.time.time_stop_active {
                self.time.release_time_stop();
                self.release_delayed_hits();
                self.sounds.time_stop(self.sound_enabled);
            } else if self.time.activate_time_stop() {
                self.sounds.time_stop(self.sound_enabled);
            }
        }

        let dash_was_active = self.player.dash_remaining > 0.0;
        self.player.update(&self.input, dt);
        if !dash_was_active && self.player.dash_remaining > 0.0 {
            self.sounds.dash(self.sound_enabled);
        }

        if self.player.attack_fired {
            self.sounds.strike(self.sound_enabled);
            self.stand_attack();
        }
        if self.input.throw_pressed {
            self.throw_knife();
        }
        if self.input.ult_pressed {
            self.activate_road_roller();
        }

        let world_dt = self.effects.world_dt(self.time.world_dt);
        if world_dt > 0.0 {
            let enemy_update = update_enemies(&mut self.enemies, &self.player, world_dt);
            let contact_damage = enemy_update.contact_damage;
            let contact_parried = self
                .player
                .take_combat_damage(contact_damage, self.input.block_held);
            self.enemy_projectiles.extend(enemy_update.projectiles);
            let projectile_damage =
                update_enemy_projectiles(&mut self.enemy_projectiles, &self.player, world_dt);
            let projectile_parried = self
                .player
                .take_combat_damage(projectile_damage, self.input.block_held);
            if contact_parried || projectile_parried {
                self.time.time_charge = (self.time.time_charge + 12.0).min(100.0);
                self.effects.spawn_impact(
                    self.player.position + self.player.attack_direction * 9.0,
                    self.player.attack_direction,
                    Color::new(0.25, 0.88, 1.0, 1.0),
                    2.4,
                );
                self.sounds.impact(self.sound_enabled);
            } else if contact_damage > 0.0 || projectile_damage > 0.0 {
                self.sounds.impact(self.sound_enabled);
            }
            self.player
                .take_damage(self.room.update_traps(&self.player, world_dt));
            self.update_knives(world_dt);
            self.update_road_roller(world_dt);
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
            if self.room.claim_clear_reward() {
                self.screen = AppScreen::Upgrade;
                self.menu_selection = 0;
                return;
            }
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
            self.road_roller = None;
        }
    }

    fn draw(&self) {
        self.renderer.draw_frame(
            &self.player,
            &self.room,
            &self.enemies,
            &self.enemy_projectiles,
            &self.knives,
            &self.road_roller,
            &self.effects,
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
            self.road_roller_cooldown_remaining,
            self.skills.road_roller_unlocked,
            self.left_handed,
        );
        if self.screen != AppScreen::Playing {
            let (title, entries) = match self.screen {
                AppScreen::MainMenu => ("MY WORLD", self.menu_entries()),
                AppScreen::Paused => ("PAUSED", self.menu_entries()),
                AppScreen::Settings => ("SETTINGS", self.menu_entries()),
                AppScreen::Upgrade => ("ROOM CLEARED", self.menu_entries()),
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

        if self.time.time_stop_active {
            if let Some(roller) = &mut self.road_roller {
                if self.input.aim_position.distance(roller.position) <= 38.0
                    && roller.add_barrage_hit()
                {
                    self.effects.spawn_impact(
                        roller.position,
                        direction,
                        Color::new(1.0, 0.7, 0.22, 1.0),
                        1.0,
                    );
                    return;
                }
            }
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
            let target_position = self
                .enemies
                .iter()
                .find(|target| target.id == target_id)
                .map_or(self.player.position, |target| target.position);
            let armored = self
                .enemies
                .iter()
                .find(|target| target.id == target_id)
                .is_some_and(|target| target.is_armored_from(self.player.position));
            if armored {
                self.effects.spawn_impact(
                    target_position,
                    direction,
                    Color::new(0.35, 0.8, 1.0, 1.0),
                    0.8,
                );
                return;
            }
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
            self.effects.spawn_impact(
                target_position,
                direction,
                Color::new(0.1, 0.9, 1.0, 1.0),
                1.4,
            );
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
        self.road_roller = None;
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
        self.road_roller = None;
        self.road_roller_cooldown_remaining = 0.0;
        self.effects = CombatEffects::default();
        self.knife_cooldown_remaining = 0.0;
        self.knife_cooldown_duration = KNIFE_COOLDOWN;
        self.skills = SkillTree::new();
        self.run_started = true;
        self.menu_selection = 0;
    }

    fn apply_upgrade(&mut self, upgrade: SkillUpgrade) {
        let Some(level) = self.skills.upgrade(upgrade) else {
            return;
        };
        match upgrade {
            SkillUpgrade::TimeStop => {
                self.time.time_stop_duration = (DEFAULT_TIME_STOP_DURATION
                    + TIME_STOP_DURATION_BONUS * level as f32)
                    .min(MAX_TIME_STOP_DURATION);
            }
            SkillUpgrade::KnifeCooldown => {
                self.knife_cooldown_duration = (KNIFE_COOLDOWN
                    - KNIFE_COOLDOWN_REDUCTION * level as f32)
                    .max(MIN_KNIFE_COOLDOWN);
            }
            SkillUpgrade::DashCooldown => {
                self.player.dash_cooldown_duration =
                    (DASH_COOLDOWN - DASH_COOLDOWN_REDUCTION * level as f32).max(MIN_DASH_COOLDOWN);
            }
            SkillUpgrade::RoadRoller => {}
        }
    }

    fn activate_road_roller(&mut self) {
        if !self.skills.road_roller_unlocked
            || self.road_roller_cooldown_remaining > 0.0
            || self.time.time_charge < ROAD_ROLLER_COST
            || self.road_roller.is_some()
        {
            return;
        }

        self.time.time_charge -= ROAD_ROLLER_COST;
        self.road_roller_cooldown_remaining = ROAD_ROLLER_COOLDOWN;
        self.road_roller = Some(RoadRoller::new(
            self.input
                .aim_position
                .clamp(vec2(32.0, 36.0), vec2(448.0, 234.0)),
        ));
    }

    fn update_road_roller(&mut self, dt: f32) {
        let Some(mut roller) = self.road_roller else {
            return;
        };
        if roller.update(dt) {
            let radius = 52.0;
            let damage = ROAD_ROLLER_DAMAGE * roller.damage_multiplier();
            for enemy in &mut self.enemies {
                let offset = enemy.position - roller.position;
                let distance = offset.length();
                if distance <= radius + enemy.radius {
                    enemy.health -= damage;
                    enemy.position += offset.normalize_or_zero() * 34.0;
                }
            }
            self.effects.spawn_impact(
                roller.position,
                Vec2::X,
                Color::new(1.0, 0.48, 0.12, 1.0),
                5.0,
            );
            self.effects.spawn_impact(
                roller.position,
                Vec2::Y,
                Color::new(1.0, 0.82, 0.35, 1.0),
                3.0,
            );
            self.sounds.roller(self.sound_enabled);
        }
        if roller.impacted && roller.impact_remaining <= 0.0 {
            self.road_roller = None;
        } else {
            self.road_roller = Some(roller);
        }
    }

    fn throw_knife(&mut self) {
        if self.knife_cooldown_remaining > 0.0 {
            return;
        }
        self.knife_cooldown_remaining = self.knife_cooldown_duration;
        self.sounds.knife(self.sound_enabled);
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
                hits.push((
                    enemy.id,
                    knife.position,
                    knife.velocity.normalize_or_zero() * 45.0,
                ));
                false
            } else {
                true
            }
        });

        for (target_id, position, impulse) in hits {
            if let Some(enemy) = self.enemies.iter_mut().find(|enemy| enemy.id == target_id) {
                if enemy.is_armored_from(position - impulse) {
                    self.effects.spawn_impact(
                        position,
                        impulse,
                        Color::new(0.35, 0.8, 1.0, 1.0),
                        0.8,
                    );
                } else {
                    enemy.health -= 24.0;
                    enemy.position += impulse * 0.03;
                    self.effects.spawn_impact(
                        position,
                        impulse,
                        Color::new(0.1, 0.9, 1.0, 1.0),
                        1.2,
                    );
                }
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
                self.effects.spawn_impact(
                    enemy.position,
                    impulse_vector,
                    Color::new(1.0, 0.28, 0.42, 1.0),
                    2.2,
                );
            }
        }
    }
}

#[macroquad::main("MY WORLD")]
async fn main() {
    simulate_mouse_with_touch(false);
    let sounds = SoundBank::load()
        .await
        .expect("failed to initialize the built-in MY WORLD sound effects");
    let mut app = App::new(sounds);

    loop {
        app.update();
        app.draw();
        next_frame().await;
    }
}
