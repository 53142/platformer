use bevy::prelude::*;

use crate::components::{
    CurrentLevel, GameTimer, HealthText, LevelText, ScoreText, TimerText,
};

use crate::tiles::spawn_level;

pub(crate) fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, current_level: Res<CurrentLevel>) {
    // Make camera
    commands.spawn(Camera2d);

    // Make health text
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

    // Make score text
    commands.spawn((
        Text::new("Score: 0"),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(12),
            ..default()
        },
        ScoreText,
    ));

    // Timer text
    commands.spawn((
        Text::new("Time: 0.0s"),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: Val::Percent(45.0),
            ..default()
        },
        TimerText,
    ));

    // Level text
    commands.spawn((
        Text::new("Level: 0 / 0"),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: Val::Percent(65.0),
            ..default()
        },
        LevelText,
    ));

    // Create level
    spawn_level(&mut commands, &mut meshes, &mut materials, &*current_level);
}

pub(crate) fn tick_game_timer(time: Res<Time>, mut timer: ResMut<GameTimer>) {
    timer.0 += time.delta_secs();
}