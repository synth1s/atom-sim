use bevy::prelude::*;
use crate::common::ActiveEra;
use crate::common::sandbox::EraParameters;

pub struct SnapshotPlugin;

impl Plugin for SnapshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, snapshot_system);
    }
}

/// Marker para a mensagem temporaria de "Snapshot salvo!".
#[derive(Component)]
struct SnapshotMessage {
    timer: f32,
}

fn snapshot_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    era: Res<State<ActiveEra>>,
    params: Res<EraParameters>,
    mut msg_query: Query<(Entity, &mut SnapshotMessage)>,
) {
    // Atualizar timer de mensagens existentes
    let dt = time.delta_secs();
    for (entity, mut msg) in msg_query.iter_mut() {
        msg.timer -= dt;
        if msg.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }

    if keyboard.just_pressed(KeyCode::KeyS) {
        // Serializar manualmente como JSON (sem serde)
        let era_name = format!("{:?}", era.get());

        let mut param_entries = Vec::new();
        for p in &params.0 {
            param_entries.push(format!(
                "    {{ \"name\": \"{}\", \"value\": {:.2} }}",
                p.name, p.value
            ));
        }
        let params_json = param_entries.join(",\n");

        let json = format!(
            "{{\n  \"era\": \"{}\",\n  \"parameters\": [\n{}\n  ]\n}}",
            era_name, params_json
        );

        // Salvar arquivo
        match std::fs::write("atom-sim-snapshot.json", &json) {
            Ok(()) => {
                // Despawn mensagens anteriores
                for (entity, _) in msg_query.iter() {
                    commands.entity(entity).despawn();
                }

                // Mostrar mensagem temporaria
                commands.spawn((
                    SnapshotMessage { timer: 2.0 },
                    Text2d::new("Snapshot salvo!"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.3, 1.0, 0.3, 1.0)),
                    Transform::from_xyz(0.0, -340.0, 95.0),
                ));
            }
            Err(e) => {
                // Mostrar erro
                for (entity, _) in msg_query.iter() {
                    commands.entity(entity).despawn();
                }
                commands.spawn((
                    SnapshotMessage { timer: 3.0 },
                    Text2d::new(format!("Erro ao salvar: {}", e)),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 0.3, 0.3, 1.0)),
                    Transform::from_xyz(0.0, -340.0, 95.0),
                ));
            }
        }
    }
}
