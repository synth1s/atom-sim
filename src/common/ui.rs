use bevy::prelude::*;
use super::{ActiveEra, SimulationState};
use super::transition::{TransitionState, TransitionPhase};
use super::auto_tour::AutoTourState;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EraControls>()
            .init_resource::<LimitationText>()
            .init_resource::<LimitationVisible>()
            .add_systems(Startup, spawn_hud)
            .add_systems(Update, (update_status_hud, update_controls_hud, toggle_pause, switch_era, toggle_limitation));
    }
}

/// Resource holding the limitation text for the current era.
#[derive(Resource, Default)]
pub struct LimitationText(pub String, pub String);

/// Resource controlling visibility of the limitation overlay.
#[derive(Resource, Default)]
pub struct LimitationVisible(pub bool);

/// Marker for the limitation overlay entities.
#[derive(Component)]
struct LimitationOverlay;

/// Marcador para o texto principal do HUD (contexto histórico).
#[derive(Component)]
pub struct HudText;

/// Marcador para o texto de status (pausado/rodando).
#[derive(Component)]
struct StatusText;

/// Marcador para o texto de controles.
#[derive(Component)]
struct ControlsText;

/// Resource com controles específicos da era atual.
#[derive(Resource, Default)]
pub struct EraControls(pub String);

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
             [1-9] Trocar era\n\
             [X] Exportar CSV\n\
             [L] Limitacoes"
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

fn update_controls_hud(
    era_controls: Res<EraControls>,
    mut query: Query<&mut Text2d, With<ControlsText>>,
) {
    if !era_controls.is_changed() { return; }

    let base = "CONTROLES:\n\
                [Espaco] Pausar/Retomar\n\
                [Scroll] Zoom\n\
                [1-9] Trocar era\n\
                [[ ]] Era anterior/proxima\n\
                [X] Exportar CSV\n\
                [L] Limitacoes\n\
                [E] Equacoes\n\
                [Q] Quiz\n\
                [P] Parametros\n\
                [S] Snapshot\n\
                [F5] Auto-Tour\n\
                [G] Modo Guiado\n\
                [H] Evidencias\n\
                [N] Quanticos\n\
                [F1] Predicao\n\
                [F2] Experimento\n\
                [F3] Derivacao";

    let full = if era_controls.0.is_empty() {
        base.to_string()
    } else {
        format!("{}\n{}", base, era_controls.0)
    };

    for mut text in query.iter_mut() {
        *text = Text2d::new(full.clone());
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
    mut transition: ResMut<TransitionState>,
    tour: Res<AutoTourState>,
) {
    if transition.active || tour.active {
        return;
    }

    let target = if keyboard.just_pressed(KeyCode::Digit1) {
        Some(ActiveEra::Democritus)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(ActiveEra::Dalton)
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(ActiveEra::Thomson)
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        Some(ActiveEra::Rutherford)
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        Some(ActiveEra::Bohr)
    } else if keyboard.just_pressed(KeyCode::Digit6) {
        Some(ActiveEra::Sommerfeld)
    } else if keyboard.just_pressed(KeyCode::Digit7) {
        Some(ActiveEra::DeBroglie)
    } else if keyboard.just_pressed(KeyCode::Digit8) {
        Some(ActiveEra::Schrodinger)
    } else if keyboard.just_pressed(KeyCode::Digit9) {
        Some(ActiveEra::Dirac)
    } else {
        None
    };

    if let Some(era) = target {
        transition.active = true;
        transition.timer = 0.0;
        transition.phase = TransitionPhase::FadeOut;
        transition.target_era = era;
    }
}

fn toggle_limitation(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<LimitationVisible>,
    limitation_text: Res<LimitationText>,
    existing: Query<Entity, With<LimitationOverlay>>,
) {
    if keyboard.just_pressed(KeyCode::KeyL) {
        visible.0 = !visible.0;
    }

    if visible.0 {
        // Only spawn if not already present
        if existing.iter().count() == 0 && !limitation_text.0.is_empty() {
            // Dark background overlay
            commands.spawn((
                LimitationOverlay,
                Mesh2d(bevy::prelude::Handle::default()),
                Transform::from_xyz(0.0, 0.0, 90.0),
            ));

            // Title
            commands.spawn((
                LimitationOverlay,
                Text2d::new(limitation_text.0.clone()),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.8, 0.3, 1.0)),
                Transform::from_xyz(0.0, 120.0, 91.0),
                bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
            ));

            // Body text
            commands.spawn((
                LimitationOverlay,
                Text2d::new(limitation_text.1.clone()),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgba(0.9, 0.9, 0.8, 0.95)),
                Transform::from_xyz(0.0, 0.0, 91.0),
                bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
            ));
        }
    } else {
        // Despawn all overlay entities
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
    }
}
