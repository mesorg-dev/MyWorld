#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillUpgrade {
    TimeStop,
    KnifeCooldown,
    DashCooldown,
    RoadRoller,
}

impl SkillUpgrade {
    pub fn label(self) -> &'static str {
        match self {
            Self::TimeStop => "TIME STOP",
            Self::KnifeCooldown => "KNIFE",
            Self::DashCooldown => "DASH",
            Self::RoadRoller => "ROAD ROLLER",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkillTree {
    pub time_stop_level: u8,
    pub knife_level: u8,
    pub dash_level: u8,
    pub road_roller_unlocked: bool,
}

impl SkillTree {
    pub const MAX_LEVEL: u8 = 3;

    pub fn new() -> Self {
        Self {
            time_stop_level: 0,
            knife_level: 0,
            dash_level: 0,
            road_roller_unlocked: false,
        }
    }

    pub fn available_upgrades(&self) -> Vec<SkillUpgrade> {
        [
            SkillUpgrade::TimeStop,
            SkillUpgrade::KnifeCooldown,
            SkillUpgrade::DashCooldown,
            SkillUpgrade::RoadRoller,
        ]
        .into_iter()
        .filter(|upgrade| match upgrade {
            SkillUpgrade::RoadRoller => !self.road_roller_unlocked,
            _ => self.level(*upgrade) < Self::MAX_LEVEL,
        })
        .collect()
    }

    pub fn level(&self, upgrade: SkillUpgrade) -> u8 {
        match upgrade {
            SkillUpgrade::TimeStop => self.time_stop_level,
            SkillUpgrade::KnifeCooldown => self.knife_level,
            SkillUpgrade::DashCooldown => self.dash_level,
            SkillUpgrade::RoadRoller => u8::from(self.road_roller_unlocked),
        }
    }

    pub fn upgrade(&mut self, upgrade: SkillUpgrade) -> Option<u8> {
        if upgrade == SkillUpgrade::RoadRoller {
            if self.road_roller_unlocked {
                return None;
            }
            self.road_roller_unlocked = true;
            return Some(1);
        }

        let level = match upgrade {
            SkillUpgrade::TimeStop => &mut self.time_stop_level,
            SkillUpgrade::KnifeCooldown => &mut self.knife_level,
            SkillUpgrade::DashCooldown => &mut self.dash_level,
            SkillUpgrade::RoadRoller => unreachable!(),
        };
        if *level >= Self::MAX_LEVEL {
            return None;
        }
        *level += 1;
        Some(*level)
    }
}

impl Default for SkillTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrades_are_offered_until_their_branch_reaches_max_level() {
        let mut skills = SkillTree::new();
        assert_eq!(skills.available_upgrades().len(), 4);

        for _ in 0..SkillTree::MAX_LEVEL {
            assert!(skills.upgrade(SkillUpgrade::KnifeCooldown).is_some());
        }

        assert_eq!(
            skills.available_upgrades(),
            vec![
                SkillUpgrade::TimeStop,
                SkillUpgrade::DashCooldown,
                SkillUpgrade::RoadRoller
            ]
        );
        assert!(skills.upgrade(SkillUpgrade::KnifeCooldown).is_none());
    }
}
