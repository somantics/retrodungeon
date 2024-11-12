use serde::{Deserialize, Serialize};

use crate::map::{
    tile::{Los, Passable},
    utils::Coordinate,
};

pub mod attributes;
pub mod behavior;
pub mod combat;
pub mod effect;
pub mod responses;
pub mod health;
pub mod image;
pub mod items;
pub mod spell;
pub mod tags;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name(pub String);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Collision(pub Passable);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SightBlocking(pub Los);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position(pub Coordinate);

impl Position {
    pub fn new(coordinate: Coordinate) -> Self {
        Self(coordinate)
    }

    pub fn coordinate(&self) -> Coordinate {
        self.0
    }

    // NOTE, may only be called from update_position in world
    pub fn move_to(&mut self, destination: Coordinate) {
        self.0 = destination;
    }
}

impl PartialEq<Coordinate> for Position {
    fn eq(&self, other: &Coordinate) -> bool {
        &self.0 == other
    }
}

impl Into<Coordinate> for Position {
    fn into(self) -> Coordinate {
        self.0
    }
}
