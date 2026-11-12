use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Knife {
    pub position: Vec2,
    pub velocity: Vec2,
    pub remaining: f32,
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
