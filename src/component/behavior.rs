use serde::{Deserialize, Serialize};

use crate::{
    event::combat_events::{AttackEvent, ShootEvent}, map::{los::line_of_sight, GameMap}, resources::ResourceManager, world::World
};

use super::{
    combat::{AttackType, Combat},
    Position,
};
use crate::error::{Error, Result};
use crate::system::error::MonsterTurnError;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Behavior {
    pub state: BehaviorState,
    pub behavior: BehaviorName,
    pub action_count: u32,
}

impl Behavior {
    pub fn new(behavior: BehaviorName, custom_action_count: Option<u32>) -> Self {
        Self {
            behavior,
            action_count: custom_action_count.unwrap_or(1),
            ..Default::default()
        }
    }

    pub fn new_melee() -> Self {
        Self {
            state: BehaviorState::Awake,
            behavior: BehaviorName::AggressiveMelee,
            action_count: 1,
        }
    }

    pub fn new_ranged() -> Self {
        Self {
            state: BehaviorState::Awake,
            behavior: BehaviorName::AggressiveRanged,
            action_count: 1,
        }
    }

    pub fn new_fast() -> Self {
        Self {
            state: BehaviorState::Awake,
            behavior: BehaviorName::AggressiveMelee,
            action_count: 2,
        }
    }

    pub fn perform_actions(
        &self,
        own_entity: usize,
        world: &mut World,
        map: &mut GameMap,
        resources: &ResourceManager,
    ) -> Result<()> {
        for _ in 0..self.action_count {
            let action = self.behavior.choose_action(own_entity, self.state, world, map, resources)?;
            action.perform(own_entity, world, map, resources)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AIAction {
    Approach,
    Attack(usize),
    Shoot(usize),
    Sleep,
    Wander,
    Flee,
    WakeUp,
}

impl AIAction {
    fn perform(
        &self,
        own_entity: usize,
        world: &mut World,
        map: &mut GameMap,
        resources: &ResourceManager,
    ) -> Result<()> {
        match self {
            AIAction::Approach => approach_player(own_entity, world, map, resources),
            AIAction::Attack(target) => attack_entity(own_entity, *target, world, map, resources),
            AIAction::Shoot(target) => shoot_entity(own_entity, *target, world, map, resources),
            AIAction::WakeUp => wake_up(own_entity, world),
            AIAction::Sleep => Ok(()),
            _ => Err(MonsterTurnError::FailedToCompleteAction {
                entity: own_entity,
                action: *self,
            }
            .into()),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BehaviorState {
    #[default]
    Asleep,
    Awake,
    Alerted,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum BehaviorName {
    None,
    #[default]
    AggressiveMelee,
    AggressiveRanged,
}

impl BehaviorName {
    pub fn choose_action(
        &self,
        entity: usize,
        ai_state: BehaviorState,
        world: &World,
        map: &GameMap,
        resources: &ResourceManager,
    ) -> Result<AIAction> {
        match self {
            BehaviorName::AggressiveMelee => aggressive_melee(entity, ai_state, world),
            BehaviorName::AggressiveRanged => {
                aggressive_ranged(entity, ai_state, world, map, resources)
            }
            _ => Err(MonsterTurnError::BehaviorNotImplemented {
                entity,
                behavior: *self,
            }
            .into()),
        }
    }
}

fn approach_player(
    own_entity: usize,
    world: &mut World,
    map: &GameMap,
    _resources: &ResourceManager,
) -> Result<()> {
    let Some(Position(origin)) = world.borrow_entity_component::<Position>(own_entity) else {
        return Err(MonsterTurnError::NoPositionFound { entity: own_entity }.into());
    };

    let Some(step) = map.pathing_grid.get(origin) else {
        return Err(MonsterTurnError::NoPathfindingData {
            entity: own_entity,
            coordinate: *origin,
        }
        .into());
    };

    let destination = *origin + *step;
    if world.get_blocking_entity(destination).is_none() {
        world.update_position(own_entity, destination);
    }

    Ok(())
}

fn attack_entity(
    source: usize,
    target: usize,
    world: &mut World,
    map: &mut GameMap,
    resources: &ResourceManager,
) -> Result<()> {
    world.send_event(map, resources, &AttackEvent::new(source), target)?;
    Ok(())
}

fn shoot_entity(
    source: usize,
    target: usize,
    world: &mut World,
    map: &mut GameMap,
    resources: &ResourceManager,
) -> Result<()> {
    world.send_event(map, resources, &ShootEvent::new(source), target)?;
    Ok(())
}

fn wake_up(
    entity: usize,
    world: &mut World,
) -> Result<()> {
    if let Some(Behavior {state, ..}) = world.borrow_entity_component_mut(entity) {
        *state = BehaviorState::Awake;
    };
    Ok(())
}

fn aggressive_melee(
    entity: usize,
    ai_state: BehaviorState,
    world: &World,
) -> Result<AIAction> {
    let action;

    if ai_state == BehaviorState::Asleep {
        action = AIAction::Sleep;
        return Ok(action);
    }

    if ai_state == BehaviorState::Alerted {
        action = AIAction::WakeUp;
        return Ok(action);
    }

    let Ok(player) = world.get_player_id() else {
        return Err(Error::NoPlayerFound);
    };

    let Some(Position(origin)) = world.borrow_entity_component::<Position>(entity) else {
        return Err(MonsterTurnError::NoPositionFound { entity }.into());
    };

    let Ok(player_position) = world.get_player_position() else {
        return Err(MonsterTurnError::NoPositionFound { entity: player }.into());
    };

    let Some(Combat {
        melee_attack: Some(attack),
        ..
    }) = world.borrow_entity_component::<Combat>(entity)
    else {
        return Err(MonsterTurnError::NoAttackFound {
            entity,
            attack_type: AttackType::Melee,
        }
        .into());
    };

    if origin.distance(player_position) <= attack.range {
        action = AIAction::Attack(player);
    } else {
        action = AIAction::Approach;
    }

    Ok(action)
}

fn aggressive_ranged(
    entity: usize,
    ai_state: BehaviorState,
    world: &World,
    map: &GameMap,
    resources: &ResourceManager,
) -> Result<AIAction> {
    let action;

    if ai_state == BehaviorState::Asleep {
        action = AIAction::Sleep;
        return Ok(action);
    }

    if ai_state == BehaviorState::Alerted {
        action = AIAction::WakeUp;
        return Ok(action);
    }

    let Ok(player) = world.get_player_id() else {
        return Err(Error::NoPlayerFound);
    };

    let Some(Position(origin)) = world.borrow_entity_component::<Position>(entity) else {
        return Err(MonsterTurnError::NoPositionFound { entity }.into());
    };

    let Ok(player_position) = world.get_player_position() else {
        return Err(MonsterTurnError::NoPositionFound { entity: player }.into());
    };

    let Some(Combat {
        ranged_attack: Some(attack),
        ..
    }) = world.borrow_entity_component::<Combat>(entity)
    else {
        return Err(MonsterTurnError::NoAttackFound {
            entity,
            attack_type: AttackType::Ranged,
        }
        .into());
    };

    if origin.distance(player_position) <= attack.range
        && line_of_sight(*origin, player_position, map, world, resources)
    {
        action = AIAction::Shoot(player);
    } else {
        action = AIAction::Approach;
    }

    Ok(action)
}
