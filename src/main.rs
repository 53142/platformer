use bevy::prelude::*;


const DIRT_COLOR: (f32, f32, f32) = (0.1, 0.07, 0.07);
const GRASS_COLOR: (f32, f32, f32) = (0.1, 0.3, 0.03);

const TILE_SIZE: f32 = 32.0;
const PLAYER_SIZE: Vec2 = vec2(32.0, 32.0 * 1.5);
enum TileType {
    DIRT,
    GRASS,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    InGame,
    GameOver,
}

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Movement {
    velocity: Vec2,
}

#[derive(Component)]
struct SolidTile;

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
        .add_systems(Startup, setup)
        .add_systems(Update, process_controls)
        .add_systems(Update, move_player)
        .add_systems(Update, check_game_over)
        .run();
}

fn setup(mut commands: Commands) {
    // Make camera
    commands.spawn(Camera2d);

    // Make tiles
    let tiles: &[(i32, i32, TileType)] = &[
        (0, 0, TileType::DIRT),
        (0, 1, TileType::GRASS),
        (1, 0, TileType::DIRT),
        (1, 1, TileType::GRASS),
        (2, 0, TileType::DIRT),
        (2, 1, TileType::GRASS),
        (4, 1, TileType::GRASS),
        (5, 1, TileType::DIRT),
        (6, 1, TileType::GRASS),
        (10, 1, TileType::DIRT),
    ];

    for tile in tiles {
        commands.spawn((
            Sprite::from_color(
                match tile.2 {
                    TileType::DIRT => Color::srgb(DIRT_COLOR.0, DIRT_COLOR.1, DIRT_COLOR.2),
                    TileType::GRASS => Color::srgb(GRASS_COLOR.0, GRASS_COLOR.1, GRASS_COLOR.2),
                },
                Vec2::splat(TILE_SIZE),
            ),
            Transform::from_xyz(
                tile.0 as f32 * TILE_SIZE,
                tile.1 as f32 * TILE_SIZE,
                0.0,
            ),
            SolidTile,
        ));
    }

    // Draw player
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.95, 0.35, 0.25),
            PLAYER_SIZE,
        ),
        Transform::from_xyz(0.0, 100.0, 1.0),
        Player,
        Movement::default(),
    ));
}

fn process_controls(keyboard: Res<ButtonInput<KeyCode>>, mut player: Query<&mut Movement, With<Player>>) {
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

    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::ArrowUp) {
        movement.velocity.y = 200.0;
    }
}

fn move_player(time: Res<Time>, query: Query<(&mut Transform, &mut Movement), With<Player>>, solids: Query<&Transform, (With<SolidTile>, Without<Player>)>) {
    for (mut transform, mut movement) in query {
        let dt = time.delta_secs();

        // Resolve x-axis collisions to prevent phasing through wall when falling
        transform.translation.x += dt * movement.velocity.x;

        for solid in solids.iter() {
            if (transform.translation.x - solid.translation.x).abs() < ((PLAYER_SIZE.x + TILE_SIZE) / 2.0)
                && (transform.translation.y - solid.translation.y).abs() < ((PLAYER_SIZE.y + TILE_SIZE) / 2.0) {
                // Push player out of the wall horizontally
                if movement.velocity.x > 0.0 {
                    transform.translation.x = solid.translation.x - (PLAYER_SIZE.x + TILE_SIZE) / 2.0;
                } else if movement.velocity.x < 0.0 {
                    transform.translation.x = solid.translation.x + (PLAYER_SIZE.x + TILE_SIZE) / 2.0;
                }
            }
        }

        // Resolve y-axis collisions next
        transform.translation.y += dt * movement.velocity.y;

        let mut on_ground = false;

        for solid in solids.iter() {
            if (transform.translation.x - solid.translation.x).abs() < ((PLAYER_SIZE.x + TILE_SIZE) / 2.0)
                && (transform.translation.y - solid.translation.y).abs() < ((PLAYER_SIZE.y + TILE_SIZE) / 2.0) {
                if movement.velocity.y <= 0.0 {
                    // Landing on top of a tile
                    on_ground = true;
                    transform.translation.y = solid.translation.y + (TILE_SIZE + PLAYER_SIZE.y) / 2.0;
                } else {
                    // Hitting head on tile
                    transform.translation.y = solid.translation.y - (TILE_SIZE + PLAYER_SIZE.y) / 2.0;
                }
                movement.velocity.y = 0.0;
            }
        }

        // Only apply gravity if not on ground
        if !on_ground {
            let gravity = 200.0;
            movement.velocity.y -= gravity * dt;
        }
    }
}

fn check_game_over(mut commands: Commands, time: Res<Time>, query: Query<&Transform, With<Player>>) {
    for transform in query {
        let total_time = time.elapsed_secs();
        if total_time > 100.0 || transform.translation.y < -150.0 {
            commands.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("GAME OVER"),
                    TextColor(Color::srgb(1.0, 0.0, 0.0)),
                ));
            });
        }
    }
}