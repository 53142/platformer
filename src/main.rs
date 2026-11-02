use bevy::prelude::*;

mod components;
mod constants;
mod levels;
mod tiles;
mod player;
mod environment;
mod ui;
mod startup;

use components::{
    CurrentLevel,
    GameState,
    Score,
    GameTimer,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Platformer Game".into(),
                resolution: (960, 540).into(),
                ..default()
            }),
            ..default()
        }))

        .add_systems(
            Startup,
            (
                startup::setup,
                ui::show_initial_message,
            ),
        )

        .add_systems(
            Update,
            (
                ui::hide_initial_message,
                startup::tick_game_timer,
                player::process_controls,
                tiles::move_tiles,
                player::move_player,
                environment::process_environment,
                player::check_game_over,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        )

        .add_systems(
            Update,
            ui::update_damage_flash
                .after(environment::process_environment)
                .run_if(in_state(GameState::InGame)),
        )

        .add_systems(
            Update,
            ui::update_game_ui
                .after(environment::process_environment)
                .run_if(in_state(GameState::InGame)),
        )

        .add_systems(
            Update,
            ui::restart_button_system
                .after(ui::update_game_ui)
                .after(player::check_game_over),
        )

        .add_systems(
            OnEnter(GameState::GameLost),
            ui::show_game_lost,
        )

        .add_systems(
            OnEnter(GameState::GameWon),
            ui::show_game_won,
        )

        .init_resource::<Score>()
        .init_resource::<CurrentLevel>()
        .init_resource::<GameTimer>()
        .init_state::<GameState>()

        .run();
}