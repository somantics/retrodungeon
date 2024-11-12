use log::{debug, warn};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use slint::ModelRc;

use crate::component::tags::Player;
use crate::event::combat_events::{ARG_DAMAGE_MULTIPLIER, ARG_DAMAGE_MULTIPLIER_OVERRIDE};
use crate::event::spell_events::SpellEvent;
use crate::event::{Event, EventArguments};
use crate::game::RANGE_EPSILON;
use crate::logger;
use crate::map::los::line_of_sight;
use crate::map::utils::Coordinate;
use crate::map::GameMap;
use crate::component::Position;
use crate::resources::id::SpellDefinitionId;
use crate::resources::serialize_spell::SpellDefinition;
use crate::resources::ResourceManager;
use crate::system::TimeSlot;
use crate::world::World;

use super::attributes::{spell_damage_boost, Attributes};
use super::combat::DamageRange;
use super::health::Health;
use super::image::ImageState;
use super::items::Inventory;
use super::Name;
use crate::error::{Error, Result};

type SpellEffect = fn(SpellEffectArguments) -> Result<()>;

// a spell creates and broadcasts evets to the correct targets
// all actual changes should happen in event applications
// event type is usually a SpellEvent, so that SpellResponses react
// if something is to bypass spell resistance (like, rubble from exploding a barrel)
// use Attack or Shoot events instead.
//
// spells that only affect the map are exceptions: such as scry, that explores rooms using the map directly

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SpellEffectName {
    Fireball,
    Scry,
}

impl SpellEffectName {
    pub fn get_callable(&self) -> Result<SpellEffect> {
        match self {
            SpellEffectName::Fireball => Ok(fireball),
            SpellEffectName::Scry => Ok(scry),
            _ => Err("No effect listed for that spell name".into())
        }
    }
}

pub struct SpellEffectArguments<'a> {
    pub source: usize,
    pub target: Coordinate,
    pub world: &'a mut World, 
    pub map: &'a mut GameMap, 
    pub stats: Option<&'a Attributes>, 
    pub items: Option<&'a Inventory>, 
    pub resources: &'a ResourceManager
}

#[derive(Debug, Clone)]
pub struct SpellInstance {
    pub name: Name,
    pub icon_states: ImageState,
    pub effect: SpellEffectName,
    pub casts_left: u32,
    pub casts_max: u32,
    pub reset_timeslot: TimeSlot,
}

impl SpellInstance {
    pub fn new(definition: &SpellDefinition) -> Self {
        Self {
            name: definition.name.clone(),
            icon_states: definition.icon_states.clone(),
            effect: definition.effect,
            casts_left: definition.casts,
            casts_max: definition.casts,
            reset_timeslot: definition.reset_time_slot,
        }
    }

    pub fn cast(&self, args: SpellEffectArguments) -> Result<()> {
        if self.casts_left <= 0 {
            logger::log_message("You have no more casts of that spell this floor.");
            return Err("Out of casts of that spell".into());
        };

        match self.effect.get_callable() {
            Ok(effect_function) => {
                (effect_function)(args)?;
            },
            Err(error) => warn!("{error}"),
        }
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Spellbook {
    spells: Vec<SpellInstance>,
}

impl Spellbook {
    pub fn new(definitions: Vec<&SpellDefinition>) -> Self {
        let spells: Vec<SpellInstance> = definitions
            .iter()
            .map(|definition| SpellInstance::new(definition))
            .collect();

        Self { spells }
    }

    pub fn get_spells(&self) -> Vec<&SpellInstance> {
        self.spells
            .iter()
            .collect()
    }

    pub fn reset_spells(&mut self, time_slot: TimeSlot) {
        let matching_spells = self
            .spells
            .iter_mut()
            .filter(|spell| spell.reset_timeslot == time_slot);

        for spell in matching_spells {
            spell.casts_left = spell.casts_max;
        }
    }

    pub fn get_spell(&self, index: usize) -> Option<&SpellInstance> {
        self.spells.get(index)
    }

    pub fn get_spell_mut(&mut self, index: usize) -> Option<&mut SpellInstance> {
        self.spells.get_mut(index)
    }

    pub fn register_cast(&mut self, spell_index: usize) -> Result<()> {
        let Some(spell) = self.spells.get_mut(spell_index) else {
            return Err("No such spell instance.".into());
        };

        if spell.casts_left > 0 {
            spell.casts_left -= 1;
        }

        Ok(())
    }

    pub fn add_spell(&mut self, id: SpellDefinitionId, stats: Option<&Attributes>, items: Option<&Inventory>, resources: &ResourceManager) -> Result<()> {
        let Some(definition) = resources.get_spell(id) else {
            return Err(format!("No spell of that ID found: {id:?}").into());
        };
        let mut spell = SpellInstance::new(definition);

        if let Some(attributes) = stats {
            spell.casts_left += attributes.wit;
            spell.casts_max += attributes.wit;
        };

        self.spells.push(spell);

        Ok(())
    }

    pub fn on_wit_increased(&mut self, amount: u32) {
        for spell in &mut self.spells {
            spell.casts_left += amount;
            spell.casts_max += amount;
        }
    }
}

fn fade(args: SpellEffectArguments) -> Result<()> {
    // make hero invis
    Ok(())
}

fn scry(args: SpellEffectArguments) -> Result<()> {
    logger::log_message("Casting scry!");
    let SpellEffectArguments {source, target, world, map, stats, items, resources } = args;

    let distance_to_target= move |best: Coordinate, current: Coordinate| -> Coordinate {
        let best_distance = best.distance(target);
        let current_distance = current.distance(target);

        if current_distance < best_distance {
            current
        } else {
            best
        }
    };

    let room_target = map.room_graph
        .node_weights()
        .map(|room| room.extends.center())
        .reduce(distance_to_target);

    let Some(room_target) = room_target else {
        return Err("Couldn't find a target room".into());
    };

    map.explore_room(room_target);

    Ok(())
}

fn fireball(args: SpellEffectArguments) -> Result<()> {
    let SpellEffectArguments {source, target, world, map, stats, items, resources } = args;
    let range = 12.0 + RANGE_EPSILON;
    let radius = 3.5 + RANGE_EPSILON;

    if !map.is_tile_explored(target) {
        logger::log_message("Can't cast that in unexplored areas");
        return Err(Error::InvalidTarget);
    };

    let Some(Position(caster_position)) = world.borrow_entity_component(source) else {
        return Err("Can't find caster position".into());
    };

    if caster_position.distance(target) > range {
        logger::log_message("Target is out of range");
        return Err(Error::InvalidTarget);
    }

    if !line_of_sight(*caster_position, target, map, world, resources) {
        logger::log_message("Can't see to that tile");
        return Err(Error::InvalidTarget);
    }

    logger::log_message("Casting fireball!");

    let mut damage = DamageRange(8, 14);
    if let Some(stats)  = stats{
        damage += spell_damage_boost(stats);
    }

    let effect = apply_spell_damage_factory(damage);
    let event = SpellEvent::new(
        source,
        effect,
    );

    let entities_in_range: Vec<usize> = world.get_entities_in_room(target)
        .iter()
        .filter(|entity| {
            let Some(Position(pos)) = world.borrow_entity_component(**entity) else {
                return false;
            };
            pos.distance(target) < radius
                && line_of_sight(target, *pos, map, world, resources)
        })
        .map(|entity| *entity)
        .collect();

    for entity in entities_in_range {
        if world.borrow_entity_component::<Player>(entity).is_some() {
            continue;
        }
        world.send_event(map, resources, &event, entity)?;
    }
    
    Ok(())
}

fn apply_spell_damage_factory( damage: DamageRange<u32>) -> impl Fn(EventArguments) -> Result<()>{
    move |args| {
        apply_spell_damage(damage, args)
    }
}

fn apply_spell_damage(
    damage: DamageRange<u32>,
    event_data: EventArguments,
) -> Result<()> {
    let EventArguments { world, map, resources, source, target, args } = event_data;

    let mut message_addendum = "";
    let mut damage = thread_rng().gen_range(damage.0..=damage.1) as f64;

    if let Some(dmg_multiplier) = args.get(ARG_DAMAGE_MULTIPLIER_OVERRIDE) {
        damage *= dmg_multiplier;

        if *dmg_multiplier > 1.05 {
            message_addendum = "It's very effective.";
        } else if *dmg_multiplier < 0.95 {
            message_addendum = "It seems to have little effect.";
        }
    } else if let Some(dmg_multiplier) = args.get(ARG_DAMAGE_MULTIPLIER) {
        damage *= dmg_multiplier;

        
        if *dmg_multiplier > 1.05 {
            message_addendum = "It's very effective.";
        } else if *dmg_multiplier < 0.95 {
            message_addendum = "It seems to have little effect.";
        }
    }
    let damage = damage as u32;

    let Some(health) = world.borrow_entity_component_mut::<Health>(target) else {
        return Err("Target has no health component.".into());
    };
    health.sub_current(damage );
    
    let name = world.borrow_entity_component::<Name>(target);
    let msg = logger::generate_take_damage_message(name, damage, &message_addendum);
    logger::log_message(&msg);

    Ok(())
}
