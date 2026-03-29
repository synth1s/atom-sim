use bevy::prelude::*;
use super::ActiveEra;
use super::transition::{TransitionState, TransitionPhase};

pub struct AutoTourPlugin;

impl Plugin for AutoTourPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutoTourState>()
            .add_systems(Update, (tour_controls, auto_tour_system).chain());
    }
}

/// Duration each era is displayed during the tour (seconds).
const ERA_DURATION: f32 = 12.0;

/// All 9 eras in chronological order.
const TOUR_ERAS: [ActiveEra; 9] = [
    ActiveEra::Democritus,
    ActiveEra::Dalton,
    ActiveEra::Thomson,
    ActiveEra::Rutherford,
    ActiveEra::Bohr,
    ActiveEra::Sommerfeld,
    ActiveEra::DeBroglie,
    ActiveEra::Schrodinger,
    ActiveEra::Dirac,
];

/// Resource controlling the auto-tour state.
#[derive(Resource)]
pub struct AutoTourState {
    pub active: bool,
    era_index: usize,
    era_timer: f32,
}

impl Default for AutoTourState {
    fn default() -> Self {
        Self {
            active: false,
            era_index: 0,
            era_timer: 0.0,
        }
    }
}

/// Marker for the tour indicator text.
#[derive(Component)]
struct TourIndicator;

fn tour_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut tour: ResMut<AutoTourState>,
    mut transition: ResMut<TransitionState>,
    mut commands: Commands,
    indicator_query: Query<Entity, With<TourIndicator>>,
) {
    if keyboard.just_pressed(KeyCode::F5) {
        if tour.active {
            // Deactivate tour
            tour.active = false;
            for entity in indicator_query.iter() {
                commands.entity(entity).despawn();
            }
        } else {
            // Activate tour
            tour.active = true;
            tour.era_index = 0;
            tour.era_timer = 0.0;
            // Navigate to first era
            transition.active = true;
            transition.timer = 0.0;
            transition.phase = TransitionPhase::FadeOut;
            transition.target_era = TOUR_ERAS[0].clone();
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) && tour.active {
        tour.active = false;
        for entity in indicator_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn auto_tour_system(
    time: Res<Time>,
    mut tour: ResMut<AutoTourState>,
    mut transition: ResMut<TransitionState>,
    mut commands: Commands,
    indicator_query: Query<Entity, With<TourIndicator>>,
) {
    if !tour.active {
        return;
    }

    // Spawn indicator if not present
    if indicator_query.iter().count() == 0 {
        commands.spawn((
            TourIndicator,
            Text2d::new("TOUR [ESC para sair]"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 0.2, 0.95)),
            Transform::from_xyz(450.0, 330.0, 90.0),
        ));
    }

    // Don't advance timer while a transition is in progress
    if transition.active {
        return;
    }

    tour.era_timer += time.delta_secs();

    if tour.era_timer >= ERA_DURATION {
        tour.era_timer = 0.0;
        tour.era_index += 1;

        if tour.era_index >= TOUR_ERAS.len() {
            // Tour complete
            tour.active = false;
            for entity in indicator_query.iter() {
                commands.entity(entity).despawn();
            }
            return;
        }

        // Transition to next era
        transition.active = true;
        transition.timer = 0.0;
        transition.phase = TransitionPhase::FadeOut;
        transition.target_era = TOUR_ERAS[tour.era_index].clone();
    }
}
