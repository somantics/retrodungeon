use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct RoomTemplateID(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct SpawnEntryID(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct SpawnableID(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct TileID(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct ImageID(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct SpellInstanceId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct SpellDefinitionId(pub usize);