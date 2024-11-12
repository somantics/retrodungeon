use std::{collections::HashMap, path::Path};

use id::*;
use log::info;
use serialize_spell::SpellDefinition;

use crate::component::image::ImageState;
use crate::component::spell::SpellEffectName;
use crate::component::Name;
use crate::error::Result;
use crate::system::TimeSlot;
use crate::{
    map::tile::GameTile,
    spawning::{
        entitytemplate::EntityTemplateEnum, roomtemplate::RoomTemplate, spawnentry::SpawnEntryType,
    },
};

pub mod id;
pub mod serialize_gametile;
pub mod serialize_roomtemplate;
pub mod serialize_spawnable;
pub mod serialize_spawnentry;
pub mod serialize_spell;

pub const DEFAULT_IMAGE_ID: ImageID = ImageID(4);

pub const DEFAULT_TILEID: TileID = TileID(0);
pub const WALL_TILEID: TileID = TileID(2);
pub const FLOOR_TILEID: TileID = TileID(1);

pub const DOOR_SPAWNABLE: SpawnableID = SpawnableID(0);
pub const GOLD_PILE_SPAWNABLE: SpawnableID = SpawnableID(5);

pub const PLAYER_SPAWNENTRY: SpawnEntryID = SpawnEntryID(0);
pub const STAIRS_SPAWNENTRY: SpawnEntryID = SpawnEntryID(1);


pub const TILESET_SIZE: f32 = 32.0;

const TILES_PATH: &'static str = "data/tiles.yaml";
const SPAWNABLES_PATH: &'static str = "data/spawnables.yaml";
const SPAWN_ENTRY_PATH: &'static str = "data/spawnentries.yaml";
const ROOM_TEMPLATE_PATH: &'static str = "data/roomtemplates.yaml";

#[derive(Default)]
pub struct ResourceManager {
    tile_definitions: HashMap<TileID, GameTile>,
    spawnable_definitions: HashMap<SpawnableID, Vec<EntityTemplateEnum>>,
    spawn_entry_definitions: HashMap<SpawnEntryID, SpawnEntryType>,
    room_template_definitions: HashMap<RoomTemplateID, RoomTemplate>,
    spell_definition: HashMap<SpellDefinitionId, SpellDefinition>,
}

impl ResourceManager {
    pub fn new() -> Result<Self> {
        let mut resources = Self::default();

        info!("Reading tile definitions");
        resources.load_tile_definitions()?;

        info!("Reading spawnables definitions");
        resources.load_spawnable_definitions()?;

        info!("Reading spawn tables");
        resources.load_spawn_entry_definitions()?;

        info!("Reading room templates");
        resources.load_room_template_definitions()?;

        let fireball = SpellDefinition { 
            name: Name("Fireball".to_string()), 
            effect: SpellEffectName::Fireball, 
            icon_states: ImageState {
                current: "available".to_string(),
                states: HashMap::from([("available".to_string(), ImageID(0))]),
            }, 
            casts: 2, 
            reset_time_slot: TimeSlot::EndOfLevel, 
        };
        resources.spell_definition.insert(SpellDefinitionId(0), fireball);

        let scry = SpellDefinition { 
            name: Name("Scry".to_string()), 
            effect: SpellEffectName::Scry, 
            icon_states: ImageState {
                current: "available".to_string(),
                states: HashMap::from([("available".to_string(), ImageID(0))]),
            }, 
            casts: 1, 
            reset_time_slot: TimeSlot::EndOfLevel, 
        };
        resources.spell_definition.insert(SpellDefinitionId(1), scry);

        Ok(resources)
    }

    //  TILES

    pub fn get_tile(&self, tile_id: TileID) -> Option<&GameTile> {
        self.tile_definitions.get(&tile_id)
    }

    pub fn load_tile_definitions(&mut self) -> Result<()> {
        self.tile_definitions = serialize_gametile::load_from_yaml(Path::new(TILES_PATH))?;
        Ok(())
    }

    pub fn save_tile_definitions(&self) -> Result<()> {
        serialize_gametile::save_to_yaml(&self.tile_definitions, Path::new(TILES_PATH))?;
        Ok(())
    }

    //  SPAWNABLES

    pub fn get_entity_templates(&self, spawnable: SpawnableID) -> Vec<&EntityTemplateEnum> {
        match self.spawnable_definitions.get(&spawnable) {
            Some(vec) => vec.iter().collect(),
            None => vec![],
        }
    }

    pub fn load_spawnable_definitions(&mut self) -> Result<()> {
        self.spawnable_definitions =
            serialize_spawnable::load_from_yaml(Path::new(SPAWNABLES_PATH))?;
        Ok(())
    }

    pub fn save_spawnable_definitions(&self) -> Result<()> {
        serialize_spawnable::save_to_yaml(&self.spawnable_definitions, Path::new(SPAWNABLES_PATH))?;
        Ok(())
    }

    //  SPAWN TABLES

    pub fn get_spawn_entry(&self, spawn_entry: SpawnEntryID) -> Option<&SpawnEntryType> {
        self.spawn_entry_definitions.get(&spawn_entry)
    }

    pub fn load_spawn_entry_definitions(&mut self) -> Result<()> {
        self.spawn_entry_definitions =
            serialize_spawnentry::load_from_yaml(Path::new(SPAWN_ENTRY_PATH))?;
        Ok(())
    }

    pub fn save_spawn_entry_definitions(&mut self) -> Result<()> {
        serialize_spawnentry::save_to_yaml(
            &self.spawn_entry_definitions,
            Path::new(SPAWN_ENTRY_PATH),
        )?;
        Ok(())
    }

    //  ROOM TEMPLATES

    pub fn get_all_room_templates(&self) -> impl Iterator<Item = (&RoomTemplateID, &RoomTemplate)> {
        self.room_template_definitions.iter()
    }

    pub fn get_room_template(&self, room_template: RoomTemplateID) -> Option<&RoomTemplate> {
        self.room_template_definitions.get(&room_template)
    }

    pub fn load_room_template_definitions(&mut self) -> Result<()> {
        self.room_template_definitions =
            serialize_roomtemplate::load_from_yaml(Path::new(ROOM_TEMPLATE_PATH))?;
        Ok(())
    }

    pub fn save_room_template_definitions(&mut self) -> Result<()> {
        serialize_roomtemplate::save_to_yaml(
            &self.room_template_definitions,
            Path::new(ROOM_TEMPLATE_PATH),
        )?;
        Ok(())
    }

    //  SPELLS

    pub fn get_spell(&self, spell: SpellDefinitionId) -> Option<&SpellDefinition> {
        self.spell_definition.get(&spell)
    }
}
