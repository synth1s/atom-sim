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

/// Formata ExportableData como string CSV.
#[allow(dead_code)]
pub fn format_csv(data: &ExportableData) -> String {
    let mut csv = String::new();
    csv.push_str(&data.headers.join(","));
    csv.push('\n');
    for row in &data.rows {
        csv.push_str(&row.join(","));
        csv.push('\n');
    }
    csv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_csv_basic() {
        let data = ExportableData {
            headers: vec!["Name".to_string(), "Value".to_string()],
            rows: vec![
                vec!["H".to_string(), "1".to_string()],
                vec!["He".to_string(), "2".to_string()],
            ],
            era_name: "test".to_string(),
        };
        let csv = format_csv(&data);
        assert_eq!(csv, "Name,Value\nH,1\nHe,2\n");
    }

    #[test]
    fn test_format_csv_empty_rows() {
        let data = ExportableData {
            headers: vec!["A".to_string(), "B".to_string()],
            rows: vec![],
            era_name: "empty".to_string(),
        };
        let csv = format_csv(&data);
        assert_eq!(csv, "A,B\n");
    }

    #[test]
    fn test_exportable_data_default() {
        let data = ExportableData::default();
        assert!(data.headers.is_empty());
        assert!(data.rows.is_empty());
        assert!(data.era_name.is_empty());
    }
}
