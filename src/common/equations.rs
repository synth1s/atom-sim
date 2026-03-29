use bevy::prelude::*;

pub struct EquationsPlugin;

impl Plugin for EquationsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EraEquations>()
            .init_resource::<EquationsVisible>()
            .add_systems(Update, toggle_equations);
    }
}

/// Resource holding the equations text for the current era.
#[derive(Resource, Default)]
pub struct EraEquations(pub String);

/// Resource controlling visibility of the equations overlay.
#[derive(Resource, Default)]
pub struct EquationsVisible(pub bool);

/// Marker for the equations overlay entities.
#[derive(Component)]
struct EquationsOverlay;

fn toggle_equations(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<EquationsVisible>,
    equations: Res<EraEquations>,
    existing: Query<Entity, With<EquationsOverlay>>,
) {
    if keyboard.just_pressed(KeyCode::KeyE) {
        visible.0 = !visible.0;
    }

    if visible.0 {
        // Only spawn if not already present
        if existing.iter().count() == 0 && !equations.0.is_empty() {
            // Dark background rectangle
            commands.spawn((
                EquationsOverlay,
                Sprite {
                    color: Color::srgba(0.05, 0.05, 0.1, 0.92),
                    custom_size: Some(Vec2::new(500.0, 250.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 85.0),
            ));

            // Title
            commands.spawn((
                EquationsOverlay,
                Text2d::new("EQUACOES"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.85, 0.3, 1.0)),
                Transform::from_xyz(0.0, 100.0, 86.0),
                bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
            ));

            // Equations body
            commands.spawn((
                EquationsOverlay,
                Text2d::new(equations.0.clone()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(0.9, 0.95, 0.8, 0.95)),
                Transform::from_xyz(0.0, -10.0, 86.0),
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
