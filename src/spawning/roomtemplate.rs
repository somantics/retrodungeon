use serde::{Deserialize, Serialize};

use crate::{
    map::room::{QuestGenerationData, RoomGenerationData},
    resources::id::SpawnEntryID,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomTemplate {
    pub requirements: Vec<RoomCriterion>,
    pub spawns: Vec<SpawnEntryID>,
}

impl RoomTemplate {
    pub fn validate(
        &self,
        room_data: &RoomGenerationData,
        quest_data: &QuestGenerationData,
    ) -> bool {
        !self
            .requirements
            .iter()
            .any(|criterion| !criterion.validate(room_data, quest_data))
    }
}

// Criterions are predicates
// store them as enums that may have primitive arguments
//  such as: LargerThan(12), SmallerThan(20)
// predicates receive following input:
//  size of room (box extends)
//  depth of level
//  distance from start (in rooms)
//  (future) budget requirement
//  (future) quest generation state

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RoomCriterion {
    LargerThan(u32),
    SmallerThan(u32),
    DepthLessThan(u32),
    DepthGreaterThan(u32),
    EarlierThan(u32),
    LaterThan(u32),
}

impl RoomCriterion {
    pub fn validate(
        &self,
        room_data: &RoomGenerationData,
        _quest_data: &QuestGenerationData,
    ) -> bool {
        match self {
            Self::LargerThan(threshold) => room_data.get_area() > *threshold,
            Self::SmallerThan(threshold) => room_data.get_area() < *threshold,
            Self::DepthGreaterThan(threshold) => room_data.level_depth > *threshold,
            Self::DepthLessThan(threshold) => room_data.level_depth < *threshold,
            Self::EarlierThan(threshold) => room_data.room_depth > *threshold,
            Self::LaterThan(threshold) => room_data.room_depth < *threshold,
        }
    }
}
