use bevy::prelude::*;

const PLAYER_SIZE: Vec2 = vec2(26.0, 32.0 * 1.5);

const LEVEL: &[&str] = &[
    "....G......G....................",
    ".....................G..........",
    "............................L...",
    "............................D...",
    "....GG.......GGG................",
    "................................",
    "GGGLLLLLLLLLLLLL................",
    "DDDDDDDDDDDDDDDD................",
];

#[derive(Clone, Copy, Debug)]
pub struct TileAttributes {
    pub color: (f32, f32, f32),
    pub size: Vec2, // Overall dimensions of tile
    pub damage_on_touch: i32,
    pub speed_multiplier: f32,
    pub is_solid: bool, // Whether or not collisions with other entities occur
}

// Make it so you don't have to type every single attribute all the time
impl Default for TileAttributes {
    fn default() -> Self {
        Self {
            color: (0.0, 0.0, 0.0),
            size: vec2(32.0, 32.0),
            damage_on_touch: 0,
            speed_multiplier: 1.0,
            is_solid: true,
        }
    }
}

// Make Tiles components so we easily select them
#[derive(Component, Copy, Clone, PartialEq)]
enum Tile {
    Dirt,
    Grass,
    Lava,
}
impl Tile {
    pub fn attributes(&self) -> TileAttributes {
        match self {
            Tile::Dirt => TileAttributes {
                color: (0.6078, 0.4627, 0.3255),
                ..Default::default()
            },
            Tile::Grass => TileAttributes {
                color: (0.1, 0.3, 0.03),
                ..Default::default()
            },
            Tile::Lava => TileAttributes {
                color: (1.0, 0.1, 0.1),
                speed_multiplier: 0.3,
                is_solid: false,
                damage_on_touch: 1,
                ..Default::default()
            },
        }
    }
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

    for (i, row) in LEVEL.into_iter().rev().enumerate() {
        for (j, column) in row.chars().enumerate() {
            match column {
                '.' => continue,
                'D' => spawn_block(&mut commands, Tile::Dirt, (j, i)),
                'G' => spawn_block(&mut commands, Tile::Grass, (j, i)),
                'L' => spawn_block(&mut commands, Tile::Lava, (j, i)),
                _ => ()
            }
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

fn spawn_block(commands: &mut Commands, tile: Tile, dims: (usize, usize)) {
    let attributes = tile.attributes();

    let sprite = Sprite::from_color(
        Color::srgb(tile.attributes().color.0, tile.attributes().color.1, tile.attributes().color.2),
        tile.attributes().size,
    );
    let transform = Transform::from_xyz(
        dims.0 as f32 * tile.attributes().size.x,
        dims.1 as f32 * tile.attributes().size.y,
        0.0,
    );

    if attributes.is_solid {
        commands.spawn((sprite, transform, SolidTile, tile));
    } else {
        commands.spawn((sprite, transform, tile));
    }
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

fn move_player(time: Res<Time>, mut player: Query<(&mut Transform, &mut Movement), With<Player>>, tiles: Query<(&Transform, &Tile), Without<Player>>, mut camera: Query<&mut Transform, (With<Camera>, Without<Player>, Without<Tile>)>,) {
    for (mut player_transform, mut movement) in &mut player {
        let dt = time.delta_secs();

        // Resolve x-axis collisions to prevent phasing through wall when falling
        player_transform.translation.x += dt * movement.velocity.x;

        for (tile_transform, tile) in &tiles {
            if !tile.attributes().is_solid {
                continue;
            }
            if (player_transform.translation.x - tile_transform.translation.x).abs() < ((PLAYER_SIZE.x + tile.attributes().size.x) / 2.0)
                && (player_transform.translation.y - tile_transform.translation.y).abs() < ((PLAYER_SIZE.y + tile.attributes().size.y) / 2.0) {
                // Push player out of the wall horizontally
                if movement.velocity.x > 0.0 {
                    player_transform.translation.x = tile_transform.translation.x - (PLAYER_SIZE.x + tile.attributes().size.x) / 2.0;
                } else if movement.velocity.x < 0.0 {
                    player_transform.translation.x = tile_transform.translation.x + (PLAYER_SIZE.x + tile.attributes().size.y) / 2.0;
                }
            }
        }

        // Resolve y-axis collisions next
        player_transform.translation.y += dt * movement.velocity.y;

        let mut on_ground = false;

        for (tile_transform, tile) in &tiles {
            if !tile.attributes().is_solid {
                continue;
            }
            if (player_transform.translation.x - tile_transform.translation.x).abs() < ((PLAYER_SIZE.x + tile.attributes().size.x) / 2.0)
                && (player_transform.translation.y - tile_transform.translation.y).abs() < ((PLAYER_SIZE.y + tile.attributes().size.y) / 2.0) {
                if movement.velocity.y <= 0.0 {
                    // Landing on top of a tile
                    on_ground = true;
                    player_transform.translation.y = tile_transform.translation.y + (PLAYER_SIZE.y + tile.attributes().size.y) / 2.0;
                    // Reset number of jumps
                    movement.number_of_jumps = 0;
                } else {
                    // Hitting head on tile
                    player_transform.translation.y = tile_transform.translation.y - (tile.attributes().size.y + PLAYER_SIZE.y) / 2.0;
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

        // Move camera
        let Ok(mut camera_transform) = camera.single_mut() else {
            return;
        };
        camera_transform.translation.x = player_transform.translation.x;
        camera_transform.translation.y = player_transform.translation.y;
    }
}

fn process_environment(
    time: Res<Time>,
    mut player: Query<
        (&Transform, &mut Health, &mut DamageCooldown),
        With<Player>,
    >,
    tiles: Query<(&Transform, &Tile)>,
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

    for (tile_transform, tile) in &tiles {
        // Skip tiles that don't affect player health
        if tile.attributes().damage_on_touch == 0 {
            continue;
        }

        let touching_lava =
            (player_pos.x - tile_transform.translation.truncate().x).abs() < (PLAYER_SIZE.x + tile.attributes().size.x) / 2.0
            && (player_pos.y - tile_transform.translation.truncate().y).abs() < (PLAYER_SIZE.y + tile.attributes().size.y) / 2.0;

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