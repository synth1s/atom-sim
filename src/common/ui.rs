use bevy::prelude::*;
use super::{ActiveEra, SimulationState};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, (update_status_hud, toggle_pause, switch_era));
    }
}

/// Marcador para o texto principal do HUD (contexto histórico).
#[derive(Component)]
pub struct HudText;

/// Marcador para o texto de status (pausado/rodando).
#[derive(Component)]
struct StatusText;

/// Marcador para o texto de controles.
#[derive(Component)]
struct ControlsText;

fn spawn_hud(mut commands: Commands) {
    // Título e contexto histórico (topo esquerdo) — será atualizado por cada era
    commands.spawn((
        HudText,
        Text2d::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgba(0.9, 0.9, 0.7, 0.9)),
        Transform::from_xyz(-380.0, 320.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));

    // Status da simulação (topo direito)
    commands.spawn((
        StatusText,
        Text2d::new("[RODANDO]"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgba(0.4, 1.0, 0.4, 0.9)),
        Transform::from_xyz(500.0, 340.0, 10.0),
    ));

    // Controles (canto inferior esquerdo)
    commands.spawn((
        ControlsText,
        Text2d::new(
            "CONTROLES:\n\
             [Espaco] Pausar/Retomar\n\
             [Clique] Adicionar atomo\n\
             [Scroll] Zoom\n\
             [1-8] Trocar era"
        ),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
        Transform::from_xyz(-500.0, -280.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));
}

fn update_status_hud(
    state: Res<State<SimulationState>>,
    mut query: Query<(&mut Text2d, &mut TextColor), With<StatusText>>,
) {
    for (mut text, mut color) in query.iter_mut() {
        match state.get() {
            SimulationState::Running => {
                *text = Text2d::new("[RODANDO]");
                *color = TextColor(Color::srgba(0.4, 1.0, 0.4, 0.9));
            }
            SimulationState::Paused => {
                *text = Text2d::new("[PAUSADO]");
                *color = TextColor(Color::srgba(1.0, 0.4, 0.4, 0.9));
            }
        }
    }
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<SimulationState>>,
    mut next_state: ResMut<NextState<SimulationState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        match state.get() {
            SimulationState::Running => next_state.set(SimulationState::Paused),
            SimulationState::Paused => next_state.set(SimulationState::Running),
        }
    }
}

fn switch_era(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_era: ResMut<NextState<ActiveEra>>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        next_era.set(ActiveEra::Democritus);
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        next_era.set(ActiveEra::Dalton);
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        next_era.set(ActiveEra::Thomson);
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        next_era.set(ActiveEra::Rutherford);
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        next_era.set(ActiveEra::Bohr);
    } else if keyboard.just_pressed(KeyCode::Digit6) {
        next_era.set(ActiveEra::Sommerfeld);
    } else if keyboard.just_pressed(KeyCode::Digit7) {
        next_era.set(ActiveEra::DeBroglie);
    } else if keyboard.just_pressed(KeyCode::Digit8) {
        next_era.set(ActiveEra::Schrodinger);
    }
}
