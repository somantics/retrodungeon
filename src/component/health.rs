use rand::{thread_rng, Rng};

pub const LEVEL_UP_MULTIPLIER: f64 = 1.15;
const DEPTH_MULTIPLIER: f64 = 1.1;
const RNG_SPAN: f64 = 0.1;

#[derive(Debug, Clone, Copy)]
pub struct Health(pub u32, pub u32);

impl Health {
    pub fn new(max: u32) -> Self {
        Health(max, max)
    }

    pub fn add_max(&mut self, amount: u32) {
        self.0 += amount;
        self.1 += amount;
    }

    pub fn add_current(&mut self, amount: u32) {
        self.0 = (self.0 + amount).min(self.1)
    }

    pub fn sub_current(&mut self, amount: u32) {
        if self.0 > amount {
            self.0 = self.0 - amount;
        } else {
            self.0 = 0;
        }
    }

    pub fn reset_to_max(&mut self) {
        self.0 = self.1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Full,
    Hurt,
    Dead,
}

impl From<Health> for HealthStatus {
    fn from(value: Health) -> Self {
        match value.0 {
            current if current == value.1 => HealthStatus::Full,
            current if current <= 0 => HealthStatus::Dead,
            _ => HealthStatus::Hurt,
        }
    }
}


pub fn get_adjusted_health(listed_health: u32, depth: u32) -> u32 {
    let depth_adjusted = listed_health as f64 * DEPTH_MULTIPLIER.powf((depth-1) as f64);
    let random_range = (1.0 - RNG_SPAN)..(1.0 + RNG_SPAN);
    let randomized = thread_rng().gen_range(random_range) * depth_adjusted;
    randomized as u32
}