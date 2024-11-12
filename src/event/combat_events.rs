use std::collections::HashMap;

use crate::{
    component::{self, attributes::Attributes, combat::Combat, health::Health, items::Inventory, responses::{AttackResponse, DeathResponse, ShootResponse}, Name}, error::Result, logger, map::GameMap, resources::ResourceManager, world::World
};

use super::{Event, EventArguments};

pub const ARG_DAMAGE_MULTIPLIER: &'static str = "DMG_MULTIPLIER";
pub const ARG_DAMAGE_MULTIPLIER_OVERRIDE: &'static str = "DMG_MULTIPLIER_OVERRIDE";

pub struct AttackEvent {
    pub source: usize,
}

impl AttackEvent {
    pub fn new(source: usize) -> Self {
        Self { source }
    }
}

impl Event for AttackEvent {
    type Response = AttackResponse;
    fn apply(
        &self,
        event_data: EventArguments,
    ) -> Result<()> {
        apply_attack(
            event_data.world,
            event_data.source,
            event_data.target,
            component::combat::calculate_melee_attack,
            event_data.args,
        )
    }

    fn source(&self) -> usize {
        self.source
    }
}


pub struct ShootEvent {
    pub source: usize,
}

impl ShootEvent {
    pub fn new(source: usize) -> Self {
        Self { source }
    }
}

impl Event for ShootEvent {
    type Response = ShootResponse;
    fn apply(
        &self,
        event_data: EventArguments,
    ) -> Result<()> {
        apply_attack(
            event_data.world,
            event_data.source,
            event_data.target,
            component::combat::calculate_ranged_attack,
            event_data.args,
        )
    }

    fn source(&self) -> usize {
        self.source
    }
}

pub struct DeathEvent {
    pub source: usize,
}

impl Event for DeathEvent {
    type Response = DeathResponse;
    fn apply(
        &self,
        _event_data: EventArguments,
    ) -> Result<()> {
        Ok(())
    }

    fn source(&self) -> usize {
        self.source
    }
}

fn apply_attack(
    world: &mut World,
    source: usize,
    target: usize,
    attack: component::combat::AttackFunction,
    args: &HashMap<String, f64>,
) -> Result<()> {
    let Some(combat) = world.borrow_entity_component::<Combat>(source) else {
        return Err("Attacker has no combat component".into());
    };
    let attributes = world.borrow_entity_component::<Attributes>(source);
    let items = world.borrow_entity_component::<Inventory>(source);

    let attack_report = (attack)(combat, attributes, items)?;
    let mut damage = attack_report.damage as f64;
    let hit_message = attack_report.hit_message;
    let mut message_addendum = "";

    if let Some(dmg_multiplier) = args.get(ARG_DAMAGE_MULTIPLIER_OVERRIDE) {
        damage *= dmg_multiplier;

        if *dmg_multiplier > 1.05 {
            message_addendum = "It's very effective.";
        } else if *dmg_multiplier < 0.95 {
            message_addendum = "It seems to have little effect.";
        }
    } else if let Some(dmg_multiplier) = args.get(ARG_DAMAGE_MULTIPLIER) {
        damage *= dmg_multiplier;

        
        if *dmg_multiplier > 1.05 {
            message_addendum = "It's very effective.";
        } else if *dmg_multiplier < 0.95 {
            message_addendum = "It seems to have little effect.";
        }
    }

    let Some(health_vec) = world.borrow_component_vec_mut::<Health>() else {
        return Err("No storage for health components".into());
    };
    let Some(ref mut health) = health_vec[target] else {
        return Err("Defender has no health component".into());
    };
    let damage = damage as u32;
    health.sub_current(damage);

    let attacker_name = world.borrow_entity_component::<Name>(source);
    let defender_name = world.borrow_entity_component::<Name>(target);

    let log_msg = logger::generate_attack_message(
        attacker_name, 
        defender_name, 
        hit_message, message_addendum,  
        damage);
    logger::log_message(&log_msg);

    Ok(())
}