use std::{collections::HashMap, fmt::Debug};

use derive_entity_template::EntityTemplateEnum;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::component::combat;
use crate::component::responses::{DeathResponse, NoiseResponse, PickupResponse, SpellResponse};
use crate::component::health::{self};
use crate::component::items::{self, Coins};
use crate::component::spell::Spellbook;
use crate::component::tags::StairsDown;
use crate::error::Result;
use crate::resources::id::SpellDefinitionId;
use crate::resources::{self, ResourceManager};
use crate::{
    component::{
        attributes::{Attributes, Xp},
        behavior::{Behavior, BehaviorName},
        combat::{Attack, Combat, DamageRange},
        responses::{AttackResponse, InteractResponse, ShootResponse},
        health::Health,
        image::ImageState,
        tags::{Door, Monster},
        Collision, Name, SightBlocking,
    },
    event::ResponseFuctionName,
    map::tile::{Los, Passable},
    resources::id::ImageID,
    world::World,
};

pub trait EntityTemplate: Debug {
    fn add_components(&self, entity: usize, world: &mut World, depth: u32, resources: &ResourceManager) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize, EntityTemplateEnum)]
pub enum EntityTemplateEnum {
    Core(CoreTemplate),
    Combat(CombatTemplate),
    Door(DoorTemplate),
    Monster(MonsterTemplate),
    Player(PlayerTemplate),
    Stairs(StairsTemplate),
    Interactable(InteractableTemplate),
    Pickup(PickupTemplate),
    Inventory(InventoryTemplate),
    Destructible(DestructibleTemplate),
    Spellbook(SpellbookTemplate),
}

// Template Definitions

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyTemplate();

impl EntityTemplate for EmptyTemplate {
    fn add_components(&self, _entity: usize, _world: &mut World, _depth: u32, _resources: &ResourceManager) -> Result<()> {
        Err("Attempted to spawn from empty template.".into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreTemplate {
    pub name: String,
    pub image: ImageID,
    pub collision: Passable,
    pub los: Los,
}

impl EntityTemplate for CoreTemplate {
    fn add_components(&self, entity: usize, world: &mut World, _depth: u32, _resources: &ResourceManager) -> Result<()> {
        let name = Name(self.name.clone());
        let image = self.image;
        let collision = Collision(self.collision);
        let sight_block = SightBlocking(self.los);

        world.add_component(entity, name)?;
        world.add_component(entity, image)?;
        world.add_component(entity, collision)?;
        world.add_component(entity, sight_block)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CombatTemplate {
    pub health: u32,
    pub melee_damage: Option<DamageRange<u32>>,
    pub ranged_damage: Option<DamageRange<u32>>,
    pub attack_response: Option<AttackResponse>,
    pub shoot_response: Option<ShootResponse>,
    pub spell_response: Option<SpellResponse>,
    pub death_response: Option<DeathResponse>,
}

impl EntityTemplate for CombatTemplate {
    fn add_components(&self, entity: usize, world: &mut World, _depth: u32, _resources: &ResourceManager) -> Result<()> {
        let health = Health::new(self.health);
        let melee_attack = Attack::optional_melee(self.melee_damage);
        let ranged_attack = Attack::optional_ranged(self.ranged_damage);
        let combat = Combat::new(melee_attack, ranged_attack);

        let attack_response = self.attack_response.clone().unwrap_or_default();
        let shoot_response = self.shoot_response.clone().unwrap_or_default();
        let spell_response = self.spell_response.clone().unwrap_or_default();

        world.add_component(entity, health)?;
        world.add_component(entity, combat)?;
        world.add_component(entity, attack_response)?;
        world.add_component(entity, shoot_response)?;
        world.add_component(entity, spell_response)?;

        if let Some(death_response) = self.death_response.clone() {
            world.add_component(entity, death_response)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DestructibleTemplate {
    health: u32,
    attack_response: Option<AttackResponse>,
    shoot_response: Option<ShootResponse>,
    spell_response: Option<SpellResponse>,
    death_response: Option<DeathResponse>,
}

impl EntityTemplate for DestructibleTemplate {
    fn add_components(&self, entity: usize, world: &mut World, _depth: u32, _resources: &ResourceManager) -> Result<()> {
        let health = Health::new(self.health);
        let attack_response = self.attack_response.clone().unwrap_or_default();
        let shoot_response = self.shoot_response.clone().unwrap_or_default();
        let spell_response = self.spell_response.clone().unwrap_or_default();

        world.add_component(entity, health)?;
        world.add_component(entity, attack_response)?;
        world.add_component(entity, shoot_response)?;
        world.add_component(entity, spell_response)?;

        if let Some(death_response) = self.death_response.clone() {
            world.add_component(entity, death_response)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InventoryTemplate {
    coins: Option<u32>,
    //items: Option<>
}

impl EntityTemplate for InventoryTemplate {
    fn add_components(&self, entity: usize, world: &mut World, depth: u32, _resources: &ResourceManager) -> Result<()> {
        if let Some(amount) = self.coins {
            let adjusted_amount = items::get_adjusted_coins(amount, depth);
            world.add_component(entity, Coins(adjusted_amount))?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StairsTemplate;

impl EntityTemplate for StairsTemplate {
    fn add_components(&self, entity: usize, world: &mut World, _depth: u32, _resources: &ResourceManager) -> Result<()> {
        world.add_component(entity, StairsDown)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTemplate {
    level: u32,
    attributes: Attributes,
}

impl EntityTemplate for PlayerTemplate {
    fn add_components(&self, entity: usize, world: &mut World, _depth: u32, _resources: &ResourceManager) -> Result<()> {
        let xp = Xp::new(self.level);

        let mut attributes = self.attributes.clone();
        let random_stat_index = thread_rng().gen_range(0..=2);
        match random_stat_index {
            0 => attributes.might += 1,
            1 => attributes.wit += 1,
            2 => attributes.skill += 1,
            _ => {}
        };

        world.add_component(entity, attributes)?;
        world.add_component(entity, xp)?;
        world.mark_as_player(entity)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpellbookTemplate {
    pub spells: Vec<SpellDefinitionId>
}

impl EntityTemplate for SpellbookTemplate {
    fn add_components(&self, entity: usize, world: &mut World, _depth: u32, resources: &ResourceManager) -> Result<()> {
        let definitions = self.spells
            .iter()
            .filter_map(|id| resources.get_spell(*id))
            .collect();

        let mut spellbook = Spellbook::new(definitions);

        if let Some(Attributes {wit, ..}) = world.borrow_entity_component(entity) {
            if *wit > 1 {
                let wit_increase = wit - 1;
                spellbook.on_wit_increased(wit_increase);
            }
        }

        world.add_component(entity, spellbook)?;

        Ok(())   
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MonsterTemplate {
    pub combat_template: CombatTemplate,
    pub behavior: BehaviorName,
    pub noise_tolerance: u32,
    pub action_count: Option<u32>
}

impl MonsterTemplate {
    pub fn new_from(combat_template: CombatTemplate) -> Self {
        Self {
            combat_template,
            behavior: BehaviorName::AggressiveMelee,
            noise_tolerance: 10,
            action_count: Some(1),
        }
    }
}

impl EntityTemplate for MonsterTemplate {
    fn add_components(&self, entity: usize, world: &mut World, depth: u32, resources: &ResourceManager) -> Result<()> {
        let noise_response = NoiseResponse::new(self.noise_tolerance);
        let behavior = Behavior::new(self.behavior, self.action_count);

        self.combat_template.add_components(entity, world, depth, resources)?;
        world.add_component(entity, noise_response)?;
        world.add_component(entity, behavior)?;
        world.add_component(entity, Monster)?;

        if let Some(Health(current, max)) = world.borrow_entity_component_mut(entity) {
            let adjusted_health = health::get_adjusted_health(self.combat_template.health, depth);
            *current = adjusted_health;
            *max = adjusted_health;
        };

        if let Some(combat) = world.borrow_entity_component_mut::<Combat>(entity) {
            if let Some(ref mut melee) = combat.melee_attack {
                melee.damage_min = combat::get_adjusted_damage(melee.damage_min, depth);
                melee.damage_max = combat::get_adjusted_damage(melee.damage_max, depth);
            } 
            if let Some(ref mut ranged) = combat.ranged_attack {
                ranged.damage_min = combat::get_adjusted_damage(ranged.damage_min, depth);
                ranged.damage_max = combat::get_adjusted_damage(ranged.damage_max, depth);
            } 
        };

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractableTemplate {
    response: InteractResponse,
    image_states: Option<ImageState>,
}

impl EntityTemplate for InteractableTemplate {
    fn add_components(&self, entity: usize, world: &mut World, _depth: u32, _resources: &ResourceManager) -> Result<()> {
        let response = self.response.clone();
        world.add_component(entity, response)?;

        if let Some(image_states) = self.image_states.clone() {
            world.add_component(entity, image_states)?;
        };

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PickupTemplate {
    pub inventory: InventoryTemplate,
    pub pickup_response: Option<PickupResponse>,
}

impl EntityTemplate for PickupTemplate {
    fn add_components(&self, entity: usize, world: &mut World, depth: u32, resources: &ResourceManager) -> Result<()> {
        let pickup_response = self.pickup_response.clone().unwrap_or_default();
        self.inventory.add_components(entity, world, depth, resources)?;
        world.add_component(entity, pickup_response)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoorTemplate {
    pub image_states: ImageState,
    pub interact_response: Option<InteractResponse>,
}

impl Default for DoorTemplate {
    fn default() -> Self {
        let mut states = HashMap::new();
        states.insert("open".to_string(), ImageID(7));
        states.insert("closed".to_string(), ImageID(8));
        let image_states = ImageState {
            current: "open".to_string(),
            states,
        };
        Self {
            image_states,
            interact_response: None,
        }
    }
}

impl EntityTemplate for DoorTemplate {
    fn add_components(&self, entity: usize, world: &mut World, _depth: u32, _resources: &ResourceManager) -> Result<()> {
        let default_resonse = InteractResponse {
            args: HashMap::new(),
            msg_args: HashMap::new(),
            response: ResponseFuctionName::OpenDoor,
        };
        let image_states = self.image_states.clone();
        let interact_response = self
            .interact_response
            .clone()
            .unwrap_or(default_resonse);

        world.add_component(entity, image_states)?;
        world.add_component(entity, interact_response)?;
        world.add_component(entity, Door)?;

        Ok(())
    }
}
