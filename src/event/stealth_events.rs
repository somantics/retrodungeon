use std::collections::HashMap;

use crate::{
    component::{behavior::{Behavior, BehaviorState}, responses::NoiseResponse, Position}, error::Result, map::GameMap, resources::ResourceManager, world::World
};

use super::{Event, EventArguments};

pub struct NoiseEvent {
    pub source: usize,
    pub magnitude: u32,
}

impl NoiseEvent {
    pub fn new(source: usize, magnitude: u32) -> Self {
        Self { source, magnitude }
    }
}

impl Event for NoiseEvent {
    type Response = NoiseResponse;
    fn apply(
        &self,
        event_data: EventArguments,
    ) -> Result<()> {
        try_wake_up(event_data.target, event_data.source, self.magnitude, event_data.world)?;
        Ok(())
    }

    fn source(&self) -> usize {
        self.source
    }
}


pub fn try_wake_up(own_entity: usize, source: usize, magnitude: u32, world: &mut World) -> Result<()> {
    let Some(Position(own_position)) = world.borrow_entity_component::<Position>(own_entity) else {
        return Err(format!("Entity not found {own_entity}").into());
    };
    let own_position = own_position.clone();

    let Some(Position(source_position)) = world.borrow_entity_component::<Position>(source) else {
        return Err(format!("Entity not found {own_entity}").into());
    };
    let source_position = source_position.clone();

    let Some(NoiseResponse { threshold, ..}) = world.borrow_entity_component::<NoiseResponse>(own_entity) else {
        return Err(format!("Tried to wake entity without noise response: {own_entity}").into());
    };
    let threshold = threshold.clone();

    let Some(Behavior { state, ..}) = world.borrow_entity_component_mut::<Behavior>(own_entity) else {
        return Err(format!("Tried to wake entity without behavior: {own_entity}").into());
    };
    *state = BehaviorState::Awake;

    // let distance = own_position.distance(source_position);
    // if threshold <= magnitude - distance as u32 {
    //     *state = BehaviorState::Awake;

    //     let name = world.borrow_entity_component(own_entity);
    //     let msg = logger::generate_wake_up_message(name);
    //     logger::log_message(&msg);
    // } else {
    //     let name = world.borrow_entity_component(own_entity);
    //     let msg = logger::generate_sleep_message(name);
    //     logger::log_message(&msg);
    // }

    Ok(())
}