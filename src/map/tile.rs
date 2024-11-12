use serde::{Deserialize, Serialize};

use crate::resources::{self, id::ImageID};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Passable {
    Walk,
    Swim,
    Fly,
    None,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Los {
    Block,
    Obstruct,
    Clear,
}

#[derive(Debug, Clone)]
pub struct GameTile {
    pub name: String,
    pub image: ImageID,
    pub passable: Passable,
    pub los: Los,
}

impl Default for GameTile {
    fn default() -> Self {
        GameTile {
            name: "Void".to_string(),
            image: resources::DEFAULT_IMAGE_ID,
            passable: Passable::None,
            los: Los::Clear,
        }
    }
}
