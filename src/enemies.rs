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
    pub difficulty_scale: f32,
    pub facing_direction: Vec2,
    pub telegraph_remaining: f32,
    pub telegraph_direction: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub struct EnemyProjectile {
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub damage: f32,
    pub remaining: f32,
    pub kind: EnemyKind,
}

#[derive(Debug, Default)]
pub struct EnemyUpdate {
    pub contact_damage: f32,
    pub projectiles: Vec<EnemyProjectile>,
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
            attack_cooldown: match kind {
                EnemyKind::Shooter => 0.8,
                EnemyKind::Sniper => 1.4,
                _ => 0.4,
            },
            difficulty_scale: 1.0,
            facing_direction: Vec2::X,
            telegraph_remaining: 0.0,
            telegraph_direction: Vec2::X,
        }
    }

    pub fn update(&mut self, player: &Player, dt: f32) -> (f32, Option<EnemyProjectile>) {
        let to_player = player.position - self.position;
        let distance_sq = to_player.length_squared();
        let distance = distance_sq.sqrt();
        let (desired_speed, preferred_range, range_tolerance) = match self.kind {
            EnemyKind::Shooter => (20.0, Some(92.0), 16.0),
            EnemyKind::Rusher => (48.0, None, 0.0),
            EnemyKind::Brute => (26.0, None, 0.0),
            EnemyKind::Sniper => (14.0, Some(145.0), 12.0),
        };

        if distance_sq > 0.01 {
            let direction = to_player / distance_sq.sqrt();
            let movement = match preferred_range {
                Some(range) if distance < range - range_tolerance => -direction,
                Some(range) if distance > range + range_tolerance => direction,
                Some(_) => Vec2::ZERO,
                None if self.kind == EnemyKind::Brute && distance <= 22.0 => Vec2::ZERO,
                None => direction,
            };
            self.velocity = movement * desired_speed;
            if movement.length_squared() > 0.0 {
                self.facing_direction = if self.kind == EnemyKind::Brute {
                    rotate_towards(self.facing_direction, movement, 1.8 * dt)
                } else {
                    direction
                };
            } else if self.kind != EnemyKind::Brute {
                self.facing_direction = direction;
            }
        }

        self.position += self.velocity * dt;
        self.position.x = self.position.x.clamp(20.0, 460.0);
        self.position.y = self.position.y.clamp(20.0, 250.0);

        self.attack_cooldown = (self.attack_cooldown - dt).max(0.0);
        if self.telegraph_remaining > 0.0 {
            self.telegraph_remaining = (self.telegraph_remaining - dt).max(0.0);
            if self.telegraph_remaining == 0.0 {
                return (0.0, self.fire_projectile());
            }
            return (0.0, None);
        }

        if distance_sq < (self.radius + player.radius + 5.0).powi(2) && self.attack_cooldown <= 0.0
        {
            self.attack_cooldown = 0.8;
            let damage = match self.kind {
                EnemyKind::Brute => 18.0,
                _ => 10.0,
            } * self.difficulty_scale;
            return (damage, None);
        } else if let Some((range, cooldown, windup)) = match self.kind {
            EnemyKind::Shooter => Some((210.0, 1.15, 0.28)),
            EnemyKind::Sniper => Some((250.0, 1.9, 0.55)),
            _ => None,
        } {
            if distance <= range && distance_sq > 0.01 && self.attack_cooldown <= 0.0 {
                self.attack_cooldown = cooldown;
                self.telegraph_direction = to_player / distance;
                self.telegraph_remaining = windup;
            }
        }
        (0.0, None)
    }

    pub fn is_armored_from(&self, attacker_position: Vec2) -> bool {
        if self.kind != EnemyKind::Brute {
            return false;
        }
        let toward_attacker = (attacker_position - self.position).normalize_or_zero();
        toward_attacker.dot(self.facing_direction) > 0.35
    }

    fn fire_projectile(&self) -> Option<EnemyProjectile> {
        let (speed, damage) = match self.kind {
            EnemyKind::Shooter => (72.0, 8.0 * self.difficulty_scale),
            EnemyKind::Sniper => (132.0, 15.0 * self.difficulty_scale),
            _ => return None,
        };
        Some(EnemyProjectile {
            position: self.position + self.telegraph_direction * (self.radius + 2.0),
            velocity: self.telegraph_direction * speed,
            radius: if self.kind == EnemyKind::Sniper {
                3.0
            } else {
                2.5
            },
            damage,
            remaining: 3.0,
            kind: self.kind,
        })
    }
}

fn rotate_towards(current: Vec2, target: Vec2, max_radians: f32) -> Vec2 {
    let current_angle = current.y.atan2(current.x);
    let target_angle = target.y.atan2(target.x);
    let delta = (target_angle - current_angle + std::f32::consts::PI)
        .rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    let angle = current_angle + delta.clamp(-max_radians, max_radians);
    vec2(angle.cos(), angle.sin())
}

pub fn spawn_room(room_number: u32) -> Vec<Enemy> {
    let kinds = [
        EnemyKind::Shooter,
        EnemyKind::Rusher,
        EnemyKind::Brute,
        EnemyKind::Sniper,
    ];
    let count = (4 + room_number.saturating_sub(1) as usize / 2).min(12);
    let difficulty_scale = (1.0 + room_number.saturating_sub(1) as f32 * 0.08).min(2.5);
    let room_offset = room_number as f32 * 0.37;

    let mut enemies = Vec::with_capacity(count);
    for i in 0..count {
        let kind = kinds[(i + room_number as usize - 1) % kinds.len()];
        let angle = std::f32::consts::TAU * i as f32 / count as f32 + room_offset;
        let position = vec2(240.0 + angle.cos() * 185.0, 135.0 + angle.sin() * 92.0);
        let mut enemy = Enemy::new((i + 1) as u32, kind, position);
        enemy.health *= difficulty_scale;
        enemy.max_health = enemy.health;
        enemy.difficulty_scale = difficulty_scale;
        enemies.push(enemy);
    }

    enemies
}

pub fn update_enemies(enemies: &mut [Enemy], player: &Player, dt: f32) -> EnemyUpdate {
    let mut update = EnemyUpdate::default();
    for enemy in enemies.iter_mut() {
        let (damage, projectile) = enemy.update(player, dt);
        update.contact_damage += damage;
        update.projectiles.extend(projectile);
    }
    update
}

pub fn update_enemy_projectiles(
    projectiles: &mut Vec<EnemyProjectile>,
    player: &Player,
    dt: f32,
) -> f32 {
    let mut damage = 0.0;
    projectiles.retain_mut(|projectile| {
        projectile.position += projectile.velocity * dt;
        projectile.remaining -= dt;

        if projectile.remaining <= 0.0
            || !(18.0..462.0).contains(&projectile.position.x)
            || !(20.0..250.0).contains(&projectile.position.y)
        {
            return false;
        }

        if projectile.position.distance(player.position) <= projectile.radius + player.radius {
            damage += projectile.damage;
            false
        } else {
            true
        }
    });
    damage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranged_enemies_keep_their_distance_and_fire_toward_the_player() {
        let player = Player::new();
        let mut shooter = Enemy::new(1, EnemyKind::Shooter, vec2(150.0, 135.0));

        let (_, projectile) = shooter.update(&player, 0.016);
        assert!(shooter.velocity.x.abs() < f32::EPSILON);
        assert!(projectile.is_none());

        shooter.attack_cooldown = 0.0;
        let (_, projectile) = shooter.update(&player, 0.016);
        assert!(projectile.is_none());
        assert!(shooter.telegraph_remaining > 0.0);
        let (_, projectile) = shooter.update(&player, 0.3);
        let projectile = projectile.expect("shooter fires after telegraphing");
        assert!(projectile.velocity.x > 0.0);
        assert!(projectile.velocity.y.abs() < f32::EPSILON);
    }

    #[test]
    fn brute_armor_blocks_frontal_attacks_but_exposes_its_back() {
        let mut brute = Enemy::new(1, EnemyKind::Brute, vec2(100.0, 100.0));
        brute.facing_direction = Vec2::X;
        assert!(brute.is_armored_from(vec2(120.0, 100.0)));
        assert!(!brute.is_armored_from(vec2(80.0, 100.0)));
    }

    #[test]
    fn projectile_damages_the_player_once_and_is_removed() {
        let player = Player::new();
        let mut projectiles = vec![EnemyProjectile {
            position: player.position - vec2(2.0, 0.0),
            velocity: vec2(120.0, 0.0),
            radius: 2.5,
            damage: 8.0,
            remaining: 1.0,
            kind: EnemyKind::Shooter,
        }];

        let damage = update_enemy_projectiles(&mut projectiles, &player, 0.016);
        assert_eq!(damage, 8.0);
        assert!(projectiles.is_empty());
    }

    #[test]
    fn later_rooms_have_more_and_tougher_enemies() {
        let first = spawn_room(1);
        let later = spawn_room(5);

        assert_eq!(first.len(), 4);
        assert_eq!(later.len(), 6);
        assert!(later[0].max_health > first[0].max_health);
    }
}
