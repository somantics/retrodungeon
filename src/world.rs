use log::{debug, info};
use petgraph::{graph::NodeIndex, Graph};

use crate::component::attributes::{Attributes, Xp};
use crate::component::combat::Combat;
use crate::component::health::Health;
use crate::component::image::Image;
use crate::component::items::{Coins, Inventory};
use crate::component::spell::Spellbook;
use crate::component::Name;
use crate::error::{Error, Result};
use crate::event::ResponseArguments;
use crate::map::GameMap;
use crate::resources::ResourceManager;
use crate::{
    component::{
        tags::{Hazard, Player},
        Collision, Position, SightBlocking,
    },
    event::{Event, EventResponse},
    map::{
        room::{EntityContainer, Room},
        tile::{Los, Passable},
        utils::Coordinate,
    },
    spawning::entitytemplate::{EntityTemplate, EntityTemplateEnum},
};

pub type EntityGraph = Graph<EntityContainer, (), petgraph::Undirected>;

pub struct World {
    entity_count: usize,
    component_vecs: Vec<Box<dyn ComponentVec>>,
    bsp: EntityGraph,
    player: Option<usize>,
}

impl World {
    pub fn new_with(bsp: EntityGraph) -> Self {
        Self {
            entity_count: 0,
            component_vecs: Vec::new(),
            bsp,
            player: None,
        }
    }

    pub fn import_player(&mut self, old_world: &World) -> Result<()> {
        let Ok(old_player) = old_world.get_player_id() else {
            return Err(Error::NoPlayerFound);
        };

        let new_player = self.get_player_id()?;

        self.mark_as_player(new_player)?;

        // With dynamic list of components, there is no easy way to iterate over the components of a single entity
        // CORE COMPONENTS
        if let Some(name) = old_world.borrow_entity_component::<Name>(old_player) {
            self.add_component(new_player, name.clone())?;
        }

        if let Some(image) = old_world.borrow_entity_component::<Image>(old_player) {
            self.add_component(new_player, image.clone())?;
        }

        if let Some(collision) = old_world.borrow_entity_component::<Collision>(old_player) {
            self.add_component(new_player, collision.clone())?;
        }

        if let Some(sight_block) = old_world.borrow_entity_component::<SightBlocking>(old_player) {
            self.add_component(new_player, sight_block.clone())?;
        }

        // COMBAT COMPONENTS
        if let Some(component) = old_world.borrow_entity_component::<Combat>(old_player) {
            self.add_component(new_player, component.clone())?;
        }

        if let Some(component) = old_world.borrow_entity_component::<Health>(old_player) {
            self.add_component(new_player, component.clone())?;
        }

        // STATS COMPONENTS
        if let Some(component) = old_world.borrow_entity_component::<Xp>(old_player) {
            self.add_component(new_player, component.clone())?;
        }

        if let Some(component) = old_world.borrow_entity_component::<Attributes>(old_player) {
            self.add_component(new_player, component.clone())?;
        }

        // INVENTORY COMPONENTS
        if let Some(component) = old_world.borrow_entity_component::<Coins>(old_player) {
            self.add_component(new_player, component.clone())?;
        }

        if let Some(component) = old_world.borrow_entity_component::<Inventory>(old_player) {
            self.add_component(new_player, component.clone())?;
        }

        // SPELLS
        if let Some(component) = old_world.borrow_entity_component::<Spellbook>(old_player) {
            self.add_component(new_player, component.clone())?;
        }

        Ok(())
    }

    pub fn debug_print_entity(&self, entity: usize) {
        let name = self.borrow_entity_component::<crate::component::Name>(entity);
        let position = self.borrow_entity_component::<crate::component::Position>(entity);
        let image = self.borrow_entity_component::<crate::resources::id::ImageID>(entity);

        debug!("{entity}: {name:?} {position:?} {image:?}");
    }

    pub fn debug_print_all(&self) {
        for entity in 0..self.entity_count {
            self.debug_print_entity(entity);
        }
    }

    pub fn spawn_from_templates(
        &mut self,
        templates: &Vec<&EntityTemplateEnum>,
        depth: u32,
        position: Coordinate,
        resources: &ResourceManager,
    ) -> Result<usize> {
        let entity: usize = self.new_entity();

        // this is dangerous and may leave broken state if failure happens half-way through!
        for template in templates {
            template.add_components(entity, self, depth, resources)?;
        }

        self.add_position(entity, Position::new(position))?;

        Ok(entity)
    }

    pub fn new_entity(&mut self) -> usize {
        for vec in self.component_vecs.iter_mut() {
            vec.push_none();
        }

        let entity_id = self.entity_count;
        self.entity_count += 1;
        entity_id
    }

    pub fn remove_entity(&mut self, entity: usize) -> Result<()> {
        if entity >= self.entity_count {
            Err("Entity id out of bounds")?
        }

        for vec in &mut self.component_vecs {
            vec.set_none(entity);
        }

        debug!("Removed entity {entity}");
        Ok(())
    }

    pub fn add_position(&mut self, entity: usize, component: Position) -> Result<()> {
        if entity >= self.entity_count {
            Err("Entity id out of bounds")?
        }

        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<Vec<Option<Position>>>()
            {
                let destination = component.coordinate();
                component_vec[entity] = Some(component);
                self.update_position(entity, destination);
                return Ok(());
            }
        }

        self.add_new_component_type::<Position>(Some((entity, component)))?;
        let destination = component.coordinate();
        self.update_position(entity, destination);

        Ok(())
    }

    pub fn add_component<ComponentType: 'static>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) -> Result<()> {
        if entity >= self.entity_count {
            Err("Entity id out of bounds")?
        }

        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<Vec<Option<ComponentType>>>()
            {
                component_vec[entity] = Some(component);
                return Ok(());
            }
        }

        self.add_new_component_type::<ComponentType>(Some((entity, component)))?;

        Ok(())
    }

    pub fn remove_component<ComponentType: 'static>(&mut self, entity: usize) -> Result<()> {
        if entity >= self.entity_count {
            Err("Entity id out of bounds")?
        }

        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<Vec<Option<ComponentType>>>()
            {
                component_vec[entity] = None;
                return Ok(());
            }
        }

        Err("Tried to remove unknown component type".into())
    }

    fn add_new_component_type<ComponentType: 'static>(
        &mut self,
        new_component: Option<(usize, ComponentType)>,
    ) -> Result<()> {
        let mut new_component_vec: Vec<Option<ComponentType>> =
            Vec::with_capacity(self.entity_count);

        for _ in 0..self.entity_count {
            new_component_vec.push_none();
        }

        if let Some((index, component)) = new_component {
            if index >= self.entity_count {
                Err("Entity id out of bounds")?
            }

            new_component_vec[index] = Some(component);
        }
        self.component_vecs.push(Box::new(new_component_vec));
        Ok(())
    }

    pub fn borrow_component_vec<ComponentType: 'static>(
        &self,
    ) -> Option<&Vec<Option<ComponentType>>> {
        self.component_vecs
            .iter()
            .find_map(|vec| vec.as_any().downcast_ref::<Vec<Option<ComponentType>>>())
    }

    pub fn borrow_component_vec_mut<ComponentType: 'static>(
        &mut self,
    ) -> Option<&mut Vec<Option<ComponentType>>> {
        self.component_vecs.iter_mut().find_map(|vec| {
            vec.as_any_mut()
                .downcast_mut::<Vec<Option<ComponentType>>>()
        })
    }

    pub fn borrow_entity_component<ComponentType: 'static>(
        &self,
        entity: usize,
    ) -> Option<&ComponentType> {
        let Some(component_vec) = self.borrow_component_vec::<ComponentType>() else {
            return None;
        };

        component_vec[entity].as_ref()
    }

    pub fn borrow_entity_component_mut<ComponentType: 'static>(
        &mut self,
        entity: usize,
    ) -> Option<&mut ComponentType> {
        let Some(component_vec) = self.borrow_component_vec_mut::<ComponentType>() else {
            return None;
        };

        component_vec[entity].as_mut()
    }

    pub fn send_event<T: Event>(
        &mut self,
        map: &mut GameMap,
        resources: &ResourceManager,
        event: &T,
        target: usize,
    ) -> Result<()> {
        let Some(response) = self.borrow_entity_component::<T::Response>(target) else {
            return Ok(());
        };
        let response = response.clone();
        let response_data = ResponseArguments::new(
            self,
            map,
            resources,
            target,
        );

        response.respond(event, response_data)
    }

    pub fn get_entities_in_room(&self, coordinate: Coordinate) -> &Vec<usize> {
        let room = self.get_room_at_coordinate(coordinate);
        &room.entities
    }

    pub fn get_entities_at_coordinate(&self, coordinate: Coordinate) -> Vec<usize> {
        let room = self.get_room_at_coordinate(coordinate);
        room.entities
            .iter()
            .filter_map(
                |entity| match self.borrow_entity_component::<Position>(*entity) {
                    Some(position) if *position == coordinate => Some(*entity),
                    _ => None,
                },
            )
            .collect()
    }

    pub fn get_room_at_coordinate(&self, coordinate: Coordinate) -> &EntityContainer {
        let root_index = NodeIndex::<u32>::new(0);
        let last_searched =
            Self::binary_search_rooms(root_index, root_index, coordinate, &self.bsp);

        &self.bsp[last_searched]
    }

    fn get_room_at_coordinate_mut(&mut self, coordinate: Coordinate) -> &mut EntityContainer {
        let root_index = NodeIndex::<u32>::new(0);
        let last_searched =
            Self::binary_search_rooms(root_index, root_index, coordinate, &self.bsp);

        &mut self.bsp[last_searched]
    }

    pub fn get_room_data_from(&self, other: &Room) -> EntityContainer {
        let coordinate = other.extends.top_left;
        let entities = self.get_room_at_coordinate(coordinate).entities.clone();
        EntityContainer {
            extends: other.extends,
            entities,
        }
    }

    fn binary_search_rooms(
        index: NodeIndex<u32>,
        parent: NodeIndex<u32>,
        coord: Coordinate,
        graph: &EntityGraph,
    ) -> NodeIndex<u32> {
        if graph.edges(index).count() == 1 {
            return index;
        }

        let mut children = graph
            .neighbors(index)
            .filter(|node_index| *node_index != parent);
        let (first_child, second_child) = (children.next().unwrap(), children.next().unwrap());
        let first_room = graph.node_weight(first_child).unwrap().extends;

        match first_room.contains_point(coord) {
            true => Self::binary_search_rooms(first_child, index, coord, graph),
            false => Self::binary_search_rooms(second_child, index, coord, graph),
        }
    }

    pub fn update_position(&mut self, entity: usize, destination: Coordinate) {
        let Some(Position(origin)) = self.borrow_entity_component::<Position>(entity) else {
            return;
        };

        let old_room = self.get_room_at_coordinate_mut(*origin);
        if let Some(index) = old_room.entities.iter().position(|value| *value == entity) {
            old_room.entities.remove(index);
        }

        let new_room = self.get_room_at_coordinate_mut(destination);
        new_room.entities.push(entity);

        let Some(position) = self.borrow_entity_component_mut::<Position>(entity) else {
            return;
        };
        position.move_to(destination);
    }

    pub fn mark_as_player(&mut self, entity: usize) -> Result<()> {
        if let Ok(old_player) = self.get_player_id() {
            self.remove_component::<Player>(old_player)?;
        }

        self.add_component(entity, Player {})?;
        self.player = Some(entity);
        Ok(())
    }

    pub fn get_blocking_entity(&self, coordinate: Coordinate) -> Option<usize> {
        let entities = self.get_entities_at_coordinate(coordinate);
        entities.into_iter().find(|entity| {
            match self.borrow_entity_component::<Collision>(*entity) {
                Some(Collision(Passable::None)) => true,
                _ => false,
            }
        })
    }

    pub fn get_sight_blocking_entity(&self, coordinate: Coordinate) -> Option<usize> {
        let entities = self.get_entities_at_coordinate(coordinate);
        entities.into_iter().find(|entity| {
            match self.borrow_entity_component::<SightBlocking>(*entity) {
                Some(SightBlocking(Los::Block)) => true,
                _ => false,
            }
        })
    }

    pub fn coordinate_has_hazard(&self, coordinate: Coordinate) -> bool {
        let entities = self.get_entities_at_coordinate(coordinate);
        entities
            .into_iter()
            .find(|entity| self.borrow_entity_component::<Hazard>(*entity).is_some())
            .is_some()
    }

    pub fn get_player_id(&self) -> Result<usize> {
        self.player.ok_or("No player has been set".into())
    }

    pub fn get_player_position(&self) -> Result<Coordinate> {
        let Some(player) = self.player else {
            return Err("No player has been set".into());
        };

        let Some(Position(coordinate)) = self.borrow_entity_component::<Position>(player) else {
            return Err("can't find player position".into());
        };

        Ok(*coordinate)
    }
}

trait ComponentVec {
    fn push_none(&mut self);
    fn set_none(&mut self, entity: usize);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: 'static> ComponentVec for Vec<Option<T>> {
    fn push_none(&mut self) {
        self.push(None);
    }

    fn set_none(&mut self, entity: usize) {
        self[entity] = None;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
