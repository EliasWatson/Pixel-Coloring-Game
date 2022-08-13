use crate::vec_2d::{Index2d, Vec2d};
use bevy::prelude::*;
use image::RgbImage;
use std::collections::HashMap;

use crate::disjoint_set_2d::DisjointSet2d;

#[derive(Component)]
pub struct Pixel {
    id: u32,
    colored: bool,
}

#[derive(Debug, Clone)]
pub struct ArtBoard {
    pub width: u32,
    pub height: u32,
    pixel_ids: Vec2d<u32>,
    pixel_disjoint_set: DisjointSet2d,
    pixel_entities: Vec2d<Option<Entity>>,
    color_id_map: HashMap<u32, Color>,
}

impl ArtBoard {
    pub fn from_image(image: RgbImage) -> ArtBoard {
        let mut inverse_color_id_map: HashMap<image::Rgb<u8>, u32> = HashMap::new();

        let mut id_rows = vec![];
        for pixel_row in image.rows() {
            let mut id_row = vec![];
            for image_pixel in pixel_row {
                let color_id = match inverse_color_id_map.get(image_pixel) {
                    Some(id) => *id,
                    None => {
                        let id = inverse_color_id_map.len() as u32;
                        inverse_color_id_map.insert(*image_pixel, id);
                        id
                    }
                };

                id_row.push(color_id);
            }
            id_rows.push(id_row);
        }
        id_rows.reverse();

        let ids = Vec2d {
            width: image.width() as usize,
            height: image.height() as usize,
            data: id_rows,
        };

        ArtBoard {
            width: image.width(),
            height: image.height(),
            pixel_disjoint_set: DisjointSet2d::from_vec_2d(&ids),
            pixel_ids: ids,
            pixel_entities: Vec2d::fill(image.width() as usize, image.height() as usize, None),
            color_id_map: inverse_color_id_map
                .iter()
                .map(|(color, id)| (*id, Color::rgb_u8(color[0], color[1], color[2])))
                .collect(),
        }
    }

    pub fn spawn_pixels(&mut self, commands: &mut Commands) {
        for y in 0..self.height {
            for x in 0..self.width {
                let color_id = self.pixel_ids.get((x as usize, y as usize)).unwrap();
                let color = self.color_id_map.get(color_id).unwrap();
                let pale_color = mix_color(color, &Color::rgb(0.75, 0.75, 0.75), 0.9);

                let id = commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: pale_color,
                            custom_size: Some(Vec2::new(1.0, 1.0)),
                            ..default()
                        },
                        transform: Transform::from_xyz(x as f32, y as f32, 0.0),
                        ..default()
                    })
                    .insert(Pixel {
                        id: *color_id,
                        colored: false,
                    })
                    .id();

                self.pixel_entities.set((x as usize, y as usize), Some(id));
            }
        }
    }

    pub fn fill_pixels_at_position(
        &self,
        pixel_query: &mut Query<(&mut Pixel, &mut Sprite)>,
        pos: Vec2,
    ) {
        let tile_pos = (pos + Vec2::new(0.5, 0.5)).floor();
        if tile_pos.x < 0.0 || tile_pos.y < 0.0 {
            return;
        }

        if let Some(parent) = self
            .pixel_disjoint_set
            .get_parent((tile_pos.x as usize, tile_pos.y as usize))
        {
            self.fill_pixels(pixel_query, parent);
        }
    }

    fn fill_pixels(&self, pixel_query: &mut Query<(&mut Pixel, &mut Sprite)>, parent: Index2d) {
        let linked_indices = match self.pixel_disjoint_set.get_linked(parent) {
            Some(i) => i,
            None => return,
        };

        for index in linked_indices {
            self.color_pixel(pixel_query, *index);
        }
    }

    fn color_pixel(&self, pixel_query: &mut Query<(&mut Pixel, &mut Sprite)>, index: Index2d) {
        let pixel_entity = match self.pixel_entities.get(index) {
            Some(Some(entity)) => entity,
            _ => return,
        };

        let (mut pixel, mut sprite) = match pixel_query.get_mut(*pixel_entity) {
            Ok(comps) => comps,
            Err(_) => return,
        };

        let color = match self.color_id_map.get(&pixel.id) {
            Some(c) => c,
            None => return,
        };

        pixel.colored = true;
        sprite.color = *color;
    }
}

fn mix_color(a: &Color, b: &Color, t: f32) -> Color {
    Color::rgba(
        mix(a.r(), b.r(), t),
        mix(a.g(), b.g(), t),
        mix(a.b(), b.b(), t),
        mix(a.a(), b.a(), t),
    )
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    (a * (1.0 - t)) + (b * t)
}
