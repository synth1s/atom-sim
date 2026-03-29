use bevy::prelude::*;
use super::ActiveEra;
use super::transition::{TransitionState, TransitionPhase};
use super::auto_tour::AutoTourState;

pub struct TimelinePlugin;

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ExperimentDescriptions(vec![
                "Argumento filosofico: materia\nnao pode ser dividida infinitamente".to_string(),
                "Lei das Proporcoes Multiplas:\nCO tem 1.33g O/g C, CO2 tem 2.67g".to_string(),
                "Tubo de raios catodicos:\ne/m = 1.759e11 C/kg".to_string(),
                "Folha de ouro: 1/8000 alfas\nretroespalhadas a >90 graus".to_string(),
                "Espectro do H: formula de Rydberg\ncom 4 casas de precisao".to_string(),
                "Interferometro de Michelson:\nH-alfa e um dupleto (~0.14 cm^-1)".to_string(),
                "Davisson-Germer (1927):\nlambda = 1.65 A (previsto: 1.67 A)".to_string(),
                "Eq. de onda: numeros quanticos\nemergem das condicoes de contorno".to_string(),
                "Anderson (1932): positron em\nraios cosmicos confirma previsao".to_string(),
            ]))
            .add_systems(Startup, spawn_timeline)
            .add_systems(Update, (update_timeline, timeline_nav, experiment_hover));
    }
}

/// Marker for the entire timeline bar.
#[derive(Component)]
struct TimelineBar;

/// Marker for individual timeline items; stores the era index (0-8).
#[derive(Component)]
struct TimelineItem(usize);

/// Marker for the highlight text of a timeline item.
#[derive(Component)]
struct TimelineHighlight(usize);

/// Marker for the experiment description panel (hover popup).
#[derive(Component)]
struct ExperimentPanel;

/// Resource holding a short experiment description for each era (0-8).
#[derive(Resource)]
struct ExperimentDescriptions(Vec<String>);

const ERA_LABELS: [&str; 9] = [
    "1:Democrito",
    "2:Dalton",
    "3:Thomson",
    "4:Rutherford",
    "5:Bohr",
    "6:Sommerfeld",
    "7:De Broglie",
    "8:Schrodinger",
    "9:Dirac",
];

const ERA_YEARS: [&str; 9] = [
    "~400 a.C.",
    "1803",
    "1897",
    "1911",
    "1913",
    "1916",
    "1924",
    "1926",
    "1928",
];

const ITEM_WIDTH: f32 = 120.0;
const ITEM_HEIGHT: f32 = 25.0;
const ITEM_GAP: f32 = 4.0;
const BAR_Y: f32 = -340.0;

const COLOR_INACTIVE: Color = Color::srgba(0.25, 0.25, 0.25, 0.85);
const COLOR_ACTIVE: Color = Color::srgba(0.95, 0.85, 0.2, 0.95);
const TEXT_COLOR_INACTIVE: Color = Color::srgba(0.6, 0.6, 0.6, 0.9);
const TEXT_COLOR_ACTIVE: Color = Color::srgba(0.1, 0.1, 0.1, 1.0);

fn era_to_index(era: &ActiveEra) -> usize {
    match era {
        ActiveEra::Democritus => 0,
        ActiveEra::Dalton => 1,
        ActiveEra::Thomson => 2,
        ActiveEra::Rutherford => 3,
        ActiveEra::Bohr => 4,
        ActiveEra::Sommerfeld => 5,
        ActiveEra::DeBroglie => 6,
        ActiveEra::Schrodinger => 7,
        ActiveEra::Dirac => 8,
    }
}

fn index_to_era(index: usize) -> Option<ActiveEra> {
    match index {
        0 => Some(ActiveEra::Democritus),
        1 => Some(ActiveEra::Dalton),
        2 => Some(ActiveEra::Thomson),
        3 => Some(ActiveEra::Rutherford),
        4 => Some(ActiveEra::Bohr),
        5 => Some(ActiveEra::Sommerfeld),
        6 => Some(ActiveEra::DeBroglie),
        7 => Some(ActiveEra::Schrodinger),
        8 => Some(ActiveEra::Dirac),
        _ => None,
    }
}

fn spawn_timeline(mut commands: Commands) {
    let total_width = 9.0 * ITEM_WIDTH + 8.0 * ITEM_GAP;
    let start_x = -total_width / 2.0 + ITEM_WIDTH / 2.0;

    for i in 0..9 {
        let x = start_x + i as f32 * (ITEM_WIDTH + ITEM_GAP);
        let label = format!("{} ({})", ERA_LABELS[i], ERA_YEARS[i]);

        // Background rectangle (using a Sprite)
        commands.spawn((
            TimelineBar,
            TimelineItem(i),
            Sprite {
                color: if i == 0 { COLOR_ACTIVE } else { COLOR_INACTIVE },
                custom_size: Some(Vec2::new(ITEM_WIDTH, ITEM_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(x, BAR_Y, 50.0),
        ));

        // Text label on top
        commands.spawn((
            TimelineBar,
            TimelineHighlight(i),
            Text2d::new(label),
            TextFont {
                font_size: 10.0,
                ..default()
            },
            TextColor(if i == 0 { TEXT_COLOR_ACTIVE } else { TEXT_COLOR_INACTIVE }),
            Transform::from_xyz(x, BAR_Y, 51.0),
        ));
    }
}

fn update_timeline(
    era: Res<State<ActiveEra>>,
    mut bg_query: Query<(&TimelineItem, &mut Sprite), Without<TimelineHighlight>>,
    mut text_query: Query<(&TimelineHighlight, &mut TextColor), Without<TimelineItem>>,
) {
    if !era.is_changed() {
        return;
    }

    let active_index = era_to_index(era.get());

    for (item, mut sprite) in bg_query.iter_mut() {
        sprite.color = if item.0 == active_index {
            COLOR_ACTIVE
        } else {
            COLOR_INACTIVE
        };
    }

    for (highlight, mut color) in text_query.iter_mut() {
        *color = if highlight.0 == active_index {
            TextColor(TEXT_COLOR_ACTIVE)
        } else {
            TextColor(TEXT_COLOR_INACTIVE)
        };
    }
}

fn timeline_nav(
    keyboard: Res<ButtonInput<KeyCode>>,
    era: Res<State<ActiveEra>>,
    mut transition: ResMut<TransitionState>,
    tour: Res<AutoTourState>,
) {
    if transition.active || tour.active {
        return;
    }

    let current = era_to_index(era.get());

    let target = if keyboard.just_pressed(KeyCode::BracketLeft) {
        if current > 0 { index_to_era(current - 1) } else { None }
    } else if keyboard.just_pressed(KeyCode::BracketRight) {
        if current < 8 { index_to_era(current + 1) } else { None }
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

fn experiment_hover(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    timeline_items: Query<(&TimelineItem, &GlobalTransform), Without<ExperimentPanel>>,
    existing_panels: Query<Entity, With<ExperimentPanel>>,
    descriptions: Res<ExperimentDescriptions>,
) {
    // Despawn existing panels first
    for entity in existing_panels.iter() {
        commands.entity(entity).despawn();
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };
    let world_pos = ray.origin.truncate();

    // Find which timeline item the cursor is hovering over (within 30px)
    let mut hovered: Option<(usize, Vec2)> = None;
    for (item, global_transform) in timeline_items.iter() {
        let item_pos = global_transform.translation().truncate();
        let dx = (world_pos.x - item_pos.x).abs();
        let dy = (world_pos.y - item_pos.y).abs();
        // Check within the rectangle bounds (half-width, half-height) + a margin
        if dx < ITEM_WIDTH / 2.0 + 5.0 && dy < ITEM_HEIGHT / 2.0 + 5.0 {
            let dist = world_pos.distance(item_pos);
            if dist < 80.0 {
                if hovered.is_none() || dist < world_pos.distance(hovered.unwrap().1) {
                    hovered = Some((item.0, item_pos));
                }
            }
        }
    }

    if let Some((index, pos)) = hovered {
        if index < descriptions.0.len() {
            let desc = &descriptions.0[index];

            // Dark background
            commands.spawn((
                ExperimentPanel,
                Sprite {
                    color: Color::srgba(0.08, 0.08, 0.12, 0.92),
                    custom_size: Some(Vec2::new(250.0, 50.0)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y + 45.0, 55.0),
            ));

            // Description text
            commands.spawn((
                ExperimentPanel,
                Text2d::new(desc.clone()),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgba(0.9, 0.92, 0.8, 0.95)),
                Transform::from_xyz(pos.x, pos.y + 45.0, 56.0),
                bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
            ));
        }
    }
}
