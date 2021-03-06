#![allow(clippy::redundant_field_names)]
#![allow(dead_code)]
#![feature(const_for)]
#![feature(const_mut_refs)]

use bevy::{prelude::*, window::PresentMode};

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const RESOLUTION: f32 = 16.0 / 9.0;
pub const WINDOWHEIGHT: f32 = 1080.;

pub const TILE_SIZE: f32 = 1.0;
pub const PLAYERSPEED: f32 = 5.0 * TILE_SIZE;

mod ascii;
mod camera;
mod debug;
mod player;
mod worldgen;

use ascii::AsciiPlugin;
use camera::CameraPlugin;
use debug::DebugPlugin;
use player::PlayerPlugin;
use worldgen::WorldgenPlugin;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum GameState {
    Overworld,
}

fn main() {
    let height: f32 = WINDOWHEIGHT;

    App::new()
        // Initialize the game and camera.
        .add_state(GameState::Overworld)
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(WindowDescriptor {
            width: height * RESOLUTION,
            height: height,
            title: "Bevy Tutorial".to_string(),
            present_mode: PresentMode::Fifo,
            resizable: false,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugPlugin)
        // Add the different game systems and components via Plugins
        .add_plugin(AsciiPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(WorldgenPlugin)
        .add_plugin(CameraPlugin)
        .run();
}
