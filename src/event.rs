use std::collections::HashMap;

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::map::GameMap;
use crate::resources::ResourceManager;
use crate::world::World;

use crate::error::Result;

pub mod response_functions;
pub mod interact_events;
pub mod combat_events;
pub mod spell_events;
pub mod stealth_events;
pub mod argument_names;


// All events send a payload of code to run, and source entity
// all responses decide whether to run that code and on whom (self or other)
// responses may pass string -> float arguments to the code
// responses may add additional code to run
//

// Events I have
//     Attack,
//     Shoot,
//     Interact,
//     Pick up
//     Spell,
//     Noise, ( not done yet )
//     Death

// Events I want
//     Spread, (like spread fire, spread disease, and so on. Use the effect component)
//     Dialogue,
//     Quest,


pub trait Event {
    type Response: EventResponse;
    fn apply(
        &self,
        event_data: EventArguments,
    ) -> Result<()>;
    fn source(&self) -> usize;
}

pub trait EventResponse: Sized + Clone + 'static {
    fn respond(
        &self,
        event: &dyn Event<Response = Self>,
        response_data: ResponseArguments,
    ) -> Result<()>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum ResponseFuctionName {
    None,
    #[default]
    Default,
    ReflectAll,
    ReflectSome(f64),
    OpenDoor,
    OpenChest,
    RevealRoom,
    Close,
    Pickup,
    GrantLevelUp,
    DropInventory,
}

type ResponseFunction<T> = fn(
    event: &dyn Event<Response = T>,
    response_data: ResponseArguments,
    args: &HashMap<String, f64>,
    msg_args: &HashMap<String, String>,
) -> Result<()>;

impl ResponseFuctionName {
    pub fn get_callable<T: EventResponse>(&self) -> Result<ResponseFunction<T>> {
        match self {
            Self::Default => Ok(response_functions::respond_default),
            Self::ReflectAll => Ok(response_functions::respond_reflect),
            Self::ReflectSome(chance) => 
                match thread_rng().gen_bool(*chance) {
                    true => Ok(response_functions::respond_reflect),
                    false => Ok(response_functions::respond_default),
                },
            Self::OpenDoor => Ok(response_functions::respond_open_door),
            Self::OpenChest => Ok(response_functions::respond_open_chest),
            Self::DropInventory => Ok(response_functions::respond_drop),
            Self::Close => Ok(response_functions::respond_close),
            Self::Pickup => Ok(response_functions::respond_pickup),
            Self::GrantLevelUp => Ok(response_functions::respond_levelup),
            Self::RevealRoom => Ok(response_functions::respond_reveal_room),
            _ => Err("No callable registered for this name".into()),
        }
    }
}

pub struct EventArguments<'a> {
    pub world: &'a mut World,
    pub map: &'a mut GameMap,
    pub resources: &'a ResourceManager,
    pub source: usize,
    pub target: usize,
    pub args: &'a HashMap<String, f64>,
    pub msg_args: &'a HashMap<String, String>,
}

impl<'a> EventArguments<'a> {

    pub fn new(source: usize, target: usize, args: &'a HashMap<String, f64>, msg_args: &'a HashMap<String, String>, world: &'a mut World, map: &'a mut GameMap, resources: &'a ResourceManager) -> Self {
        Self {
            source,
            target,
            args,
            msg_args,
            world: world,
            map: map,
            resources: resources,
        }
    }
    pub fn new_from(source: usize, target: usize, args: &'a HashMap<String, f64>, msg_args: &'a HashMap<String, String>, response_data: ResponseArguments<'a>) -> Self {
        Self {
            source,
            target,
            args,
            msg_args,
            world: response_data.world,
            map: response_data.map,
            resources: response_data.resources,
        }
    }
}

pub struct ResponseArguments<'a> {
    pub world: &'a mut World,
    pub map: &'a mut GameMap,
    pub resources: &'a ResourceManager,
    pub entity: usize,
}

impl<'a> ResponseArguments<'a> {
    pub fn new(
        world: &'a mut World,
        map: &'a mut GameMap,
        resources: &'a ResourceManager,
        entity: usize
    ) -> Self 
    {
        Self { world, map, resources, entity }
    }
}