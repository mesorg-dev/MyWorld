#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillUpgrade {
    TimeStop,
    KnifeCooldown,
    DashCooldown,
}

impl SkillUpgrade {
    pub fn label(self) -> &'static str {
        match self {
            Self::TimeStop => "TIME STOP",
            Self::KnifeCooldown => "KNIFE",
            Self::DashCooldown => "DASH",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkillTree {
    pub time_stop_level: u8,
    pub knife_level: u8,
    pub dash_level: u8,
}

impl SkillTree {
    pub const MAX_LEVEL: u8 = 3;

    pub fn new() -> Self {
        Self {
            time_stop_level: 0,
            knife_level: 0,
            dash_level: 0,
        }
    }

    pub fn available_upgrades(&self) -> Vec<SkillUpgrade> {
        [
            SkillUpgrade::TimeStop,
            SkillUpgrade::KnifeCooldown,
            SkillUpgrade::DashCooldown,
        ]
        .into_iter()
        .filter(|upgrade| self.level(*upgrade) < Self::MAX_LEVEL)
        .collect()
    }

    pub fn level(&self, upgrade: SkillUpgrade) -> u8 {
        match upgrade {
            SkillUpgrade::TimeStop => self.time_stop_level,
            SkillUpgrade::KnifeCooldown => self.knife_level,
            SkillUpgrade::DashCooldown => self.dash_level,
        }
    }

    pub fn upgrade(&mut self, upgrade: SkillUpgrade) -> Option<u8> {
        let level = match upgrade {
            SkillUpgrade::TimeStop => &mut self.time_stop_level,
            SkillUpgrade::KnifeCooldown => &mut self.knife_level,
            SkillUpgrade::DashCooldown => &mut self.dash_level,
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
        assert_eq!(skills.available_upgrades().len(), 3);

        for _ in 0..SkillTree::MAX_LEVEL {
            assert!(skills.upgrade(SkillUpgrade::KnifeCooldown).is_some());
        }

        assert_eq!(
            skills.available_upgrades(),
            vec![SkillUpgrade::TimeStop, SkillUpgrade::DashCooldown]
        );
        assert!(skills.upgrade(SkillUpgrade::KnifeCooldown).is_none());
    }
}
