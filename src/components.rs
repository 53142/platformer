use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct CurrentLevel(pub(crate) usize);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub(crate) enum GameState {
    #[default]
    InGame,
    GameWon,
    GameLost,
}

#[derive(Component)]
pub(crate) struct Player;

#[derive(Component)]
pub(crate) struct Movement {
    pub(crate) velocity: Vec2, // Velocity without speed multiplied
    pub(crate) speed_multiplier: f32,
    pub(crate) number_of_jumps: u8,
    pub(crate) coyote_time_remaining: f32,
}

#[derive(Component)]
pub(crate) struct Health {
    pub(crate) max_health: u32,
    pub(crate) current_health: u32,
}

impl Health {
    pub(crate) fn new(max_health: u32) -> Self {
        Self {
            max_health: max_health,
            current_health: max_health,
        }
    }

    pub(crate) fn damage(&mut self, amount: u32) {
        self.current_health = self.current_health.saturating_sub(amount);
    }

    pub(crate) fn player_dead(&self) -> bool {
        self.current_health == 0
    }
}

#[derive(Component)]
pub(crate) struct HealthText;

#[derive(Component)]
pub(crate) struct ScoreText;

#[derive(Resource, Default)]
pub(crate) struct Score(pub(crate) u32);

#[derive(Component)]
pub(crate) struct InitialMessage(pub(crate) Timer);

#[derive(Component)]
pub(crate) struct DamageCooldown {
    pub(crate) timer: Timer,
    pub(crate) active: bool,
}

impl DamageCooldown {
    pub(crate) fn new(seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
            active: false,
        }
    }
}

#[derive(Component)]
pub(crate) struct DamageFlash {
    pub(crate) timer: Timer,
    pub(crate) active: bool,
}

impl DamageFlash {
    pub(crate) fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            active: false,
        }
    }
}

#[derive(Component)]
pub(crate) struct GameEntity;

#[derive(Component)]
pub(crate) struct EndScreenRoot;

#[derive(Component)]
pub(crate) struct RestartButton;

#[derive(Resource, Default)]
pub(crate) struct GameTimer(pub(crate) f32);

#[derive(Component)]
pub(crate) struct TimerText;