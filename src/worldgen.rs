use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use rand::Rng;

use crate::{
    ascii::{spawn_ascii_sprite, AsciiSheet},
    player::Player,
    TILE_SIZE,
};

const SPAWN_RADIUS: i32 = 5;
const POINT_IN_RADIUS: usize = {
    let mut count: usize = 0;
    let mut i: i32 = 0;
    loop {
        if i > SPAWN_RADIUS {
            break;
        }
        let mut x: i32 = -i;
        loop {
            if x > i {
                break;
            }
            let mut y: i32 = -i;
            loop {
                if y > i {
                    break;
                }
                if x.abs() == i.abs() || y.abs() == i.abs() {
                    if x * x + y * y < SPAWN_RADIUS * SPAWN_RADIUS {
                        count += 1;
                    }
                }
                y += 1;
            }
            x += 1;
        }
        i += 1;
    }
    count
};

const SPAWN_LIST: [(i32, i32); POINT_IN_RADIUS] = {
    let mut count: usize = 0;
    let mut list: [(i32, i32); POINT_IN_RADIUS] = [(0, 0); POINT_IN_RADIUS];
    let mut i: i32 = 0;
    loop {
        if i > SPAWN_RADIUS {
            break;
        }
        let mut x: i32 = -i;
        loop {
            if x > i {
                break;
            }
            let mut y: i32 = -i;
            loop {
                if y > i {
                    break;
                }
                if x.abs() == i.abs() || y.abs() == i.abs() {
                    if x * x + y * y < SPAWN_RADIUS * SPAWN_RADIUS {
                        list[count] = (x, y);
                        count += 1;
                    }
                }
                y += 1;
            }
            x += 1;
        }
        i += 1;
    }
    list
};

const LOCAL_RADIUS: i32 = 8;
const LOCAL_MODIFIER: usize = 2;

const DIRT_POPULATION: f32 = 8.0;
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

// The maximal distance with the one-norm in a triangle built from the the unit vectors.
const BARYCENTRIC_MAX_ABS_DIFF: f32 = 1.0;

#[derive(Component, Inspectable, Default)]
pub struct Map {
    dirtcount: usize,
    grasscount: usize,
    stonecount: usize,
}

impl Map {
    fn new(dirtcount: usize, grasscount: usize, stonecount: usize) -> Map {
        Map {
            dirtcount,
            grasscount,
            stonecount,
        }
    }

    fn one_of_each(mut self) -> Map {
        self.add_one(&TileType::Dirt)
            .add_one(&TileType::Grass)
            .add_one(&TileType::Stone)
    }

    fn add_one(&mut self, tile_type: &TileType) -> Map {
        match tile_type {
            TileType::Dirt => self.dirtcount += 1,
            TileType::Grass => self.grasscount += 1,
            TileType::Stone => self.stonecount += 1,
        }
        Map::new(self.dirtcount, self.grasscount, self.stonecount)
    }

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
        for (_, tile_type, _distance) in tile_query.iter().filter_map(|(tile, tile_type)| {
            let distance: i32 = ((tile.0.x - new_tile.0.x) * (tile.0.x - new_tile.0.x))
                + ((tile.0.y - new_tile.0.y) * (tile.0.y - new_tile.0.y));
            if distance > LOCAL_RADIUS * LOCAL_RADIUS {
                None
            } else {
                Some((tile, tile_type, distance))
            }
        }) {
            match tile_type {
                TileType::Dirt => local_map.dirtcount += _distance as usize,
                TileType::Grass => local_map.grasscount += _distance as usize,
                TileType::Stone => local_map.stonecount += _distance as usize,
            }
        }

        let local = local_map.get_normalized();

        let global = map.get_normalized();

        let ideal = renormalize_barycentric(IDEAL_TILE_DISTRIBUTION);

        // Should be zero if the global distribution equals the ideal.
        let weight: f32 =
            (global.0 - ideal.0).abs() + (global.1 - ideal.1).abs() + (global.2 - ideal.2).abs();
        let weight = weight / BARYCENTRIC_MAX_ABS_DIFF;

        // Should be equal to the local distribution if the weight is zero.
        // Should be equal to the ideal if the weight is one.
        let blend = renormalize_barycentric((
            cubic_blend(local.0, ideal.0, weight),
            cubic_blend(local.1, ideal.1, weight),
            cubic_blend(local.2, ideal.2, weight),
        ));

        let draw: f32 = rand::thread_rng().gen_range(0.0..1.0);
        match draw {
            n if n < blend.0 => TileType::Dirt,
            n if n < blend.0 + blend.1 => TileType::Grass,
            _ => TileType::Stone,
        }
    };

    map.add_one(&tile_type);

    spawn_tile_type(commands, ascii, new_tile, tile_type)
}

fn linear_blend(local: f32, ideal: f32, weight: f32) -> f32 {
    local * (1.0 - weight) + ideal * weight
}

fn cubic_blend(local: f32, ideal: f32, weight: f32) -> f32 {
    linear_blend(local, ideal, weight * weight * weight)
}

fn spawn_tile_type(
    commands: &mut Commands,
    ascii: &Res<AsciiSheet>,
    new_tile: MapTile,
    tile_type: TileType,
) -> Entity {
    let (index, color, name) = match tile_type {
        TileType::Dirt => (
            0 as usize,
            Color::rgb(194.0 / 255.0, 126.0 / 255.0, 64.0 / 255.0),
            "Dirt".to_owned(),
        ),
        TileType::Grass => (0 as usize, Color::LIME_GREEN, "Grass".to_owned()),
        TileType::Stone => (
            0,
            Color::rgb(192.0 / 255.0, 192.0 / 255.0, 192.0 / 255.0),
            "Stone".to_owned(),
        ),
    };

    let sprite = spawn_ascii_sprite(
        commands,
        ascii,
        index,
        color,
        new_tile.0.as_vec2().extend(100.0) * TILE_SIZE,
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

    let points: Vec<MapTile> = SPAWN_LIST
        .into_iter()
        .map(|(x, y)| MapTile(IVec2::new(x, y) + pos))
        .collect();

    points
}
