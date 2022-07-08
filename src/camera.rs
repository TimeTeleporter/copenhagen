use bevy::{prelude::*,
    render::camera::ScalingMode, input::mouse::{MouseWheel, MouseScrollUnit},
};
use crate::{RESOLUTION, TILE_SIZE,
    player::Player
};

const ZOOM: f32 = 10.0;
const ZOOM_SCALE: f32 = 1.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin{
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera)
            .add_system(camera_follow)
            .add_system(camera_zoom);
    }
}

fn camera_zoom(
    mut camera_query: Query<&mut OrthographicProjection>,
    mut scroll_event_reader: EventReader<MouseWheel>,
) {
    let mut projec: Mut<'_, OrthographicProjection> = camera_query.single_mut();

    for event in scroll_event_reader.iter() {
        match event.unit {
            MouseScrollUnit::Line => {
                projec.top = (projec.top + event.y * ZOOM_SCALE).abs();
                projec.bottom = -(projec.bottom - event.y * ZOOM_SCALE).abs();
                projec.right = (projec.right + event.y * ZOOM_SCALE * RESOLUTION).abs();
                projec.left = -(projec.left - event.y * ZOOM_SCALE * RESOLUTION).abs();
            }
            MouseScrollUnit::Pixel => {}
        }
    }
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let player_transform: &Transform = player_query.single();
    let mut camera_transform: Mut<Transform> = camera_query.single_mut();

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();

    camera.orthographic_projection.top = 1.0 * ZOOM;
    camera.orthographic_projection.bottom = -1.0 * ZOOM;

    camera.orthographic_projection.right = 1.0 * RESOLUTION * TILE_SIZE * ZOOM;
    camera.orthographic_projection.left = -1.0 * RESOLUTION * TILE_SIZE * ZOOM;

    camera.orthographic_projection.scaling_mode = ScalingMode::None;

    commands.spawn_bundle(camera);
}