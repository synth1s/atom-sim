use bevy::prelude::*;
use super::ActiveEra;

pub struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransitionState>()
            .add_systems(Startup, spawn_overlay)
            .add_systems(Update, transition_system);
    }
}

/// Phase of the fade transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionPhase {
    FadeOut,
    FadeIn,
}

/// Resource controlling the animated era transition.
#[derive(Resource)]
pub struct TransitionState {
    pub active: bool,
    pub timer: f32,
    pub phase: TransitionPhase,
    pub target_era: ActiveEra,
}

impl Default for TransitionState {
    fn default() -> Self {
        Self {
            active: false,
            timer: 0.0,
            phase: TransitionPhase::FadeOut,
            target_era: ActiveEra::Democritus,
        }
    }
}

/// Marker for the fullscreen black overlay used during transitions.
#[derive(Component)]
struct TransitionOverlay;

const FADE_DURATION: f32 = 0.3;

fn spawn_overlay(mut commands: Commands) {
    commands.spawn((
        TransitionOverlay,
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(1400.0, 800.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 80.0),
    ));
}

fn transition_system(
    time: Res<Time>,
    mut state: ResMut<TransitionState>,
    mut next_era: ResMut<NextState<ActiveEra>>,
    mut overlay_query: Query<&mut Sprite, With<TransitionOverlay>>,
) {
    if !state.active {
        return;
    }

    state.timer += time.delta_secs();

    let Ok(mut sprite) = overlay_query.single_mut() else {
        return;
    };

    match state.phase {
        TransitionPhase::FadeOut => {
            let alpha = (state.timer / FADE_DURATION).min(1.0);
            sprite.color = Color::srgba(0.0, 0.0, 0.0, alpha);

            if state.timer >= FADE_DURATION {
                next_era.set(state.target_era.clone());
                state.phase = TransitionPhase::FadeIn;
                state.timer = 0.0;
            }
        }
        TransitionPhase::FadeIn => {
            let alpha = 1.0 - (state.timer / FADE_DURATION).min(1.0);
            sprite.color = Color::srgba(0.0, 0.0, 0.0, alpha);

            if state.timer >= FADE_DURATION {
                sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
                state.active = false;
            }
        }
    }
}
