use bevy::prelude::*;
use super::SimulationState;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, (update_hud, toggle_pause));
    }
}

/// Marcador para o texto principal do HUD.
#[derive(Component)]
pub struct HudText;

/// Marcador para o texto de status (pausado/rodando).
#[derive(Component)]
struct StatusText;

/// Marcador para o texto de controles.
#[derive(Component)]
struct ControlsText;

fn spawn_hud(mut commands: Commands) {
    // Título e contexto histórico (topo esquerdo)
    commands.spawn((
        HudText,
        Text2d::new(
            "DEMOCRITO (~400 a.C.) \u{2014} Atomos indivisiveis no vazio\n\n\
             \"Nada existe exceto atomos e vazio; tudo o mais e opiniao.\"\n\n\
             Particulas identicas e eternas movendo-se no vazio,\n\
             colidindo mecanicamente. Sem forcas a distancia.\n\n\
             LIMITACAO: Todos os atomos sao identicos.\n\
             Como explicar a diversidade da materia?"
        ),
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
             [Scroll] Zoom"
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

fn update_hud(
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
