use bevy::prelude::*;

use crate::components::{
    CurrentLevel,
    DamageCooldown,
    DamageFlash,
    GameEntity,
    Health,
    Movement,
    Player,
};

use crate::constants::{
    PLAYER_SIZE,
    TILE_SIZE,
    MOVING_TILE_DISTANCE,
    MOVING_TILE_SPEED,
};

use crate::levels::{LEVEL_1, LEVELS};

#[derive(Clone, Copy, Debug)]
pub(crate) struct TileAttributes {
    pub(crate) color: (f32, f32, f32), // Original color of tile
    pub(crate) animate_color: Option<(f32, f32, f32)>, // Color the tile will fade to
    pub(crate) animate_speed: Option<f32>, // Speed at which tile will fade to different color; lower value is slower; higher value is faster
    pub(crate) is_circle: bool, // Whether or not the tile is circular shaped
    pub(crate) damage: u32, // The amount of damage the tile inflicts on the player when they touch it
    pub(crate) has_damage_cooldown: bool, // Whether the damage cooldown is applied to the tile
    pub(crate) speed_multiplier: f32, // How much the tile impacts user speed
    pub(crate) bounce: f32, // Amount of bounce a tile has
    pub(crate) movement_translate_pos: Option<(i32, i32)>, // If tile moves, the translation of the tile's movement
    pub(crate) movement_distance: f32, // Will implement later; will allow for manually adjusting tile's movement attributes
    pub(crate) movement_speed: f32, // Will implement later; will allow for manually adjusting tile's movement attributes
    pub(crate) is_solid: bool, // Whether or not collisions with other entities occur
}

// Make it so you don't have to type every single attribute all the time
impl Default for TileAttributes {
    fn default() -> Self {
        Self {
            color: (0.0, 0.0, 0.0),
            animate_color: None,
            animate_speed: None, // Higher number is faster animation
            is_circle: false,
            damage: 0,
            has_damage_cooldown: true,
            speed_multiplier: 1.0,
            bounce: 0.0,
            movement_translate_pos: None, // No tile movement by default
            movement_distance: 0.0,
            movement_speed: 0.0,
            is_solid: true,
        }
    }
}

// Make Tiles components so we easily select them
#[derive(Component, Copy, Clone, PartialEq)]
pub(crate)enum Tile {
    Dirt,
    Grass,
    Lava,
    Bedrock,
    Coin,
    Bounce,
    Moving,
    Win, // Tile that user needs to contact to win the game
}
impl Tile {
    pub(crate) fn attributes(&self) -> TileAttributes {
        match self {
            Tile::Dirt => TileAttributes {
                color: (0.6078, 0.4627, 0.3255),
                ..Default::default()
            },
            Tile::Grass => TileAttributes {
                color: (0.1, 0.3, 0.03),
                ..Default::default()
            },
            Tile::Lava => TileAttributes {
                color: (1.0, 0.35, 0.0),
                animate_color: Some((0.7, 0.15, 0.1)),
                animate_speed: Some(0.5),
                speed_multiplier: 0.3,
                is_solid: false,
                damage: 1,
                ..Default::default()
            },
            Tile::Bedrock => TileAttributes {
                damage: 10000,
                has_damage_cooldown: false,
                is_solid: false,
                ..Default::default()
            },
            Tile::Win => TileAttributes {
                is_solid: false,
                color: (0.8, 0.2, 0.8),
                animate_color: Some((0.5, 0.9, 0.5)),
                animate_speed: Some(1.5),
                ..Default::default()
            },
            Tile::Coin => TileAttributes {
                is_solid: false,
                is_circle: true,
                color: (1.0, 1.0, 0.0),
                animate_color: Some((0.8, 0.8, 0.3)),
                animate_speed: Some(6.0),
                ..Default::default()
            },
            Tile::Bounce => TileAttributes {
                color: (0.2, 1.0, 0.2),
                bounce: 450.0,
                ..Default::default()
            },
            Tile::Moving => TileAttributes {
                color:  (0.1, 0.1, 0.1),
                movement_translate_pos: Some((1, 0)),
                movement_distance: TILE_SIZE * 4.0,
                movement_speed: 2.0,
                ..Default::default()
            }
        }
    }
}

#[derive(Component)]
struct SolidTile;

// Stores moving tile start point and movement speed
#[derive(Component)]
pub(crate) struct MovingTileMotion {
    origin: Vec3,
    direction: Vec2,
    pub(crate) previous_translation: Vec3,
}

// Read tilemap to spawn all tiles
pub(crate) fn spawn_level(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    current_level: &CurrentLevel,
) {
    let level = LEVELS.get(current_level.0).unwrap_or(&LEVEL_1);
    for (i, row) in level.iter().rev().enumerate() {
        for (j, cell) in row.chars().enumerate() {
            match cell {
                '.' => continue,
                'D' => spawn_block(commands, meshes, materials, Tile::Dirt, (j, i)),
                'G' => spawn_block(commands, meshes, materials, Tile::Grass, (j, i)),
                'L' => spawn_block(commands, meshes, materials, Tile::Lava, (j, i)),
                'B' => spawn_block(commands, meshes, materials, Tile::Bedrock, (j, i)),
                'W' => spawn_block(commands, meshes, materials, Tile::Win, (j, i)),
                'C' => spawn_block(commands, meshes, materials, Tile::Coin, (j, i)),
                'J' => spawn_block(commands, meshes, materials, Tile::Bounce, (j, i)),
                'M' => spawn_block(commands, meshes, materials, Tile::Moving, (j, i)),
                'P' => {
                    // Draw player
                    commands.spawn((
                        Sprite::from_color(
                            Color::srgb(0.95, 0.35, 0.25),
                            PLAYER_SIZE,
                        ),
                        Transform::from_xyz(j as f32 * TILE_SIZE, i as f32 * TILE_SIZE, 1.0),
                        Player,
                        GameEntity,
                        Movement {
                            velocity: vec2(0.0, 0.0),
                            speed_multiplier: 1.0,
                            number_of_jumps: 0,
                            coyote_time_remaining: 0.0,
                        },
                        Health::new(2),
                        DamageCooldown::new(1.0),
                        DamageFlash::new(0.15),
                    ));
                }
                _ => ()
            }
        }
    }
}

// Helper function to create the entity for a tile
pub(crate)fn spawn_block(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    tile: Tile,
    dims: (usize, usize)
) {
    let attributes = tile.attributes();    

    let transform = Transform::from_xyz(
        dims.0 as f32 * TILE_SIZE,
        dims.1 as f32 * TILE_SIZE,
        0.0,
    );

    // Coins need different stuff apparently
    if attributes.is_circle {
        let mesh = meshes.add(Circle::new(TILE_SIZE / 2.0));
        let material = materials.add(ColorMaterial::from(
            Color::srgb(
                attributes.color.0,
                attributes.color.1,
                attributes.color.2,
            )
        ));

        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            transform,
            tile,
            GameEntity,
        ));
    } else {
        let sprite = Sprite::from_color(
            Color::srgb(
                attributes.color.0,
                attributes.color.1,
                attributes.color.2,
            ),
            vec2(TILE_SIZE, TILE_SIZE),
        );
        if attributes.is_solid {
            // If moving tile
            if let Some((dx, dy)) = attributes.movement_translate_pos {
                let motion = MovingTileMotion {
                    origin: transform.translation,
                    direction: Vec2::new(dx as f32, dy as f32).normalize_or_zero(),
                    previous_translation: transform.translation,
                };

                commands.spawn((
                    sprite,
                    transform,
                    SolidTile,
                    tile,
                    GameEntity,
                    motion,
                ));
            } else { // Tile doesn't move
                commands.spawn((
                    sprite,
                    transform,
                    SolidTile,
                    tile,
                    GameEntity,
                ));
            }
        } else {
            commands.spawn((
                sprite,
                transform,
                tile,
                GameEntity,
            ));
        }
    }
}

pub(crate)fn move_tiles(
    time: Res<Time>,
    mut tiles: Query<(&mut Transform, &mut MovingTileMotion)>,
) {
    // Time for tile to move one cycle
    let phase = time.elapsed_secs() * MOVING_TILE_SPEED;
    // Offset sin function to influence distance moved
    let offset = phase.sin() * MOVING_TILE_DISTANCE;

    // Move offset distance
    for (mut transform, mut motion) in &mut tiles {
        motion.previous_translation = transform.translation;
        transform.translation = motion.origin + Vec3::new(motion.direction.x * offset, motion.direction.y * offset, 0.0);
    }
}
