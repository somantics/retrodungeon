use std::{fs::File, rc::Rc};
use game::Game;
use simplelog::*;

slint::include_modules!();

pub mod component;
pub mod error;
pub mod event;
pub mod game;
pub mod logger;
pub mod map;
pub mod resources;
pub mod spawning;
pub mod system;
pub mod ui;
pub mod world;

use crate::error::Result;

fn main() -> Result<()> {
    CombinedLogger::init(vec![
        // TermLogger::new(
        //     LevelFilter::Debug,
        //     Config::default(),
        //     TerminalMode::Mixed,
        //     ColorChoice::Auto,
        // ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("game.log").unwrap(),
        ),
    ])
    .unwrap();

    let resources = Rc::new(resources::ResourceManager::new()?);
    let game = Game::new(&resources)?;
    let main_window = ui::create_window(game, resources);
    main_window.run().unwrap();

    Ok(())
}
