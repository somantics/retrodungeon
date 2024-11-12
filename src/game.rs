use log::{debug, warn};
use slint::ModelRc;

use crate::component::attributes::{attack_damage_boost, Attribute};
use crate::component::spell::{self, SpellEffectArguments, Spellbook};
use crate::component::tags::StairsDown;
use crate::component::{health, Position};
use crate::error::{Error, Result};
use crate::ui::SpellbookModel;
use crate::{
    component::{
        attributes::{Attributes, Xp, XpStatus},
        combat::{AttackType, Combat},
        health::{Health, HealthStatus},
        items::Coins,
        tags::Monster,
        Name,
    },
    event::{
        Event,
        combat_events::{AttackEvent, ShootEvent},
        interact_events::{InteractEvent, PickupEvent},
    },
    logger,
    map::{
        generation,
        los::line_of_sight,
        pathfinding::{astar_heuristic_factory, pathfind},
        tile::GameTile,
        utils::Coordinate,
        GameMap,
    },
    resources::{id::ImageID, ResourceManager},
    spawning::spawn_all_entities,
    system::{self, Scheduler},
    ui::{MapModel, PlayerModel},
    world::World,
};
use crate::system::definitions::*;

// GAME COMMANDS
// move player (dir)            check
// perform attack (dir)         check 
// interact (dir)               check
// shoot (position)             check
// interact (position)          check (only in melee range)
// pathfind player (position)   check (takes steps but doesn't slow down for the player to see)
// cast spell (id, position)    check
// wait/end turn                check

pub const MAP_SIZE_X: u32 = 16 * 2;
pub const MAP_SIZE_Y: u32 = 9 * 2;

pub const RANGE_EPSILON: f64 = 0.25;
pub const INTERACT_RANGE: f64 = 1.0 + RANGE_EPSILON;

pub struct Game {
    map: GameMap,
    world: World,
    scheduler: Scheduler,
}

impl Game {
    pub fn new(resources: &ResourceManager) -> Result<Self> {
        let mut world;
        let mut scheduler;
        let mut map;
        let mut bsp;let mut attempts = 0;
        loop {
            if attempts > 5 {
                return Err("Failed to generate level".into());
            } else {
                attempts += 1;
            }

            (map, bsp) = generation::generate_new(MAP_SIZE_X, MAP_SIZE_Y, 1, &resources);
            world = World::new_with(bsp);
            scheduler = Scheduler::new();

            let result = spawn_all_entities(&map, &mut world, resources);
            if let Err(_) = result {
                continue;
            }

            let result = explore_player_room(&mut map, &world);
            if let Err(_) = result {
                continue;
            }
            break;
        }

        scheduler.add_system(reap_units, system::TimeSlot::EndOfTurn);
        scheduler.add_system(level_up_check, system::TimeSlot::EndOfTurn);
        scheduler.add_system(monster_turns, system::TimeSlot::EndOfTurn);

        scheduler.add_system(spell_cooldowns, system::TimeSlot::EndOfLevel);

        Ok(Self {
            map,
            world,
            scheduler,
        })
    }

    pub fn get_map_info(&self) -> MapModel {
        (&self.map).into()
    }

    pub fn get_player_info(&self) -> PlayerModel {
        let mut model = PlayerModel::default();

        let Ok(player) = self.world.get_player_id() else {
            return model;
        };

        if let Some(name) = self.world.borrow_entity_component::<Name>(player) {
            model.name = name.0.clone();
        }

        if let Some(xp) = self.world.borrow_entity_component::<Xp>(player) {
            model.level = xp.level as i32;
            model.xp_current = xp.current as i32;
            model.xp_goal = xp.max as i32;
        }

        if let Some(health) = self.world.borrow_entity_component::<Health>(player) {
            model.hp_current = health.0 as i32;
            model.hp_max = health.1 as i32;
        }

        let maybe_attributes =  self.world.borrow_entity_component::<Attributes>(player);
        if let Some(attributes) = maybe_attributes {
            model.might = attributes.might as i32;
            model.wit = attributes.wit as i32;
            model.skill = attributes.skill as i32;
        }

        if let Some(combat) = self.world.borrow_entity_component::<Combat>(player) {
            if let Some(melee) = &combat.melee_attack {
                let mut min_damage = melee.damage_min;
                let mut max_damage = melee.damage_max;

                if let Some(attributes) = maybe_attributes {
                    min_damage  = attack_damage_boost(melee.damage_min as f64, melee, attributes) as u32;
                    max_damage = attack_damage_boost(melee.damage_max as f64, melee, attributes) as u32;
                };
                model.melee_damage = [min_damage as i32, max_damage as i32].into()
            }
            model.melee_crit = 0.0;

            if let Some(ranged) = &combat.ranged_attack {
                let mut min_damage = ranged.damage_min;
                let mut max_damage = ranged.damage_max;

                if let Some(attributes) = maybe_attributes {
                    min_damage  = attack_damage_boost(ranged.damage_min as f64, ranged, attributes) as u32;
                    max_damage = attack_damage_boost(ranged.damage_max as f64, ranged, attributes) as u32;
                };
                model.ranged_damage = [min_damage as i32, max_damage as i32].into()
            }
            model.ranged_crit = 0.0;
        }

        if let Some(coins) = self.world.borrow_entity_component::<Coins>(player) {
            model.coins = coins.0 as i32;
        }

        if let Some(xp) = self.world.borrow_entity_component::<Xp>(player) {
            model.xp_current = xp.current as i32;
            model.xp_goal = xp.max as i32;
        }

        model
    }

    pub fn get_spell_info(&self) -> SpellbookModel {
        let mut model = SpellbookModel::default();
        let Ok(player) = self.world.get_player_id() else {
            return model;
        };

        if let Some(spellbook) = self.world.borrow_entity_component::<Spellbook>(player) {
            let spells = spellbook.get_spells();

            let mut names = Vec::new();
            let mut casts: Vec<ModelRc<i32>> = Vec::new();
            //let mut damages = Vec::new();

            for spell in spells {
                let name = spell.name.0.clone();
                names.push(name.into());
                casts.push([spell.casts_left as i32, spell.casts_max as i32].into());
                //damages.push(vec![spell.casts_left, spell.casts_max]);
            }

            model.names = names.as_slice().into();
            model.casts = casts.as_slice().into();
        };

        model
    }

    pub fn get_sprite_ids(&self, resources: &ResourceManager) -> Vec<Vec<i32>> {
        let max_tile_index = self.map.height * self.map.width;

        let mut tile_images = Vec::new();

        for index in 0..max_tile_index {
            let mut images = Vec::new();

            if self.map.is_tile_explored_from_index(index) {
                let tile_id = self.map.get_game_tile_from(index);
                let tile = resources.get_tile(tile_id);
                let ImageID(image) = tile.unwrap_or(&GameTile::default()).image;
                images.push(image as i32);

                let entity_images = self.get_images_at_index(index);
                images.extend(entity_images);
            } else {
                images.push(crate::resources::DEFAULT_IMAGE_ID.0 as i32);
            }

            tile_images.push(images);
        }
        tile_images
    }

    fn get_images_at_index(&self, index: u32) -> impl Iterator<Item = i32> + use<'_> {
        let coordinate = Coordinate {
            x: (index % self.map.width) as i32,
            y: (index / self.map.width) as i32,
        };
        self.world
            .get_entities_at_coordinate(coordinate)
            .into_iter()
            .filter_map(|entity| self.get_entity_image(entity))
            .map(|image_id| image_id.0 as i32)
        //sort by depth later
    }

    fn get_entity_image(&self, entity: usize) -> Option<ImageID> {
        self.world
            .borrow_entity_component::<ImageID>(entity)
            .copied()
    }

    pub fn player_health_status(&self) -> HealthStatus {
        let Ok(player) = self.world.get_player_id() else {
            return HealthStatus::Full;
        };
        let Some(health) = self.world.borrow_entity_component::<Health>(player) else {
            return HealthStatus::Full;
        };
        (*health).into()
    }

    pub fn player_xp_status(&self) -> XpStatus {
        let Ok(player) = self.world.get_player_id() else {
            return XpStatus::Default;
        };

        let Some(xp) = self.world.borrow_entity_component::<Xp>(player) else {
            return XpStatus::Default;
        };

        xp.status
    }

    pub fn force_attack_command(
        &mut self,
        direction: Coordinate,
        resources: &ResourceManager,
    ) -> Result<()> {
        let Ok(origin) = self.world.get_player_position() else {
            return Err(Error::NoPlayerFound);
        };
        let destination = origin + direction;

        let result = self.perform_attack(destination, AttackType::Melee, resources, true);
        match result {
            Err(Error::InvalidTarget) => return Ok(()),
            Err(error) => debug!("{error}"),
            Ok(_) => {},
        }

        self.end_turn(resources)?;
        Ok(())
    }

    pub fn cast_spell_command(&mut self, index: usize, target: Coordinate, resources: &ResourceManager) -> Result<()> {
        let Ok(player) = self.world.get_player_id() else {
            Err(Error::NoPlayerFound)?
        };

        let Some(spellbook) = self.world.borrow_entity_component::<Spellbook>(player) else {
            return Err("Player has no spellbook".into());
        };
        let Some(spell) = spellbook.get_spell(index) else {
            return Err("That spell doesn't exist".into());
        };
        let spell = spell.clone();

        let args = SpellEffectArguments {
            source: player,
            target,
            world: &mut self.world,
            map: &mut self.map,
            stats: None,
            items: None,
            resources,
        };
        spell.cast(args)?;

        let Some(spellbook) = self.world.borrow_entity_component_mut::<Spellbook>(player) else {
            return Err("Player has no spellbook".into());
        };
        spellbook.register_cast(index)?;
        
        self.end_turn(resources)
    }

    pub fn direction_command(
        &mut self,
        direction: Coordinate,
        resources: &ResourceManager,
    ) -> Result<()> {
        let Ok(origin) = self.world.get_player_position() else {
            return Err(Error::NoPlayerFound);
        };
        let destination = origin + direction;

        let blocking_entity = self.world.get_blocking_entity(destination);
        let Some(blocking_entity) = blocking_entity else {
            self.move_direction(destination, resources)?;
            return Ok(());
        };

        let monster_tag = self
            .world
            .borrow_entity_component::<Monster>(blocking_entity);
        if monster_tag.is_some() {
            let result = self.perform_attack(destination, AttackType::Melee, resources, false);
            match result {
                Err(Error::InvalidTarget) => return Ok(()),
                Err(error) => debug!("{error}"),
                Ok(_) => {},
            }
            self.end_turn(resources)?;
            return Ok(());
        };

        self.interact_command(destination, resources)?;
        Ok(())
    }

    pub fn move_to_command(
        &mut self,
        destination: Coordinate,
        resources: &ResourceManager,
    ) -> Result<()> {
        let Ok(origin) = self.world.get_player_position() else {
            return Err(Error::NoPlayerFound);
        };

        if !self.map.is_tile_walkable(destination, resources) {
            logger::log_message("Destination tile is not walkable.");
            return Ok(());
        };

        if !self.map.is_tile_explored(destination) {
            logger::log_message("You haven't explored that tile.");
            return Ok(());
        }

        if self.world.get_blocking_entity(destination).is_some() {
            logger::log_message("That tile is blocked.");
            return Ok(());
        }

        if self.are_enemies_in_sight(origin, resources) {
            logger::log_message("Can't auto travel while enemies are in sight.");
            return Ok(());
        };

        let heuristic = astar_heuristic_factory(destination);

        let Some(path) = pathfind(
            origin,
            destination,
            &self.map,
            &self.world,
            resources,
            heuristic,
            false,
            false,
            std::u32::MAX,
        ) else {
            logger::log_message("Can't find path to destination.");
            return Ok(());
        };

        for step_direction in path {
            let Ok(origin) = self.world.get_player_position() else {
                return Err("No player position found".into());
            };

            if self.are_enemies_in_sight(origin, resources) {
                logger::log_message("Can't auto travel while enemies are in sight.");
                return Ok(());
            };

            let step = origin + step_direction;
            self.move_player(step, resources)?;
            self.end_turn(resources)?;
        }

        logger::log_message("Arrived at destination.");
        Ok(())
    }

    pub fn interact_command(
        &mut self,
        target: Coordinate,
        resources: &ResourceManager,
    ) -> Result<()> {
        let Ok(origin) = self.world.get_player_position() else {
            return Err(Error::NoPlayerFound);
        };

        if origin.distance(target) > INTERACT_RANGE {
            logger::log_message("Too far away to interact");
            return Ok(());
        }
        
        self.broadcast_interact(target, resources)?;
        self.end_turn(resources)?;
        Ok(())
    }

    pub fn shoot_command(&mut self, target: Coordinate, resources: &ResourceManager, force_attack: bool) -> Result<()> {
        let Ok(origin) = self.world.get_player_position() else {
            return Err(Error::NoPlayerFound);
        };

        if !self.map.is_tile_explored(target) {
            logger::log_message("You haven't explored that tile.");
            return Ok(());
        }

        if !line_of_sight(origin, target, &self.map, &self.world, resources) {
            logger::log_message("You can't see to that tile.");
            return Ok(());
        }

        let result = self.perform_attack(target, AttackType::Ranged, resources, force_attack);
            match result {
                Err(Error::InvalidTarget) => return Ok(()),
                Err(error) => debug!("{error}"),
                Ok(_) => {},
            }
        self.end_turn(resources)?;
        Ok(())
    }

    pub fn descend_command(&mut self, resources: &ResourceManager) -> Result<()> {
        let Ok(player_location) = self.world.get_player_position() else {
            return Err(Error::InvalidTarget);
        };

        let location_has_stairs = self
            .world
            .get_entities_at_coordinate(player_location)
            .into_iter()
            .find(|entity| {
                self.world
                    .borrow_entity_component::<StairsDown>(*entity)
                    .is_some()
            })
            .is_some();

        if !location_has_stairs {
            return Err(Error::InvalidTarget);
        }

        let new_depth = self.map.depth + 1;

        let mut new_world;
        let mut new_map;
        let mut new_bsp;
        let mut attempts = 0;
        loop {
            if attempts > 5 {
                return Err("Failed to generate level".into());
            } else {
                attempts += 1;
            }

            (new_map, new_bsp) = generation::generate_new(MAP_SIZE_X, MAP_SIZE_Y, new_depth, resources);
            new_world = World::new_with(new_bsp);

            let result = spawn_all_entities(&new_map, &mut new_world, resources);
            if let Err(_) = result {
                continue;
            };

            let result = new_world.import_player(&self.world);
            if let Err(_) = result {
                continue;
            };

            let result = explore_player_room(&mut new_map, &new_world);
            if let Err(_) = result {
                continue;
            };

            break;
        }

        self.map = new_map;
        self.world = new_world;

        self.scheduler.on_descend_floor(&mut self.world, &mut self.map, resources)?;

        Ok(())
    }

    pub fn wait_command(&mut self, resources: &ResourceManager) -> Result<()> {
        let result = self.descend_command(resources);
        match  result {
            Ok(_) => Ok(()),
            Err(Error::InvalidTarget) => {
                logger::log_message("Waited for a turn.");
                self.end_turn(resources)
            }  
            Err(error) => {
                Err(error)
            }
        }
    }

    fn end_turn(&mut self, resources: &ResourceManager) -> Result<()> {
        if let Ok(player_position) = self.world.get_player_position() {
            self.map
                .update_pathing_grid(player_position, &self.world, resources);
        };

        self.scheduler
            .on_end_turn(&mut self.world, &mut self.map, resources)?;
        Ok(())
    }

    pub fn level_up_command(&mut self, stat: Attribute, amount: u32) -> Result<()> {
        let Ok(player) = self.world.get_player_id() else {
            Err(Error::NoPlayerFound)?
        };

        let Some(attibutes_component) = self.world.borrow_entity_component_mut::<Attributes>(player) else {
            return Err("No player attributes".into());
        };

        match stat {
            Attribute::Might => { attibutes_component.might += amount; },
            Attribute::Wit => { 
                attibutes_component.wit += amount; 
                if let Some(spellbook) = self.world.borrow_entity_component_mut::<Spellbook>(player) {
                    spellbook.on_wit_increased(amount);
                }
            },
            Attribute::Skill => { attibutes_component.skill += amount; },
        }

        let Some(xp_component) = self.world.borrow_entity_component_mut::<Xp>(player) else {
            return Err("No player Xp".into());
        };   

        xp_component.level += 1;
        xp_component.max = (0..=xp_component.level).sum::<u32>() * 100;
        xp_component.status = XpStatus::Default;
        
        let Some(Health(current_hp, max_hp)) = self.world.borrow_entity_component_mut::<Health>(player) else {
            return Err("No player health".into());
        };

        *max_hp = (*max_hp as f64 * health::LEVEL_UP_MULTIPLIER) as u32;  
        *current_hp = *max_hp;


        Ok(())
    }

    fn move_direction(
        &mut self,
        destination: Coordinate,
        resources: &ResourceManager,
    ) -> Result<()> {
        let true = self.map.is_tile_walkable(destination, resources) else {
            logger::log_message("Can't walk there.");
            return Ok(());
        };
        self.move_player(destination, resources)?;
        self.end_turn(resources)?;
        Ok(())
    }

    fn move_player(&mut self, destination: Coordinate, resources: &ResourceManager) -> Result<()> {
        let Ok(player) = self.world.get_player_id() else {
            return Err(Error::NoPlayerFound);
        };
        self.world.update_position(player, destination);
        self.broadcast_pickup(destination, resources)?;
        Ok(())
    }

    fn broadcast_event(
        &mut self,
        event: impl Event,
        destination: Coordinate,
        resources: &ResourceManager,
    ) -> Result<()> {
        let targets = self.world.get_entities_at_coordinate(destination);

        for target in targets {
            let result = self
                .world
                .send_event(&mut self.map, resources, &event, target);

            match result {
                Err(error) => warn!("{error}"),
                Ok(_) => {}
            }
        }
        Ok(())
    }

    fn broadcast_interact(
        &mut self,
        destination: Coordinate,
        resources: &ResourceManager,
    ) -> Result<()> {
        let Ok(player_id) = self.world.get_player_id() else {
            return Err(Error::NoPlayerFound);
        };

        let event = InteractEvent { source: player_id };
        self.broadcast_event(event, destination, resources)
    }

    fn broadcast_pickup(
        &mut self,
        destination: Coordinate,
        resources: &ResourceManager,
    ) -> Result<()> {
        let Ok(player_id) = self.world.get_player_id() else {
            return Err(Error::NoPlayerFound);
        };

        let event = PickupEvent { source: player_id };
        self.broadcast_event(event, destination, resources)
    }

    fn are_enemies_in_sight(
        &mut self,
        coordinate: Coordinate,
        resources: &ResourceManager,
    ) -> bool {
        self.world
            .get_entities_in_room(coordinate)
            .into_iter()
            .filter(|entity| {
                self.world
                    .borrow_entity_component::<Monster>(**entity)
                    .is_some()
            })
            .find(|entity| {
                if let Some(Position(other)) =
                    self.world.borrow_entity_component::<Position>(**entity)
                {
                    line_of_sight(coordinate, *other, &self.map, &self.world, resources)
                } else {
                    false
                }
            })
            .is_some()
    }

    fn perform_attack(
        &mut self,
        target: Coordinate,
        attack_type: AttackType,
        resources: &ResourceManager,
        attack_non_hostile: bool,
    ) -> Result<()> {
        let Ok(player) = self.world.get_player_id() else {
            return Err(Error::NoPlayerFound);
        };
        let Ok(origin) = self.world.get_player_position() else {
            return Err(Error::NoPlayerFound);
        };
        let Some(combat) = self.world.borrow_entity_component::<Combat>(player) else {
            return Err("No player combat found".into());
        };

        let attack = match attack_type {
            AttackType::Melee => &combat.melee_attack,
            AttackType::Ranged => &combat.ranged_attack,
        };

        let Some(attack) = attack else {
            logger::log_message("You don't have an appropriate attack.");
            return Err(Error::InvalidTarget);
        };

        if origin.distance(target) > attack.range {
            logger::log_message("Target is out of range.");
            return Err(Error::InvalidTarget);
        }

        let Some(entity) = self.world.get_blocking_entity(target) else {
            logger::log_message("Nothing there to attack.");
            return Err(Error::InvalidTarget);
        };

        if entity == self.world.get_player_id()? {
            logger::log_message("Can't attack yourself.");
            return Err(Error::InvalidTarget);
        }

        if self.world.borrow_entity_component::<Monster>(entity).is_some() 
            || attack_non_hostile
        {
            match attack_type {
                AttackType::Melee => {
                    self.world.send_event(
                        &mut self.map,
                        resources,
                        &AttackEvent::new(player),
                        entity,
                    )?;
                }
                AttackType::Ranged => {
                    self.world.send_event(
                        &mut self.map,
                        resources,
                        &ShootEvent::new(player),
                        entity,
                    )?;
                }
            };
        } else {
            logger::log_message("Non hostile target, use ctrl to force an attack.");
            return Err(Error::InvalidTarget);
        }
        
        Ok(())
    }
}

fn explore_player_room(map: &mut GameMap, world: &World) -> Result<()> {
    let Ok(position) = world.get_player_position() else {
        warn!("can't explore player room");
        return Err(Error::NoPlayerFound);
    };

    map.explore_room(position);
    Ok(())
}
