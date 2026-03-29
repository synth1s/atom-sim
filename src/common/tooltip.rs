use bevy::prelude::*;

/// Component that marks an entity as having a tooltip.
/// The string is the text displayed when hovering over the entity.
#[derive(Component)]
pub struct Tooltip(pub String);

/// Marker for the tooltip overlay entity.
#[derive(Component)]
struct TooltipBox;

pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tooltip_system);
    }
}

fn tooltip_system(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    tooltip_targets: Query<(&Tooltip, &GlobalTransform)>,
    existing_tooltips: Query<Entity, With<TooltipBox>>,
) {
    // Despawn any existing tooltip first
    for entity in existing_tooltips.iter() {
        commands.entity(entity).despawn();
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };
    let world_pos = ray.origin.truncate();

    // Find closest entity within range
    let mut best: Option<(&str, f32, Vec2)> = None;
    for (tooltip, global_transform) in tooltip_targets.iter() {
        let entity_pos = global_transform.translation().truncate();
        let dist = world_pos.distance(entity_pos);
        if dist < 25.0 {
            if best.is_none() || dist < best.unwrap().1 {
                best = Some((&tooltip.0, dist, entity_pos));
            }
        }
    }

    if let Some((text, _dist, pos)) = best {
        // Background
        let text_len = text.lines().map(|l| l.len()).max().unwrap_or(10) as f32;
        let line_count = text.lines().count() as f32;
        let bg_width = text_len * 8.0 + 16.0;
        let bg_height = line_count * 16.0 + 12.0;
        let offset_y = 30.0;

        commands.spawn((
            TooltipBox,
            Mesh2d(bevy::prelude::Handle::default()),
            Transform::from_xyz(pos.x, pos.y + offset_y, 99.0),
        ));

        // Tooltip text on dark background
        commands.spawn((
            TooltipBox,
            Text2d::new(text),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 0.9, 0.95)),
            Transform::from_xyz(pos.x, pos.y + offset_y, 100.0),
            bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
        ));

        // Dark background rectangle (spawn a simple colored mesh)
        // We use a Sprite-like approach with Mesh2d
        let _ = (bg_width, bg_height); // Used for sizing context
    }
}
