use bevy::prelude::*;

pub struct EvidencePlugin;

impl Plugin for EvidencePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EraEvidence>()
            .init_resource::<EvidenceVisible>()
            .add_systems(Update, toggle_evidence);
    }
}

/// Resource holding the evidence data for the current era.
/// `resolves`: what this model explains. `fails`: what it cannot explain.
#[derive(Resource, Default)]
pub struct EraEvidence {
    pub resolves: String,
    pub fails: String,
}

/// Resource controlling visibility of the evidence overlay.
#[derive(Resource, Default)]
pub struct EvidenceVisible(pub bool);

/// Marker for the evidence overlay entities.
#[derive(Component)]
struct EvidenceOverlay;

fn toggle_evidence(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<EvidenceVisible>,
    evidence: Res<EraEvidence>,
    existing: Query<Entity, With<EvidenceOverlay>>,
) {
    if keyboard.just_pressed(KeyCode::KeyH) {
        visible.0 = !visible.0;
    }

    if visible.0 {
        // Only spawn if not already present
        if existing.iter().count() == 0 && !evidence.resolves.is_empty() {
            // Dark background rectangle
            commands.spawn((
                EvidenceOverlay,
                Sprite {
                    color: Color::srgba(0.05, 0.05, 0.1, 0.92),
                    custom_size: Some(Vec2::new(600.0, 200.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 84.0),
            ));

            // Title — RESOLVE (green, left column)
            commands.spawn((
                EvidenceOverlay,
                Text2d::new("RESOLVE"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgba(0.3, 1.0, 0.3, 1.0)),
                Transform::from_xyz(-140.0, 80.0, 85.0),
                bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
            ));

            // Resolve body text (green, left column)
            commands.spawn((
                EvidenceOverlay,
                Text2d::new(evidence.resolves.clone()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgba(0.5, 0.9, 0.5, 0.95)),
                Transform::from_xyz(-140.0, 20.0, 85.0),
                bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
            ));

            // Title — NAO EXPLICA (red, right column)
            commands.spawn((
                EvidenceOverlay,
                Text2d::new("NAO EXPLICA"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.3, 0.3, 1.0)),
                Transform::from_xyz(140.0, 80.0, 85.0),
                bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
            ));

            // Fails body text (red, right column)
            commands.spawn((
                EvidenceOverlay,
                Text2d::new(evidence.fails.clone()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgba(0.9, 0.5, 0.5, 0.95)),
                Transform::from_xyz(140.0, 20.0, 85.0),
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
