use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::map::tile::{GameTile, Los, Passable};

use super::id::{ImageID, TileID};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedGameTile {
    pub id: TileID,
    pub name: String,
    pub image: ImageID,
    pub passable: Passable,
    pub los: Los,
}

impl SerializedGameTile {
    pub fn new(id: TileID, tile: GameTile) -> Self {
        Self {
            id,
            name: tile.name,
            image: tile.image,
            passable: tile.passable,
            los: tile.los,
        }
    }

    pub fn decompose(self) -> (TileID, GameTile) {
        (self.id, self.into())
    }
}

impl From<&SerializedGameTile> for GameTile {
    fn from(value: &SerializedGameTile) -> Self {
        Self {
            name: value.name.to_string(),
            image: value.image,
            passable: value.passable,
            los: value.los,
        }
    }
}

impl From<SerializedGameTile> for GameTile {
    fn from(value: SerializedGameTile) -> Self {
        Self {
            name: value.name,
            image: value.image,
            passable: value.passable,
            los: value.los,
        }
    }
}

pub fn save_to_yaml(tiles: &HashMap<TileID, GameTile>, path: &Path) -> Result<()> {
    let mut tiles: Vec<SerializedGameTile> = tiles
        .into_iter()
        .map(|(id, tile)| SerializedGameTile::new(*id, tile.clone()))
        .collect();

    tiles.sort_by_key(|tile| tile.id);

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_yaml::to_writer(writer, &tiles)?;

    Ok(())
}

pub fn load_from_yaml(path: &Path) -> Result<HashMap<TileID, GameTile>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tiles: Vec<SerializedGameTile> = serde_yaml::from_reader(reader)?;

    let tiles = tiles
        .into_iter()
        .map(|serialized_tile| serialized_tile.decompose())
        .collect();

    Ok(tiles)
}
