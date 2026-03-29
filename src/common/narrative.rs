use bevy::prelude::*;

/// Plugin para modo guiado com narrativa passo-a-passo.
pub struct NarrativePlugin;

impl Plugin for NarrativePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NarrativeState>()
            .add_systems(Update, narrative_system);
    }
}

/// Um passo da narrativa: texto explicativo + dica de acao.
pub struct NarrativeStep {
    pub text: String,
    pub action_hint: String,
}

/// Resource com os passos narrativos da era atual.
#[derive(Resource, Default)]
pub struct EraNarrative(pub Vec<NarrativeStep>);

/// Estado do modo guiado.
#[derive(Resource, Default)]
pub struct NarrativeState {
    pub active: bool,
    pub step: usize,
}

/// Marcador para entidades do overlay de narrativa.
#[derive(Component)]
struct NarrativeOverlay;

/// Marcador para o indicador "GUIA" no canto.
#[derive(Component)]
struct NarrativeIndicator;

fn narrative_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NarrativeState>,
    narrative: Option<Res<EraNarrative>>,
    existing: Query<Entity, With<NarrativeOverlay>>,
    indicator: Query<Entity, With<NarrativeIndicator>>,
) {
    // Toggle modo guiado com G
    if keyboard.just_pressed(KeyCode::KeyG) {
        state.active = !state.active;
        if state.active {
            state.step = 0;
        } else {
            // Despawnar overlay e indicador
            for entity in existing.iter() {
                commands.entity(entity).despawn();
            }
            for entity in indicator.iter() {
                commands.entity(entity).despawn();
            }
            return;
        }
    }

    if !state.active {
        // Garantir que nao ha overlay remanescente
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
        for entity in indicator.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let Some(narrative) = narrative else {
        return;
    };

    // Avancar passo com Enter
    if keyboard.just_pressed(KeyCode::Enter) {
        if state.step < narrative.0.len().saturating_sub(1) {
            state.step += 1;
        }
        // Despawnar overlay antigo para redesenhar
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
    }

    // Spawnar indicador "GUIA" se nao existe
    if indicator.iter().count() == 0 {
        commands.spawn((
            NarrativeIndicator,
            Text2d::new("GUIA"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgba(0.2, 1.0, 0.4, 0.95)),
            Transform::from_xyz(540.0, 310.0, 89.0),
        ));
    }

    // Spawnar painel inferior se nao existe
    if existing.iter().count() == 0 {
        let steps = &narrative.0;
        if steps.is_empty() {
            return;
        }

        let current_step = state.step.min(steps.len().saturating_sub(1));

        let display_text = if current_step < steps.len() {
            let step = &steps[current_step];
            format!(
                "[{}/{}] {}\n    >> {}",
                current_step + 1,
                steps.len(),
                step.text,
                step.action_hint
            )
        } else {
            "Avance para proxima era [->]".to_string()
        };

        // Fundo do painel (retangulo escuro)
        commands.spawn((
            NarrativeOverlay,
            Mesh2d(Handle::default()),
            Transform::from_xyz(0.0, -260.0, 88.0),
        ));

        // Texto do passo atual
        commands.spawn((
            NarrativeOverlay,
            Text2d::new(display_text),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 0.95, 0.7, 0.95)),
            Transform::from_xyz(0.0, -260.0, 89.0),
            TextLayout::new_with_justify(Justify::Center),
        ));

        // Hint de navegacao
        let nav_hint = if current_step >= steps.len().saturating_sub(1) {
            "Avance para proxima era [->]".to_string()
        } else {
            "[Enter] Proximo passo  |  [G] Fechar guia".to_string()
        };

        commands.spawn((
            NarrativeOverlay,
            Text2d::new(nav_hint),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
            Transform::from_xyz(0.0, -290.0, 89.0),
            TextLayout::new_with_justify(Justify::Center),
        ));
    }
}
