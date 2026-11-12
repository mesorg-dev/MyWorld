use macroquad::prelude::*;

use crate::player::Player;
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Shooter,
    Rusher,
    Brute,
    Sniper,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Enemy {
    pub id: u32,
    pub kind: EnemyKind,
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub health: f32,
    pub max_health: f32,
    pub attack_cooldown: f32,
}

impl Enemy {
    pub fn new(id: u32, kind: EnemyKind, position: Vec2) -> Self {
        Self {
            id,
            kind,
            position,
            velocity: Vec2::ZERO,
            radius: match kind {
                EnemyKind::Shooter => 7.0,
                EnemyKind::Rusher => 8.0,
                EnemyKind::Brute => 10.0,
                EnemyKind::Sniper => 7.0,
            },
            health: match kind {
                EnemyKind::Shooter => 18.0,
                EnemyKind::Rusher => 24.0,
                EnemyKind::Brute => 40.0,
                EnemyKind::Sniper => 28.0,
            },
            max_health: match kind {
                EnemyKind::Shooter => 18.0,
                EnemyKind::Rusher => 24.0,
                EnemyKind::Brute => 40.0,
                EnemyKind::Sniper => 28.0,
            },
            attack_cooldown: 0.4,
        }
    }

    pub fn update(&mut self, player: &Player, dt: f32) -> f32 {
        let to_player = player.position - self.position;
        let distance_sq = to_player.length_squared();
        let desired_speed = match self.kind {
            EnemyKind::Shooter => 20.0,
            EnemyKind::Rusher => 48.0,
            EnemyKind::Brute => 26.0,
            EnemyKind::Sniper => 14.0,
        };

        if distance_sq > 0.01 {
            let direction = to_player / distance_sq.sqrt();
            self.velocity = direction * desired_speed;
        }

        self.position += self.velocity * dt;
        self.position.x = self.position.x.clamp(20.0, 460.0);
        self.position.y = self.position.y.clamp(20.0, 250.0);

        self.attack_cooldown -= dt;
        if distance_sq < (self.radius + player.radius + 5.0).powi(2) && self.attack_cooldown <= 0.0
        {
            self.attack_cooldown = 0.8;
            match self.kind {
                EnemyKind::Brute => 18.0,
                _ => 10.0,
            }
        } else {
            0.0
        }
    }
}

pub fn spawn_wave() -> Vec<Enemy> {
    let mut enemies = Vec::new();
    let kinds = [
        EnemyKind::Shooter,
        EnemyKind::Rusher,
        EnemyKind::Brute,
        EnemyKind::Sniper,
    ];

    for (i, kind) in kinds.iter().enumerate() {
        let offset = vec2((i as f32) * 40.0 + 60.0, 90.0 + (i as f32 * 18.0));
        enemies.push(Enemy::new((i + 1) as u32, *kind, offset));
    }

    enemies
}

pub fn update_enemies(enemies: &mut [Enemy], player: &Player, dt: f32) -> f32 {
    let mut damage = 0.0;
    for enemy in enemies.iter_mut() {
        damage += enemy.update(player, dt);
    }
    damage
}
