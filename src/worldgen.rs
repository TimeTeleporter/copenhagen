use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use rand::Rng;

use crate::{
    ascii::{spawn_ascii_sprite, AsciiSheet},
    player::Player,
    TILE_SIZE,
};

const SPAWN_RADIUS: i32 = 10;
const CHECK_RADIUS: i32 = 2;
const TILE_Z: f32 = 100.0;

const DIRT_POPULATION: f32 = 1.0;
const GRASS_POPULATION: f32 = 1.0;
const STONE_POPULATION: f32 = 1.0;

const IDEAL_TILE_DISTRIBUTION: (f32, f32, f32) = {
    let sum = DIRT_POPULATION + GRASS_POPULATION + STONE_POPULATION;
    (
        DIRT_POPULATION / sum,
        GRASS_POPULATION / sum,
        STONE_POPULATION / sum,
    )
};

const BARYCENTRIC_MAX_ABS_DIFF: f32 = 1.0; // The maximal distance with the "simply add/p=1" norm in a triangle built from the the unit vectors.

#[derive(Component, Inspectable)]
pub struct Map {
    dirtcount: usize,
    grasscount: usize,
    stonecount: usize,
}

impl Map {
    fn get_normalized(&self) -> (f32, f32, f32) {
        renormalize_barycentric((
            self.dirtcount as f32,
            self.grasscount as f32,
            self.stonecount as f32,
        ))
    }
}

fn renormalize_barycentric(xyz: (f32, f32, f32)) -> (f32, f32, f32) {
    let (x, y, z) = xyz;
    let sum = x + y + z;

    (x / sum, y / sum, z / sum)
}

impl Default for Map {
    fn default() -> Self {
        Self {
            dirtcount: 0,
            grasscount: 0,
            stonecount: 0,
        }
    }
}

#[derive(Component, Debug)]
pub struct MapTile(IVec2);

#[derive(Component)]
enum TileType {
    Dirt,
    Grass,
    Stone,
}

pub struct WorldgenPlugin;

impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_map)
            .add_system(spawn_ground_around_player);
    }
}

fn spawn_ground_around_player(
    mut commands: Commands,
    player_query: Query<&Transform, (With<Player>, Without<Map>, Without<MapTile>)>,
    mut map_query: Query<(Entity, &Children, &mut Map), Without<Player>>,
    tile_query: Query<(&MapTile, &TileType), Without<Map>>,
    ascii: Res<AsciiSheet>,
) {
    let player_transform: &Transform = player_query.single();
    let lattice: Vec<MapTile> = get_lattice_points_in_radius(player_transform.translation);

    match map_query.get_single_mut() {
        Ok((entity, children, mut map)) => {
            let mut tiles: Vec<Entity> = Vec::new();

            'new_tile_loop: for new_tile in lattice.into_iter() {
                'check_exsiting_tiles: for &child in children.iter() {
                    let old_tile = match tile_query.get(child) {
                        Ok(x) => {
                            let (tile, _) = x;
                            tile
                        }
                        Err(e) => {
                            println!("{:?}", e);
                            break 'check_exsiting_tiles;
                        }
                    };

                    if new_tile.0 == old_tile.0 {
                        continue 'new_tile_loop;
                    }
                }

                let tile = spawn_tile(&mut commands, &ascii, new_tile, &mut map, &tile_query);

                tiles.push(tile);
            }

            commands.entity(entity).push_children(&tiles);
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}

fn spawn_tile(
    commands: &mut Commands,
    ascii: &Res<AsciiSheet>,
    new_tile: MapTile,
    map: &mut Map,
    tile_query: &Query<(&MapTile, &TileType), Without<Map>>,
) -> Entity {
    // Decide tile_type
    let tile_type: TileType = {
        // Get local barycentric coordinates
        let mut local_map = Map::default();
        for (_tile, tile_type, _distance) in tile_query.iter().filter_map(|(tile, tile_type)| {
            let distance = ((tile.0.x - new_tile.0.x) * (tile.0.x - new_tile.0.x))
                + ((tile.0.y - new_tile.0.y) * (tile.0.y - new_tile.0.y));
            if distance > CHECK_RADIUS * CHECK_RADIUS {
                None
            } else {
                Some((tile, tile_type, distance))
            }
        }) {
            match tile_type {
                TileType::Dirt => local_map.dirtcount += 1,
                TileType::Grass => local_map.grasscount += 1,
                TileType::Stone => local_map.stonecount += 1,
            }
        }

        let local = local_map.get_normalized();

        let global = map.get_normalized();

        let ideal = renormalize_barycentric(IDEAL_TILE_DISTRIBUTION);

        // Should be zero if the global distribution equals the ideal.
        let weight: f32 =
            (global.0 - ideal.0).abs() + (global.1 - ideal.1).abs() + (global.2 - ideal.2).abs();
        let weight = weight / BARYCENTRIC_MAX_ABS_DIFF;

        println!("weight: {}", weight);

        // Should be equal to the local distribution if the weight is zero.
        // Should be equal to the ideal if the weight is one.
        let blend = renormalize_barycentric((
            linear_blend(local.0, ideal.0, weight),
            linear_blend(local.1, ideal.1, weight),
            linear_blend(local.2, ideal.2, weight),
        ));

        let draw: f32 = rand::thread_rng().gen_range(0.0..1.0);
        println!("blend: {:?}, draw: {}", blend, draw);
        match draw {
            n if n < blend.0 => TileType::Dirt,
            n if n < blend.0 + blend.1 => TileType::Grass,
            _ => TileType::Stone,
        }
    };

    match tile_type {
        TileType::Dirt => map.dirtcount += 1,
        TileType::Grass => map.grasscount += 1,
        TileType::Stone => map.stonecount += 1,
    }

    spawn_tile_type(commands, ascii, new_tile, tile_type)
}

fn linear_blend(local: f32, ideal: f32, weight: f32) -> f32 {
    local * (1.0 - weight) + ideal * weight
}

fn spawn_tile_type(
    commands: &mut Commands,
    ascii: &Res<AsciiSheet>,
    new_tile: MapTile,
    tile_type: TileType,
) -> Entity {
    let (index, color, name) = match tile_type {
        TileType::Dirt => (
            '#' as usize,
            Color::rgb(194.0 / 255.0, 126.0 / 255.0, 64.0 / 255.0),
            "Dirt".to_owned(),
        ),
        TileType::Grass => ('~' as usize, Color::LIME_GREEN, "Grass".to_owned()),
        TileType::Stone => (
            176,
            Color::rgb(192.0 / 255.0, 192.0 / 255.0, 192.0 / 255.0),
            "Stone".to_owned(),
        ),
    };

    let sprite = spawn_ascii_sprite(
        commands,
        ascii,
        index,
        color,
        new_tile.0.as_vec2().extend(TILE_Z) * TILE_SIZE,
    );
    let tile = commands
        .entity(sprite)
        .insert(MapTile(new_tile.0))
        .insert(tile_type)
        .insert(Name::new(
            name + &": "
                + new_tile.0.x.to_string().as_str()
                + " "
                + new_tile.0.y.to_string().as_str(),
        ))
        .id();
    tile
}

fn spawn_map(mut commands: Commands, ascii: Res<AsciiSheet>) {
    let map = commands
        .spawn()
        .insert(Name::new("Map"))
        .insert(Map::default())
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .id();

    let mut tiles: Vec<Entity> = Vec::new();

    tiles.push(spawn_tile_type(
        &mut commands,
        &ascii,
        MapTile(IVec2::new(0, 0)),
        TileType::Grass,
    ));

    commands.entity(map).push_children(&tiles);
}

fn get_lattice_points_in_radius(pos: Vec3) -> Vec<MapTile> {
    let pos = pos.truncate() / TILE_SIZE;

    let pos: IVec2 = pos.round().as_ivec2();

    let mut points: Vec<MapTile> = Vec::new();

    for x in -SPAWN_RADIUS..SPAWN_RADIUS + 1 {
        for y in -SPAWN_RADIUS..SPAWN_RADIUS + 1 {
            points.push(MapTile(IVec2::new(x, y)));
        }
    }

    let points = points
        .iter()
        .filter(|tile| tile.0.x * tile.0.x + tile.0.y * tile.0.y < SPAWN_RADIUS * SPAWN_RADIUS)
        .map(|tile| MapTile(tile.0 + pos))
        .collect();

    points
}
