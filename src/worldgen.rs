use bevy::prelude::*;

use crate::{ascii::{AsciiSheet, spawn_ascii_sprite}, player::Player, TILE_SIZE};

const LATTICE_RADIUS: i32 = 10;
const TILE_Z: f32 = 100.0;

#[derive(Component)]
pub struct Map;

#[derive(Component, Debug)]
pub struct MapTile(IVec2);

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
    tile_query: Query<&MapTile, Without<Map>>,
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
                        Ok(x) => { x },
                        Err(e) => {
                            println!("{:?}", e);
                            break 'check_exsiting_tiles;
                        }
                    };
        
                    if new_tile.0 ==  old_tile.0 {
                        break 'new_tile_loop; 
                    }
                }
        
                let tile = spawn_tile(&mut commands, &ascii, new_tile);
                
                tiles.push(tile);
            }

            commands.entity(map).push_children(&tiles);
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }


}

fn spawn_tile(commands: &mut Commands, ascii: &Res<AsciiSheet>, new_tile: MapTile) -> Entity {
    let sprite = spawn_ascii_sprite(
        commands,
        ascii,
        '#' as usize,
        Color::RED,
        new_tile.0.as_vec2().extend(TILE_Z) * TILE_SIZE,
    );
    let tile = commands.entity(sprite)
        .insert(MapTile(new_tile.0))
        .insert(Name::new("Tile ".to_owned() + new_tile.0.x.to_string().as_str() + " " + new_tile.0.y.to_string().as_str()))
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

    tiles.push(spawn_tile(
        &mut commands,
        &ascii,
        MapTile(IVec2::new(0, 0))
    ));
    
    commands.entity(map).push_children(&tiles);
}

fn get_lattice_points_in_radius(pos: Vec3) -> Vec<MapTile> {
    let pos = pos.truncate() / TILE_SIZE;

    let pos: IVec2 = pos.round().as_ivec2();

    let mut points: Vec<MapTile> = Vec::new();

    for x in -LATTICE_RADIUS..LATTICE_RADIUS+1 {
        for y in -LATTICE_RADIUS..LATTICE_RADIUS+1 {
            points.push(MapTile(IVec2::new(x, y)));
        }
    }
    
    let points = points.iter()
        .filter(|tile| tile.0.dot(tile.0) < LATTICE_RADIUS * LATTICE_RADIUS )
        .map(|tile| MapTile(tile.0 + pos))
        .collect();

    points
}