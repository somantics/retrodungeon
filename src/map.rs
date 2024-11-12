pub mod boxextends;
pub mod generation;
pub mod los;
pub mod pathfinding;
pub mod room;
pub mod tile;
pub mod utils;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use itertools::Itertools;
use petgraph::Graph;
use utils::flood_fill_explore;


use crate::{
    resources::{id::TileID, ResourceManager},
    world::World,
};

use {
    pathfinding::calculate_pathing_grid,
    room::Room,
    tile::{Los, Passable},
    utils::Coordinate,
};

const DEFAULT_HEIGHT: u32 = 32;
const DEFAULT_WIDTH: u32 = 32;

#[derive(Clone)]
pub struct GameMap {
    pub map: HashMap<Coordinate, TileID>,
    pub explored: RefCell<HashSet<Coordinate>>,
    pub room_graph: Graph<Room, (), petgraph::Undirected>,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub pathing_grid: HashMap<Coordinate, Coordinate>,
}

impl GameMap {
    pub fn new(width: u32, height: u32) -> Self {
        let map = HashMap::<Coordinate, TileID>::new();
        let explored = RefCell::new(HashSet::<Coordinate>::new());
        let room_graph = Graph::default();
        let pathing_grid = HashMap::new();

        Self {
            map,
            explored,
            width,
            height,
            depth: 0,
            room_graph,
            pathing_grid,
        }
    }

    pub fn is_tile_walkable(&self, coordinate: Coordinate, resources: &ResourceManager) -> bool {
        let Some(tile_id) = self.map.get(&coordinate) else {
            return false;
        };

        let Some(tile) = resources.get_tile(*tile_id) else {
            return false;
        };

        tile.passable == Passable::Walk
    }

    pub fn is_tile_sight_blocking(
        &self,
        coordinate: Coordinate,
        resources: &ResourceManager,
    ) -> bool {
        let Some(tile_id) = self.map.get(&coordinate) else {
            return false;
        };

        let Some(tile) = resources.get_tile(*tile_id) else {
            return false;
        };

        tile.los == Los::Block
    }

    pub fn is_tile_explored(&self, coordinate: Coordinate) -> bool {
        self.explored.borrow().contains(&coordinate)
    }

    pub fn is_tile_explored_from_index(&self, index: u32) -> bool {
        let coordinate = Coordinate {
            x: (index % self.width) as i32,
            y: (index / self.width) as i32,
        };
        self.explored.borrow().contains(&coordinate)
    }

    pub fn is_tile_void(&self, coordinate: Coordinate) -> bool {
        let Some(tile_id) = self.map.get(&coordinate) else {
            return false;
        };

        *tile_id == crate::resources::DEFAULT_TILEID
    }

    pub fn set_game_tile(&mut self, coordinate: Coordinate, tile: TileID) {
        self.map.insert(coordinate, tile);
    }

    pub fn get_game_tile(&self, coordinate: Coordinate) -> TileID {
        match self.map.get(&coordinate) {
            Some(tile) => *tile,
            None => TileID(0),
        }
    }

    pub fn get_game_tile_from(&self, index: u32) -> TileID {
        let coordinate = Coordinate {
            x: (index % self.width) as i32,
            y: (index / self.width) as i32,
        };
        match self.map.get(&coordinate) {
            Some(tile) => *tile,
            None => TileID(0),
        }
    }

    pub fn update_pathing_grid(
        &mut self,
        destination: Coordinate,
        world: &World,
        resources: &ResourceManager,
    ) {
        let safe_pathing_grid = calculate_pathing_grid(
            destination,
            self,
            world,
            resources,
            true,
            false,
            std::u32::MAX,
        );

        self.pathing_grid = safe_pathing_grid;
    }

    pub fn explore_room(&mut self, coordinate: Coordinate) {
        if let Some(room) = self.get_room_at_coordinate_mut(coordinate) {
            let x_range = room.extends.top_left.x..=room.extends.bottom_right.x;
            let y_range = room.extends.top_left.y..=room.extends.bottom_right.y;

            let inside_locations = x_range
                .cartesian_product(y_range)
                .map(|(x, y)| Coordinate { x, y });

            for coordinate in inside_locations {
                self.explored.borrow_mut().insert(coordinate);
            }
        };
    }

    pub fn explore_hallway(
        &mut self,
        coordinate: Coordinate,
        world: &World,
        resources: &ResourceManager,
    ) {
        flood_fill_explore(coordinate, self, world, resources);
    }

    fn get_room_at_coordinate(&self, coordinate: Coordinate) -> Option<&Room> {
        self.room_graph
            .node_weights()
            .find(|room| room.extends.contains_point(coordinate))
    }

    fn get_room_at_coordinate_mut(&mut self, coordinate: Coordinate) -> Option<&mut Room> {
        self.room_graph
            .node_weights_mut()
            .find(|room| room.extends.contains_point(coordinate))
    }
}

impl Default for GameMap {
    fn default() -> Self {
        Self::new(DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }
}
