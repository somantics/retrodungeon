use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::spawning::spawnentry::SpawnEntryType;

use super::SpawnEntryID;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedSpawnEntry {
    pub id: SpawnEntryID,
    pub category: SpawnEntryType,
}

impl SerializedSpawnEntry {
    pub fn new(id: SpawnEntryID, spawn_entry: SpawnEntryType) -> Self {
        Self {
            id,
            category: spawn_entry,
        }
    }

    pub fn decompose(self) -> (SpawnEntryID, SpawnEntryType) {
        (self.id, self.into())
    }
}

impl From<&SerializedSpawnEntry> for SpawnEntryType {
    fn from(value: &SerializedSpawnEntry) -> Self {
        value.category.clone()
    }
}

impl From<SerializedSpawnEntry> for SpawnEntryType {
    fn from(value: SerializedSpawnEntry) -> Self {
        value.category
    }
}

pub fn save_to_yaml(tiles: &HashMap<SpawnEntryID, SpawnEntryType>, path: &Path) -> Result<()> {
    let mut tiles: Vec<SerializedSpawnEntry> = tiles
        .into_iter()
        .map(|(id, tile)| SerializedSpawnEntry::new(*id, tile.clone()))
        .collect();

    tiles.sort_by_key(|tile| tile.id);

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_yaml::to_writer(writer, &tiles)?;

    Ok(())
}

pub fn load_from_yaml(path: &Path) -> Result<HashMap<SpawnEntryID, SpawnEntryType>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let entries: Vec<SerializedSpawnEntry> = serde_yaml::from_reader(reader)?;

    let entries = entries
        .into_iter()
        .map(|serialized_entry| serialized_entry.decompose())
        .collect();

    Ok(entries)
}
