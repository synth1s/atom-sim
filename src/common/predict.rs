use bevy::prelude::*;

pub struct PredictPlugin;

impl Plugin for PredictPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EraPredictions>()
            .init_resource::<PredictState>()
            .add_systems(Update, predict_system);
    }
}

/// A single prediction challenge: question, options, correct answer index, and explanation.
pub struct Prediction {
    pub question: String,
    pub options: Vec<String>,
    pub correct: usize,
    pub explanation: String,
}

/// Resource holding the predictions for the current era.
#[derive(Resource, Default)]
pub struct EraPredictions(pub Vec<Prediction>);

/// Phase of the predict-observe-explain cycle.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum PredictPhase {
    #[default]
    Predicting,
    Revealed,
}

/// Resource tracking prediction interaction state.
#[derive(Resource, Default)]
pub struct PredictState {
    pub visible: bool,
    pub current: usize,
    pub phase: PredictPhase,
    pub score: usize,
    pub total: usize,
    selected: Option<usize>,
    finished: bool,
}

/// Marker for prediction overlay entities.
#[derive(Component)]
struct PredictOverlay;

fn predict_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<PredictState>,
    predictions: Res<EraPredictions>,
    existing: Query<Entity, With<PredictOverlay>>,
) {
    // Toggle with F1 (only when not in revealed phase)
    if keyboard.just_pressed(KeyCode::F1) && state.phase != PredictPhase::Revealed {
        state.visible = !state.visible;
        if state.visible {
            state.current = 0;
            state.phase = PredictPhase::Predicting;
            state.selected = None;
            state.score = 0;
            state.total = predictions.0.len();
            state.finished = false;
        }
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
        if !state.visible {
            return;
        }
    }

    if !state.visible || predictions.0.is_empty() {
        return;
    }

    // Phase: Predicting — select answer with 1-4
    if state.phase == PredictPhase::Predicting && !state.finished {
        let selection = if keyboard.just_pressed(KeyCode::Digit1) {
            Some(0)
        } else if keyboard.just_pressed(KeyCode::Digit2) {
            Some(1)
        } else if keyboard.just_pressed(KeyCode::Digit3) {
            Some(2)
        } else if keyboard.just_pressed(KeyCode::Digit4) {
            Some(3)
        } else {
            None
        };

        if let Some(sel) = selection {
            if sel < predictions.0[state.current].options.len() {
                state.selected = Some(sel);
                state.phase = PredictPhase::Revealed;
                if sel == predictions.0[state.current].correct {
                    state.score += 1;
                }
                for entity in existing.iter() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    // Phase: Revealed — Enter to advance
    if state.phase == PredictPhase::Revealed && !state.finished && keyboard.just_pressed(KeyCode::Enter) {
        state.phase = PredictPhase::Predicting;
        state.selected = None;
        state.current += 1;
        if state.current >= state.total {
            state.finished = true;
        }
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
    }

    // Close after finishing with F1
    if state.finished && keyboard.just_pressed(KeyCode::F1) {
        state.visible = false;
        state.finished = false;
        state.phase = PredictPhase::Predicting;
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Only spawn overlay if nothing is present
    if existing.iter().count() > 0 {
        return;
    }

    // Dark background
    commands.spawn((
        PredictOverlay,
        Sprite {
            color: Color::srgba(0.03, 0.05, 0.12, 0.95),
            custom_size: Some(Vec2::new(600.0, 360.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 92.0),
    ));

    if state.finished {
        let result_text = format!(
            "PREDICAO COMPLETA!\n\nPontuacao: {}/{}\n\n[F1] Fechar",
            state.score, state.total
        );
        commands.spawn((
            PredictOverlay,
            Text2d::new(result_text),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgba(0.9, 0.95, 0.5, 1.0)),
            Transform::from_xyz(0.0, 0.0, 93.0),
            bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
        ));
        return;
    }

    let p = &predictions.0[state.current];

    // Header
    let header = format!("PREDICAO ({}/{})", state.current + 1, state.total);
    commands.spawn((
        PredictOverlay,
        Text2d::new(header),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgba(0.3, 0.85, 1.0, 1.0)),
        Transform::from_xyz(0.0, 150.0, 93.0),
        bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
    ));

    // Question text
    commands.spawn((
        PredictOverlay,
        Text2d::new(p.question.clone()),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::srgba(0.95, 0.95, 0.9, 1.0)),
        Transform::from_xyz(0.0, 100.0, 93.0),
        bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
    ));

    // Options
    let option_labels = ["A)", "B)", "C)", "D)"];
    for (i, opt) in p.options.iter().enumerate() {
        let color = if state.phase == PredictPhase::Revealed {
            if i == p.correct {
                Color::srgba(0.3, 1.0, 0.3, 1.0)
            } else if state.selected == Some(i) {
                Color::srgba(1.0, 0.3, 0.3, 1.0)
            } else {
                Color::srgba(0.5, 0.5, 0.5, 0.7)
            }
        } else {
            Color::srgba(0.8, 0.85, 0.9, 0.95)
        };

        let label = format!("{} {}", option_labels[i], opt);
        commands.spawn((
            PredictOverlay,
            Text2d::new(label),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(color),
            Transform::from_xyz(-200.0, 50.0 - i as f32 * 28.0, 93.0),
            bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Left),
        ));
    }

    // Revealed phase: show explanation
    if state.phase == PredictPhase::Revealed {
        let feedback = if state.selected == Some(p.correct) {
            "CORRETO!"
        } else {
            "INCORRETO"
        };
        let fb_color = if state.selected == Some(p.correct) {
            Color::srgba(0.3, 1.0, 0.3, 1.0)
        } else {
            Color::srgba(1.0, 0.3, 0.3, 1.0)
        };

        commands.spawn((
            PredictOverlay,
            Text2d::new(feedback.to_string()),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(fb_color),
            Transform::from_xyz(0.0, -50.0, 93.0),
            bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
        ));

        // Explanation
        commands.spawn((
            PredictOverlay,
            Text2d::new(p.explanation.clone()),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgba(0.8, 0.9, 1.0, 0.9)),
            Transform::from_xyz(0.0, -85.0, 93.0),
            bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
        ));

        commands.spawn((
            PredictOverlay,
            Text2d::new(format!("Score: {}/{}  [Enter] Proximo", state.score, state.total)),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
            Transform::from_xyz(0.0, -120.0, 93.0),
            bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
        ));
    } else {
        commands.spawn((
            PredictOverlay,
            Text2d::new("[1-4] Faca sua predicao"),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgba(0.6, 0.6, 0.6, 0.7)),
            Transform::from_xyz(0.0, -80.0, 93.0),
            bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predict_state_defaults() {
        let state = PredictState::default();
        assert!(!state.visible);
        assert_eq!(state.current, 0);
        assert_eq!(state.phase, PredictPhase::Predicting);
        assert_eq!(state.score, 0);
        assert_eq!(state.total, 0);
        assert!(state.selected.is_none());
        assert!(!state.finished);
    }

    #[test]
    fn era_predictions_default_is_empty() {
        let preds = EraPredictions::default();
        assert!(preds.0.is_empty());
    }

    #[test]
    fn prediction_struct_fields() {
        let pred = Prediction {
            question: "Test?".to_string(),
            options: vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()],
            correct: 2,
            explanation: "Because C".to_string(),
        };
        assert_eq!(pred.correct, 2);
        assert_eq!(pred.options.len(), 4);
    }
}
