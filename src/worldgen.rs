use bevy::prelude::*;
use rand::Rng;

use crate::{ascii::{AsciiSheet, spawn_ascii_sprite}, player::Player, TILE_SIZE};

const SPAWN_RADIUS: i32 = 10;
const CHECK_RADIUS: i32 = 2;
const TILE_Z: f32 = 100.0;

const DIRT_MODIFIER: usize = 40;
const GRASS_MODIFIER: usize = 10;
const STONE_MODIFIER: usize = 40;

#[derive(Component)]
pub struct Map;

#[derive(Component, Debug)]
pub struct MapTile(IVec2);

#[derive(Component)]
enum TileType {
    Grass,
    Dirt,
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
    map_query: Query<(Entity, &Children), (With<Map>, Without<Player>)>,
    tile_query: Query<(&MapTile, &TileType), Without<Map>>,
    ascii: Res<AsciiSheet>,
) {
    let player_transform: &Transform = player_query.single();
    let lattice: Vec<MapTile> = get_lattice_points_in_radius(player_transform.translation);

    match map_query.get_single() {
        Ok((map, children)) => {
            let mut tiles: Vec<Entity> = Vec::new();

            'new_tile_loop:
            for new_tile in lattice.into_iter() {
                'check_exsiting_tiles:
                for &child in children.iter() {
                    let old_tile = match tile_query.get(child) {
                        Ok(x) => {
                            let (tile, _) = x;
                            tile
                        },
                        Err(e) => {
                            println!("{:?}", e);
                            break 'check_exsiting_tiles;
                        }
                    };
        
                    if new_tile.0 ==  old_tile.0 {
                        continue 'new_tile_loop; 
                    }
                }
        
                let tile = spawn_tile(&mut commands, &ascii, new_tile, &tile_query);
                
                tiles.push(tile);
            }

            commands.entity(map).push_children(&tiles);
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }


}

fn spawn_tile(
    commands: &mut Commands,
    ascii: &Res<AsciiSheet>,
    new_tile: MapTile,
    tile_query: &Query<(&MapTile, &TileType), Without<Map>>,
) -> Entity {
    let (mut dirtcount, mut grasscount, mut stonecount): (usize, usize, usize) = (1, 1, 1);

    // Find neighbours
    for (tile, tile_type) in tile_query.iter() {
        if ((tile.0.x - new_tile.0.x) * (tile.0.x - new_tile.0.x)) + ((tile.0.y - new_tile.0.y) * (tile.0.y - new_tile.0.y)) < CHECK_RADIUS * CHECK_RADIUS {
            match tile_type {
                TileType::Dirt => { dirtcount += DIRT_MODIFIER },
                TileType::Grass => { grasscount += GRASS_MODIFIER },
                TileType::Stone => { stonecount += STONE_MODIFIER }
            }
        }
    }

    let draw: usize = rand::thread_rng().gen_range(0..(dirtcount + grasscount + stonecount));
    let tile_type: TileType = match draw {
        n if n < dirtcount => { TileType::Dirt },
        n if n < dirtcount + grasscount => {TileType::Grass}
        _ => {TileType::Stone},
    };

    spawn_tile_type(commands, ascii, new_tile, tile_type)
}

fn spawn_tile_type(commands: &mut Commands, ascii: &Res<AsciiSheet>, new_tile: MapTile, tile_type: TileType) -> Entity {
    let (index, color, name) = match tile_type {
        TileType::Dirt => ('#' as usize, Color::rgb(194.0 / 255.0, 126.0 / 255.0, 64.0 / 255.0), "Dirt".to_owned()),
        TileType::Grass => ('~' as usize, Color::LIME_GREEN, "Grass".to_owned()),
        TileType::Stone => (176, Color::rgb(192.0 / 255.0, 192.0 / 255.0, 192.0 / 255.0), "Stone".to_owned()),
    };
    
    let sprite = spawn_ascii_sprite(
        commands,
        ascii,
        index,
        color,
        new_tile.0.as_vec2().extend(TILE_Z) * TILE_SIZE,
    );
    let tile = commands.entity(sprite)
        .insert(MapTile(new_tile.0))
        .insert(tile_type)
        .insert(Name::new(name +  &": " + new_tile.0.x.to_string().as_str() + " " + new_tile.0.y.to_string().as_str()))
        .id();
    tile
}

fn spawn_map(mut commands: Commands, ascii: Res<AsciiSheet>) {
    let map = commands.spawn()
        .insert(Name::new("Map"))
        .insert(Map)
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .id();
    
    let mut tiles: Vec<Entity> = Vec::new();

    tiles.push(
        spawn_tile_type(&mut commands, &ascii, MapTile(IVec2::new(0, 0)), TileType::Grass)
    );
    
    commands.entity(map).push_children(&tiles);
}

fn get_lattice_points_in_radius(pos: Vec3) -> Vec<MapTile> {
    let pos = pos.truncate() / TILE_SIZE;

    let pos: IVec2 = pos.round().as_ivec2();

    let mut points: Vec<MapTile> = Vec::new();

    for x in -SPAWN_RADIUS..SPAWN_RADIUS+1 {
        for y in -SPAWN_RADIUS..SPAWN_RADIUS+1 {
            points.push(MapTile(IVec2::new(x, y)));
        }
    }

    let points = points.iter()
        .filter(|tile| tile.0.x * tile.0.x + tile.0.y * tile.0.y < SPAWN_RADIUS * SPAWN_RADIUS )
        .map(|tile| MapTile(tile.0 + pos))
        .collect();
    
    points
}