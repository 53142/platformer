use bevy::prelude::*;

use crate::components::{
    CurrentLevel, GameTimer, HealthText, LevelText, ScoreText, TimerText,
};

use crate::tiles::{spawn_level, spawn_player};

pub(crate) fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, current_level: Res<CurrentLevel>) {
    // Spawn camera
    commands.spawn(Camera2d);

    // Spawn health text
    commands.spawn((
        Text::new("Health: 0 / 0"),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
        HealthText,
    ));

    // Spawn level text
    commands.spawn((
        Text::new("Level: 0 / 0"),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: Val::Percent(40.0),
            ..default()
        },
        LevelText,
    ));

    // Spawn score text
    commands.spawn((
        Text::new("Score: 0"),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: Val::Percent(60.0),
            ..default()
        },
        ScoreText,
    ));

    // Spawn timer text
    commands.spawn((
        Text::new("Time: 0.0s"),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(12),
            ..default()
        },
        TimerText,
    ));

    // Spawn level
    if let Some(spawn_position) = spawn_level(
        &mut commands,
        &mut meshes,
        &mut materials,
        &current_level,
    ) {
        spawn_player(&mut commands, spawn_position);
    }
}

pub(crate) fn tick_game_timer(time: Res<Time>, mut timer: ResMut<GameTimer>) {
    timer.0 += time.delta_secs();
}