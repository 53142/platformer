use bevy::prelude::*;

pub(crate) const PLAYER_SIZE: Vec2 = vec2(26.0, 32.0 * 1.5);
pub(crate) const TILE_SIZE: f32 = 32.0;

// Total distance moving tile moves (moves half of this value in each direction)
pub(crate) const MOVING_TILE_DISTANCE: f32 = TILE_SIZE * 3.0;
// Speed of moving tile - controls how long it takes to move a specific distance
pub(crate) const MOVING_TILE_SPEED: f32 = 1.5;

// Give player grace period to jump after walking off edge
pub(crate) const COYOTE_TIME: f32 = 0.1;