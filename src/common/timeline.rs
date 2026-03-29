use bevy::prelude::*;
use super::ActiveEra;

pub struct TimelinePlugin;

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_timeline)
            .add_systems(Update, (update_timeline, timeline_nav));
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
    mut next_era: ResMut<NextState<ActiveEra>>,
) {
    let current = era_to_index(era.get());

    if keyboard.just_pressed(KeyCode::BracketLeft) {
        if current > 0 {
            if let Some(new_era) = index_to_era(current - 1) {
                next_era.set(new_era);
            }
        }
    } else if keyboard.just_pressed(KeyCode::BracketRight) {
        if current < 8 {
            if let Some(new_era) = index_to_era(current + 1) {
                next_era.set(new_era);
            }
        }
    }
}
