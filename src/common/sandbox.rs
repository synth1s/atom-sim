use bevy::prelude::*;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EraParameters>()
            .init_resource::<SandboxVisible>()
            .init_resource::<SandboxSelected>()
            .add_systems(Update, sandbox_system);
    }
}

/// Um parametro ajustavel de uma era.
pub struct Parameter {
    pub name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

/// Resource contendo os parametros ajustaveis da era atual.
#[derive(Resource, Default)]
pub struct EraParameters(pub Vec<Parameter>);

impl EraParameters {
    pub fn from_tuples(tuples: &[(&str, f32, f32, f32, f32)]) -> Self {
        Self(
            tuples
                .iter()
                .map(|(name, value, min, max, step)| Parameter {
                    name: name.to_string(),
                    value: *value,
                    min: *min,
                    max: *max,
                    step: *step,
                })
                .collect(),
        )
    }
}

/// Visibilidade do painel sandbox.
#[derive(Resource, Default)]
pub struct SandboxVisible(pub bool);

/// Qual parametro esta selecionado no painel.
#[derive(Resource, Default)]
pub struct SandboxSelected(pub usize);

/// Marker para entidades do overlay sandbox.
#[derive(Component)]
struct SandboxOverlay;

fn sandbox_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<SandboxVisible>,
    mut selected: ResMut<SandboxSelected>,
    mut params: ResMut<EraParameters>,
    existing: Query<Entity, With<SandboxOverlay>>,
) {
    // Toggle com P
    if keyboard.just_pressed(KeyCode::KeyP) {
        visible.0 = !visible.0;
    }

    if !visible.0 {
        // Despawn overlay
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let param_count = params.0.len();
    if param_count == 0 {
        // Sem parametros, despawn qualquer overlay residual
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Clamp selected
    if selected.0 >= param_count {
        selected.0 = 0;
    }

    // Navegacao
    if keyboard.just_pressed(KeyCode::ArrowUp) && selected.0 > 0 {
        selected.0 -= 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) && selected.0 + 1 < param_count {
        selected.0 += 1;
    }

    // Ajustar valor
    if keyboard.just_pressed(KeyCode::Minus) {
        let p = &mut params.0[selected.0];
        p.value = (p.value - p.step).max(p.min);
    }
    if keyboard.just_pressed(KeyCode::Equal) {
        let p = &mut params.0[selected.0];
        p.value = (p.value + p.step).min(p.max);
    }

    // Rebuild overlay a cada frame quando visivel (simples, sem otimizacao)
    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }

    // Montar texto do painel
    let mut lines = String::from("PARAMETROS (Sandbox)\n--------------------\n");
    for (i, p) in params.0.iter().enumerate() {
        let marker = if i == selected.0 { ">> " } else { "   " };
        lines.push_str(&format!(
            "{}{}: {:.1}  [{:.1} .. {:.1}]\n",
            marker, p.name, p.value, p.min, p.max
        ));
    }
    lines.push_str("--------------------\n[-/=] Ajustar  [Setas] Selecionar");

    // Fundo do painel
    let panel_height = 30.0 + param_count as f32 * 20.0 + 60.0;
    commands.spawn((
        SandboxOverlay,
        Mesh2d(bevy::prelude::Handle::default()),
        Transform::from_xyz(480.0, 0.0, 83.0),
    ));

    // Texto do painel
    let selected_idx = selected.0;
    commands.spawn((
        SandboxOverlay,
        Text2d::new(lines),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgba(0.9, 0.9, 0.7, 0.95)),
        Transform::from_xyz(480.0, panel_height / 2.0 - 30.0, 84.0),
        TextLayout::new_with_justify(Justify::Left),
    ));

    // Indicador de selecao (cor amarela) — spawnar texto individual para o parametro selecionado
    if selected_idx < params.0.len() {
        let p = &params.0[selected_idx];
        let highlight_text = format!(">> {}: {:.1}", p.name, p.value);
        let y_offset = panel_height / 2.0 - 30.0 - (selected_idx as f32 + 2.0) * 16.0;
        commands.spawn((
            SandboxOverlay,
            Text2d::new(highlight_text),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 0.3, 1.0)),
            Transform::from_xyz(480.0, y_offset, 85.0),
            TextLayout::new_with_justify(Justify::Left),
        ));
    }
}
