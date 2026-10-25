use bevy::prelude::*;

const DIRT_COLOR: (f32, f32, f32) = (155.0/255.0, 118.0/255.0, 83.0/255.0);
const GRASS_COLOR: (f32, f32, f32) = (0.1, 0.3, 0.03);
const LAVA_COLOR: (f32, f32, f32) = (1.0, 0.1, 0.1);

const TILE_SIZE: f32 = 32.0;
const PLAYER_SIZE: Vec2 = vec2(26.0, 32.0 * 1.5);

// Make TileTypes a component so can easily select them
#[derive(Component, Copy, Clone, PartialEq)]
enum TileType {
    Dirt,
    Grass,
    Lava,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    InGame,
    GameOver,
}

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Movement {
    velocity: Vec2,
    number_of_jumps: u8,
}

#[derive(Component)]
struct Health {
    max_health: i32,
    current_health: i32,
}

impl Health {
    fn new(max_health: i32) -> Self {
        Self {
            max_health: max_health,
            current_health: max_health,
        }
    }

    fn damage(&mut self, amount: i32) {
        self.current_health = self.current_health.saturating_sub(amount);
    }

    fn player_dead(&self) -> bool {
        self.current_health == 0
    }
}

#[derive(Component)]
struct SolidTile;

#[derive(Component)]
struct DamageCooldown {
    timer: Timer,
    active: bool,
}

impl DamageCooldown {
    fn new(seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
            active: false,
        }
    }
}

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

        .add_systems(Update, (
            process_controls,
            move_player,
            process_environment,
            check_game_over,
        ).run_if(in_state(GameState::InGame)))

        .add_systems(OnEnter(GameState::GameOver), show_game_over)
        .init_state::<GameState>()
        .run();
}

fn setup(mut commands: Commands) {
    // Make camera
    commands.spawn(Camera2d);

    // Make tiles
    let tiles: &[(i32, i32, TileType)] = &[
        (0, 0, TileType::Dirt),
        (0, 1, TileType::Grass),
        (1, 0, TileType::Dirt),
        (1, 1, TileType::Grass),
        (2, 0, TileType::Dirt),
        (2, 1, TileType::Grass),
        (4, 1, TileType::Grass),
        (5, 1, TileType::Dirt),
        (6, 1, TileType::Grass),
        (8, 2, TileType::Grass),
        (10, 4, TileType::Dirt),
        (12, 4, TileType::Lava),
        (12, 3, TileType::Dirt),
    ];

    for tile in tiles {
        if tile.2 != TileType::Lava {
            commands.spawn((
                Sprite::from_color(
                    match tile.2 {
                        TileType::Dirt => Color::srgb(DIRT_COLOR.0, DIRT_COLOR.1, DIRT_COLOR.2),
                        TileType::Grass => Color::srgb(GRASS_COLOR.0, GRASS_COLOR.1, GRASS_COLOR.2),
                        _ => Color::srgb(0.0,0.0,0.0),
                    },
                    Vec2::splat(TILE_SIZE),
                ),
                Transform::from_xyz(
                    tile.0 as f32 * TILE_SIZE,
                    tile.1 as f32 * TILE_SIZE,
                    0.0,
                ),
                SolidTile,
                tile.2,
            ));
        } else {
            commands.spawn((
                Sprite::from_color(
                    match tile.2 {
                        TileType::Lava => Color::srgb(LAVA_COLOR.0, LAVA_COLOR.1, LAVA_COLOR.2),
                        _ => Color::srgb(0.0,0.0,0.0),
                    },
                    Vec2::splat(TILE_SIZE),
                ),
                Transform::from_xyz(
                    tile.0 as f32 * TILE_SIZE,
                    tile.1 as f32 * TILE_SIZE,
                    0.0,
                ),
                tile.2,
            ));
        }
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
        Health::new(3),
        DamageCooldown::new(1.0)
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

    // Can only jumps twice in air
    if (keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::ArrowUp)) && movement.number_of_jumps < 2 {
        movement.velocity.y = 300.0;
        movement.number_of_jumps += 1;
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
                    // Reset number of jumps
                    movement.number_of_jumps = 0;
                } else {
                    // Hitting head on tile
                    transform.translation.y = solid.translation.y - (TILE_SIZE + PLAYER_SIZE.y) / 2.0;
                }
                movement.velocity.y = 0.0;
            }
        }

        // Only apply gravity if not on ground
        if !on_ground {
            let gravity = 500.0;
            // Set terminal velocity
            movement.velocity.y = (movement.velocity.y - gravity * dt).max(-500.0);
        }
    }
}

fn process_environment(
    time: Res<Time>,
    mut player: Query<
        (&Transform, &mut Health, &mut DamageCooldown),
        With<Player>,
    >,
    tiles: Query<(&Transform, &TileType)>,
) {
    let Ok((player_transform, mut health, mut cooldown)) =
        player.single_mut()
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

    for (tile_transform, tile_type) in &tiles {
        if *tile_type != TileType::Lava {
            continue;
        }

        let touching_lava =
            (player_pos.x - tile_transform.translation.truncate().x).abs() < (PLAYER_SIZE.x + TILE_SIZE) / 2.0
            && (player_pos.y - tile_transform.translation.truncate().y).abs() < (PLAYER_SIZE.y + TILE_SIZE) / 2.0;

        // Only damage when touching lava and cooldown complete
        if touching_lava && !cooldown.active {
            health.damage(1);
            cooldown.timer.reset();
            cooldown.active = true;
            break;
        }
    }
}

fn check_game_over(mut next_state: ResMut<NextState<GameState>>, player: Query<(&Transform, &Health), With<Player>>) {
    let Ok((transform, health)) = player.single() else {
        return;
    };

    if health.player_dead() || transform.translation.y < -150.0 {
        next_state.set(GameState::GameOver)
    }
}

fn show_game_over(mut commands: Commands) {
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