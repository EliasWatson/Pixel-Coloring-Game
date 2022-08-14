use crate::vec_2d::{Index2d, Vec2d};
use bevy::prelude::*;
use image::RgbImage;
use std::collections::HashMap;

use crate::disjoint_set_2d::DisjointSet2d;

const PIXEL_EDGE_THICKNESS: f32 = 0.1;

#[derive(Component)]
pub struct Pixel {
    id: u32,
    colored: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PixelEdgeDirection {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

const ALL_PIXEL_EDGE_DIRECTIONS: [PixelEdgeDirection; 4] = [
    PixelEdgeDirection::Up,
    PixelEdgeDirection::Right,
    PixelEdgeDirection::Down,
    PixelEdgeDirection::Left,
];

pub type PixelEdgeEntities = [Option<Entity>; 4];

#[derive(Debug, Clone)]
pub struct ArtBoard {
    pub width: u32,
    pub height: u32,
    pixel_ids: Vec2d<u32>,
    pixel_disjoint_set: DisjointSet2d,
    pixel_entities: Vec2d<Option<Entity>>,
    pixel_edge_entities: Vec2d<PixelEdgeEntities>,
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
            pixel_edge_entities: Vec2d::fill(
                image.width() as usize,
                image.height() as usize,
                [None; 4],
            ),
            color_id_map: inverse_color_id_map
                .iter()
                .map(|(color, id)| (*id, Color::rgb_u8(color[0], color[1], color[2])))
                .collect(),
        }
    }

    pub fn spawn_pixels(&mut self, commands: &mut Commands) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = (x as usize, y as usize);

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

                self.pixel_entities.set(pos, Some(id));

                self.spawn_pixel_edges(commands, pos);
            }
        }
    }

    fn spawn_pixel_edges(&mut self, commands: &mut Commands, pos: Index2d) {
        let pixel_parent_index = match self.pixel_disjoint_set.get_parent(pos) {
            Some(i) => i,
            None => return,
        };

        for dir in ALL_PIXEL_EDGE_DIRECTIONS {
            let neighbor_index = match dir.offset_index(pos) {
                Some(i) => i,
                None => continue,
            };

            let neighbor_parent_index = match self.pixel_disjoint_set.get_parent(neighbor_index) {
                Some(i) => i,
                None => continue,
            };

            if pixel_parent_index != neighbor_parent_index {
                self.spawn_pixel_edge(commands, pos, dir);
            }
        }
    }

    fn spawn_pixel_edge(&mut self, commands: &mut Commands, pos: Index2d, dir: PixelEdgeDirection) {
        let edge_entities = match self.pixel_edge_entities.get_mut(pos) {
            Some(e) => e,
            None => return,
        };

        if edge_entities[dir as usize].is_some() {
            return;
        }

        let id = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(dir.get_size()),
                    ..default()
                },
                transform: Transform::from_translation(
                    (Vec2::new(pos.0 as f32, pos.1 as f32) + dir.get_offset()).extend(0.1),
                ),
                ..default()
            })
            .id();

        edge_entities[dir as usize] = Some(id);
    }

    fn clear_pixel_edges(&mut self, commands: &mut Commands, pos: Index2d) {
        let edge_entities = match self.pixel_edge_entities.get_mut(pos) {
            Some(e) => e,
            None => return,
        };

        for dir in ALL_PIXEL_EDGE_DIRECTIONS {
            if let Some(entity) = edge_entities[dir as usize] {
                commands.entity(entity).despawn();
                edge_entities[dir as usize] = None;
            }
        }
    }

    pub fn fill_pixels_at_position(
        &mut self,
        commands: &mut Commands,
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
            self.fill_pixels(commands, pixel_query, parent);
        }
    }

    fn fill_pixels(
        &mut self,
        commands: &mut Commands,
        pixel_query: &mut Query<(&mut Pixel, &mut Sprite)>,
        parent: Index2d,
    ) {
        let linked_indices = match self.pixel_disjoint_set.get_linked(parent) {
            Some(i) => i.clone(),
            None => return,
        };

        for index in linked_indices {
            self.color_pixel(pixel_query, index);
            self.clear_pixel_edges(commands, index);
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

impl PixelEdgeDirection {
    pub fn get_size(&self) -> Vec2 {
        match self {
            PixelEdgeDirection::Up => Vec2::new(1.0, PIXEL_EDGE_THICKNESS),
            PixelEdgeDirection::Right => Vec2::new(PIXEL_EDGE_THICKNESS, 1.0),
            PixelEdgeDirection::Down => Vec2::new(1.0, PIXEL_EDGE_THICKNESS),
            PixelEdgeDirection::Left => Vec2::new(PIXEL_EDGE_THICKNESS, 1.0),
        }
    }

    pub fn get_offset(&self) -> Vec2 {
        match self {
            PixelEdgeDirection::Up => Vec2::new(0.0, 0.5),
            PixelEdgeDirection::Right => Vec2::new(0.5, 0.0),
            PixelEdgeDirection::Down => Vec2::new(0.0, -0.5),
            PixelEdgeDirection::Left => Vec2::new(-0.5, 0.0),
        }
    }

    pub fn offset_index(&self, index: Index2d) -> Option<Index2d> {
        match self {
            PixelEdgeDirection::Up => Some((index.0, index.1 + 1)),
            PixelEdgeDirection::Right => Some((index.0 + 1, index.1)),
            PixelEdgeDirection::Down if index.1 > 0 => Some((index.0, index.1 - 1)),
            PixelEdgeDirection::Left if index.0 > 0 => Some((index.0 - 1, index.1)),
            _ => None,
        }
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
