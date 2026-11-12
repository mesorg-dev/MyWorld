use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Knife {
    pub position: Vec2,
    pub velocity: Vec2,
    pub remaining: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ImpactMark {
    pub position: Vec2,
    pub radius: f32,
    pub color: Color,
}

#[derive(Clone, Copy, Debug)]
pub struct CombatParticle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub remaining: f32,
    pub lifetime: f32,
    pub radius: f32,
    pub color: Color,
}

#[derive(Clone, Debug, Default)]
pub struct CombatEffects {
    pub marks: Vec<ImpactMark>,
    pub particles: Vec<CombatParticle>,
    shake: f32,
    shake_phase: f32,
    hit_stop_remaining: f32,
}

impl CombatEffects {
    pub fn spawn_impact(&mut self, position: Vec2, direction: Vec2, color: Color, strength: f32) {
        let direction = direction.normalize_or_zero();
        let base_angle = direction.y.atan2(direction.x);
        for index in 0..7 {
            let angle = base_angle + (index as f32 - 3.0) * 0.34;
            let speed = 26.0 + (index % 3) as f32 * 13.0;
            let lifetime = 0.16 + (index % 3) as f32 * 0.045;
            self.particles.push(CombatParticle {
                position,
                velocity: vec2(angle.cos(), angle.sin()) * speed,
                remaining: lifetime,
                lifetime,
                radius: 1.3 + (index % 2) as f32,
                color,
            });
        }

        self.marks.push(ImpactMark {
            position,
            radius: 2.0 + strength,
            color: Color::new(0.35, 0.045, 0.12, 0.62),
        });
        if self.marks.len() > 160 {
            self.marks.remove(0);
        }
        self.shake = (self.shake + strength).min(5.0);
        if strength >= 1.2 {
            self.hit_stop_remaining = self.hit_stop_remaining.max(0.045);
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.shake = (self.shake - dt * 18.0).max(0.0);
        self.shake_phase += dt * 92.0;
        self.hit_stop_remaining = (self.hit_stop_remaining - dt).max(0.0);
        for particle in &mut self.particles {
            particle.position += particle.velocity * dt;
            particle.velocity *= (1.0 - dt * 4.5).max(0.0);
            particle.remaining -= dt;
        }
        self.particles.retain(|particle| particle.remaining > 0.0);
    }

    pub fn world_dt(&self, world_dt: f32) -> f32 {
        if self.hit_stop_remaining > 0.0 {
            0.0
        } else {
            world_dt
        }
    }

    pub fn camera_offset(&self) -> Vec2 {
        vec2(
            self.shake_phase.sin() * self.shake,
            (self.shake_phase * 1.37).cos() * self.shake * 0.62,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RoadRoller {
    pub position: Vec2,
    pub windup: f32,
    pub impact_remaining: f32,
    pub barrage_hits: u8,
    pub impacted: bool,
}

impl RoadRoller {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            windup: 0.4,
            impact_remaining: 0.32,
            barrage_hits: 0,
            impacted: false,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        if self.impacted {
            self.impact_remaining = (self.impact_remaining - dt).max(0.0);
            return false;
        }

        self.windup = (self.windup - dt).max(0.0);
        if self.windup == 0.0 {
            self.impacted = true;
            true
        } else {
            false
        }
    }

    pub fn add_barrage_hit(&mut self) -> bool {
        if self.impacted || self.barrage_hits >= 4 {
            return false;
        }
        self.barrage_hits += 1;
        true
    }

    pub fn damage_multiplier(&self) -> f32 {
        1.0 + self.barrage_hits as f32 * 0.25
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct CombatHit {
    pub target_id: u32,
    pub damage: f32,
    pub impulse: Vec2,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct CombatSystem {
    next_id: u32,
}

#[allow(dead_code)]
impl CombatSystem {
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    pub fn allocate_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn resolve_hit(&self, hit: CombatHit) -> CombatHit {
        hit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn road_roller_waits_for_world_time_and_barrage_increases_impact() {
        let mut roller = RoadRoller::new(vec2(100.0, 100.0));
        assert!(!roller.update(0.2));
        assert!(!roller.impacted);
        assert!(roller.add_barrage_hit());
        assert_eq!(roller.damage_multiplier(), 1.25);
        assert!(roller.update(0.2));
        assert!(!roller.add_barrage_hit());
    }

    #[test]
    fn combat_effects_keep_marks_bounded_and_decay_particles() {
        let mut effects = CombatEffects::default();
        for _ in 0..170 {
            effects.spawn_impact(Vec2::ZERO, Vec2::X, WHITE, 1.5);
        }
        assert_eq!(effects.marks.len(), 160);
        assert!(!effects.particles.is_empty());
        assert_eq!(effects.world_dt(0.016), 0.0);

        effects.update(1.0);
        assert!(effects.particles.is_empty());
        assert_eq!(effects.camera_offset(), Vec2::ZERO);
        assert_eq!(effects.world_dt(0.016), 0.016);
    }
}
