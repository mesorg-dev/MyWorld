mod combat;
mod enemies;
mod input;
mod player;
mod render;
mod skills;
mod time;

use macroquad::prelude::*;

use crate::combat::Knife;
use crate::enemies::{spawn_wave, update_enemies, Enemy};
use crate::input::{read_input, InputState};
use crate::player::Player;
use crate::render::ArenaRenderer;
use crate::time::{DelayedHit, FinisherKind, TimeState};

struct App {
    time: TimeState,
    input: InputState,
    player: Player,
    enemies: Vec<Enemy>,
    knives: Vec<Knife>,
    renderer: ArenaRenderer,
    wave: u32,
}

impl App {
    fn new() -> Self {
        Self {
            time: TimeState::new(),
            input: InputState::default(),
            player: Player::new(),
            enemies: spawn_wave(),
            knives: Vec::new(),
            renderer: ArenaRenderer::new(),
            wave: 1,
        }
    }

    fn update(&mut self) {
        self.input = read_input(self.player.position);
        let dt = get_frame_time().min(0.033);
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
            let damage = update_enemies(&mut self.enemies, &self.player, self.time.world_dt);
            self.player.take_damage(damage);
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
            self.wave += 1;
            self.enemies = spawn_wave();
        }

        if self.player.health <= 0.0 {
            self.player.health = self.player.max_health;
            self.player.position = vec2(240.0, 135.0);
            self.enemies = spawn_wave();
            self.knives.clear();
        }
    }

    fn draw(&self) {
        self.renderer.draw_frame(
            &self.player,
            &self.enemies,
            &self.knives,
            self.input.aim_position,
            self.input.touch_controls_visible,
            self.input.move_stick_position,
            self.input.move_stick_active,
            self.input.aim_stick_position,
            self.input.aim_stick_active,
            self.wave,
            self.time.time_charge,
            self.time.time_stop_active,
            self.time.time_stop_remaining,
        );
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

    fn throw_knife(&mut self) {
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
