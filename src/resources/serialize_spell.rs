use serde::{Deserialize, Serialize};

use crate::{component::{image::ImageState, spell::SpellEffectName, Name}, system::TimeSlot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellDefinition {
    pub name: Name,
    pub effect: SpellEffectName,
    pub icon_states: ImageState,
    pub casts: u32,
    pub reset_time_slot: TimeSlot,
}