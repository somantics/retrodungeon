use log::debug;
use serde::{Deserialize, Serialize};

use crate::{map::GameMap, resources::ResourceManager, world::World};

pub mod definitions;
pub mod error;

use crate::error::Result;

pub type System =
    fn(world: &mut World, map: &mut GameMap, resources: &ResourceManager) -> Result<()>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSlot {
    EndOfTurn,
    EndOfLevel,
    EndOfRoom,
}

pub struct Scheduler {
    turn_end_systems: Vec<System>,
    floor_end_systems: Vec<System>,
    room_end_systems: Vec<System>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            turn_end_systems: Vec::new(),
            floor_end_systems: Vec::new(),
            room_end_systems: Vec::new(),
        }
    }

    pub fn add_system(&mut self, system: System, time: TimeSlot) {
        match time {
            TimeSlot::EndOfTurn => {
                self.turn_end_systems.push(system);
            }
            TimeSlot::EndOfLevel => {
                self.floor_end_systems.push(system);
            }
            TimeSlot::EndOfRoom => {
                self.room_end_systems.push(system);
            }
        }
    }

    pub fn on_end_turn(
        &self,
        world: &mut World,
        map: &mut GameMap,
        resources: &ResourceManager,
    ) -> Result<()> {
        Self::run_systems(&self.turn_end_systems, world, map, resources)?;
        
        Ok(())
    }

    pub fn on_descend_floor(
        &self,
        world: &mut World,
        map: &mut GameMap,
        resources: &ResourceManager,
    ) -> Result<()> {
        Self::run_systems(&self.floor_end_systems, world, map, resources)
    }

    pub fn on_clear_room(
        &self,
        world: &mut World,
        map: &mut GameMap,
        resources: &ResourceManager,
    ) -> Result<()> {
        Self::run_systems(&self.room_end_systems, world, map, resources)
    }

    fn run_systems(
        systems: &Vec<System>,
        world: &mut World,
        map: &mut GameMap,
        resources: &ResourceManager,
    ) -> Result<()> {
        for system in systems {
            let result = (system)(world, map, resources);
            match result {
                Err(error) => debug!("{error}"),
                Ok(_) => {}
            }
        }
        Ok(())
    }
}
