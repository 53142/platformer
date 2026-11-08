use bevy::prelude::*;

use crate::components::{
    CurrentLevel,
    GameEntity,
    Health,
    PlayerMovement,
    Player,
    GameState,
};

use crate::constants::{
    COYOTE_TIME,
    PLAYER_SIZE,
    TILE_SIZE,
};

use crate::environment::increment_level;
use crate::tiles::{
    MovingTileMotion,
    Tile,
};

// User controls
pub(crate) fn process_controls(
    commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: Query<&mut PlayerMovement, With<Player>>,
    current_level: ResMut<CurrentLevel>,
    world_entities: Query<Entity, With<GameEntity>>,
    meshes: ResMut<Assets<Mesh>>,
    color_materials: ResMut<Assets<ColorMaterial>>,
    next_state: ResMut<NextState<GameState>>,
) {
    let Ok(mut movement) = player.single_mut() else {
        return;
    };

    // Horizontal movement
    let mut direction = 0.0;

    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction = -1.0;
    }

    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction = 1.0;
    }

    movement.velocity.x = direction * 100.0;

    // Can only jump twice in air
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::ArrowUp) {
        if movement.number_of_jumps == 0 && movement.coyote_time_remaining > 0.0 {
            movement.velocity.y = 300.0;
            movement.number_of_jumps = 1;
            movement.coyote_time_remaining = 0.0;
        } else if movement.number_of_jumps == 1 {
            movement.velocity.y = 300.0;
            movement.number_of_jumps = 2;
        }
    }

    // Skip level
    if keyboard.just_pressed(KeyCode::Semicolon) {
        increment_level(
            commands,
            current_level,
            world_entities,
            meshes,
            color_materials,
            next_state,
        );
    }
}

// Player and camera movement
pub(crate) fn move_player(time: Res<Time>, mut player: Query<(&mut Transform, &mut PlayerMovement), With<Player>>, tiles: Query<(&Transform, &Tile, Option<&MovingTileMotion>), Without<Player>>, mut camera: Query<&mut Transform, (With<Camera>, Without<Player>, Without<Tile>)>,) {
    for (mut player_transform, mut movement) in &mut player {
        let dt = time.delta_secs();

        for (tile_transform, tile, motion) in &tiles {
            let Some(motion) = motion else {
                continue;
            };

            if !tile.attributes().is_solid || movement.velocity.y > 0.0 {
                continue;
            }


            let player_bottom = player_transform.translation.y - PLAYER_SIZE.y / 2.0;
            let previous_tile_top = motion.previous_translation.y + TILE_SIZE / 2.0;

            let vertical_distance = (player_bottom - previous_tile_top).abs();
            let horizontal_distance = (player_transform.translation.x - motion.previous_translation.x).abs();

            let was_standing_on_tile = vertical_distance <= 2.0 && horizontal_distance < (PLAYER_SIZE.x + TILE_SIZE) / 2.0;

            // If player contacting tile, move player by the tile's movement
            if was_standing_on_tile {
                let tile_movement = tile_transform.translation - motion.previous_translation;
                player_transform.translation += tile_movement;
            }
        }

        // Resolve x-axis collisions to prevent phasing through wall when falling
        player_transform.translation.x += dt * movement.velocity.x * movement.speed_multiplier;

        for (tile_transform, tile, _) in &tiles {
            if !tile.attributes().is_solid {
                continue;
            }
            if (player_transform.translation.x - tile_transform.translation.x).abs() < ((PLAYER_SIZE.x + TILE_SIZE) / 2.0)
                && (player_transform.translation.y - tile_transform.translation.y).abs() < ((PLAYER_SIZE.y + TILE_SIZE) / 2.0) {
                // Push player out of the wall horizontally
                if movement.velocity.x > 0.0 {
                    player_transform.translation.x = tile_transform.translation.x - (PLAYER_SIZE.x + TILE_SIZE) / 2.0;
                } else if movement.velocity.x < 0.0 {
                    player_transform.translation.x = tile_transform.translation.x + (PLAYER_SIZE.x + TILE_SIZE) / 2.0;
                }
            }
        }

        // Resolve y-axis collisions next
        player_transform.translation.y += dt * movement.velocity.y * movement.speed_multiplier;

        let mut on_ground = false;

        for (tile_transform, tile, _) in &tiles {
            if !tile.attributes().is_solid {
                continue;
            }
            if (player_transform.translation.x - tile_transform.translation.x).abs() < ((PLAYER_SIZE.x + TILE_SIZE) / 2.0)
                && (player_transform.translation.y - tile_transform.translation.y).abs() < ((PLAYER_SIZE.y + TILE_SIZE) / 2.0) {
                if movement.velocity.y <= 0.0 {
                    // Landing on top of a tile
                    on_ground = true;
                    player_transform.translation.y = tile_transform.translation.y + (PLAYER_SIZE.y + TILE_SIZE) / 2.0;
                } else {
                    // Hitting head on tile
                    player_transform.translation.y = tile_transform.translation.y - (TILE_SIZE + PLAYER_SIZE.y) / 2.0;
                }
                movement.velocity.y = 0.0;
            }
        }

        if on_ground {
            // Reset number of jumps & coyote time when touch ground
            movement.number_of_jumps = 0;
            movement.coyote_time_remaining = COYOTE_TIME;
        } else { // Count coyote time in air
            // Tick timer
            movement.coyote_time_remaining = (movement.coyote_time_remaining - dt).max(0.0);
        }

        // Only apply gravity if not on ground
        if !on_ground {
            let gravity = 500.0;
            // Set terminal velocity
            movement.velocity.y = (movement.velocity.y - gravity * dt).max(-500.0);
        }

        let Ok(mut camera_transform) = camera.single_mut() else {
            return;
        };

        // Ease-out camera movement so it looks nice
        let smoothing = 1.0 - (1.0 - dt).powf(3.0);

        // Move camera
        camera_transform.translation.x += (player_transform.translation.x - camera_transform.translation.x) * smoothing;
        camera_transform.translation.y += (player_transform.translation.y - camera_transform.translation.y) * smoothing;
    }
}

// Check if player dead, if so queue death screen
pub(crate) fn check_game_over(mut next_state: ResMut<NextState<GameState>>, player: Query<&Health, With<Player>>) {
    let Ok(health) = player.single() else {
        return;
    };

    if health.player_dead() {
        next_state.set(GameState::GameLost)
    }
}