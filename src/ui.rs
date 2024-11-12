use std::rc::Rc;

use log::warn;
use slint::ModelRc;

slint::include_modules!();

use crate::{
    component::{attributes::{Attribute, XpStatus}, health::HealthStatus},
    game::{self, Game},
    logger,
    map::{utils::Coordinate, GameMap},
    resources::{self, ResourceManager},
};

pub fn create_window(game: Game, resources: Rc<ResourceManager>) -> MainWindow {
    let window = MainWindow::new().unwrap();
    window.set_tile_size(resources::TILESET_SIZE);
    window.set_grid_width(game::MAP_SIZE_X as i32);
    window.set_grid_height(game::MAP_SIZE_Y as i32);

    update_game_info(&game, &window);
    window.invoke_display_intro_popup();
    update_tile_map(&game, &window, resources.clone());
    set_up_input(game, &window, resources);
    window
}

fn set_up_input(mut game: Game, window: &MainWindow, resources: Rc<ResourceManager>) {
    let weak_window = window.as_weak();
    window.on_received_input(move |command, x, y, z| {
        match command {
            InputCommand::Direction => {
                let result = game.direction_command(Coordinate { x, y }, &resources);
                match result {
                    Ok(_) => {}
                    Err(error) => warn!("{error}"),
                };
            }
            InputCommand::Position => {
                let result = game.move_to_command(Coordinate { x, y }, &resources);
                match result {
                    Ok(_) => {}
                    Err(error) => warn!("{error}"),
                };
            }
            InputCommand::Shoot => {
                let result = game.shoot_command(Coordinate { x, y }, &resources, false);
                match result {
                    Ok(_) => {}
                    Err(error) => warn!("{error}"),
                };
            }
            InputCommand::ForceAttack => {
                let result = game.force_attack_command(Coordinate { x, y }, &resources);
                match result {
                    Ok(_) => {}
                    Err(error) => warn!("{error}"),
                };
            }
            InputCommand::ForceShoot => {
                let result = game.shoot_command(Coordinate { x, y }, &resources, true);
                match result {
                    Ok(_) => {}
                    Err(error) => warn!("{error}"),
                };
            }
            InputCommand::Spell => {
                if z < 0 {
                    warn!("Invalid spell index");
                    return;
                } 
                let result = game.cast_spell_command(z as usize, Coordinate { x, y }, &resources);
                match result {
                    Ok(_) => {}
                    Err(error) => warn!("{error}"),
                };
            }
            InputCommand::Descend => {
                let result = game.descend_command(&resources);
                match result {
                    Ok(_) => {}
                    Err(error) => warn!("{error}"),
                };
            }
            InputCommand::Wait => {
                let result = game.wait_command(&resources);
                match result {
                    Ok(_) => {}
                    Err(error) => warn!("{error}"),
                };
            }
            InputCommand::LevelUp => {
                let (stat, amount ) = (x, y as u32);
                let stat = match stat {
                    0 => Attribute::Might,
                    1 => Attribute::Wit,
                    2 => Attribute::Skill,
                    _ => {
                        warn!("Unrecognized attribute selected for level up.");
                        return;
                    }
                };

                let result = game.level_up_command(stat, amount);
                match result {
                    Ok(_) => {}
                    Err(error) => warn!("{error}"),
                }
            }
            InputCommand::Quit => {
                close_window(&weak_window.unwrap());
            }
            InputCommand::Restart => {
                if let Ok(new_game) = Game::new(&resources) {
                    game = new_game;
                } else {
                    logger::log_message("Failed to initialize new game.");
                }
            }
            _ => {
                warn!("Unrecognized input command: {command:?}.");
            }
        }
        update_game_info(&game, &weak_window.unwrap());
        logger::LOG.with(|log| display_messages(&log, &weak_window.unwrap()));
        display_popup(&game, &weak_window.unwrap());
        update_tile_map(&game, &weak_window.unwrap(), resources.clone());
    });
}

fn display_popup(game: &Game, window: &MainWindow) {
    if game.player_health_status() == HealthStatus::Dead {
        window.invoke_display_death_popup();
    }
    if game.player_xp_status() == XpStatus::LevelUp {
        window.invoke_display_level_up_popup();
    }
}

fn display_messages(message_log: &logger::MessageLog, window: &MainWindow) {
    while let Some(msg) = message_log.next_message() {
        window.invoke_display_message(msg.into());
    }
}

fn close_window(window: &MainWindow) {
    window.window().hide().unwrap();
}

fn update_game_info(game: &Game, window: &MainWindow) {
    let PlayerModel {
        name,
        level,
        coins,
        xp_current,
        xp_goal,
        hp_current,
        hp_max,
        might,
        wit,
        skill,
        melee_damage,
        melee_crit,
        ranged_damage,
        ranged_crit,
    } = game.get_player_info();

    let depth = game.get_map_info().depth;

    window.set_depth(depth);
    window.set_character_name(name.into());
    window.set_player_level(level);
    window.set_player_coins(coins);
    window.set_player_xp_current(xp_current);
    window.set_player_xp_goal(xp_goal);
    window.set_player_health_current(hp_current);
    window.set_player_health_max(hp_max);
    window.set_player_might(might);
    window.set_player_wit(wit);
    window.set_player_skill(skill);
    window.set_player_melee_damage(melee_damage.into());
    window.set_player_melee_crit(melee_crit);
    window.set_player_ranged_damage(ranged_damage.into());
    window.set_player_ranged_crit(ranged_crit);

    let SpellbookModel { names, casts, damages } = game.get_spell_info();

    window.set_spell_names(names);
    window.set_spell_casts(casts);
    window.set_spell_damages(damages);
}

fn update_tile_map(game: &Game, window: &MainWindow, resources: Rc<ResourceManager>) {
    // Updates frontend's internal data for tiles, which triggers redraw.
    let tiles: Vec<TileGraphics> = game
        .get_sprite_ids(&resources)
        .into_iter()
        .map(|vec| Rc::new(slint::VecModel::from(vec)))
        .map(|vec_model| TileGraphics {
            image_ids: vec_model.into(),
        })
        .collect();

    let tiles = std::rc::Rc::new(slint::VecModel::from(tiles));

    window.set_memory_tiles(tiles.into());
}

#[derive(Debug, Clone, Default)]
pub struct PlayerModel {
    pub name: String,
    pub level: i32,
    pub coins: i32,
    pub xp_current: i32,
    pub xp_goal: i32,
    pub hp_current: i32,
    pub hp_max: i32,
    pub might: i32,
    pub wit: i32,
    pub skill: i32,
    pub melee_damage: ModelRc<i32>,
    pub melee_crit: f32,
    pub ranged_damage: ModelRc<i32>,
    pub ranged_crit: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MapModel {
    pub width: i32,
    pub height: i32,
    pub depth: i32,
}

impl From<GameMap> for MapModel {
    fn from(value: GameMap) -> Self {
        Self {
            width: value.width as i32,
            height: value.height as i32,
            depth: value.depth as i32,
        }
    }
}

impl From<&GameMap> for MapModel {
    fn from(value: &GameMap) -> Self {
        Self {
            width: value.width as i32,
            height: value.height as i32,
            depth: value.depth as i32,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SpellbookModel {
    pub names: ModelRc<slint::SharedString>,
    pub casts: ModelRc<ModelRc<i32>>,
    pub damages: ModelRc<ModelRc<i32>>,
}