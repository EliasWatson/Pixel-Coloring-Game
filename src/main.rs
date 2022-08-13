mod art_board;
mod disjoint_set_2d;
mod vec_2d;

use art_board::{ArtBoard, Pixel};
use bevy::{
    prelude::*,
    render::camera::{RenderTarget, ScalingMode},
};

fn main() {
    let image = image::open("input.png").unwrap().into_rgb8();

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ArtBoard::from_image(image))
        .add_startup_system(setup)
        .add_system(tile_painting)
        .run();
}

fn setup(mut commands: Commands, mut art_board: ResMut<ArtBoard>) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.transform.translation = Vec3::new(
        (art_board.width as f32 / 2.0) - 0.5,
        (art_board.height as f32 / 2.0) - 0.5,
        0.0,
    );
    camera_bundle.projection.scaling_mode = ScalingMode::Auto {
        min_width: art_board.width as f32,
        min_height: art_board.height as f32,
    };

    commands.spawn_bundle(camera_bundle);

    art_board.spawn_pixels(&mut commands);
}

fn tile_painting(
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    art_board: Res<ArtBoard>,
    mut pixel_query: Query<(&mut Pixel, &mut Sprite)>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        let (camera, camera_transform) = camera_query.single();

        let window = if let RenderTarget::Window(id) = camera.target {
            windows.get(id).unwrap()
        } else {
            windows.get_primary().unwrap()
        };

        if let Some(screen_pos) = window.cursor_position() {
            let window_size = Vec2::new(window.width() as f32, window.height() as f32);
            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
            let ndc_to_world =
                camera_transform.compute_matrix() * camera.projection_matrix().inverse();
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
            let world_pos: Vec2 = world_pos.truncate();

            art_board.fill_pixels_at_position(&mut pixel_query, world_pos);
        }
    }
}
