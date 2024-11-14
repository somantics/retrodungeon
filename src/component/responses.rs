use std::collections::HashMap;

use derive_entity_template::EventResponse;
use serde::{Deserialize, Serialize};

use crate::event::{argument_names::ARG_DAMAGE_MULTIPLIER, Event, EventResponse, ResponseFuctionName, EventArguments, ResponseArguments};

use crate::error::Result;


#[derive(Debug, Clone, Serialize, Default, Deserialize, EventResponse)]
pub struct SpellResponse {
    args: HashMap<String, f64>,
    msg_args: HashMap<String, String>,
    response: ResponseFuctionName,
}

#[derive(Debug, Clone, Serialize, Default, Deserialize, EventResponse)]
pub struct NoiseResponse {
    pub threshold: u32,
    args: HashMap<String, f64>,
    msg_args: HashMap<String, String>,
    response: ResponseFuctionName,
}

impl NoiseResponse {
    pub fn new(threshold: u32) -> Self {
        Self { threshold, ..Default::default() }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, EventResponse)]
pub struct AttackResponse {
    args: HashMap<String, f64>,
    msg_args: HashMap<String, String>,
    response: ResponseFuctionName,
}

impl AttackResponse {
    pub fn new_half_damage() -> Self {
        let mut args = HashMap::new();
        args.insert(ARG_DAMAGE_MULTIPLIER.to_string(), 0.5);

        Self {
            args,
            msg_args: HashMap::default(),
            response: ResponseFuctionName::Default,
        }
    }

    pub fn new_with_args(args: HashMap<String, f64>, msg_args: HashMap<String, String>,) -> Self {
        Self {
            args,
            msg_args,
            response: ResponseFuctionName::Default,
        }
    }

    pub fn new_reflect() -> Self {
        Self {
            args: HashMap::new(),
            msg_args: HashMap::default(),
            response: ResponseFuctionName::ReflectAll,
        }
    }
}


#[derive(Debug, Clone, Default, Serialize, Deserialize, EventResponse)]
pub struct ShootResponse {
    args: HashMap<String, f64>,
    msg_args: HashMap<String, String>,
    response: ResponseFuctionName,
}

impl ShootResponse {
    pub fn new_half_damage() -> Self {
        let mut args = HashMap::new();
        args.insert(ARG_DAMAGE_MULTIPLIER.to_string(), 0.5);

        Self {
            args,
            msg_args: HashMap::default(),
            response: ResponseFuctionName::Default,
        }
    }

    pub fn new_with_args(args: HashMap<String, f64>, msg_args: HashMap<String, String>,) -> Self {
        Self {
            args,
            msg_args,
            response: ResponseFuctionName::Default,
        }
    }

    pub fn new_reflect() -> Self {
        Self {
            args: HashMap::new(),
            msg_args: HashMap::default(),
            response: ResponseFuctionName::ReflectAll,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, EventResponse)]
pub struct InteractResponse {
    pub args: HashMap<String, f64>,
    pub msg_args: HashMap<String, String>,
    pub response: ResponseFuctionName,
}

#[derive(Debug, Clone, Serialize, Deserialize, EventResponse)]
pub struct PickupResponse {
    pub args: HashMap<String, f64>,
    pub msg_args: HashMap<String, String>,
    pub response: ResponseFuctionName,
}

impl Default for PickupResponse {
    fn default() -> Self {
        Self {
            args: HashMap::default(),
            msg_args: HashMap::default(),
            response: ResponseFuctionName::Pickup,
        }
    }
}


#[derive(Debug, Clone, Default, Serialize, Deserialize, EventResponse)]
pub struct DeathResponse {
    pub args: HashMap<String, f64>,
    pub msg_args: HashMap<String, String>,
    pub response: ResponseFuctionName,
}

