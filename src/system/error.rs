use derive_more::derive::From;

use crate::{
    component::{
        behavior::{AIAction, BehaviorName},
        combat::AttackType,
    },
    map::utils::Coordinate,
};

#[derive(Debug, From)]
pub enum MonsterTurnError {
    NoPlayerFound,
    NoAttackFound {
        entity: usize,
        attack_type: AttackType,
    },
    NoPathfindingData {
        entity: usize,
        coordinate: Coordinate,
    },
    NoPositionFound {
        entity: usize,
    },
    ActionNotImplemented {
        entity: usize,
        action: AIAction,
    },
    BehaviorNotImplemented {
        entity: usize,
        behavior: BehaviorName,
    },
    FailedToCompleteAction {
        entity: usize,
        action: AIAction,
    },
    #[from]
    SerdeYaml(serde_yaml::Error),
}
