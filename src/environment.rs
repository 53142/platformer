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
    Score,
};

use crate::player::reset_player_position;

use crate::constants::{
    FALL_DISTANCE_TO_DIE, PLAYER_SIZE, TILE_SIZE,
};

use crate::levels::LEVELS;

use crate::tiles::{ Tile, TileAttributes, spawn_level };

// Process tiles and other environment stuff
pub(crate) fn process_environment(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut player: Query<(&mut Transform, &mut PlayerMovement, &mut Lives, &mut DamageCooldown, &mut DamageFlash), With<Player>>,
    mut tiles: Query<(Entity, &Transform, Option<&mut Sprite>, Option<&MeshMaterial2d<ColorMaterial>>, &Tile), Without<Player>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    world_entities: Query<Entity, With<GameEntity>>,
    mut current_level: ResMut<CurrentLevel>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok((
        mut player_transform,
        mut movement,
        mut lives,
        mut cooldown,
        mut damage_flash,
    )) = player.single_mut() else {
        return;
    };

    // Make player die if they fall too far
    if player_transform.translation.y < FALL_DISTANCE_TO_DIE && !cooldown.active {
        lives.remove_lives(1);

        // Show damage flash
        damage_flash.timer.reset();
        damage_flash.active = true;

        // Move player to last check point position
        player_transform.translation.x = movement.respawn_location.x;
        player_transform.translation.y = movement.respawn_location.y;

        // Reset player movement
        movement.velocity.x = 0.0;
        movement.velocity.y = 0.0;
    }

    if cooldown.active {
        cooldown.timer.tick(time.delta());
        if cooldown.timer.is_finished() {
            cooldown.active = false;
        }
    }

    movement.speed_multiplier = 1.0;

    for (entity, tile_transform, sprite_opt, material_opt, tile) in &mut tiles {
        let player_pos = player_transform.translation.truncate();
        let attributes = tile.attributes();

        // Animate tiles (handles both sprites and mesh color materials)
        if let (Some(target), Some(speed)) = (attributes.animate_color, attributes.animate_speed) {
            let base = attributes.color;
            let t = (time.elapsed_secs() * speed).sin() * 0.5 + 0.5;

            let new_color = Color::srgb(
                base.0 + (target.0 - base.0) * t,
                base.1 + (target.1 - base.1) * t,
                base.2 + (target.2 - base.2) * t
            );

            if let Some(mut sprite) = sprite_opt {
                sprite.color = new_color;
            } else if let Some(mat_handle) = material_opt {
                if let Some(mut material) = color_materials.get_mut(&mat_handle.0) {
                    material.color = new_color;
                }
            }
        }

        let touching_tile = (player_pos.x - tile_transform.translation.truncate().x).abs() <= (PLAYER_SIZE.x + TILE_SIZE) / 2.0
            && (player_pos.y - tile_transform.translation.truncate().y).abs() <= (PLAYER_SIZE.y + TILE_SIZE) / 2.0;

        if !touching_tile {
            continue;
        }

        // Take minimum speed multiplier between player and thew touching tile
        movement.speed_multiplier = movement.speed_multiplier.min(attributes.speed_multiplier);

        // Actions to do for each tile the player touches
        match *tile {
          Tile::Win => {
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
                &mut damage_flash,
            );
            break; // Don't need to process anything else if won
          }
          Tile::Coin => {
            score.0 += 1;
            commands.entity(entity).despawn();
            continue;
          }
          Tile::Checkpoint => {
            movement.respawn_location.x = tile_transform.translation.x;
            movement.respawn_location.y = tile_transform.translation.y + PLAYER_SIZE.y / 2.0;
            commands.entity(entity).despawn();
            continue;
          }
          _ => {}
        }

        // If tile damages and player cooldown is inactive/tile doesn't have cooldown, damage player
        // Or if player falls too far damage them
        if attributes.damage > 0 && (!cooldown.active || !attributes.has_damage_cooldown) {
          damage_player(&mut lives, attributes, &mut damage_flash, &mut player_transform, &mut movement, &mut cooldown);
          break;
        }
    }
}

fn damage_player(
    lives: &mut Lives,
    attributes: TileAttributes,
    damage_flash: &mut DamageFlash,
    player_transform: &mut Transform,
    movement: &mut PlayerMovement,
    cooldown: &mut DamageCooldown,
) {
    lives.remove_lives(attributes.damage);

    damage_flash.timer.reset();
    damage_flash.active = true;

    // Respawn at checkpoint
    player_transform.translation.x =
        movement.respawn_location.x;

    player_transform.translation.y =
        movement.respawn_location.y;

    // Reset movement
    movement.velocity = Vec2::ZERO;
    movement.number_of_jumps = 0;
    movement.coyote_time_remaining = 0.0;

    if attributes.has_damage_cooldown {
        cooldown.timer.reset();
        cooldown.active = true;
    }
}

pub(crate) fn increment_level(
    commands: &mut Commands,
    current_level: &mut CurrentLevel,
    world_entities: &Query<Entity, With<GameEntity>>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    next_state: &mut NextState<GameState>,
    player_transform: &mut Transform,
    movement: &mut PlayerMovement,
    cooldown: &mut DamageCooldown,
    flash: &mut DamageFlash,
) {
    // Check whether another level exists
    if current_level.0 + 1 >= LEVELS.len() {
        next_state.set(GameState::GameWon);
        return;
    }

    current_level.0 += 1;

    // Despawn old level entities, but NOT the player
    for entity in world_entities.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn next level
    let Some(spawn_position) = spawn_level(
        commands,
        meshes,
        materials,
        current_level,
    ) else {
        error!("Failed to spawn next level!");
        return;
    };

    // Reset player position and movement
    reset_player_position(
        player_transform,
        movement,
        cooldown,
        flash,
        spawn_position,
    );
}