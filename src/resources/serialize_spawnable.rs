use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::spawning::entitytemplate::EntityTemplateEnum;

use super::SpawnableID;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEntityTemplate {
    pub id: SpawnableID,
    pub data: Vec<EntityTemplateEnum>,
}

impl SerializedEntityTemplate {
    pub fn new(id: SpawnableID, data: Vec<EntityTemplateEnum>) -> Self {
        Self { id, data }
    }

    pub fn decompose(self) -> (SpawnableID, Vec<EntityTemplateEnum>) {
        (self.id, self.data)
    }
}

pub fn save_to_yaml(
    spawnables: &HashMap<SpawnableID, Vec<EntityTemplateEnum>>,
    path: &Path,
) -> Result<()> {
    let mut spawnables: Vec<SerializedEntityTemplate> = spawnables
        .iter()
        .map(|(id, data)| SerializedEntityTemplate::new(*id, data.clone()))
        .collect();

    spawnables.sort_by_key(|entry| entry.id);

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_yaml::to_writer(writer, &spawnables)?;

    Ok(())
}

pub fn load_from_yaml(path: &Path) -> Result<HashMap<SpawnableID, Vec<EntityTemplateEnum>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let spawnables: Vec<SerializedEntityTemplate> = serde_yaml::from_reader(reader)?;
    let spawnables = spawnables.into_iter().map(|entry| entry.decompose());
    let spawnables = HashMap::from_iter(spawnables);

    Ok(spawnables)
}
