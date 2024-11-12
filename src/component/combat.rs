use std::ops::{Add, AddAssign};

use num::Num;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::game::RANGE_EPSILON;

use super::{attributes::Attributes, items::Inventory};
use crate::error::{Error, Result};

const DEPTH_MULTIPLIER: f64 = 1.1;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DamageRange<T: Num>(pub T, pub T);

impl From<DamageRange<u32>> for DamageRange<i32> {
    fn from(value: DamageRange<u32>) -> Self {
        Self(value.0 as i32, value.1 as i32)
    }
}

impl From<DamageRange<i32>> for DamageRange<u32> {
    fn from(value: DamageRange<i32>) -> Self {
        Self(value.0 as u32, value.1 as u32)
    }
}

impl From<DamageRange<u32>> for DamageRange<f64> {
    fn from(value: DamageRange<u32>) -> Self {
        Self(value.0 as f64, value.1 as f64)
    }
}

impl From<DamageRange<f64>> for DamageRange<u32> {
    fn from(value: DamageRange<f64>) -> Self {
        Self(value.0 as u32, value.1 as u32)
    }
}

impl<T: Num + Copy> Add<T> for DamageRange<T> {
    type Output = DamageRange<T>;
    fn add(self, rhs: T) -> Self::Output {
        DamageRange(self.0 + rhs, self.1 + rhs)
    }
}

impl<T: Num + Copy> Add<DamageRange<T>> for DamageRange<T> {
    type Output = DamageRange<T>;
    fn add(self, rhs: DamageRange<T>) -> Self::Output {
        DamageRange(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T: Num + Copy + AddAssign> AddAssign<T> for DamageRange<T> {
    fn add_assign(&mut self, rhs: T) {
        self.0 += rhs;
        self.1 += rhs;
    }
}

impl<T: Num + Copy + AddAssign> AddAssign<DamageRange<T>> for DamageRange<T> {
    fn add_assign(&mut self, rhs: DamageRange<T>) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DamageType {
    Physical,
    Burning,
    Magical,
}

#[derive(Debug, Clone, Copy)]
pub enum AttackType {
    Melee,
    Ranged,
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub damage_min: u32,
    pub damage_max: u32,
    pub damage_type: DamageType,
    pub attack_type: AttackType,
    pub range: f64,
}

impl Attack {
    pub fn new(
        min: u32,
        max: u32,
        damage_type: DamageType,
        attack_type: AttackType,
        range: f64,
    ) -> Self {
        Self {
            damage_min: min,
            damage_max: max,
            damage_type,
            attack_type,
            range: range + RANGE_EPSILON,
        }
    }

    pub fn new_melee(min: u32, max: u32) -> Self {
        Self {
            damage_min: min,
            damage_max: max,
            damage_type: DamageType::Physical,
            attack_type: AttackType::Melee,
            range: 1.0 + RANGE_EPSILON,
        }
    }

    pub fn new_ranged(min: u32, max: u32) -> Self {
        Self {
            damage_min: min,
            damage_max: max,
            damage_type: DamageType::Physical,
            attack_type: AttackType::Ranged,
            range: 5.0 + RANGE_EPSILON,
        }
    }

    pub fn optional_melee(values: Option<DamageRange<u32>>) -> Option<Self> {
        match values {
            Some(DamageRange(min, max)) => Some(Attack::new_melee(min, max)),
            None => None,
        }
    }

    pub fn optional_ranged(values: Option<DamageRange<u32>>) -> Option<Self> {
        match values {
            Some(DamageRange(min, max)) => Some(Attack::new_ranged(min, max)),
            None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Combat {
    pub melee_attack: Option<Attack>,
    pub ranged_attack: Option<Attack>,
}

impl Combat {
    pub fn new(melee: Option<Attack>, ranged: Option<Attack>) -> Self {
        Self {
            melee_attack: melee,
            ranged_attack: ranged,
        }
    }

    pub fn new_melee(attack: Attack) -> Self {
        Self {
            melee_attack: Some(attack),
            ranged_attack: None,
        }
    }

    pub fn new_ranged(attack: Attack) -> Self {
        Self {
            melee_attack: None,
            ranged_attack: Some(attack),
        }
    }
}

pub struct AttackReport {
    pub damage: u32,
    pub damage_type: DamageType,
    pub attack_type: AttackType,
    pub hit_message: &'static str,
}

pub type AttackFunction = fn(
    combat: &Combat,
    stats: Option<&Attributes>,
    items: Option<&Inventory>,
) -> Result<AttackReport>;

pub fn calculate_melee_attack(
    combat: &Combat,
    stats: Option<&Attributes>,
    items: Option<&Inventory>,
) -> Result<AttackReport> {
    let Some(attack) = &combat.melee_attack else {
        return Err("Attacker has no melee attack.".into());
    };

    Ok(calculate_attack(attack, stats, items))
}

pub fn calculate_ranged_attack(
    combat: &Combat,
    stats: Option<&Attributes>,
    items: Option<&Inventory>,
) -> Result<AttackReport> {
    let Some(attack) = &combat.ranged_attack else {
        return Err("Attacker has no melee attack.".into());
    };

    Ok(calculate_attack(attack, stats, items))
}

pub fn calculate_attack(
    attack: &Attack,
    stats: Option<&Attributes>,
    items: Option<&Inventory>,
) -> AttackReport {
    let hit_message = match attack.attack_type {
        AttackType::Melee => "attacked",
        AttackType::Ranged => "shot",
        
    };

    let mut damage = thread_rng().gen_range(attack.damage_min..=attack.damage_max) as f64;

    if let Some(stats) = stats {
        damage = super::attributes::attack_damage_boost(damage, attack, stats);
    }

    if let Some(items) = items {
        damage = super::items::item_damage_boost(damage, attack, items);
    }

    AttackReport {
        damage: damage as u32,
        damage_type: attack.damage_type,
        attack_type: attack.attack_type,
        hit_message
    }
}

pub fn calculate_armor_reduction(
    attack: &AttackReport,
    stats: Option<&Attributes>,
    items: Option<&Inventory>,
) -> f64 {
    attack.damage as f64
}

pub fn get_adjusted_damage(listed_damage: u32, depth: u32) -> u32 {
    let damage_increase = DEPTH_MULTIPLIER.powf((depth-1) as f64) as u32;
    listed_damage + damage_increase
}