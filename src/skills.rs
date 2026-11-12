#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SkillNode {
    pub id: &'static str,
    pub name: &'static str,
    pub prereq_ids: Vec<&'static str>,
    pub unlocked: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SkillTree {
    pub nodes: Vec<SkillNode>,
}

#[allow(dead_code)]
impl SkillTree {
    pub fn new() -> Self {
        let nodes = vec![
            SkillNode {
                id: "chronos-awakening",
                name: "Awakening",
                prereq_ids: Vec::new(),
                unlocked: true,
            },
            SkillNode {
                id: "chronos-acceleration",
                name: "Acceleration",
                prereq_ids: vec!["chronos-awakening"],
                unlocked: false,
            },
            SkillNode {
                id: "blade-belt",
                name: "Full Belt",
                prereq_ids: vec!["chronos-awakening"],
                unlocked: false,
            },
            SkillNode {
                id: "road-roller",
                name: "Road Roller",
                prereq_ids: vec!["blade-belt"],
                unlocked: false,
            },
        ];

        Self { nodes }
    }

    pub fn unlock(&mut self, id: &str) -> bool {
        let Some(idx) = self.nodes.iter().position(|node| node.id == id) else {
            return false;
        };

        let prereqs_met = self.nodes.iter().filter(|node| node.id == id).all(|node| {
            node.prereq_ids.iter().all(|prereq| {
                self.nodes
                    .iter()
                    .any(|candidate| candidate.id == *prereq && candidate.unlocked)
            })
        });

        if prereqs_met {
            self.nodes[idx].unlocked = true;
            true
        } else {
            false
        }
    }
}
