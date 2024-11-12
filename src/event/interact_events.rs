use std::collections::HashMap;

use crate::{
    component::responses::{InteractResponse, PickupResponse}, error::Result, map::GameMap, resources::ResourceManager, world::World
};

use super::{Event, EventArguments};


pub struct InteractEvent {
    pub source: usize,
}

impl Event for InteractEvent {
    type Response = InteractResponse;
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


pub struct PickupEvent {
    pub source: usize,
}

impl Event for PickupEvent {
    type Response = PickupResponse;
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



