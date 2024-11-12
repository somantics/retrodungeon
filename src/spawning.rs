use std::collections::{HashSet, VecDeque};
use log::{debug, warn};
use petgraph::{graph::NodeIndex, visit::IntoNodeReferences, Graph};
use rand::{thread_rng, Rng};
use roomtemplate::RoomTemplate;
use spawnentry::{SpawnEntry, SpawnEntryType};

use crate::error::Result;
use crate::{
    map::{
        generation::RoomGraph,
        room::{QuestGenerationData, Room, RoomGenerationData},
        utils::Coordinate,
        GameMap,
    },
    resources::{id::RoomTemplateID, ResourceManager, DOOR_SPAWNABLE},
    world::World,
};

pub mod entitytemplate;
pub mod roomtemplate;
pub mod spawnentry;

pub fn spawn_all_entities(
    map: &GameMap,
    world: &mut World,
    resources: &ResourceManager,
) -> Result<()> {
    let rooms = map.room_graph.node_weights();

    for room in rooms {
        spawn_doors(room, map, world, resources)?;

        // if spawn player, don't also spawn template
        if let Some(SpawnEntryType::Player(_)) = room.extra_spawn {
            spawn_player(room, map, world, resources)?;
            continue;
        }

        if let Some(SpawnEntryType::Stairs(_)) = room.extra_spawn {
            spawn_stairs(room, map, world, resources)?;
        }

        // spawn from templates
        let Some(template_id) = &room.template else {
            continue;
        };
        let Some(room_template) = resources.get_room_template(*template_id) else {
            continue;
        };

        spawn_room(room, room_template, map, world, resources)?;
    }

    world.debug_print_all();
    Ok(())
}

fn spawn_player(
    room: &Room,
    map: &GameMap,
    world: &mut World,
    resources: &ResourceManager,
) -> Result<()> {
    let spawn_entry = SpawnEntryType::Player(0);
    let mut quest_data = QuestGenerationData {};
    let room_data = RoomGenerationData {
        room: room.into(),
        level_depth: map.depth,
        room_depth: 0,
    };

    spawn_entry.spawn(&room_data, &mut quest_data, world, map, resources)?;
    Ok(())
}

fn spawn_doors(
    room: &Room,
    map: &GameMap,
    world: &mut World,
    resources: &ResourceManager,
) -> Result<()> {
    let id = DOOR_SPAWNABLE;
    let door_templates = resources.get_entity_templates(id);

    for position in &room.door_locations {
        debug!("Attempting to spawn door at {position:?}");
        match world.spawn_from_templates(&door_templates, map.depth, *position, resources) {
            Err(error) => warn!("{error}"),
            Ok(_) => {}
        }
    }
    Ok(())
}

fn spawn_room(
    room: &Room, 
    room_template: &RoomTemplate,
    map: &GameMap,
    world: &mut World,
    resources: &ResourceManager,
) -> Result<()> {
    let room_depth = room.room_depth.unwrap();
    let mut quest_data = QuestGenerationData {};

    for spawn_entry in &room_template.spawns {
        let Some(spawn_entry) = resources.get_spawn_entry(*spawn_entry) else {
            continue;
        };

        let container = world.get_room_data_from(room);
        let room_data = RoomGenerationData {
            room: container,
            level_depth: map.depth,
            room_depth,
        };

        if spawn_entry.evaluate(&room_data, &quest_data) {
            spawn_entry.spawn(&room_data, &mut quest_data, world, map, resources)?;
        }
    }

    Ok(())
}

fn spawn_stairs(
    room: &Room,
    map: &GameMap,
    world: &mut World,
    resources: &ResourceManager,
) -> Result<()> {
    let spawn_entry = SpawnEntryType::Stairs(0);
    let mut quest_data = QuestGenerationData {};
    let room_data = RoomGenerationData {
        room: room.into(),
        level_depth: map.depth,
        room_depth: 0,
    };

    spawn_entry.spawn(&room_data, &mut quest_data, world, map, resources)?;
    Ok(())
}

pub fn flood_fill_spawn_tables(map: &GameMap, resources: &ResourceManager) -> GameMap {
    let mut new_graph: RoomGraph = Graph::default();
    new_graph.clone_from(&map.room_graph);

    let mut top_left_corners: Vec<(NodeIndex, Coordinate)> = new_graph
        .node_references()
        .map(|(index, room)| (index, room.extends.top_left))
        .collect();
    top_left_corners.sort_unstable_by_key(|(_, coord)| *coord);
    let (start_index, _) = top_left_corners[0];
    let mut visited: HashSet<NodeIndex> = HashSet::new();
    let mut fill_queue: VecDeque<NodeIndex> = VecDeque::new();

    fill_queue.push_front(start_index);
    while let Some(index) = fill_queue.pop_back() {
        visited.insert(index);

        let unvisited_neighbors: Vec<NodeIndex> = new_graph
            .neighbors(index)
            .filter_map(|idx| (!visited.contains(&idx)).then(move || idx))
            .collect();

        if index == start_index {
            new_graph[index].room_depth = Some(0);
        }

        for unvisited_index in unvisited_neighbors {
            let new_depth = new_graph[index].room_depth.map(|x| x + 1);
            let old_depth = new_graph[unvisited_index].room_depth;

            new_graph[unvisited_index].room_depth = option_min(new_depth, old_depth);
            fill_queue.push_front(unvisited_index);
        }

        let room = new_graph[index].clone();
        let room_depth = new_graph[index].room_depth.unwrap();
        let room_data = RoomGenerationData {
            room: room.into(),
            level_depth: map.depth,
            room_depth,
        };

        let quest_data = QuestGenerationData {};

        let mut template = None;
        let viable_templates: Vec<RoomTemplateID> = resources
            .get_all_room_templates()
            .into_iter()
            .filter_map(
                |(id, template)| match template.validate(&room_data, &quest_data) {
                    true => Some(*id),
                    false => None,
                },
            )
            .collect();

        if viable_templates.len() > 0 {
            let random_index = thread_rng().gen_range(0..viable_templates.len());
            template = Some(viable_templates[random_index]);
        };

        let mut extra_spawn = None;

        if index == start_index {
            extra_spawn = Some(SpawnEntryType::Player(0));
        }

        if fill_queue.is_empty() {
            extra_spawn = Some(SpawnEntryType::Stairs(0));
        }

        new_graph[index] = Room {
            template,
            extra_spawn,
            ..new_graph[index].clone()
        };
    }

    GameMap {
        room_graph: new_graph,
        ..map.clone()
    }
}

fn option_min(lhs: Option<u32>, rhs: Option<u32>) -> Option<u32> {
    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => Some(lhs.min(rhs)),
        (Some(lhs), None) => Some(lhs),
        (None, Some(rhs)) => Some(rhs),
        (None, None) => None,
    }
}
