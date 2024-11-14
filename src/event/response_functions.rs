use std::collections::HashMap;

use log::debug;

use crate::{
    component::{
        attributes::Xp, responses::InteractResponse, image::ImageState, items::Coins, Collision,
        Position, SightBlocking,
    },
    logger,
    map::{
        tile::{Los, Passable}, utils::Coordinate, GameMap
    },
    resources::{self, id::ImageID, ResourceManager},
    world::World,
};

use super::{argument_names::{MSG_ARG_ATTACKER, MSG_ARG_ATTACK_MESSAGE}, Event, EventArguments, EventResponse, ResponseArguments, ResponseFuctionName};
use crate::error::Result;

pub fn respond_default<T: EventResponse>(
    event: &dyn Event<Response = T>,
    response_data: ResponseArguments,
    args: &HashMap<String, f64>,
    msg_args: &HashMap<String, String>,
) -> Result<()> {
    let event_data = EventArguments::new_from(event.source(), response_data.entity, args, msg_args, response_data);
    event.apply(event_data)
}

pub fn respond_reflect<T: EventResponse>(
    event: &dyn Event<Response = T>,
    response_data: ResponseArguments,
    args: &HashMap<String, f64>,
    msg_args: &HashMap<String, String>,
) -> Result<()> {
    let mut reflect_msg_args = msg_args.clone();
    reflect_msg_args.insert(MSG_ARG_ATTACK_MESSAGE.into(), "reflected your attack".into());
    reflect_msg_args.insert(MSG_ARG_ATTACKER.into(), response_data.entity.to_string());

    let event_data = EventArguments::new_from(event.source(), event.source(), args, &reflect_msg_args, response_data);
    event.apply(event_data)
}

pub fn respond_open_door<T: EventResponse>(
    event: &dyn Event<Response = T>,
    response_data: ResponseArguments,
    args: &HashMap<String, f64>,
    msg_args: &HashMap<String, String>,
) -> Result<()> {
    let ResponseArguments { world, map, resources, entity } = response_data;

    change_collision(entity, Passable::Walk, world)?;
    change_sight_blocking(entity, Los::Clear, world)?;
    change_interact_response(entity, ResponseFuctionName::Close, world)?;
    change_image_state(entity, "open", world)?;
    explore_room_of_entity(entity, &world, map, resources)?;
    make_noise(50, entity, world, map, resources)?;

    let event_data = EventArguments::new(
        response_data.entity, 
        event.source(), 
        args, 
        msg_args,
        world, 
        map, 
        resources
    );
    event.apply(event_data)
}

pub fn respond_open_chest<T: EventResponse>(
    event: &dyn Event<Response = T>,
    response_data: ResponseArguments,
    args: &HashMap<String, f64>,
    msg_args: &HashMap<String, String>,
) -> Result<()> {
    let ResponseArguments { world, map, resources, entity } = response_data;
    give_coins(entity, event.source(), world)?;
    make_noise(50, entity, world, map, resources)?;
    change_image_state(entity, "open", world)?;
    change_interact_response(entity, ResponseFuctionName::Default, world)?;

    let event_data = EventArguments::new(
        response_data.entity, 
        event.source(), 
        args, 
        msg_args, 
        world, 
        map, 
        resources
    );
    event.apply(event_data)
}

pub fn respond_close<T: EventResponse>(
    event: &dyn Event<Response = T>,
    response_data: ResponseArguments,
    args: &HashMap<String, f64>,
    msg_args: &HashMap<String, String>,
) -> Result<()> {
    let ResponseArguments { world, map, resources, entity } = response_data;
    change_collision(entity, Passable::Walk, world)?;
    change_sight_blocking(entity, Los::Block, world)?;
    change_interact_response(entity, ResponseFuctionName::OpenDoor, world)?;
    change_image_state(entity, "closed", world)?;

    let event_data = EventArguments::new(
        response_data.entity, 
        event.source(), 
        args, 
        msg_args, 
        world, 
        map, 
        resources
    );
    event.apply(event_data)
}

pub fn respond_reveal_room<T: EventResponse>(
    event: &dyn Event<Response = T>,
    response_data: ResponseArguments,
    args: &HashMap<String, f64>,
    msg_args: &HashMap<String, String>,
) -> Result<()> {
    let ResponseArguments { world, map, resources, entity } = response_data;
    explore_room_of_entity(entity, &world, map, resources)?;

    let event_data = EventArguments::new(
        response_data.entity, 
        event.source(), 
        args, 
        msg_args, 
        world, 
        map, 
        resources
    );
    event.apply(event_data)
}

pub fn respond_pickup<T: EventResponse>(
    event: &dyn Event<Response = T>,
    response_data: ResponseArguments,
    args: &HashMap<String, f64>,
    msg_args: &HashMap<String, String>,
) -> Result<()> {
    let ResponseArguments { world, map, resources, entity } = response_data;
    give_coins(entity, event.source(), world)?;
    //give_inventory(entity, event.source(), world)?;

    let event_data = EventArguments::new(
        response_data.entity, 
        event.source(), 
        args, 
        msg_args, 
        world, 
        map, 
        resources
    );
    event.apply(event_data)?;

    world.remove_entity(entity)?;

    Ok(())
}

pub fn respond_drop<T: EventResponse>(
    event: &dyn Event<Response = T>,
    response_data: ResponseArguments,
    args: &HashMap<String, f64>,
    msg_args: &HashMap<String, String>,
) -> Result<()> {
    let ResponseArguments { world, map, resources, entity } = response_data;
    if let Some(Position(location)) = world.borrow_entity_component::<Position>(entity) {
        drop_coins(entity, *location, world, map, resources)?;
    }

    let event_data = EventArguments::new(
        response_data.entity, 
        event.source(), 
        args, 
        msg_args, 
        world, 
        map, 
        resources
    );
    event.apply(event_data)
}

pub fn respond_levelup<T: EventResponse>(
    event: &dyn Event<Response = T>,
    response_data: ResponseArguments,
    args: &HashMap<String, f64>,
    msg_args: &HashMap<String, String>,
) -> Result<()> {
    let ResponseArguments { world, map, resources, entity } = response_data;
    give_full_xp(entity, event.source(), world)?;

    let event_data = EventArguments::new(
        response_data.entity, 
        event.source(), 
        args, 
        msg_args, 
        world, 
        map, 
        resources
    );
    event.apply(event_data)?;

    Ok(())
}

fn change_sight_blocking(entity: usize, new_state: Los, world: &mut World) -> Result<()> {
    let sight_block = world.borrow_entity_component_mut::<SightBlocking>(entity);
    match sight_block {
        Some(SightBlocking(los)) => {
            *los = new_state;
            Ok(())
        }
        _ => Err("No sight block to change".into()),
    }
}

fn change_collision(entity: usize, new_state: Passable, world: &mut World) -> Result<()> {
    let collision = world.borrow_entity_component_mut::<Collision>(entity);
    match collision {
        Some(Collision(passable)) => {
            *passable = new_state;
            Ok(())
        }
        _ => Err("No collision to change".into()),
    }
}

fn change_interact_response(
    entity: usize,
    new_response: ResponseFuctionName,
    world: &mut World,
) -> Result<()> {
    let response = world.borrow_entity_component_mut::<InteractResponse>(entity);
    match response {
        Some(response) => {
            response.response = new_response;
            Ok(())
        }
        _ => Err("No interact response to change".into()),
    }
}

fn change_image_state(entity: usize, new_state: &'static str, world: &mut World) -> Result<()> {
    let new_image: Option<&ImageID>;

    match world.borrow_entity_component_mut::<ImageState>(entity) {
        Some(image_states) => {
            if !image_states.states.contains_key(new_state) {
                return Err("Invalid image state".into());
            }

            image_states.current = new_state.to_string();
            new_image = image_states.states.get(new_state);
        }
        _ => return Err("No image state to change".into()),
    }

    if let Some(new_image) = new_image.cloned() {
        match world.borrow_entity_component_mut::<ImageID>(entity) {
            Some(id) => *id = new_image,
            _ => return Err("No image to update while changing image state".into()),
        }
    }
    Ok(())
}

fn explore_room_of_entity(
    entity: usize,
    world: &World,
    map: &mut GameMap,
    resources: &ResourceManager,
) -> Result<()> {
    let Some(Position(coordinate)) = world.borrow_entity_component::<Position>(entity) else {
        return Err("Entity has no position".into());
    };
    map.explore_room(*coordinate);
    map.explore_hallway(*coordinate, world, resources);
    Ok(())
}

fn give_coins(own_entity: usize, other_entity: usize, world: &mut World) -> Result<()> {
    let Some(coins_vec) = world.borrow_component_vec_mut::<Coins>() else {
        return Err("No coin vector found".into());
    };

    let Some(Coins(amount)) = coins_vec[own_entity] else {
        return Err("No coins to give".into());
    };

    let Some(ref mut their_coins) = coins_vec[other_entity] else {
        return Err("No coin purse to give to".into());
    };

    *their_coins += Coins(amount);

    if let Some(ref mut own_coins) = coins_vec[own_entity] {
        own_coins.0 = 0;
    };

    if let Some(xp) = world.borrow_entity_component_mut::<Xp>(other_entity) {
        xp.current += amount;
    }

    logger::log_message(&format!("Found {amount} gold!"));
    Ok(())
}

fn give_full_xp(_own_entity: usize, other_entity: usize, world: &mut World) -> Result<()> {
    if let Some(xp) = world.borrow_entity_component_mut::<Xp>(other_entity) {
        xp.current = xp.max;
    }

    Ok(())
}

fn _give_inventory(_own_entity: usize, _other_entity: usize, _world: &mut World) -> Result<()> {
    todo!()
}


fn drop_coins(own_entity: usize, location: Coordinate, world: &mut World, map: &GameMap, resources: &ResourceManager) -> Result<()> {
    let Some(own_coins) = world.borrow_entity_component::<Coins>(own_entity) else {
        return Err("No coins to drop".into());
    };
    let Coins(amount) = own_coins.clone();


    debug!("Coins to drop: {amount}");

    if amount == 0 {
        debug!("No coins detected.");
        return Ok(());
    }

    let templates = resources.get_entity_templates(resources::GOLD_PILE_SPAWNABLE);
    let gold_pile = world.spawn_from_templates(&templates, map.depth, location, resources)?;


    let Some(Coins(pile_amount)) = world.borrow_entity_component_mut::<Coins>(gold_pile) else {
        return Err("No coins on new gold pile".into());
    };

    *pile_amount = amount;

    Ok(())
}

fn make_noise(magnitude: u32, own_entity: usize, world: &mut World, map: &mut GameMap, resources: &ResourceManager) -> Result<()> {
    let event = super::stealth_events::NoiseEvent::new(own_entity, magnitude);

    let Some(Position(location)) = world.borrow_entity_component(own_entity) else {
        return Err(format!("Position not found: {own_entity}").into());
    };
    let room = world.get_room_at_coordinate(*location);

    let entities = room.entities.clone();
    for entity in entities {
        world.send_event(map, resources, &event, entity)?;
    }

    Ok(())
}