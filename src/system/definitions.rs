use log::{debug, warn};

use crate::component::attributes::{Xp, XpStatus};
use crate::component::spell::{self, Spellbook};
use crate::error::{Error, Result};
use crate::event::combat_events::DeathEvent;
use crate::event::spell_events;
use crate::{
    component::{behavior::Behavior, health::Health},
    map::GameMap,
    resources::ResourceManager,
    world::World,
};

pub fn reap_units(world: &mut World, map: &mut GameMap, resources: &ResourceManager) -> Result<()> {
    let Some(health_components) = world.borrow_component_vec::<Health>() else {
        return Err("No health components registered".into());
    };

    let components_by_entity = health_components.into_iter().enumerate().filter_map(
        |(entity, component)| match component {
            Some(component) => Some((entity, component)),
            None => None,
        },
    );

    let mut reaped_entities: Vec<usize> = Vec::new();
    for (entity, Health(current, _max)) in components_by_entity {
        if *current <= 0 {
            reaped_entities.push(entity);
        }
    }

    let event = DeathEvent { source: 0 };
    for entity in reaped_entities {
        world.send_event(map, resources, &event, entity)?;
        world.remove_entity(entity)?;
    }

    Ok(())
}

pub fn monster_turns(
    world: &mut World,
    map: &mut GameMap,
    resources: &ResourceManager,
) -> Result<()> {
    let Some(behavior_components) = world.borrow_component_vec::<Behavior>() else {
        return Err("No behavior components registered".into());
    };

    let components_by_entity: Vec<usize> = behavior_components
        .into_iter()
        .enumerate()
        .filter_map(|(entity, component)| match component {
            Some(_) => Some(entity),
            None => None,
        })
        .collect();

    for entity in components_by_entity {
        let Some(behavior) = world.borrow_entity_component::<Behavior>(entity) else {
            return Err("Missing behavior component".into());
        };
        let behavior = behavior.clone();
        let result = behavior.perform_actions(entity, world, map, resources);
        match result {
            Ok(_) => {}
            Err(error) => debug!("{error}"),
        };
    }

    Ok(())
}

pub fn level_up_check(
    world: &mut World,
    _map: &mut GameMap,
    _resources: &ResourceManager,
) -> Result<()> {
    let Ok(player) = world.get_player_id() else {
        Err(Error::NoPlayerFound)?
    };

    let Some(xp) = world.borrow_entity_component_mut::<Xp>(player) else {
        return Err("Player has no xp component".into());
    };

    if xp.current >= xp.max {
        xp.status = XpStatus::LevelUp;
    } else {
        xp.status = XpStatus::Default;
    }
    Ok(())
}

pub fn spell_cooldowns(
    world: &mut World,
    _map: &mut GameMap,
    _resources: &ResourceManager,
) -> Result<()> {
    let Some(spellbooks) = world.borrow_component_vec_mut::<Spellbook>() else {
        return Ok(());
    };

    for spellbook in spellbooks.iter_mut().flatten() {
        spellbook.reset_spells(super::TimeSlot::EndOfLevel);
    }

    Ok(())
}