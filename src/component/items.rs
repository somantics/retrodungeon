use std::ops::{Add, AddAssign};

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use super::combat::Attack;

const DEPTH_MULTIPLIER: f64 = 1.2;
const RNG_SPAN: f64 = 0.2;

#[derive(Debug, Clone)]
pub struct Inventory;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coins(pub u32);

impl Add<Coins> for Coins {
    type Output = Coins;

    fn add(self, rhs: Coins) -> Self::Output {
        Coins(self.0 + rhs.0)
    }
}

impl AddAssign for Coins {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

pub fn item_damage_boost(in_damage: f64, _attack: &Attack, _stats: &Inventory) -> f64 {
    in_damage
}

pub fn get_adjusted_coins(listed_coins: u32, depth: u32) -> u32 {
    let depth_adjusted = listed_coins as f64 * DEPTH_MULTIPLIER.powf((depth-1) as f64);
    let random_range = (1.0 - RNG_SPAN)..(1.0 + RNG_SPAN);
    let randomized = thread_rng().gen_range(random_range) * depth_adjusted;
    randomized as u32
}