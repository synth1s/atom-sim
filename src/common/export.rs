use bevy::prelude::*;

/// Resource holding exportable simulation data.
#[derive(Resource, Default)]
pub struct ExportableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub era_name: String,
}

pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExportableData>()
            .add_systems(Update, export_csv_system);
    }
}

fn export_csv_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    data: Res<ExportableData>,
) {
    if keyboard.just_pressed(KeyCode::KeyX) {
        if data.era_name.is_empty() || data.headers.is_empty() {
            return;
        }

        let mut csv = String::new();
        csv.push_str(&data.headers.join(","));
        csv.push('\n');

        for row in &data.rows {
            csv.push_str(&row.join(","));
            csv.push('\n');
        }

        let filename = format!("atom-sim-{}.csv", data.era_name);
        match std::fs::write(&filename, &csv) {
            Ok(_) => info!("CSV exportado: {}", filename),
            Err(e) => warn!("Erro ao exportar CSV: {}", e),
        }
    }
}
