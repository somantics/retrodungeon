use std::collections::HashMap;

use crate::{
    component::{self, attributes::Attributes, combat::Combat, health::Health, items::Inventory, responses::{AttackResponse, DeathResponse, ShootResponse}, Name}, error::Result, logger, map::GameMap, resources::ResourceManager, world::World
};
use super::{argument_names::{ARG_DAMAGE_MULTIPLIER, ARG_DAMAGE_MULTIPLIER_OVERRIDE, MSG_ARG_ADDENDUM, MSG_ARG_ADDENDUM_OVERRIDE, MSG_ARG_ATTACKER, MSG_ARG_ATTACK_MESSAGE}, Event, EventArguments};

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
            event_data.msg_args,
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
            event_data.msg_args,
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
    msg_args: &HashMap<String, String>,
) -> Result<()> {
    let Some(combat) = world.borrow_entity_component::<Combat>(source) else {
        return Err("Attacker has no combat component".into());
    };
    let attributes = world.borrow_entity_component::<Attributes>(source);
    let items = world.borrow_entity_component::<Inventory>(source);
    let attack_report = (attack)(combat, attributes, items)?;
    let damage = attack_report.damage as f64;

    let mut multiplier = 1.0;
    let mut hit_message = attack_report.hit_message;
    let mut message_addendum = "";

    // CHECK FLOAT ARGS
    if let Some(dmg_multiplier) = args.get(ARG_DAMAGE_MULTIPLIER_OVERRIDE) {
        multiplier = *dmg_multiplier;
    } else if let Some(dmg_multiplier) = args.get(ARG_DAMAGE_MULTIPLIER) {
        multiplier = *dmg_multiplier;
    }


    let Some(health_vec) = world.borrow_component_vec_mut::<Health>() else {
        return Err("No storage for health components".into());
    };
    let Some(ref mut health) = health_vec[target] else {
        return Err("Defender has no health component".into());
    };
    let damage = (damage * multiplier) as u32;
    health.sub_current(damage);

    let mut attacker_name = world.borrow_entity_component::<Name>(source);
    let defender_name = world.borrow_entity_component::<Name>(target);

    // CHECK MSG ARGS
    if let Some(entity_as_str) = msg_args.get(MSG_ARG_ATTACKER.into()) {
        if let Ok(entity) = entity_as_str.parse::<usize>() {
            attacker_name = world.borrow_entity_component::<Name>(entity);
        }
    };

    if let Some(msg) = msg_args.get(MSG_ARG_ATTACK_MESSAGE.into()) {
        hit_message = msg;
    };

    if let Some(msg) = msg_args.get(MSG_ARG_ADDENDUM_OVERRIDE.into()) {
        message_addendum = msg;
    };

    if let Some(addendum) = msg_args.get(MSG_ARG_ADDENDUM) {
        message_addendum = addendum;
    } else {
        if multiplier > 1.05 {
            message_addendum = "It's very effective.";
        } else if multiplier < 0.95 {
            message_addendum = "It seems to have little effect.";
        }
    }

    let log_msg = logger::generate_attack_message(
        attacker_name, 
        defender_name, 
        hit_message, message_addendum,  
        damage);
    logger::log_message(&log_msg);

    Ok(())
}