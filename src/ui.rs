use bevy::prelude::*;
use crate::levels::LEVELS;

use crate::components::{
    CurrentLevel,
    DamageFlash,
    EndScreenRoot,
    GameEntity,
    GameState,
    GameTimer,
    Health,
    HealthText,
    InitialMessage,
    Player,
    RestartButton,
    Score,
    ScoreText,
    TimerText,
    LevelText,
};

use crate::tiles::spawn_level;

// Update HUDs
pub(crate) fn update_game_ui(
    player: Query<&Health, With<Player>>,
    score: Res<Score>,
    timer: Res<GameTimer>,
    level: Res<CurrentLevel>,
    mut health_text: Query<&mut Text, (With<HealthText>, Without<ScoreText>, Without<TimerText>, Without<LevelText>)>,
    mut score_text: Query<&mut Text, (With<ScoreText>, Without<HealthText>, Without<TimerText>, Without<LevelText>)>,
    mut timer_text: Query<&mut Text, (With<TimerText>, Without<HealthText>, Without<ScoreText>, Without<LevelText>)>,
    mut level_text: Query<&mut Text, (With<LevelText>, Without<HealthText>, Without<ScoreText>, Without<TimerText>)>,
) {
    let Ok(health) = player.single() else { return; };

    if let Ok(mut text) = health_text.single_mut() {
        text.0 = format!("Health: {} / {}", health.current_health, health.max_health);
    }
    if let Ok(mut text) = score_text.single_mut() {
        text.0 = format!("Score: {}", score.0);
    }
    if let Ok(mut text) = timer_text.single_mut() {
        // 1 decimal points looks cool lol
        text.0 = format!("Time: {:.1}s", timer.0);
    }
    if let Ok(mut text) = level_text.single_mut() {
        text.0 = format!("Level: {} / {}", level.0 + 1, LEVELS.len())
    }
}

// Flash player a different color when they are hurt
pub(crate) fn update_damage_flash(time: Res<Time>, mut player: Query<(&mut Sprite, &mut DamageFlash), With<Player>>) {
    let Ok((mut sprite, mut flash)) = player.single_mut() else {
        return;
    };

    if flash.active {
        // Advance timer
        flash.timer.tick(time.delta());

        if flash.timer.is_finished() {
            flash.active = false;
            sprite.color = Color::srgb(0.95, 0.35, 0.25);
        } else {
            // Timer running so set player to white
            sprite.color = Color::WHITE;
        }
    } else {
        sprite.color = Color::srgb(0.95, 0.35, 0.25);
    }
}

// Show start message
pub(crate) fn show_initial_message(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        InitialMessage(Timer::from_seconds(4.0, TimerMode::Once))
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Get to the color-changing block at the top to win. Try to collect as many coins as you can and avoid the lava!"),
            TextColor(Color::srgb(0.0, 1.0, 1.0)),
        ));
    });
}

// Called every loop to wait until it is time to hide the initial message
pub(crate) fn hide_initial_message(
    mut commands: Commands,
    time: Res<Time>,
    mut messages: Query<(Entity, &mut InitialMessage)>,
) {
    for (entity, mut message) in &mut messages {
        message.0.tick(time.delta());
        if message.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// Game over message
pub(crate) fn show_game_lost(mut commands: Commands) {
    show_end_screen(
        &mut commands,
        "Game Over!",
        Color::srgb(1.0, 0.0, 0.0),
    );
}

// Winner message
pub(crate) fn show_game_won(mut commands: Commands) {
    show_end_screen(
        &mut commands,
        "You won!",
        Color::srgb(0.0, 1.0, 0.0),
    );
}

// Monitor for restart button interactions and reset level when butotn pressed
pub(crate) fn restart_button_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<ColorMaterial>>,
    mut buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<RestartButton>)>,
    world_entities: Query<Entity, With<GameEntity>>,
    end_screens: Query<Entity, With<EndScreenRoot>>,
    mut score: ResMut<Score>,
    mut timer: ResMut<GameTimer>,
    mut current_level: ResMut<CurrentLevel>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut restart_requested = false;

    // Button interaction colors
    for (interaction, mut background) in &mut buttons {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(Color::srgb(0.1, 0.5, 0.2));
                restart_requested = true;
            }
            Interaction::Hovered => *background = BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            Interaction::None => *background = BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        }
    }

    if !restart_requested {
        return;
    }

    // Despawn all entities
    for entity in &world_entities {
        commands.entity(entity).despawn();
    }

    // Remove win or loss screen
    for entity in &end_screens {
        commands.entity(entity).despawn();
    }

    // Reset score
    score.0 = 0;
    timer.0 = 0.0;

    // Reset to first level again
    current_level.0 = 0;

    // Spawn new level including: player, tiles, and coins
    spawn_level(&mut commands, &mut meshes, &mut materials, &current_level);

    // Resume game
    next_state.set(GameState::InGame);
}

// End screen general template function
fn show_end_screen(commands: &mut Commands, message: &str, message_color: Color) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            EndScreenRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(message),
                TextColor(message_color),
            ));
            parent
                .spawn((
                    Button,
                    Node {
                        margin: UiRect {
                            top: px(24),
                            ..default()
                        },
                        padding: UiRect::all(px(12)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    RestartButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Restart"),
                        TextColor(Color::WHITE),
                    ));
                });
        });
}