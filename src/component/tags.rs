use serde::{Deserialize, Serialize};

pub trait Tag {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Monster;
impl Tag for Monster {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Player;
impl Tag for Player {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Door;
impl Tag for Door {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StairsDown;
impl Tag for StairsDown {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Hazard;
impl Tag for Hazard {}
