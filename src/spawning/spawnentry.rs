use itertools::Itertools;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Debug};

use crate::error::Result;
use crate::{
    component::{tags::Door, Position},
    map::{
        room::{EntityContainer, QuestGenerationData, RoomGenerationData},
        utils::{Coordinate, DOWN, LEFT, RIGHT, UP},
        GameMap,
    },
    resources::{
        id::{SpawnEntryID, SpawnableID},
        ResourceManager, PLAYER_SPAWNENTRY, STAIRS_SPAWNENTRY,
    },
    world::World,
};

pub type SpawnFunction = fn(&mut World, depth: u32, position: Coordinate) -> Result<()>;

// Spawns tables work like loot tables
// each spawn entry represents a catergory
// which may contain:
//  concrete spawnables ( x to y of type z )
//  further categories
//  room requirements
//  location requirements
//
// spawning happens in order, completely resolving an entry before moving on to the next in the list
// failure on a requirement means that entry will be skipped
// if there are no more entries in that category that category is now done spawning
// requirements are checked before evaluating sub entries

pub trait SpawnEntry: Debug {
    fn spawn(
        &self,
        room_data: &RoomGenerationData,
        quest_data: &mut QuestGenerationData,
        world: &mut World,
        map: &GameMap,
        resources: &ResourceManager,
    ) -> Result<()>;
    fn evaluate(&self, room_data: &RoomGenerationData, quest_data: &QuestGenerationData) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpawnEntryType {
    Selection(SpawnSelection),
    Concrete(ConcreteSpawn),
    Union(SpawnUnion),
    Player(u32),
    Stairs(u32),
}

impl SpawnEntry for SpawnEntryType {
    fn evaluate(&self, room_data: &RoomGenerationData, quest_data: &QuestGenerationData) -> bool {
        match self {
            Self::Concrete(spawn) => spawn.evaluate(room_data, quest_data),
            Self::Selection(spawn) => spawn.evaluate(room_data, quest_data),
            Self::Union(spawn) => spawn.evaluate(room_data, quest_data),
            Self::Player(_) => true,
            Self::Stairs(_) => true,
        }
    }

    fn spawn(
        &self,
        room_data: &RoomGenerationData,
        quest_data: &mut QuestGenerationData,
        world: &mut World,
        map: &GameMap,
        resources: &ResourceManager,
    ) -> Result<()> {
        match self {
            Self::Concrete(spawn) => spawn.spawn(room_data, quest_data, world, map, resources),
            Self::Selection(spawn) => spawn.spawn(room_data, quest_data, world, map, resources),
            Self::Union(spawn) => spawn.spawn(room_data, quest_data, world, map, resources),
            Self::Player(_) => {
                let Some(spawn) = resources.get_spawn_entry(PLAYER_SPAWNENTRY) else {
                    return Err("Could not find spawn table for player!".into());
                };
                spawn.spawn(room_data, quest_data, world, map, resources)
            }
            Self::Stairs(_) => {
                let Some(spawn) = resources.get_spawn_entry(STAIRS_SPAWNENTRY) else {
                    return Err("Could not find spawn table for stairs!".into());
                };
                spawn.spawn(room_data, quest_data, world, map, resources)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnSelection {
    sub_entries: Vec<SpawnEntryID>,
}

impl SpawnSelection {
    pub fn new(
        sub_entries: Vec<SpawnEntryID>,
    ) -> Self {
        Self {
            sub_entries,
        }
    }
}

impl SpawnEntry for SpawnSelection {
    fn spawn(
        &self,
        room_data: &RoomGenerationData,
        quest_data: &mut QuestGenerationData,
        world: &mut World,
        map: &GameMap,
        resources: &ResourceManager,
    ) -> Result<()> {
        let random_index = thread_rng().gen_range(0.. self.sub_entries.len());
        let entry_id = self.sub_entries[random_index];

        let Some(entry) = resources.get_spawn_entry(entry_id) else {
            return Err("No span entry found.".into());
        };

        if entry.evaluate(room_data, quest_data) {
            entry.spawn(room_data, quest_data, world, map, resources)?;
        }

        Ok(())
    }

    fn evaluate(&self, _room_data: &RoomGenerationData, _quest_data: &QuestGenerationData) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnUnion {
    sub_entries: Vec<SpawnEntryID>,
}

impl SpawnUnion {
    pub fn new(
        sub_entries: Vec<SpawnEntryID>,
    ) -> Self {
        Self {
            sub_entries,
        }
    }
}

impl SpawnEntry for SpawnUnion {
    fn spawn(
        &self,
        room_data: &RoomGenerationData,
        quest_data: &mut QuestGenerationData,
        world: &mut World,
        map: &GameMap,
        resources: &ResourceManager,
    ) -> Result<()> {

        for entry in &self.sub_entries {
            let Some(entry) = resources.get_spawn_entry(*entry) else {
                continue;
            };
    
            if entry.evaluate(room_data, quest_data) {
                entry.spawn(room_data, quest_data, world, map, resources)?;
            }
        }

        Ok(())
    }

    fn evaluate(&self, _room_data: &RoomGenerationData, _quest_data: &QuestGenerationData) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcreteSpawn {
    spawnable: SpawnableID,
    min_amount: u32,
    max_amount: u32,
    location_requirements: Vec<LocationCriterion>,
}

impl ConcreteSpawn {
    pub fn new(
        spawnable: SpawnableID,
        min_amount: u32,
        max_amount: u32,
        location_requirements: Vec<LocationCriterion>,
    ) -> Self {
        Self {
            spawnable,
            min_amount,
            max_amount,
            location_requirements,
        }
    }

    fn get_viable_location(
        &self,
        room: &EntityContainer,
        world: &World,
        map: &GameMap,
        resources: &ResourceManager,
    ) -> Result<Coordinate> {
        let x_range = room.extends.top_left.x + 1..room.extends.bottom_right.x;
        let y_range = room.extends.top_left.y + 1..room.extends.bottom_right.y;

        let inside_locations = x_range
            .cartesian_product(y_range)
            .map(|(x, y)| Coordinate { x, y });

        // never allow spawning on top of other entities
        let mut filtered_locations: Vec<Coordinate> = inside_locations
            .filter(|location| world.get_entities_at_coordinate(*location).is_empty())
            .collect();

        // try to enforce criteria but prioritize being able to spawn
        for criterion in &self.location_requirements {
            let passing_locations: Vec<Coordinate> = filtered_locations
                .iter()
                .filter(|location| criterion.validate(**location, room, world, map, resources))
                .map(|location| *location)
                .collect();

            if passing_locations.len() > 0 {
                filtered_locations = passing_locations;
            }
        }

        if filtered_locations.len() == 0 {
            return Err("No legal location found".into());
        };

        let random_index = thread_rng().gen_range(0..filtered_locations.len());
        Ok(filtered_locations[random_index])
    }

    fn get_random_amount(&self) -> u32 {
        thread_rng().gen_range(self.min_amount..=self.max_amount)
    }
}

impl SpawnEntry for ConcreteSpawn {
    fn spawn(
        &self,
        room_data: &RoomGenerationData,
        _quest_data: &mut QuestGenerationData,
        world: &mut World,
        map: &GameMap,
        resources: &ResourceManager,
    ) -> Result<()> {
        let amount = self.get_random_amount();
        for _ in 0..amount {
            let Ok(location) = self.get_viable_location(&room_data.room, world, map, resources)
            else {
                continue;
            };
            let templates = resources.get_entity_templates(self.spawnable);
            world.spawn_from_templates(&templates, room_data.level_depth, location, resources)?;
        }
        Ok(())
    }

    fn evaluate(&self, _room_data: &RoomGenerationData, _quest_data: &QuestGenerationData) -> bool {
        true
    }
}

pub type LocationCriterionPredicate = fn(&EntityContainer, Coordinate, &GameMap) -> bool;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LocationCriterion {
    ByWall,
    AwayFromWall,
    AwayFromDoor,
}

impl LocationCriterion {
    pub fn validate(
        &self,
        candidate: Coordinate,
        room: &EntityContainer,
        world: &World,
        map: &GameMap,
        resources: &ResourceManager,
    ) -> bool {
        match self {
            Self::ByWall => find_one_adjacent(is_a_wall, candidate, room, world, map, resources),
            Self::AwayFromWall => {
                !find_one_adjacent(is_a_wall, candidate, room, world, map, resources)
            }
            Self::AwayFromDoor => !next_to_door(candidate, room, world, map, resources),
            _ => false,
        }
    }
}

fn calculate_adjacent(candidate: Coordinate) -> [Coordinate; 4] {
    [
        candidate + UP,
        candidate + DOWN,
        candidate + LEFT,
        candidate + RIGHT,
    ]
}

fn next_to_door(
    candidate: Coordinate,
    room: &EntityContainer,
    world: &World,
    _map: &GameMap,
    _resources: &ResourceManager,
) -> bool {
    let adjacent = HashSet::from(calculate_adjacent(candidate));

    room.entities
        .iter()
        .filter(|entity| {
            let Some(Position(coordinate)) = world.borrow_entity_component::<Position>(**entity)
            else {
                return false;
            };
            adjacent.contains(coordinate)
        })
        .any(|entity| world.borrow_entity_component::<Door>(*entity).is_some())
}

fn is_a_wall(
    candidate: Coordinate,
    _room: &EntityContainer,
    _world: &World,
    map: &GameMap,
    resources: &ResourceManager,
) -> bool {
    !map.is_tile_walkable(candidate, resources)
}

fn check_all_adjacent<F>(
    predicate: F,
    candidate: Coordinate,
    room: &EntityContainer,
    world: &World,
    map: &GameMap,
    resources: &ResourceManager,
) -> bool
where
    F: Fn(Coordinate, &EntityContainer, &World, &GameMap, &ResourceManager) -> bool,
{
    let adjacent = calculate_adjacent(candidate);

    adjacent
        .iter()
        .all(|coordinate| (predicate)(*coordinate, room, world, map, resources))
}

fn find_one_adjacent<F>(
    predicate: F,
    candidate: Coordinate,
    room: &EntityContainer,
    world: &World,
    map: &GameMap,
    resources: &ResourceManager,
) -> bool
where
    F: Fn(Coordinate, &EntityContainer, &World, &GameMap, &ResourceManager) -> bool,
{
    let adjacent = calculate_adjacent(candidate);

    adjacent
        .iter()
        .any(|coordinate| (predicate)(*coordinate, room, world, map, resources))
}
