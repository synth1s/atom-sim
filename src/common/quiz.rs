use bevy::prelude::*;

pub struct QuizPlugin;

impl Plugin for QuizPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EraQuiz>()
            .init_resource::<QuizState>()
            .add_systems(Update, quiz_system);
    }
}

/// Resource holding the quiz questions for the current era.
#[derive(Resource, Default)]
pub struct EraQuiz(pub Vec<QuestionData>);

/// Public question data that eras can construct.
pub struct QuestionData {
    pub text: String,
    pub options: Vec<String>,
    pub correct: usize,
}

/// Resource tracking quiz interaction state.
#[derive(Resource, Default)]
pub struct QuizState {
    visible: bool,
    current: usize,
    answered: bool,
    selected: Option<usize>,
    score: usize,
    total: usize,
    finished: bool,
}

/// Marker for the quiz overlay entities.
#[derive(Component)]
struct QuizOverlay;

fn quiz_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<QuizState>,
    quiz: Res<EraQuiz>,
    existing: Query<Entity, With<QuizOverlay>>,
) {
    // Toggle quiz with Q (only when not in the middle of answering)
    if keyboard.just_pressed(KeyCode::KeyQ) && !state.answered {
        state.visible = !state.visible;
        if state.visible {
            // Reset quiz state on open
            state.current = 0;
            state.answered = false;
            state.selected = None;
            state.score = 0;
            state.total = quiz.0.len();
            state.finished = false;
        }
        // Despawn existing overlay
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
        if !state.visible {
            return;
        }
    }

    if !state.visible || quiz.0.is_empty() {
        return;
    }

    // Handle answer selection (keys 1-4)
    if !state.answered && !state.finished {
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
            if sel < quiz.0[state.current].options.len() {
                state.selected = Some(sel);
                state.answered = true;
                if sel == quiz.0[state.current].correct {
                    state.score += 1;
                }
                // Force redraw
                for entity in existing.iter() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    // Handle Enter to go to next question
    if state.answered && !state.finished && keyboard.just_pressed(KeyCode::Enter) {
        state.answered = false;
        state.selected = None;
        state.current += 1;
        if state.current >= state.total {
            state.finished = true;
        }
        // Force redraw
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
    }

    // Handle Q to close after finishing
    if state.finished && keyboard.just_pressed(KeyCode::KeyQ) {
        state.visible = false;
        state.finished = false;
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
        QuizOverlay,
        Sprite {
            color: Color::srgba(0.05, 0.05, 0.12, 0.94),
            custom_size: Some(Vec2::new(550.0, 300.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 87.0),
    ));

    if state.finished {
        // Show final score
        let result_text = format!(
            "QUIZ COMPLETO!\n\nPontuacao: {}/{}\n\n[Q] Fechar",
            state.score, state.total
        );
        commands.spawn((
            QuizOverlay,
            Text2d::new(result_text),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgba(0.9, 0.95, 0.5, 1.0)),
            Transform::from_xyz(0.0, 0.0, 88.0),
            bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
        ));
        return;
    }

    let q = &quiz.0[state.current];

    // Question header
    let header = format!("QUIZ ({}/{})", state.current + 1, state.total);
    commands.spawn((
        QuizOverlay,
        Text2d::new(header),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 0.85, 0.3, 1.0)),
        Transform::from_xyz(0.0, 120.0, 88.0),
        bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
    ));

    // Question text
    commands.spawn((
        QuizOverlay,
        Text2d::new(q.text.clone()),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::srgba(0.95, 0.95, 0.9, 1.0)),
        Transform::from_xyz(0.0, 70.0, 88.0),
        bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
    ));

    // Options
    let option_labels = ["1)", "2)", "3)", "4)"];
    for (i, opt) in q.options.iter().enumerate() {
        let color = if state.answered {
            if i == q.correct {
                Color::srgba(0.3, 1.0, 0.3, 1.0) // green for correct
            } else if state.selected == Some(i) {
                Color::srgba(1.0, 0.3, 0.3, 1.0) // red for wrong selection
            } else {
                Color::srgba(0.6, 0.6, 0.6, 0.8)
            }
        } else {
            Color::srgba(0.8, 0.85, 0.9, 0.95)
        };

        let label = format!("{} {}", option_labels[i], opt);
        commands.spawn((
            QuizOverlay,
            Text2d::new(label),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(color),
            Transform::from_xyz(-180.0, 30.0 - i as f32 * 25.0, 88.0),
            bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Left),
        ));
    }

    // Show result feedback if answered
    if state.answered {
        let feedback = if state.selected == Some(q.correct) {
            "CORRETO!"
        } else {
            "INCORRETO"
        };
        let fb_color = if state.selected == Some(q.correct) {
            Color::srgba(0.3, 1.0, 0.3, 1.0)
        } else {
            Color::srgba(1.0, 0.3, 0.3, 1.0)
        };

        commands.spawn((
            QuizOverlay,
            Text2d::new(format!("{}  (Score: {}/{})\n[Enter] Proximo", feedback, state.score, state.total)),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(fb_color),
            Transform::from_xyz(0.0, -90.0, 88.0),
            bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
        ));
    } else {
        commands.spawn((
            QuizOverlay,
            Text2d::new("[1-4] Selecionar resposta"),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgba(0.6, 0.6, 0.6, 0.7)),
            Transform::from_xyz(0.0, -100.0, 88.0),
            bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
        ));
    }
}
