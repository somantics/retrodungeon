use serde::{Deserialize, Serialize};

use super::combat::{Attack, AttackType};

const MIGHT_ATTACK_BONUS: f64 = 2.4;
const SKILL_ATTACK_BONUS: f64 = 1.6;
const WIT_DAMAGE_BONUS: f64 = 3.6;

pub const ATTRIBUTE_MINIMUM: u32 = 1; 

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Attributes {
    pub might: u32,
    pub wit: u32,
    pub skill: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Xp {
    pub level: u32,
    pub current: u32,
    pub max: u32,
    pub status: XpStatus,
}

impl Xp {
    pub fn new(level: u32) -> Self {
        Self {
            level,
            current: 0,
            max: next_level_requirement(level),
            status: XpStatus::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum XpStatus {
    LevelUp,
    Default,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Attribute {
    Might,
    Wit,
    Skill,
}

pub fn attack_damage_boost(in_damage: f64, attack: &Attack, stats: &Attributes) -> f64 {
    let (original_stat, per_stat_bonus) = match attack.attack_type {
        AttackType::Melee => (stats.might, MIGHT_ATTACK_BONUS),
        AttackType::Ranged => (stats.skill, SKILL_ATTACK_BONUS),
    };

    let extra_stat = match original_stat {
        stat if stat < ATTRIBUTE_MINIMUM =>  {
            0.0
        },
        stat => {
            (stat - ATTRIBUTE_MINIMUM) as f64
        }
    };

    in_damage + extra_stat * per_stat_bonus
}

pub fn spell_damage_boost(stats: &Attributes) -> u32 {

    let extra_wit = match stats.wit {
        stat if stat < ATTRIBUTE_MINIMUM =>  {
            0.0
        },
        stat => {
            (stat - ATTRIBUTE_MINIMUM) as f64
        }
    };

    (extra_wit * WIT_DAMAGE_BONUS) as u32
}

pub fn next_level_requirement(level: u32) -> u32 {
    (0..=level).sum::<u32>() * 100
}
