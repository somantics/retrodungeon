use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::resources::id::ImageID;

#[derive(Debug, Clone)]
pub struct Image {
    pub id: ImageID,
    pub depth: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImageState {
    pub current: String,
    pub states: HashMap<String, ImageID>,
}
