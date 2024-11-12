use petgraph::{algo, graph::NodeIndex, visit::IntoNodeReferences, Graph};
use std::collections::HashSet;

use crate::{
    resources::{ResourceManager, DEFAULT_TILEID, FLOOR_TILEID, WALL_TILEID},
    spawning,
    world::EntityGraph,
};

use super::{
    boxextends::{self, BoxExtends},
    room::Room,
    utils::{Coordinate, DOWN, LEFT, RIGHT, UP},
    GameMap,
};

pub type RoomGraph = Graph<Room, (), petgraph::Undirected>;

pub fn generate_new(
    size_x: u32,
    size_y: u32,
    depth: u32,
    resources: &ResourceManager,
) -> (GameMap, EntityGraph) {
    let bsp: RoomGraph;
    let mut graph: RoomGraph;
    loop {
        bsp = binary_space_partitioning(size_x, size_y, 4);
        graph = make_rooms_from_bsp(&bsp, 5);
        graph = prune_small_rooms(&graph, 6);
        graph = make_connected_graph(&graph, 6);
        graph = prune_edges(&graph, 4, 2);

        let islands = algo::connected_components(&graph);
        if islands == 1 {
            break;
        }
        break;
    }

    let map = draw_rooms_to_map(&graph, size_x, size_y, depth);
    let map = add_doors_to_rooms(&map, resources);
    let map = spawning::flood_fill_spawn_tables(&map, resources);
    let bsp = entity_bsp_from_room_bsp(bsp);
    (map, bsp)
}

fn binary_space_partitioning(size_x: u32, size_y: u32, max_depth: u32) -> RoomGraph {
    // Recursive algorithm for generating a binary space partitioning on BoxExtends.
    // Allows overlapping walls.
    let mut graph = RoomGraph::new_undirected();
    let map_box = BoxExtends {
        top_left: Coordinate::default(),
        bottom_right: Coordinate {
            x: (size_x - 1) as i32,
            y: (size_y - 1) as i32,
        },
    };
    let map_room = Room::new(map_box);
    let origin = graph.add_node(map_room);
    split_branch(origin, &mut graph, 0, max_depth);

    graph
}

// Inner recursive function, adds nodes to 'graph' directly.
fn split_branch(parent: NodeIndex, graph: &mut RoomGraph, current_depth: u32, max_depth: u32) {
    if current_depth >= max_depth {
        return;
    }

    let parent_box = graph.node_weight(parent).unwrap().extends;
    let Ok((a, b)) = boxextends::split_box(&parent_box) else {
        return; //don't split further
    };
    let branch_a = graph.add_node(Room::new(a));
    let branch_b = graph.add_node(Room::new(b));

    graph.extend_with_edges(&[(parent, branch_a), (parent, branch_b)]);

    split_branch(branch_a, graph, current_depth + 1, max_depth);

    split_branch(branch_b, graph, current_depth + 1, max_depth);
}

// Generates rooms inside the partitioned areas. Returned as a new graph.
fn make_rooms_from_bsp(bsp_tree: &RoomGraph, min_side_length: i32) -> RoomGraph {
    let bsp_leaves = leaves_from_bsp(&bsp_tree);
    let mut graph = Graph::<Room, (), petgraph::Undirected>::default();

    for index in bsp_leaves {
        let room_box = match bsp_tree.node_weight(index) {
            Some(room) => boxextends::random_subbox(&room.extends, min_side_length),
            None => continue,
        };

        // Removes any existing room data beyond extends
        graph.add_node(Room::new(room_box));
    }

    graph
}

fn leaves_from_bsp<'a>(graph: &'a RoomGraph) -> impl Iterator<Item = NodeIndex> + 'a {
    graph
        .node_indices()
        .filter(|index| graph.neighbors_undirected(*index).count() == 1)
}

// Takes a graph of nodes, removes original edges and supplies edges between geographic neighbors.
fn make_connected_graph(room_graph: &RoomGraph, max_scan_distance: i32) -> RoomGraph {
    let mut new_graph = RoomGraph::default();
    new_graph.clone_from(room_graph);
    new_graph.clear_edges();

    let mut unprocessed = room_graph.node_references(); // this moves room_graph
    let mut opened: Vec<(NodeIndex, &Room)> = vec![];
    let mut closed: Vec<NodeIndex> = vec![];

    let mut current_node: NodeIndex;
    let mut current_area: &Room;

    loop {
        // Select next node to process
        if opened.len() == 0 {
            // if none in open list, get from  unprocessed
            (current_node, current_area) = match unprocessed.next() {
                Some(tuple) => tuple,
                None => break,
            };
        } else {
            // take from opened list
            (current_node, current_area) = match opened.pop() {
                Some(tuple) => tuple,
                None => break,
            };
        }
        closed.push(current_node);

        // find neighbors using collision boxes to the top, bottom, right, left
        let collision_boxes: Vec<BoxExtends> =
            boxextends::make_edge_vicinity_boxes(&current_area.extends, max_scan_distance, 2);

        let neighbors = unprocessed
            .clone()
            .filter(|(_, area)| {
                collision_boxes
                    .iter()
                    .any(|collision| area.extends.overlaps(collision))
            })
            .filter(|(index, _)| !closed.contains(index));

        // add hits to opened list
        opened.extend(neighbors.clone());

        // make new edges
        new_graph.extend_with_edges(neighbors.map(|(index, _)| (current_node, index)));
    }

    new_graph
}

// Rebuilds graph without rooms w. floor area less than the threshold.
fn prune_small_rooms(graph: &RoomGraph, threshold: i32) -> RoomGraph {
    let mut pruned_graph = RoomGraph::default();
    let filtered_rooms = graph
        .node_indices()
        .map(|index| graph.node_weight(index).unwrap())
        .filter(|room: &&Room| room.extends.get_inner_area() > threshold);

    for weight in filtered_rooms {
        pruned_graph.add_node(weight.clone());
    }

    pruned_graph
}

// Prune edges from rooms with edge_count over the threshold, attempting to maintain connectivity
fn prune_edges(graph: &RoomGraph, edge_threshold: usize, max_trim_amount: usize) -> RoomGraph {
    let mut pruned_graph = RoomGraph::default();
    pruned_graph.clone_from(graph);

    for room in pruned_graph.node_indices() {
        let neighbor_count = pruned_graph.neighbors(room).count();
        if !(neighbor_count >= edge_threshold) {
            continue;
        }

        let mut neighbors: Vec<NodeIndex> = pruned_graph.neighbors(room).collect();
        neighbors.sort_by_key(|index| graph.neighbors(*index).count());

        for index in 0..neighbors.len().min(max_trim_amount) {
            let neighbor = neighbors[index];
            let edge_candidate = pruned_graph.find_edge(room, neighbor).unwrap();

            pruned_graph.remove_edge(edge_candidate);

            // Do not prune if connectivity is compromised.
            if algo::connected_components(&pruned_graph) != 1 {
                pruned_graph.add_edge(room, neighbor, ());
            }
        }
    }

    pruned_graph
}

fn draw_rooms_to_map(graph: &RoomGraph, size_x: u32, size_y: u32, depth: u32) -> GameMap {
    let mut map = GameMap::new(size_x, size_y);
    map.room_graph = graph.clone();
    map.depth = depth;
    let leaves = graph.node_indices();

    // Drawing empty rooms
    for index in leaves {
        let room_box: BoxExtends = match graph.node_weight(index) {
            Some(weight) => weight.extends,
            None => continue,
        };

        draw_room(room_box, &mut map);
    }

    // Drawing corridors
    let neighbor_pairs = graph
        .edge_indices()
        .map(|index| graph.edge_endpoints(index).unwrap());

    for (room_a, room_b) in neighbor_pairs {
        draw_path_between_rooms(
            &mut map,
            &graph.node_weight(room_a).unwrap().extends,
            &graph.node_weight(room_b).unwrap().extends,
        )
    }
    map
}

fn draw_room(room_box: BoxExtends, map: &mut GameMap) {
    let (left, top) = (room_box.top_left.x, room_box.top_left.y);
    let (right, bottom) = (room_box.bottom_right.x, room_box.bottom_right.y);

    for x in left..=right {
        // top row
        map.set_game_tile(Coordinate { x, y: top }, WALL_TILEID);

        // bottom row
        map.set_game_tile(Coordinate { x, y: bottom }, WALL_TILEID);

        for y in (top + 1)..bottom {
            let tile;
            if x == left || x == right {
                tile = WALL_TILEID;
            } else {
                tile = FLOOR_TILEID;
            }
            map.set_game_tile(Coordinate { x: x, y: y }, tile);
        }
    }
}

fn draw_path_between_rooms(map: &mut GameMap, box_a: &BoxExtends, box_b: &BoxExtends) {
    // case overlap in x
    let a_x_range: HashSet<i32> = HashSet::from_iter(box_a.top_left.x + 1..box_a.bottom_right.x);
    let b_x_range: HashSet<i32> = HashSet::from_iter(box_b.top_left.x + 1..box_b.bottom_right.x);
    let x_range_overlap: HashSet<i32> = a_x_range.intersection(&b_x_range).map(|i| *i).collect();

    if x_range_overlap.len() > 0 {
        let corridor_x = *x_range_overlap.iter().next().unwrap();
        let corridor_start = Coordinate {
            x: corridor_x,
            y: box_a.center().y,
        };
        let corridor_end = Coordinate {
            x: corridor_x,
            y: box_b.center().y,
        };

        draw_vertical_corridor(corridor_start, corridor_end, map);
        return;
    }
    // case overlap in y
    let a_y_range: HashSet<i32> = HashSet::from_iter(box_a.top_left.y + 1..box_a.bottom_right.y);
    let b_y_range: HashSet<i32> = HashSet::from_iter(box_b.top_left.y + 1..box_b.bottom_right.y);
    let y_range_overlap: HashSet<i32> = a_y_range.intersection(&b_y_range).map(|i| *i).collect();

    if y_range_overlap.len() > 0 {
        let corridor_y = *y_range_overlap.iter().next().unwrap();
        let corridor_start = Coordinate {
            x: box_a.center().x,
            y: corridor_y,
        };
        let corridor_end = Coordinate {
            x: box_b.center().x,
            y: corridor_y,
        };

        draw_horizontal_corridor(corridor_start, corridor_end, map);
    }
}

fn draw_vertical_corridor(start: Coordinate, end: Coordinate, map: &mut GameMap) {
    let center = |y| Coordinate { x: start.x, y: y };

    let left_of = |coord: Coordinate| Coordinate {
        x: coord.x - 1,
        ..coord
    };
    let right_of = |coord: Coordinate| Coordinate {
        x: coord.x + 1,
        ..coord
    };

    let (low_y, high_y) = if start.y < end.y {
        (start.y, end.y)
    } else {
        (end.y, start.y)
    };

    for y in low_y..=high_y {
        match (
            map.get_game_tile(left_of(center(y))),
            map.get_game_tile(center(y)),
            map.get_game_tile(right_of(center(y))),
        ) {
            (WALL_TILEID, WALL_TILEID, WALL_TILEID) => {
                map.set_game_tile(center(y), FLOOR_TILEID);
            }
            (DEFAULT_TILEID, WALL_TILEID, FLOOR_TILEID | WALL_TILEID) => {
                if map.get_game_tile(center(y-1)) == WALL_TILEID {
                    continue;
                }
                map.set_game_tile(left_of(center(y)), WALL_TILEID);
                map.set_game_tile(center(y), FLOOR_TILEID);
            }
            (FLOOR_TILEID | WALL_TILEID, WALL_TILEID, DEFAULT_TILEID) => {
                if map.get_game_tile(center(y-1)) == WALL_TILEID {
                    continue;
                }
                map.set_game_tile(center(y), FLOOR_TILEID);
                map.set_game_tile(right_of(center(y)), WALL_TILEID);
            }
            (_, DEFAULT_TILEID, _) => {
                map.set_game_tile(left_of(center(y)), WALL_TILEID);
                map.set_game_tile(center(y), FLOOR_TILEID);
                map.set_game_tile(right_of(center(y)), WALL_TILEID);
            }

            _ => {}
        }
    }
}

fn draw_horizontal_corridor(start: Coordinate, end: Coordinate, map: &mut GameMap) {
    let center = |x| Coordinate { x: x, y: start.y };
    let above = |coord: Coordinate| Coordinate {
        y: coord.y - 1,
        ..coord
    };
    let below = |coord: Coordinate| Coordinate {
        y: coord.y + 1,
        ..coord
    };

    let (low_x, high_x) = if start.x < end.x {
        (start.x, end.x)
    } else {
        (end.x, start.x)
    };

    for x in low_x..=high_x {
        match (
            map.get_game_tile(above(center(x))),
            map.get_game_tile(center(x)),
            map.get_game_tile(below(center(x))),
        ) {
            (WALL_TILEID, WALL_TILEID, WALL_TILEID) => {
                map.set_game_tile(center(x), FLOOR_TILEID);
            }

            (DEFAULT_TILEID, WALL_TILEID, FLOOR_TILEID | WALL_TILEID) => {
                if map.get_game_tile(center(x-1)) == WALL_TILEID {
                    continue;
                }
                map.set_game_tile(above(center(x)), WALL_TILEID);
                map.set_game_tile(center(x), FLOOR_TILEID);
            }

            (FLOOR_TILEID | WALL_TILEID, WALL_TILEID, DEFAULT_TILEID) => {
                if map.get_game_tile(center(x-1)) == WALL_TILEID {
                    continue;
                }
                map.set_game_tile(center(x), FLOOR_TILEID);
                map.set_game_tile(below(center(x)), WALL_TILEID);
            }

            (_, DEFAULT_TILEID, _) => {
                map.set_game_tile(above(center(x)), WALL_TILEID);
                map.set_game_tile(center(x), FLOOR_TILEID);
                map.set_game_tile(below(center(x)), WALL_TILEID);
            }

            _ => {}
        }
    }
}

fn check_door_conditions(coord: Coordinate, map: &GameMap, resources: &ResourceManager) -> bool {
    if !map.is_tile_walkable(coord, resources) {
        return false;
    }

    let directions = [UP, DOWN, LEFT, RIGHT];
    let walkable_neighbors = directions
        .iter()
        .map(|dir| coord + *dir)
        .filter(|neighbor| map.is_tile_walkable(*neighbor, resources))
        .count();

    if walkable_neighbors != 2 {
        return false;
    }

    true
}

fn add_doors_to_rooms(map: &GameMap, resources: &ResourceManager) -> GameMap {
    let mut new_graph: RoomGraph = Graph::default();
    new_graph.clone_from(&map.room_graph);

    for (node, room) in map.room_graph.node_references() {
        let (left, top) = (room.extends.top_left.x, room.extends.top_left.y);
        let (right, bottom) = (room.extends.bottom_right.x, room.extends.bottom_right.y);
        let mut door_locations = vec![];

        // Horizontal walls + corners
        for x in left..=right {
            let top_coord = Coordinate { x, y: top };
            let bottom_coord = Coordinate { x, y: bottom };

            if check_door_conditions(top_coord, map, resources) {
                door_locations.push(top_coord);
            }
            if check_door_conditions(bottom_coord, map, resources) {
                door_locations.push(bottom_coord);
            }
        }

        // Vertical walls (not the corners)
        for y in top + 1..bottom {
            let left_coord = Coordinate { x: left, y };
            let right_coord = Coordinate { x: right, y };

            if check_door_conditions(left_coord, map, resources) {
                door_locations.push(left_coord);
            }
            if check_door_conditions(right_coord, map, resources) {
                door_locations.push(right_coord);
            }
        }

        let new_room = Room {
            door_locations,
            ..room.clone()
        };
        new_graph[node] = new_room;
    }

    let mut new_map = map.clone();
    new_map.room_graph = new_graph;
    new_map
}

fn entity_bsp_from_room_bsp(room_graph: RoomGraph) -> EntityGraph {
    let mut bsp = EntityGraph::new_undirected();

    let nodes = room_graph
        .node_references()
        .into_iter()
        .map(|(_, room)| room.into());

    for node in nodes {
        bsp.add_node(node);
    }

    for edge in room_graph.edge_indices() {
        if let Some((a, b)) = room_graph.edge_endpoints(edge) {
            bsp.add_edge(a, b, ());
        }
    }
    
    bsp
}
