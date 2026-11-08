use bevy::prelude::*;

use crate::components::{
    CurrentLevel,
    DamageCooldown,
    DamageFlash,
    GameEntity,
    GameState,
    Health,
    PlayerMovement,
    Player,
    Score,
};

use crate::constants::{
    PLAYER_SIZE,
    TILE_SIZE,
};

use crate::levels::LEVELS;

use crate::tiles::{ Tile, TileAttributes, spawn_level };

// Process tiles and other environment stuff
pub(crate) fn process_environment(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut player: Query<(&mut Transform, &mut PlayerMovement, &mut Health, &mut DamageCooldown, &mut DamageFlash), With<Player>>,
    mut tiles: Query<(Entity, &Transform, Option<&mut Sprite>, Option<&MeshMaterial2d<ColorMaterial>>, &Tile), Without<Player>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    world_entities: Query<Entity, With<GameEntity>>,
    mut current_level: ResMut<CurrentLevel>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok((mut player_transform, mut movement, mut health, mut cooldown, mut damage_flash)) = player.single_mut()
    else {
        return;
    };

    if cooldown.active {
        cooldown.timer.tick(time.delta());

        if cooldown.timer.is_finished() {
            cooldown.active = false;
        }
    }

    let player_pos = player_transform.translation.truncate();

    movement.speed_multiplier = 1.0;

    for (entity, tile_transform, sprite_opt, material_opt, tile) in &mut tiles {
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
            increment_level(&mut commands, &mut current_level, &world_entities, &mut meshes, &mut color_materials, &mut next_state);
            break; // Don't need to process anything else if won
          }
          Tile::Coin => {
            score.0 += 1;
            commands.entity(entity).despawn();
            continue;
          }
          Tile::Checkpoint => {
            // TODO: Eventually have a way to not reset spawn location if user touches old spawn point (user somehow fell or something and contacted old spawn location)
            movement.respawn_location = tile_transform.translation.truncate();
            commands.entity(entity).despawn();
            continue;
          }
          _ => {}
        }

        // If tile damages and player cooldown is inactive/tile doesn't have cooldown, damage player
        // Or if player falls too far damage them
        if attributes.damage > 0 && (!cooldown.active || !attributes.has_damage_cooldown)  {
          damage_player(&mut health, attributes, &mut damage_flash, &mut player_transform, &mut movement, &mut cooldown);
        }
    }
}

fn damage_player(
    health: &mut Health,
    attributes: TileAttributes,
    damage_flash: &mut DamageFlash,
    player_transform: &mut Transform,
    movement: &mut PlayerMovement,
    cooldown: &mut DamageCooldown,
) {
      health.damage(attributes.damage);
      damage_flash.timer.reset();
      damage_flash.active = true;

      // Move player to last check point position
      player_transform.translation.x = movement.respawn_location.x;
      player_transform.translation.y = movement.respawn_location.y;

      if attributes.has_damage_cooldown {
          cooldown.timer.reset();
          cooldown.active = true;
      }
    }

pub(crate) fn increment_level(
    mut commands: &mut Commands,
    current_level: &mut ResMut<CurrentLevel>,
    world_entities: &Query<Entity, With<GameEntity>>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut color_materials: &mut ResMut<Assets<ColorMaterial>>,
    next_state: &mut ResMut<NextState<GameState>>,
) {
    if current_level.0 + 1 < LEVELS.len() {
        current_level.0 += 1;

        // Despawn current world entities
        for entity in world_entities {
            commands.entity(entity).despawn();
        }

        spawn_level(
            &mut commands,
            &mut meshes,
            &mut color_materials,
            &*current_level,
        );
    } else {
        next_state.set(GameState::GameWon);
    }
}