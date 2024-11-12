use crate::{resources::id::RoomTemplateID, spawning::spawnentry::SpawnEntryType};

use super::{boxextends::BoxExtends, utils::Coordinate};

#[derive(Debug, Default, Clone)]
pub struct Room {
    pub extends: BoxExtends,
    pub door_locations: Vec<Coordinate>,
    pub template: Option<RoomTemplateID>,
    pub room_depth: Option<u32>,
    pub extra_spawn: Option<SpawnEntryType>,
}

impl Room {
    pub fn new(extends: BoxExtends) -> Self {
        Self {
            extends,
            door_locations: vec![],
            template: None,
            room_depth: None,
            extra_spawn: None,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EntityContainer {
    pub extends: BoxExtends,
    pub entities: Vec<usize>,
}

impl From<Room> for EntityContainer {
    fn from(value: Room) -> Self {
        Self {
            extends: value.extends.to_owned(),
            entities: Vec::new(),
        }
    }
}

impl From<&Room> for EntityContainer {
    fn from(value: &Room) -> Self {
        Self {
            extends: value.extends.to_owned(),
            entities: Vec::new(),
        }
    }
}

pub struct RoomGenerationData {
    pub room: EntityContainer,
    pub level_depth: u32,
    pub room_depth: u32,
}

impl RoomGenerationData {
    pub fn get_area(&self) -> u32 {
        self.room.extends.get_inner_area() as u32
    }
}

pub struct QuestGenerationData {}
