use itertools::Itertools;
use log::debug;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{collections::{HashSet, VecDeque}, ops::Range, hash::Hash, fmt::Debug};

use crate::{resources::ResourceManager, world::World};

use super::GameMap;

pub const UP: Coordinate = Coordinate { x: 0, y: -1 };
pub const DOWN: Coordinate = Coordinate { x: 0, y: 1 };
pub const LEFT: Coordinate = Coordinate { x: -1, y: 0 };
pub const RIGHT: Coordinate = Coordinate { x: 1, y: 0 };

pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(
    Hash, PartialEq, Eq, Clone, Copy, Debug, Default, Ord, PartialOrd, Serialize, Deserialize,
)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

impl Coordinate {
    pub fn new(x: u32, y: u32) -> Self {
        Coordinate {
            x: x as i32,
            y: y as i32,
        }
    }

    pub fn zero() -> Coordinate {
        Coordinate { x: 0, y: 0 }
    }

    pub fn uniform(xy: u32) -> Self {
        Coordinate {
            x: xy as i32,
            y: xy as i32,
        }
    }

    pub fn random(x_range: &Range<i32>, y_range: &Range<i32>) -> Self {
        Self {
            x: thread_rng().gen_range(x_range.clone()),
            y: thread_rng().gen_range(y_range.clone()),
        }
    }

    pub fn distance(&self, other: Coordinate) -> f64 {
        let delta_x = self.x - other.x;
        let delta_y = self.y - other.y;

        ((delta_x.pow(2) + delta_y.pow(2)) as f64).sqrt()
    }
}

impl std::ops::AddAssign for Coordinate {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Add for Coordinate {
    type Output = Coordinate;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Coordinate {
    type Output = Coordinate;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<i32> for Coordinate {
    type Output = Coordinate;
    fn mul(self, rhs: i32) -> Self::Output {
        Coordinate {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

pub fn reverse_direction(direction: &Coordinate) -> Coordinate {
    Coordinate {
        x: -direction.x,
        y: -direction.y,
    }
}


pub fn flood_fill_explore(start: Coordinate, map: &mut GameMap, world: &World, resources: &ResourceManager) {
    let get_neighbors = |node: &Coordinate| {
        [UP,DOWN, LEFT, RIGHT]
            .into_iter()
            .map(|dir| dir + *node)
            .filter(|neighbor| {
                !map.is_tile_explored(*neighbor)
                && map.is_tile_walkable(*neighbor, resources)
                && world.get_blocking_entity(*neighbor).is_none()
                && world.get_sight_blocking_entity(*neighbor).is_none()
            })
            .collect()
    };

    let mut coordinates_to_explore = Vec::new();

    flood_fill(start, get_neighbors, |node| coordinates_to_explore.push(node));

    for coordinate in coordinates_to_explore {
        let diagonal_neighbors = (-1..=1).cartesian_product(-1..=1)
            .into_iter()
            .map(|(x, y)| Coordinate {x, y} + coordinate);

        for neighbor in diagonal_neighbors {
            debug!("Exploring: {neighbor:?} from {coordinate:?}");
            map.explored.borrow_mut().insert(neighbor);
        }
    }
}

pub fn flood_fill<T: Hash + Eq + Clone + Debug, F, W>(start: T, get_neighbors: F, mut process_node: W) 
where 
    F: Fn(&T) -> Vec<T>,
    W: FnMut(T),
{
    let mut visited: HashSet<T> = HashSet::new();
    let mut fill_queue: VecDeque<T> = VecDeque::new();

    fill_queue.push_back(start);
    while let Some(node) = fill_queue.pop_front() {
        visited.insert(node.clone());

        let unvisited_neighbors: Vec<T> =
            get_neighbors(&node)
            .into_iter()
            .filter(|neighbor| !visited.contains(neighbor))
            .collect();

        fill_queue.extend(unvisited_neighbors);

        process_node(node);
    }
}