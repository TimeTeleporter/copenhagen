use bevy::{prelude::*};
use bevy_inspector_egui::Inspectable;
use crate::{PLAYERSPEED,
    ascii::{spawn_ascii_sprite, AsciiSheet}, TILE_SIZE,
};

const PLAYER_SPRITE_Z: f32 = 900.0;
const _PLAYER_CHILD_Z: f32 = -1.0;

#[derive(Component, Inspectable)]
pub struct Player {
    speed: f32,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system(player_movement)
            ;
    }
}

fn player_movement(mut player_query: Query<(&mut Player, &mut Transform)>, keyboard: Res<Input<KeyCode>>, time: Res<Time>) {
    let (player, mut transform): (Mut<'_, Player>, Mut<'_, Transform>) = player_query.single_mut();

    let sprintmodifier: f32 = if keyboard.pressed(KeyCode::LShift) { 2.0 } else { 1.0 };

    let mut delta_y: f32 = 0.0;
    let mut delta_x: f32 = 0.0;

    if keyboard.pressed(KeyCode::W) {
        delta_y += player.speed * TILE_SIZE * time.delta_seconds() * sprintmodifier;
    }
    if keyboard.pressed(KeyCode::S) {
        delta_y += -player.speed  * TILE_SIZE * time.delta_seconds() * sprintmodifier;
    }
    if keyboard.pressed(KeyCode::A) {
        delta_x += -player.speed * TILE_SIZE * time.delta_seconds() * sprintmodifier;
    }
    if keyboard.pressed(KeyCode::D) {
        delta_x += player.speed * TILE_SIZE * time.delta_seconds() * sprintmodifier;
    }

    let target = transform.translation + Vec3::new(delta_x, 0.0, 0.0);

    // We move the player only if the collision check was negative
    /*
    if !wall_query.iter().any(|&transform| wall_collision_check(target, transform.translation)) {
        */
        transform.translation = target;
        /*
        if delta_x != 0.0 {
            player.just_moved = true;
        }
    }
    */

    let target = transform.translation + Vec3::new(0.0, delta_y, 0.0);
    /*
    if !wall_query.iter().any(|&transform| wall_collision_check(target, transform.translation)) {
        */
        transform.translation = target;
        /*

        if delta_x != 0.0 {
            player.just_moved = true;
        }
    }
    */
}

fn spawn_player(mut commands: Commands, ascii: Res<AsciiSheet>) {
    let sprite = spawn_ascii_sprite(
        &mut commands,
        &ascii,
        1,
        Color::SALMON,
        Vec3::new(0.0, 0.0, PLAYER_SPRITE_Z)
    );

    let _player = commands.entity(sprite)
        .insert(Name::new("Player"))
        .insert(Player { speed: PLAYERSPEED })
        .id();
}