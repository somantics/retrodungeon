use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::spawning::roomtemplate::{RoomCriterion, RoomTemplate};

use super::{RoomTemplateID, SpawnEntryID};
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedRoomTemplate {
    id: RoomTemplateID,
    requirements: Vec<RoomCriterion>,
    spawns: Vec<SpawnEntryID>,
}

impl SerializedRoomTemplate {
    pub fn new(id: RoomTemplateID, template: RoomTemplate) -> Self {
        Self {
            id,
            requirements: template.requirements,
            spawns: template.spawns,
        }
    }

    pub fn decompose(self) -> (RoomTemplateID, RoomTemplate) {
        (self.id, self.into())
    }
}

impl From<&SerializedRoomTemplate> for RoomTemplate {
    fn from(value: &SerializedRoomTemplate) -> Self {
        Self {
            requirements: value.requirements.clone(),
            spawns: value.spawns.clone(),
        }
    }
}

impl From<SerializedRoomTemplate> for RoomTemplate {
    fn from(value: SerializedRoomTemplate) -> Self {
        Self {
            requirements: value.requirements,
            spawns: value.spawns,
        }
    }
}

pub fn save_to_yaml(spawnables: &HashMap<RoomTemplateID, RoomTemplate>, path: &Path) -> Result<()> {
    let mut spawnables: Vec<SerializedRoomTemplate> = spawnables
        .iter()
        .map(|(id, data)| SerializedRoomTemplate::new(*id, data.clone()))
        .collect();

    spawnables.sort_by_key(|entry| entry.id);

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_yaml::to_writer(writer, &spawnables)?;

    Ok(())
}

pub fn load_from_yaml(path: &Path) -> Result<HashMap<RoomTemplateID, RoomTemplate>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let spawnables: Vec<SerializedRoomTemplate> = serde_yaml::from_reader(reader)?;
    let spawnables = spawnables.into_iter().map(|entry| entry.decompose());
    let spawnables = HashMap::from_iter(spawnables);

    Ok(spawnables)
}
