use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use std::collections::HashMap;

use crate::component::tags::{Door, Monster};
use crate::resources::ResourceManager;
use crate::world::World;

use super::utils::{Coordinate, DOWN, LEFT, RIGHT, UP};
use super::GameMap;

#[derive(Debug, Hash, Clone, Copy)]
struct NodeData {
    distance: u32,
    h_value: u32,
    parent: Option<Coordinate>,
}

impl NodeData {
    fn new(h_value: u32) -> Self {
        NodeData {
            distance: 0,
            h_value,
            parent: None,
        }
    }
    fn get_comparable(&self) -> u32 {
        self.distance + self.h_value
    }
}

impl Ord for NodeData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_comparable().cmp(&other.get_comparable())
    }
}

impl PartialOrd for NodeData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get_comparable().partial_cmp(&other.get_comparable())
    }
}

impl PartialEq for NodeData {
    fn eq(&self, other: &Self) -> bool {
        self.get_comparable() == other.get_comparable()
    }
}

impl Eq for NodeData {}

pub fn pathfind<F>(
    origin: Coordinate,
    destination: Coordinate,
    map: &GameMap,
    world: &World,
    resources: &ResourceManager,
    heuristic: F,
    ignore_units: bool,
    ignore_doors: bool,
    hazard_cost: u32,
) -> Option<impl Iterator<Item = Coordinate>>
where
    F: Fn(Coordinate) -> u32,
{
    let return_early = true;
    let origin_h_value = heuristic(origin);

    let neighbors = [DOWN, UP, LEFT, RIGHT];

    let mut open = PriorityQueue::new();
    let mut closed: HashMap<Coordinate, NodeData> = HashMap::new();
    let mut last_node: (Coordinate, NodeData) = (origin, NodeData::new(origin_h_value));

    open.push(origin, Reverse(NodeData::new(origin_h_value)));

    (last_node, closed) = fill_path_map(
        open,
        closed,
        last_node,
        &neighbors,
        &destination,
        heuristic,
        return_early,
        ignore_units,
        ignore_doors,
        hazard_cost,
        map,
        world,
        resources,
    );

    // check if we have a solution
    if last_node.0 != destination {
        return None;
    }
    backtrace_path(last_node, closed, origin)
}

fn backtrace_path(
    mut last_node: (Coordinate, NodeData),
    closed: HashMap<Coordinate, NodeData>,
    origin: Coordinate,
) -> Option<impl Iterator<Item = Coordinate>> {
    let mut sequence: Vec<Coordinate> = Vec::new();

    while let Some(parent) = last_node.1.parent {
        let current = last_node.0;
        let delta = current - parent;
        sequence.push(delta);

        if parent == origin {
            break;
        }

        last_node = (
            parent,
            *closed.get(&parent).expect("Failed to find note data."),
        );
    }

    Some(sequence.into_iter().rev())
}

fn get_passable_neighbors(
    neighbors: &[Coordinate],
    visited_coord: &Coordinate,
    ignore_units: bool,
    ignore_doors: bool,
    map: &GameMap,
    world: &World,
    resources: &ResourceManager,
) -> Vec<Coordinate> {
    neighbors
        .into_iter()
        .map(|dir| *visited_coord + *dir)
        .filter(|&coord| {
            let walkable = map.is_tile_walkable(coord, resources);
            let no_blocking_entity = match world.get_blocking_entity(coord) {
                Some(entity) => {
                    let mut can_ignore_entity = false;
                    if ignore_units && world.borrow_entity_component::<Monster>(entity).is_some() {
                        can_ignore_entity = true;
                    };
                    if ignore_doors && world.borrow_entity_component::<Door>(entity).is_some() {
                        can_ignore_entity = true;
                    };

                    can_ignore_entity
                }
                None => true,
            };
            walkable && no_blocking_entity
        })
        .collect()
}

fn fill_path_map<F>(
    mut open: PriorityQueue<Coordinate, Reverse<NodeData>>,
    mut closed: HashMap<Coordinate, NodeData>,
    mut last_node: (Coordinate, NodeData),
    neighbors: &[Coordinate],
    destination: &Coordinate,
    heuristic: F,
    return_early: bool,
    ignore_units: bool,
    ignore_doors: bool,
    hazard_cost: u32,
    map: &GameMap,
    world: &World,
    resources: &ResourceManager,
) -> ((Coordinate, NodeData), HashMap<Coordinate, NodeData>)
where
    F: Fn(Coordinate) -> u32,
{
    while let Some((visited_coord, Reverse(visited_data))) = open.pop() {
        // add visited node to closed
        closed.insert(visited_coord, visited_data);
        last_node = (visited_coord, visited_data);

        if visited_coord == *destination && return_early {
            break;
        }

        let passable_neighbors = get_passable_neighbors(
            neighbors,
            &visited_coord,
            ignore_units,
            ignore_doors,
            map,
            world,
            resources,
        );

        for neighbor_coord in passable_neighbors {
            // neighbor already visited
            if closed.contains_key(&neighbor_coord) {
                continue;
            }
            let cost = match world.coordinate_has_hazard(neighbor_coord) {
                true => hazard_cost,
                false => 1,
            };
            let distance_through_here = visited_data.distance + cost;
            // neighbor in open set already
            if let Some(Reverse(neigbor_data)) = open.get_priority(&neighbor_coord) {
                if neigbor_data.distance > distance_through_here {
                    open.change_priority(
                        &neighbor_coord,
                        Reverse(NodeData {
                            distance: distance_through_here,
                            h_value: heuristic(neighbor_coord),
                            parent: Some(visited_coord),
                        }),
                    );
                }
            // add neighbor to open set
            } else {
                open.push(
                    neighbor_coord,
                    Reverse(NodeData {
                        distance: distance_through_here,
                        h_value: heuristic(neighbor_coord),
                        parent: Some(visited_coord),
                    }),
                );
            }
        }
    }
    return (last_node, closed);
}

pub fn calculate_pathing_grid(
    destination: Coordinate,
    map: &GameMap,
    world: &World,
    resources: &ResourceManager,
    ignore_units: bool,
    ignore_doors: bool,
    hazard_cost: u32,
) -> HashMap<Coordinate, Coordinate> {
    let return_early = false;
    let origin = destination;
    let origin_h_value = 0;
    let heuristic = |_| 0;

    let neighbors = [DOWN, UP, LEFT, RIGHT];

    let mut open = PriorityQueue::new();
    let mut closed: HashMap<Coordinate, NodeData> = HashMap::new();
    let last_node: (Coordinate, NodeData) = (origin, NodeData::new(origin_h_value));

    open.push(origin, Reverse(NodeData::new(origin_h_value)));

    (_, closed) = fill_path_map(
        open,
        closed,
        last_node,
        &neighbors,
        &destination,
        heuristic,
        return_early,
        ignore_units,
        ignore_doors,
        hazard_cost,
        map,
        world,
        resources,
    );

    closed
        .into_iter()
        .filter_map(|(coord, NodeData { parent, .. })| {
            if let Some(parent) = parent {
                Some((coord, parent - coord))
            } else {
                None
            }
        })
        .collect()
}

pub fn astar_heuristic_factory(destination: Coordinate) -> impl Fn(Coordinate) -> u32 {
    move |coordinate: Coordinate| {
        ((coordinate.x - destination.x).abs() + (coordinate.y - destination.y).abs()) as u32
    }
}
