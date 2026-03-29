use bevy::prelude::*;

pub struct QuantumExplorerPlugin;

impl Plugin for QuantumExplorerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<QuantumExplorerState>()
            .add_systems(Update, quantum_explorer_system);
    }
}

/// State for the quantum number explorer panel.
#[derive(Resource)]
pub struct QuantumExplorerState {
    pub visible: bool,
    pub selected_n: u32,
    pub selected_l: u32,
    pub selected_ml: i32,
    pub selected_ms: i32,
}

impl Default for QuantumExplorerState {
    fn default() -> Self {
        Self {
            visible: false,
            selected_n: 1,
            selected_l: 0,
            selected_ml: 0,
            selected_ms: -1,
        }
    }
}

/// Marker for the quantum explorer overlay entities.
#[derive(Component)]
struct QuantumExplorerOverlay;

fn orbital_name(n: u32, l: u32) -> String {
    let l_char = match l {
        0 => 's',
        1 => 'p',
        2 => 'd',
        3 => 'f',
        _ => 'g',
    };
    format!("{}{}", n, l_char)
}

fn quantum_explorer_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<QuantumExplorerState>,
    existing: Query<Entity, With<QuantumExplorerOverlay>>,
) {
    // Toggle visibility
    if keyboard.just_pressed(KeyCode::KeyN) {
        state.visible = !state.visible;
    }

    if !state.visible {
        // Despawn all overlay entities
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Handle navigation inputs
    let mut changed = false;

    // Left/Right arrows: change n (1..4)
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        if state.selected_n < 4 {
            state.selected_n += 1;
            changed = true;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        if state.selected_n > 1 {
            state.selected_n -= 1;
            changed = true;
        }
    }

    // Clamp l to valid range for current n
    let max_l = state.selected_n - 1;
    if state.selected_l > max_l {
        state.selected_l = max_l;
        changed = true;
    }

    // W/S: change l
    if keyboard.just_pressed(KeyCode::KeyW) {
        if state.selected_l < max_l {
            state.selected_l += 1;
            changed = true;
        }
    }
    if keyboard.just_pressed(KeyCode::KeyS) && state.visible {
        if state.selected_l > 0 {
            state.selected_l -= 1;
            changed = true;
        }
    }

    // Clamp ml to valid range for current l
    let l_i32 = state.selected_l as i32;
    if state.selected_ml > l_i32 {
        state.selected_ml = l_i32;
        changed = true;
    }
    if state.selected_ml < -l_i32 {
        state.selected_ml = -l_i32;
        changed = true;
    }

    // A/D: change ml
    if keyboard.just_pressed(KeyCode::KeyD) {
        if state.selected_ml < l_i32 {
            state.selected_ml += 1;
            changed = true;
        }
    }
    if keyboard.just_pressed(KeyCode::KeyA) && state.visible {
        if state.selected_ml > -l_i32 {
            state.selected_ml -= 1;
            changed = true;
        }
    }

    // T: toggle ms
    if keyboard.just_pressed(KeyCode::KeyT) {
        state.selected_ms = if state.selected_ms == -1 { 1 } else { -1 };
        changed = true;
    }

    // Only rebuild if changed or first spawn
    let already_exists = existing.iter().count() > 0;
    if already_exists && !changed {
        return;
    }

    // Despawn existing overlay
    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }

    let n = state.selected_n;
    let l = state.selected_l;
    let ml = state.selected_ml;
    let ms = state.selected_ms;

    // Build n line
    let n_line = build_selector_line("n", n as i32, 1, 4);
    // Build l line
    let l_line = build_selector_line("l", l as i32, 0, max_l as i32);
    // Build ml line
    let ml_line = build_selector_line("ml", ml, -(l as i32), l as i32);
    // Build ms line
    let ms_str = if ms == -1 { "-1/2" } else { "+1/2" };
    let ms_other = if ms == -1 { "+1/2" } else { "-1/2" };
    let ms_line = format!(
        "ms = [{}] {}",
        ms_str, ms_other
    );

    let orbital = orbital_name(n, l);
    let radial_nodes = n - l - 1;
    let angular_nodes = l;
    let info = format!(
        "Orbital: {} (ml={})\nNos: {} radiais, {} angular",
        orbital, ml, radial_nodes, angular_nodes
    );

    let panel_text = format!(
        "{}\n{}\n{}\n{}\n\n{}",
        n_line, l_line, ml_line, ms_line, info
    );

    // Panel height based on content
    let panel_height = 220.0;
    let panel_width = 300.0;
    let panel_x = -490.0 + panel_width / 2.0;

    // Dark background
    commands.spawn((
        QuantumExplorerOverlay,
        Sprite {
            color: Color::srgba(0.05, 0.05, 0.1, 0.92),
            custom_size: Some(Vec2::new(panel_width, panel_height)),
            ..default()
        },
        Transform::from_xyz(panel_x, 0.0, 84.0),
    ));

    // Title
    commands.spawn((
        QuantumExplorerOverlay,
        Text2d::new("NUMEROS QUANTICOS"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 0.85, 0.3, 1.0)),
        Transform::from_xyz(panel_x, 90.0, 85.0),
        bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
    ));

    // Body
    commands.spawn((
        QuantumExplorerOverlay,
        Text2d::new(panel_text),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgba(0.9, 0.9, 0.5, 0.95)),
        Transform::from_xyz(panel_x, 10.0, 85.0),
        bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Left),
    ));

    // Controls hint
    commands.spawn((
        QuantumExplorerOverlay,
        Text2d::new("<-/-> n  W/S l  A/D ml  T ms"),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
        Transform::from_xyz(panel_x, -90.0, 85.0),
        bevy::text::TextLayout::new_with_justify(bevy::text::Justify::Center),
    ));
}

fn build_selector_line(label: &str, selected: i32, min: i32, max: i32) -> String {
    let mut parts = Vec::new();
    for v in min..=max {
        if v == selected {
            parts.push(format!("[{}]", v));
        } else {
            parts.push(format!(" {} ", v));
        }
    }
    format!("{} = {}", label, parts.join(" "))
}
