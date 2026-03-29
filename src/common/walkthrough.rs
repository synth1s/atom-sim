use bevy::prelude::*;

/// Plugin para derivacao passo-a-passo animada (F3).
pub struct WalkthroughPlugin;

impl Plugin for WalkthroughPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WalkthroughState>()
            .add_systems(Update, walkthrough_system);
    }
}

/// Resource com os passos da derivacao da era atual.
#[derive(Resource, Default)]
pub struct EraDerivation(pub Vec<String>);

/// Estado do painel de derivacao.
#[derive(Resource, Default)]
pub struct WalkthroughState {
    pub visible: bool,
    pub step: usize,
}

/// Marcador para entidades do overlay de derivacao.
#[derive(Component)]
struct WalkthroughOverlay;

fn walkthrough_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<WalkthroughState>,
    derivation: Option<Res<EraDerivation>>,
    existing: Query<Entity, With<WalkthroughOverlay>>,
) {
    // Toggle com F3
    if keyboard.just_pressed(KeyCode::F3) {
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

    let Some(derivation) = derivation else {
        return;
    };

    if derivation.0.is_empty() {
        return;
    }

    // Avancar passo com Enter
    if keyboard.just_pressed(KeyCode::Enter) {
        if state.step < derivation.0.len().saturating_sub(1) {
            state.step += 1;
        }
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
    }

    // Spawnar painel se nao existe
    if existing.iter().count() == 0 {
        let current = state.step.min(derivation.0.len().saturating_sub(1));

        // Fundo escuro
        commands.spawn((
            WalkthroughOverlay,
            Sprite {
                color: Color::srgba(0.05, 0.05, 0.1, 0.93),
                custom_size: Some(Vec2::new(550.0, 300.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 87.0),
        ));

        // Header
        commands.spawn((
            WalkthroughOverlay,
            Text2d::new("DERIVACAO"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 0.8, 0.3, 1.0)),
            Transform::from_xyz(0.0, 120.0, 87.5),
            TextLayout::new_with_justify(Justify::Center),
        ));

        // Mostrar todos os passos ate o atual
        let mut y_offset = 80.0;
        for (i, step_text) in derivation.0.iter().enumerate() {
            if i > current {
                break;
            }

            let color = if i == current {
                // Passo atual em AMARELO
                Color::srgba(1.0, 1.0, 0.3, 1.0)
            } else {
                // Anteriores em cinza
                Color::srgba(0.6, 0.6, 0.6, 0.7)
            };

            let label = format!("{}. {}", i + 1, step_text);
            commands.spawn((
                WalkthroughOverlay,
                Text2d::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(color),
                Transform::from_xyz(0.0, y_offset, 87.5),
                TextLayout::new_with_justify(Justify::Center),
            ));

            y_offset -= 30.0;
        }

        // Navegacao
        let nav = if current >= derivation.0.len().saturating_sub(1) {
            format!(
                "[{}/{}]  [F3] Fechar",
                current + 1,
                derivation.0.len()
            )
        } else {
            format!(
                "[{}/{}]  [Enter] Proximo passo  |  [F3] Fechar",
                current + 1,
                derivation.0.len()
            )
        };

        commands.spawn((
            WalkthroughOverlay,
            Text2d::new(nav),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
            Transform::from_xyz(0.0, -130.0, 87.5),
            TextLayout::new_with_justify(Justify::Center),
        ));
    }
}
