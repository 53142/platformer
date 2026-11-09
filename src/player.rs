use bevy::prelude::*;

use crate::components::{
    CurrentLevel,
    DamageCooldown,
    DamageFlash,
    GameEntity,
    Lives,
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
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Transform, &mut PlayerMovement, &mut DamageCooldown, &mut DamageFlash), With<Player>>,
    mut current_level: ResMut<CurrentLevel>,
    world_entities: Query<Entity, With<GameEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok((mut player_transform, mut movement, mut cooldown, mut flash)) = player.single_mut() else {
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
            &mut commands,
            &mut current_level,
            &world_entities,
            &mut *meshes,
            &mut *color_materials,
            &mut next_state,
            &mut player_transform,
            &mut movement,
            &mut cooldown,
            &mut flash,
        );

        return;
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

        // Hopefully fix player glitching through blocks
        // Basically we are processing movement in multiple sub steps an checking for contact each time
        let max_step_dist = TILE_SIZE / 4.0;

        // Resolve x-axis collisions first
        let delta_x = dt * movement.velocity.x * movement.speed_multiplier;
        let x_steps = (delta_x.abs() / max_step_dist).ceil().max(1.0) as usize;
        let x_step = delta_x / x_steps as f32;
        for _ in 0..x_steps {
            player_transform.translation.x += x_step;

            let mut hit_wall = false;

            for (tile_transform, tile, _) in &tiles {
                if !tile.attributes().is_solid {
                    continue;
                }

                let dx = (player_transform.translation.x - tile_transform.translation.x).abs();
                let dy = (player_transform.translation.y - tile_transform.translation.y).abs();

                // If player contacting tile
                if dx < (PLAYER_SIZE.x + TILE_SIZE) / 2.0 && dy < (PLAYER_SIZE.y + TILE_SIZE) / 2.0 {
                    if x_step > 0.0 {
                        // Snap to left of tile
                        player_transform.translation.x = tile_transform.translation.x - (PLAYER_SIZE.x + TILE_SIZE) / 2.0;
                    } else if x_step < 0.0 {
                        // Snap to right of tile
                        player_transform.translation.x = tile_transform.translation.x + (PLAYER_SIZE.x + TILE_SIZE) / 2.0;
                    }

                    movement.velocity.x = 0.0;
                    hit_wall = true;
                    break;
                }
            }

            // Don't need to process anything else if contacted wall
            if hit_wall {
                break;
            }
        }

        // Resolve y-axis collisions using substeps
        // Do bounce calculation here player doesn't get stuck when jumping into bounce tile
        // Also fixes the issue of being able to bounce by contacting the tile on the side
        let delta_y = dt * movement.velocity.y * movement.speed_multiplier;
        let y_steps = (delta_y.abs() / max_step_dist).ceil().max(1.0) as usize;
        let y_step = delta_y / y_steps as f32;

        let mut on_ground = false;

        for _ in 0..y_steps {
            if movement.velocity.y == 0.0 {
                break;
            }

            let prev_y = player_transform.translation.y;
            let next_y = prev_y + y_step;

            let mut hit_tile = false;

            for (tile_transform, tile, _) in &tiles {
                if !tile.attributes().is_solid {
                    continue;
                }

                let dist_x = (player_transform.translation.x - tile_transform.translation.x).abs();
                if dist_x >= (PLAYER_SIZE.x + TILE_SIZE) / 2.0 {
                    continue;
                }

                let tile_top = tile_transform.translation.y + TILE_SIZE / 2.0;
                let tile_bottom = tile_transform.translation.y - TILE_SIZE / 2.0;

                // Landing on top of a tile
                if y_step < 0.0 && (prev_y - PLAYER_SIZE.y / 2.0) >= tile_top && (next_y - PLAYER_SIZE.y / 2.0) <= tile_top {
                    player_transform.translation.y = tile_top + PLAYER_SIZE.y / 2.0;
                    
                    if tile.attributes().bounce > 0.0 {
                        // Bounce player
                        movement.velocity.y = tile.attributes().bounce;
                        movement.number_of_jumps = 1;
                        movement.coyote_time_remaining = 0.0;
                    } else {
                        // Normal landing
                        movement.velocity.y = 0.0;
                        on_ground = true;
                    }

                    hit_tile = true;
                    break;
                }
                // Hitting underside of tile

                // Moving up and hitting tile
                if y_step > 0.0 && (prev_y + PLAYER_SIZE.y / 2.0) <= tile_bottom && (next_y + PLAYER_SIZE.y / 2.0) >= tile_bottom {
                    player_transform.translation.y = tile_bottom - PLAYER_SIZE.y / 2.0;
                    movement.velocity.y = 0.0;
                    hit_tile = true;
                    break;
                }

            }

            if hit_tile {
                break;
            }

            // If no collisions, move normally
            player_transform.translation.y = next_y;
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
            let gravity = 450.0;
            // Set terminal velocity
            movement.velocity.y = (movement.velocity.y - gravity * dt).max(-800.0);
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

pub(crate) fn reset_player_position(
    transform: &mut Transform,
    movement: &mut PlayerMovement,
    cooldown: &mut DamageCooldown,
    flash: &mut DamageFlash,
    spawn_position: Vec2,
) {
    transform.translation.x = spawn_position.x;
    transform.translation.y = spawn_position.y;

    movement.velocity = Vec2::ZERO;
    movement.speed_multiplier = 1.0;
    movement.number_of_jumps = 0;
    movement.coyote_time_remaining = 0.0;
    movement.respawn_location = spawn_position;

    cooldown.active = false;
    cooldown.timer.reset();

    flash.active = false;
    flash.timer.reset();
}

// Check if player dead, if so queue death screen
pub(crate) fn check_game_over(mut next_state: ResMut<NextState<GameState>>, player: Query<&Lives, With<Player>>) {
    let Ok(health) = player.single() else {
        return;
    };

    if health.game_over() {
        next_state.set(GameState::GameLost)
    }
}