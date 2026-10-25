use bevy::prelude::*;

const PLAYER_SIZE: Vec2 = vec2(26.0, 32.0 * 1.5);
const TILE_SIZE: f32 = 32.0;

const LEVEL: &[&str] = &[
    ".......................................C..................",
    "...........................C..............................",
    ".................C........................................",
    "......................P...................................",
    "..........................................................",
    "......................D...........G........GGGG.......LLLD",
    "......................D.......................DGGG..G.LDDD",
    "......................DW.............................L...",
    "......................DGG............C................L...",
    "......................DDD.............................L...",
    "...............................................G......L...",
    "......................................................L...",
    "......................................................L...",
    "......................................C...............L...",
    "......................................................L...",
    "...............................................G......L...",    
    ".....................................................LLL..",
    "......................C..............................LDL..",
    "..............................GG.......GGG...........L.L..",
    ".....................................................L.L..",
    "..........................GGGLLLLLLLLLLLLLL..........L.L..",
    "..........................DDDDDDDDDDDDDDDDL..........L.L..",
    "..........................................L..........L.L..",
    "..........................................L..........L.L..",
    "..........................................L..........L.L..",
    "..........................................L..........L.L..",
    "..........................................L..........L.L..",
    "..........................................L..........L.L..",
    "..........................................L..........L.L..",
    "..........................................L..........L.L..",
    "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
];

#[derive(Clone, Copy, Debug)]
pub struct TileAttributes {
    pub color: (f32, f32, f32),
    pub animate_color: Option<(f32, f32, f32)>,
    pub animate_speed: Option<f32>,
    pub size: Vec2, // Overall dimensions of tile; it will prob break stuff if this is changed lol
    pub damage_on_touch: u32,
    pub has_damage_cooldown: bool,
    pub speed_multiplier: f32,
    pub is_solid: bool, // Whether or not collisions with other entities occur
}

// Make it so you don't have to type every single attribute all the time
impl Default for TileAttributes {
    fn default() -> Self {
        Self {
            color: (0.0, 0.0, 0.0),
            animate_color: None,
            animate_speed: None, // Higher number is faster animation
            size: vec2(32.0, 32.0),
            damage_on_touch: 0,
            has_damage_cooldown: true,
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
    Bedrock,
    Win,
    Coin,
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
                color: (1.0, 0.35, 0.0),
                animate_color: Some((0.7, 0.15, 0.1)),
                animate_speed: Some(0.5),
                speed_multiplier: 0.3,
                is_solid: false,
                damage_on_touch: 1,
                ..Default::default()
            },
            Tile::Bedrock => TileAttributes {
                damage_on_touch: 10000,
                has_damage_cooldown: false,
                is_solid: false,
                ..Default::default()
            },
            Tile::Win => TileAttributes {
                is_solid: false,
                color: (0.8, 0.2, 0.8),
                animate_color: Some((0.5, 0.9, 0.5)),
                animate_speed: Some(1.5),
                ..Default::default()
            },
            Tile::Coin => TileAttributes {
                is_solid: false,
                color: (1.0, 1.0, 0.0),
                animate_color: Some((0.8, 0.8, 0.3)),
                animate_speed: Some(6.0),
                ..Default::default()
            }
        }
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    InGame,
    GameWon,
    GameLost,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Movement {
    velocity: Vec2, // Velocity without speed multiplied
    speed_multiplier: f32,
    number_of_jumps: u8,
}

#[derive(Component)]
struct Health {
    max_health: u32,
    current_health: u32,
}

impl Health {
    fn new(max_health: u32) -> Self {
        Self {
            max_health: max_health,
            current_health: max_health,
        }
    }

    fn damage(&mut self, amount: u32) {
        self.current_health = self.current_health.saturating_sub(amount);
    }

    fn player_dead(&self) -> bool {
        self.current_health == 0
    }
}

#[derive(Component)]
struct HealthText;

#[derive(Component)]
struct ScoreText;

#[derive(Resource, Default)]
struct Score(u32);

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
        ).chain().run_if(in_state(GameState::InGame)))
        .add_systems(Update,update_game_ui
            .after(process_environment)
            .run_if(in_state(GameState::InGame)),
        )

        .add_systems(OnEnter(GameState::GameLost), show_game_lost)
        .add_systems(OnEnter(GameState::GameWon), show_game_won)
        .init_resource::<Score>()
        .init_state::<GameState>()
        .run();
}

fn setup(mut commands: Commands) {
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

    for (i, row) in LEVEL.into_iter().rev().enumerate() {
        for (j, column) in row.chars().enumerate() {
            match column {
                '.' => continue,
                'D' => spawn_block(&mut commands, Tile::Dirt, (j, i)),
                'G' => spawn_block(&mut commands, Tile::Grass, (j, i)),
                'L' => spawn_block(&mut commands, Tile::Lava, (j, i)),
                'B' => spawn_block(&mut commands, Tile::Bedrock, (j, i)),
                'W' => spawn_block(&mut commands, Tile::Win, (j, i)),
                'C' => spawn_block(&mut commands, Tile::Coin, (j, i)),
                'P' => {
                    // Draw player
                    commands.spawn((
                        Sprite::from_color(
                            Color::srgb(0.95, 0.35, 0.25),
                            PLAYER_SIZE,
                        ),
                        Transform::from_xyz(j as f32 * TILE_SIZE, i as f32 * TILE_SIZE, 1.0),
                        Player,
                        Movement {
                            velocity: vec2(0.0, 0.0),
                            speed_multiplier: 1.0,
                            number_of_jumps: 0,
                        },
                        Health::new(2),
                        DamageCooldown::new(1.0)
                    ));
                }
                _ => ()
            }
        }
    }
}

fn spawn_block(commands: &mut Commands, tile: Tile, dims: (usize, usize)) {
    let attributes = tile.attributes();

    let sprite = Sprite::from_color(
        Color::srgb(tile.attributes().color.0, tile.attributes().color.1, tile.attributes().color.2),
        tile.attributes().size,
    );
    let transform = Transform::from_xyz(
        dims.0 as f32 * TILE_SIZE,
        dims.1 as f32 * TILE_SIZE,
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
        player_transform.translation.x += dt * movement.velocity.x * movement.speed_multiplier;

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
        player_transform.translation.y += dt * movement.velocity.y * movement.speed_multiplier;

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
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut player: Query<(&Transform, &mut Movement, &mut Health, &mut DamageCooldown), With<Player>>,
    mut tiles: Query<(Entity, &Transform, &mut Sprite, &Tile), Without<Player>>,
) {
    let Ok((player_transform, mut movement, mut health, mut cooldown)) = player.single_mut()
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

    for (entity, tile_transform, mut sprite, tile) in &mut tiles {
        let attributes = tile.attributes();
        // Animate tiles
        if let Some(target) = attributes.animate_color {
            let base = attributes.color;

            // Oscillate between 0 and 1
            let t = (time.elapsed_secs() * attributes.animate_speed.unwrap()).sin() * 0.5 + 0.5;

            let r = base.0 + (target.0 - base.0) * t;
            let g = base.1 + (target.1 - base.1) * t;
            let b = base.2 + (target.2 - base.2) * t;

            sprite.color = Color::srgb(r, g, b);
        }

        let touching_tile =
            (player_pos.x - tile_transform.translation.truncate().x).abs() < (PLAYER_SIZE.x + tile.attributes().size.x) / 2.0
            && (player_pos.y - tile_transform.translation.truncate().y).abs() < (PLAYER_SIZE.y + tile.attributes().size.y) / 2.0;
        if !touching_tile {
            continue;
        }

        // Speed multiplier is the min from any tiles with multiplier
        movement.speed_multiplier = movement.speed_multiplier.min(attributes.speed_multiplier);

        // Win tile
        if *tile == Tile::Win {
            next_state.set(GameState::GameWon);
            break;
        }

        // Increment score and remove coin when contacted
        if *tile == Tile::Coin {
            score.0 += 1;
            commands.entity(entity).despawn();
            continue;
        }

        // Only damage when touching tile that does damage and cooldown is over or tile has no cooldown
        if tile.attributes().damage_on_touch > 0 && (!cooldown.active || !tile.attributes().has_damage_cooldown) {
            health.damage(tile.attributes().damage_on_touch);
            cooldown.timer.reset();
            cooldown.active = true;
        }
    }
}

fn update_game_ui(
    player: Query<&Health, With<Player>>,
    score: Res<Score>,
    mut health_text: Query<&mut Text, (With<HealthText>, Without<ScoreText>)>,
    mut score_text: Query<&mut Text, (With<ScoreText>, Without<HealthText>)>,
) {
    let Ok(health) = player.single() else {
        return;
    };

    if let Ok(mut text) = health_text.single_mut() {
        text.0 = format!(
            "Health: {} / {}",
            health.current_health,
            health.max_health,
        );
    }

    if let Ok(mut text) = score_text.single_mut() {
        text.0 = format!("Score: {}", score.0);
    }
}

fn check_game_over(mut next_state: ResMut<NextState<GameState>>, player: Query<(&Transform, &Health), With<Player>>) {
    let Ok((transform, health)) = player.single() else {
        return;
    };

    if health.player_dead() {
        next_state.set(GameState::GameLost)
    }
}

fn show_initial_message(mut commands: Commands) {
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
            Text::new("You won!"),
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
        ));
    });
}

fn show_game_lost(mut commands: Commands) {
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

fn show_game_won(mut commands: Commands) {
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
            Text::new("You won!"),
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
        ));
    });
}