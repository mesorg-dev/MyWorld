use macroquad::prelude::*;

use crate::player::Player;

const ARENA_CENTER: Vec2 = Vec2::new(240.0, 135.0);
const TRAP_CYCLE: f32 = 3.0;
const TRAP_WARNING_START: f32 = 1.35;
const TRAP_ACTIVE_START: f32 = 1.8;
const TRAP_ACTIVE_END: f32 = 2.25;
const TRAP_RETRACT_END: f32 = 2.7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitSide {
    Top,
    Right,
    Bottom,
    Left,
}

impl ExitSide {
    pub fn from_room_number(room_number: u32) -> Self {
        let value = room_number
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223)
            >> 30;
        match value {
            0 => Self::Top,
            1 => Self::Right,
            2 => Self::Bottom,
            _ => Self::Left,
        }
    }

    pub fn entry_position(self) -> Vec2 {
        match self {
            Self::Top => vec2(240.0, 232.0),
            Self::Right => vec2(30.0, 135.0),
            Self::Bottom => vec2(240.0, 38.0),
            Self::Left => vec2(450.0, 135.0),
        }
    }

    pub fn exit_position(self) -> Vec2 {
        match self {
            Self::Top => vec2(240.0, 28.0),
            Self::Right => vec2(452.0, 135.0),
            Self::Bottom => vec2(240.0, 242.0),
            Self::Left => vec2(28.0, 135.0),
        }
    }

    pub fn exit_reached(self, position: Vec2) -> bool {
        match self {
            Self::Top => position.y <= 34.0 && (position.x - ARENA_CENTER.x).abs() <= 28.0,
            Self::Right => position.x >= 446.0 && (position.y - ARENA_CENTER.y).abs() <= 28.0,
            Self::Bottom => position.y >= 246.0 && (position.x - ARENA_CENTER.x).abs() <= 28.0,
            Self::Left => position.x <= 34.0 && (position.y - ARENA_CENTER.y).abs() <= 28.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpikeTrap {
    pub position: Vec2,
    pub phase: f32,
    hit_this_cycle: bool,
}

impl SpikeTrap {
    fn new(position: Vec2, phase: f32) -> Self {
        Self {
            position,
            phase: phase.rem_euclid(TRAP_CYCLE),
            hit_this_cycle: false,
        }
    }

    pub fn is_active(&self) -> bool {
        (TRAP_ACTIVE_START..TRAP_ACTIVE_END).contains(&self.phase)
    }

    pub fn is_warning(&self) -> bool {
        (TRAP_WARNING_START..TRAP_ACTIVE_START).contains(&self.phase)
    }

    pub fn height(&self) -> f32 {
        match self.phase {
            phase if phase < TRAP_WARNING_START => 0.0,
            phase if phase < TRAP_ACTIVE_START => {
                (phase - TRAP_WARNING_START) / (TRAP_ACTIVE_START - TRAP_WARNING_START) * 0.55
            }
            phase if phase < TRAP_ACTIVE_END => 1.0,
            phase if phase < TRAP_RETRACT_END => {
                1.0 - (phase - TRAP_ACTIVE_END) / (TRAP_RETRACT_END - TRAP_ACTIVE_END)
            }
            _ => 0.0,
        }
    }

    fn update(&mut self, player: &Player, dt: f32) -> f32 {
        let previous_phase = self.phase;
        self.phase = (self.phase + dt).rem_euclid(TRAP_CYCLE);
        if self.phase < previous_phase {
            self.hit_this_cycle = false;
        }

        if self.is_active()
            && !self.hit_this_cycle
            && self.position.distance(player.position) <= 19.0
        {
            self.hit_this_cycle = true;
            14.0
        } else {
            0.0
        }
    }
}

#[derive(Clone, Debug)]
pub struct Room {
    pub number: u32,
    pub exit: ExitSide,
    pub exit_open: bool,
    pub traps: Vec<SpikeTrap>,
    clear_reward_claimed: bool,
}

impl Room {
    pub fn new(number: u32) -> Self {
        let number = number.max(1);
        let mut seed = number
            .wrapping_mul(2_246_822_519)
            .wrapping_add(3_266_489_917);
        let trap_count = ((number - 1) / 2).min(4) as usize;
        let mut traps = Vec::with_capacity(trap_count);

        for _ in 0..trap_count {
            let x = 56.0 + next_random(&mut seed) * 368.0;
            let y = 60.0 + next_random(&mut seed) * 150.0;
            let mut position = vec2(x, y);
            if position.distance(ARENA_CENTER) < 52.0 {
                position.x = if position.x < ARENA_CENTER.x {
                    position.x - 60.0
                } else {
                    position.x + 60.0
                };
            }
            position.x = position.x.clamp(56.0, 424.0);
            let phase = next_random(&mut seed) * TRAP_CYCLE;
            traps.push(SpikeTrap::new(position, phase));
        }

        Self {
            number,
            exit: ExitSide::from_room_number(number),
            exit_open: false,
            traps,
            clear_reward_claimed: false,
        }
    }

    pub fn update_traps(&mut self, player: &Player, dt: f32) -> f32 {
        self.traps
            .iter_mut()
            .map(|trap| trap.update(player, dt))
            .sum()
    }

    pub fn open_exit(&mut self) {
        self.exit_open = true;
    }

    pub fn claim_clear_reward(&mut self) -> bool {
        if self.clear_reward_claimed {
            return false;
        }
        self.clear_reward_claimed = true;
        true
    }

    pub fn can_leave(&self, player_position: Vec2) -> bool {
        self.exit_open && self.exit.exit_reached(player_position)
    }
}

fn next_random(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*seed >> 8) as f32 / ((u32::MAX >> 8) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_only_allows_transition_after_the_room_is_cleared() {
        let mut room = Room::new(1);
        let exit_position = room.exit.exit_position();

        assert!(!room.can_leave(exit_position));
        room.open_exit();
        assert!(room.can_leave(exit_position));
    }

    #[test]
    fn room_clear_reward_can_only_be_claimed_once() {
        let mut room = Room::new(1);
        assert!(room.claim_clear_reward());
        assert!(!room.claim_clear_reward());
    }

    #[test]
    fn trap_hits_once_per_active_cycle() {
        let player = Player::new();
        let mut trap = SpikeTrap::new(player.position, TRAP_ACTIVE_START);

        assert_eq!(trap.update(&player, 0.01), 14.0);
        assert_eq!(trap.update(&player, 0.01), 0.0);
    }

    #[test]
    fn room_generation_is_repeatable_and_scales_trap_count() {
        let room = Room::new(5);
        let same_room = Room::new(5);
        assert_eq!(room.exit, same_room.exit);
        assert_eq!(room.traps.len(), 2);
        assert_eq!(room.traps[0].position, same_room.traps[0].position);
    }

    #[test]
    fn generated_traps_stay_inside_the_arena_and_away_from_entry_edges() {
        for number in 1..=30 {
            let room = Room::new(number);
            for trap in room.traps {
                assert!((56.0..=424.0).contains(&trap.position.x));
                assert!((60.0..=210.0).contains(&trap.position.y));
            }
        }
    }
}
