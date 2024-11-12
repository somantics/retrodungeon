#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EffectType {
    Burning,
}

pub struct EffectHandler {
    active_effects: Vec<Effect>,
}

impl EffectHandler {
    pub fn add_effect(&mut self, effect: Effect) {
        if let Some(existing) = self
            .active_effects
            .iter_mut()
            .find(|elem| elem.same_type(&effect))
        {
            existing.merge_with(&effect);
        } else {
            self.active_effects.push(effect);
        }
    }

    pub fn tick_all(&mut self) {
        for effect in self.active_effects.iter_mut() {
            effect.tick(1);
        }
    }
}

pub struct Effect {
    effect_type: EffectType,
    duration: Duration,
}

impl Effect {
    fn same_type(&self, other: &Effect) -> bool {
        self.effect_type == other.effect_type
    }

    fn merge_with(&mut self, other: &Effect) {
        self.duration += other.duration;
    }

    fn tick(&mut self, amount: u32) {
        self.duration -= Duration(amount)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Duration(pub u32);

impl std::ops::Add<Duration> for Duration {
    type Output = Duration;
    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign<Duration> for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        self.0 += rhs.0
    }
}

impl std::ops::Sub<Duration> for Duration {
    type Output = Duration;
    fn sub(self, rhs: Duration) -> Self::Output {
        Self((self.0 - rhs.0).max(0))
    }
}

impl std::ops::SubAssign<Duration> for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        self.0 = (self.0 - rhs.0).max(0)
    }
}
