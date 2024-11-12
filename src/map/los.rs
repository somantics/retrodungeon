use crate::{resources::ResourceManager, world::World};

use super::{utils::Coordinate, GameMap};

pub fn line_of_sight(
    origin: Coordinate,
    destination: Coordinate,
    map: &GameMap,
    world: &World,
    resources: &ResourceManager,
) -> bool {
    if origin == destination {
        return true;
    }
    
    let full_line = linetrace(origin, destination);
    let line_between = &full_line[1..full_line.len() - 1];
    !los_block_on_line(line_between, map, world, resources)
}

fn los_block_on_line(
    line: &[Coordinate],
    map: &GameMap,
    world: &World,
    resources: &ResourceManager,
) -> bool {
    line.iter().any(|point| {
        map.is_tile_sight_blocking(*point, resources)
            || world.get_sight_blocking_entity(*point).is_some()
    })
}

fn collision_on_line(
    line: &[Coordinate],
    map: &GameMap,
    world: &World,
    resources: &ResourceManager,
) -> bool {
    line.iter().any(|point| {
        !map.is_tile_walkable(*point, resources) || world.get_blocking_entity(*point).is_some()
    })
}

fn linetrace(origin: Coordinate, destination: Coordinate) -> Vec<Coordinate> {
    let mut current_point = origin;
    let mut results: Vec<Coordinate> = Vec::new();

    let (mut dx, mut dy) = (destination.x - origin.x, destination.y - origin.y);
    let sx = if dx > 0 { 1 } else { -1 };
    let sy = if dy > 0 { 1 } else { -1 };

    (dx, dy) = (dx.abs(), -dy.abs());
    let mut error = dx + dy;
    let mut error2: i32;

    loop {
        results.push(current_point);
        if current_point.x == destination.x && current_point.y == destination.y {
            break;
        }
        error2 = 2 * error;
        if error2 >= dy {
            if current_point.x == destination.x {
                break;
            }
            error += dy;
            current_point.x += sx;
        }
        if error2 <= dx {
            if current_point.y == destination.y {
                break;
            }
            error += dx;
            current_point.y += sy;
        }
    }

    results
}
