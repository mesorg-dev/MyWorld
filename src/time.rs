use macroquad::prelude::*;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinisherKind {
    Stand,
    Knife,
    RoadRoller,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct DelayedHit {
    pub target_id: u32,
    pub damage: f32,
    pub impulse_vector: Vec2,
    pub finisher_type: FinisherKind,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TimeState {
    pub real_dt: f32,
    pub world_dt: f32,
    pub time_stop_active: bool,
    pub time_stop_duration: f32,
    pub time_stop_remaining: f32,
    pub time_charge: f32,
    pub delayed_hits: Vec<DelayedHit>,
}

#[allow(dead_code)]
impl TimeState {
    pub fn new() -> Self {
        Self {
            real_dt: 1.0 / 60.0,
            world_dt: 1.0 / 60.0,
            time_stop_active: false,
            time_stop_duration: 1.5,
            time_stop_remaining: 0.0,
            time_charge: 100.0,
            delayed_hits: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.real_dt = dt;
        let mut expired = false;
        if self.time_stop_active {
            self.time_stop_remaining -= dt;
            if self.time_stop_remaining <= 0.0 {
                self.release_time_stop();
                expired = true;
            }
        }
        self.world_dt = if self.time_stop_active { 0.0 } else { dt };
        expired
    }

    pub fn activate_time_stop(&mut self) -> bool {
        const TIME_STOP_COST: f32 = 35.0;
        if self.time_stop_active || self.time_charge < TIME_STOP_COST {
            return false;
        }
        self.time_charge -= TIME_STOP_COST;
        self.time_stop_active = true;
        self.time_stop_remaining = self.time_stop_duration;
        self.world_dt = 0.0;
        true
    }

    pub fn release_time_stop(&mut self) {
        self.time_stop_active = false;
        self.time_stop_remaining = 0.0;
        self.world_dt = self.real_dt;
    }

    pub fn queue_hit(
        &mut self,
        target_id: u32,
        damage: f32,
        impulse_vector: Vec2,
        finisher: FinisherKind,
    ) {
        self.delayed_hits.push(DelayedHit {
            target_id,
            damage,
            impulse_vector,
            finisher_type: finisher,
        });
    }

    pub fn take_delayed_hits(&mut self) -> Vec<DelayedHit> {
        std::mem::take(&mut self.delayed_hits)
    }
}

impl Default for TimeState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_stop_consumes_charge_and_freezes_world_time() {
        let mut time = TimeState::new();

        assert!(time.activate_time_stop());
        assert_eq!(time.time_charge, 65.0);

        time.update(0.1);
        assert_eq!(time.world_dt, 0.0);
        assert!(time.time_stop_active);
    }

    #[test]
    fn time_stop_expiry_releases_delayed_hits_for_the_game_to_apply() {
        let mut time = TimeState::new();
        time.activate_time_stop();
        time.queue_hit(7, 18.0, Vec2::X, FinisherKind::Stand);

        assert!(time.update(time.time_stop_duration));
        assert!(!time.time_stop_active);
        assert_eq!(time.world_dt, time.real_dt);
        assert_eq!(time.take_delayed_hits().len(), 1);
    }
}
