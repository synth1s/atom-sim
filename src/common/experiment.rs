use bevy::prelude::*;

/// Plugin para replay de experimento interativo (F2).
pub struct ExperimentPlugin;

impl Plugin for ExperimentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExperimentState>()
            .add_systems(Update, experiment_system);
    }
}

/// Um passo do experimento: descricao + acao.
pub struct ExperimentStep {
    pub description: String,
    pub action: String,
}

/// Resource com os dados do experimento da era atual.
#[derive(Resource, Default)]
pub struct EraExperiment {
    pub name: String,
    pub year: String,
    pub apparatus: String,
    pub steps: Vec<ExperimentStep>,
}

/// Estado do painel de experimento.
#[derive(Resource, Default)]
pub struct ExperimentState {
    pub visible: bool,
    pub step: usize,
}

/// Marcador para entidades do overlay de experimento.
#[derive(Component)]
struct ExperimentOverlay;

fn experiment_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ExperimentState>,
    experiment: Option<Res<EraExperiment>>,
    existing: Query<Entity, With<ExperimentOverlay>>,
) {
    // Toggle com F2
    if keyboard.just_pressed(KeyCode::F2) {
        state.visible = !state.visible;
        if state.visible {
            state.step = 0;
        } else {
            for entity in existing.iter() {
                commands.entity(entity).despawn();
            }
            return;
        }
    }

    if !state.visible {
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let Some(experiment) = experiment else {
        return;
    };

    if experiment.steps.is_empty() {
        return;
    }

    // Avancar passo com Enter
    if keyboard.just_pressed(KeyCode::Enter) {
        if state.step < experiment.steps.len().saturating_sub(1) {
            state.step += 1;
        }
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
    }

    // Spawnar painel se nao existe
    if existing.iter().count() == 0 {
        let current = state.step.min(experiment.steps.len().saturating_sub(1));
        let step = &experiment.steps[current];

        let display_text = format!(
            "EXPERIMENTO: {} ({})\nAparato: {}\n\nPasso {}: {}\nAcao: {}",
            experiment.name,
            experiment.year,
            experiment.apparatus,
            current + 1,
            step.description,
            step.action
        );

        // Fundo escuro
        commands.spawn((
            ExperimentOverlay,
            Sprite {
                color: Color::srgba(0.05, 0.05, 0.1, 0.93),
                custom_size: Some(Vec2::new(650.0, 250.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 86.0),
        ));

        // Texto do experimento
        commands.spawn((
            ExperimentOverlay,
            Text2d::new(display_text),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 0.95, 0.7, 0.95)),
            Transform::from_xyz(0.0, 20.0, 86.5),
            TextLayout::new_with_justify(Justify::Center),
        ));

        // Navegacao
        let nav = if current >= experiment.steps.len().saturating_sub(1) {
            format!("[{}/{}]  [F2] Fechar", current + 1, experiment.steps.len())
        } else {
            format!(
                "[{}/{}]  [Enter] Proximo passo  |  [F2] Fechar",
                current + 1,
                experiment.steps.len()
            )
        };

        commands.spawn((
            ExperimentOverlay,
            Text2d::new(nav),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
            Transform::from_xyz(0.0, -100.0, 86.5),
            TextLayout::new_with_justify(Justify::Center),
        ));
    }
}
