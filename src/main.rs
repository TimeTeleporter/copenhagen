#![allow(clippy::redundant_field_names)]

use bevy::{
    prelude::*, 
    window::PresentMode,
};

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const RESOLUTION: f32 = 16.0 / 9.0;
pub const WINDOWHEIGHT: f32 = 1080.;

pub const TILE_SIZE: f32 = 1.0;
pub const PLAYERSPEED: f32 = 5.0 * TILE_SIZE;

mod player;
mod worldgen;
mod ascii;
mod debug;
mod camera;

use ascii::AsciiPlugin;
use player::PlayerPlugin;
use worldgen::WorldgenPlugin;
use debug::DebugPlugin;
use camera::CameraPlugin;

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

