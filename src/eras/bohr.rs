use bevy::prelude::*;
use crate::common::{ActiveEra, SimulationState};
use crate::common::ui::HudText;
use crate::physics::spectral;

pub struct BohrPlugin;

impl Plugin for BohrPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ActiveEra::Bohr), setup_bohr)
            .add_systems(OnExit(ActiveEra::Bohr), cleanup_bohr)
            .add_systems(
                Update,
                (
                    orbit_electron,
                    update_photons,
                    update_bohr_hud,
                )
                    .run_if(in_state(ActiveEra::Bohr))
                    .run_if(in_state(SimulationState::Running)),
            )
            .add_systems(
                Update,
                bohr_controls.run_if(in_state(ActiveEra::Bohr)),
            );
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct BohrEntity;

#[derive(Component)]
struct BohrNucleus;

/// Elétron numa órbita quantizada de Bohr.
#[derive(Component)]
struct BohrElectron {
    n: u32,         // Número quântico principal (1, 2, 3, ...)
    angle: f32,     // Ângulo atual na órbita
}

/// Órbita visual (círculo tracejado).
#[derive(Component)]
struct OrbitRing {
    #[allow(dead_code)]
    n: u32,
}

/// Fóton emitido numa transição.
#[derive(Component)]
struct Photon {
    vel: Vec2,
    lifetime: f32,
}

/// Linha espectral no espectrógrafo.
#[derive(Component)]
struct SpectralLine;

#[derive(Component)]
struct BohrInfoText;

#[derive(Component)]
struct EnergyDiagramText;

#[derive(Component)]
struct SpectrumLabel;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct BohrState {
    current_n: u32,
    max_n: u32,
    transitions_recorded: Vec<(u32, u32, f64)>, // (n_upper, n_lower, wavelength_nm)
}

impl Default for BohrState {
    fn default() -> Self {
        Self {
            current_n: 3,
            max_n: 6,
            transitions_recorded: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const NUCLEUS_POS: Vec2 = Vec2::new(-200.0, 30.0);
const ORBIT_SCALE: f32 = 18.0; // rₙ visual = n² × ORBIT_SCALE

fn orbit_radius(n: u32) -> f32 {
    (n as f32).powi(2) * ORBIT_SCALE
}

const SPECTRUM_X: f32 = 250.0;
const SPECTRUM_Y: f32 = -200.0;
const SPECTRUM_WIDTH: f32 = 300.0;

// ---------------------------------------------------------------------------
// Setup & Cleanup
// ---------------------------------------------------------------------------

fn setup_bohr(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hud_query: Query<&mut Text2d, With<HudText>>,
) {
    for mut text in hud_query.iter_mut() {
        *text = Text2d::new(
            "BOHR (1913) \u{2014} Orbitas Quantizadas\n\n\
             Postulado 1: L = n*hbar (momento angular)\n\
             Postulado 2: hv = En1 - En2 (fotons)\n\n\
             rn = n^2 * a0 (a0 = 0.529 A)\n\
             En = -13.6/n^2 eV\n\
             R_H derivado = 1.0974 x 10^7 m^-1\n\
             (4 casas decimais de precisao!)\n\n\
             LIMITACAO: Nao funciona para He.\n\
             Nao explica efeito Zeeman anomalo."
        );
    }

    commands.insert_resource(BohrState::default());

    // Núcleo
    commands.spawn((
        BohrEntity,
        BohrNucleus,
        Mesh2d(meshes.add(Circle::new(6.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 0.3, 0.2, 1.0),
        ))),
        Transform::from_xyz(NUCLEUS_POS.x, NUCLEUS_POS.y, 2.0),
    ));

    // Orbits (circles for n=1..6)
    for n in 1..=6u32 {
        let r = orbit_radius(n);
        // Desenhar órbita como anel fino
        let ring_mesh = meshes.add(Annulus::new(r - 0.5, r + 0.5));
        let alpha = if n <= 3 { 0.4 } else { 0.2 };
        let ring_mat = materials.add(ColorMaterial::from_color(
            Color::srgba(0.5, 0.5, 0.7, alpha),
        ));

        commands.spawn((
            BohrEntity,
            OrbitRing { n },
            Mesh2d(ring_mesh),
            MeshMaterial2d(ring_mat),
            Transform::from_xyz(NUCLEUS_POS.x, NUCLEUS_POS.y, 0.5),
        ));

        // Label para cada órbita
        commands.spawn((
            BohrEntity,
            Text2d::new(format!("n={}", n)),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::srgba(0.5, 0.5, 0.7, 0.6)),
            Transform::from_xyz(NUCLEUS_POS.x + r + 8.0, NUCLEUS_POS.y + 5.0, 10.0),
        ));
    }

    // Elétron (inicia em n=3)
    let initial_n = 3u32;
    let r = orbit_radius(initial_n);
    commands.spawn((
        BohrEntity,
        BohrElectron {
            n: initial_n,
            angle: 0.0,
        },
        Mesh2d(meshes.add(Circle::new(7.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(0.2, 0.6, 1.0, 1.0),
        ))),
        Transform::from_xyz(NUCLEUS_POS.x + r, NUCLEUS_POS.y, 3.0),
    ));

    // Espectrógrafo — fundo
    commands.spawn((
        BohrEntity,
        Mesh2d(meshes.add(Rectangle::new(SPECTRUM_WIDTH + 10.0, 40.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(0.05, 0.05, 0.08, 0.9),
        ))),
        Transform::from_xyz(SPECTRUM_X + SPECTRUM_WIDTH / 2.0, SPECTRUM_Y, 4.0),
    ));

    // Espectrógrafo — labels
    commands.spawn((
        BohrEntity,
        SpectrumLabel,
        Text2d::new("ESPECTRO DE EMISSAO (380nm --- 780nm)"),
        TextFont { font_size: 11.0, ..default() },
        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.7)),
        Transform::from_xyz(SPECTRUM_X + SPECTRUM_WIDTH / 2.0, SPECTRUM_Y - 30.0, 10.0),
    ));

    // Diagrama de energia (lado direito)
    commands.spawn((
        BohrEntity,
        EnergyDiagramText,
        Text2d::new(""),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgba(0.7, 0.8, 0.9, 0.9)),
        Transform::from_xyz(350.0, 250.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));

    // Info/controles
    commands.spawn((
        BohrEntity,
        BohrInfoText,
        Text2d::new(""),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgba(0.8, 0.8, 0.5, 0.9)),
        Transform::from_xyz(350.0, 50.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));
}

fn cleanup_bohr(
    mut commands: Commands,
    query: Query<Entity, With<BohrEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<BohrState>();
}

// ---------------------------------------------------------------------------
// Electron orbital motion
// Angular velocity: ω = v/r, where v = nℏ/(mₑr) = ℏ/(mₑna₀)
// In simulation units, ω ∝ 1/n³ (faster for lower orbits)
// ---------------------------------------------------------------------------

fn orbit_electron(
    time: Res<Time>,
    bohr_state: Option<Res<BohrState>>,
    mut query: Query<(&mut BohrElectron, &mut Transform)>,
) {
    let Some(state) = bohr_state else { return };
    let dt = time.delta_secs();

    for (mut electron, mut transform) in query.iter_mut() {
        electron.n = state.current_n;
        let r = orbit_radius(electron.n);

        // ω ∝ 1/n³ (from Bohr model: v ∝ 1/n, r ∝ n²)
        let angular_vel = 4.0 / (electron.n as f32).powi(3);
        electron.angle += angular_vel * dt;

        transform.translation.x = NUCLEUS_POS.x + electron.angle.cos() * r;
        transform.translation.y = NUCLEUS_POS.y + electron.angle.sin() * r;
    }
}

// ---------------------------------------------------------------------------
// Photon emission and travel
// ---------------------------------------------------------------------------

fn update_photons(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Photon, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut photon, mut transform) in query.iter_mut() {
        transform.translation.x += photon.vel.x * dt;
        transform.translation.y += photon.vel.y * dt;
        photon.lifetime -= dt;
        if photon.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Controls — transition between orbits
// ---------------------------------------------------------------------------

fn bohr_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut bohr_state: Option<ResMut<BohrState>>,
    electron_query: Query<&Transform, With<BohrElectron>>,
) {
    let Some(ref mut state) = bohr_state else { return };

    let old_n = state.current_n;
    let mut new_n = old_n;

    // Setas ou teclas numéricas para mudar de órbita
    if keyboard.just_pressed(KeyCode::ArrowUp) && old_n < state.max_n {
        new_n = old_n + 1; // Absorção (sobe de nível)
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) && old_n > 1 {
        new_n = old_n - 1; // Emissão (desce de nível)
    }

    // Salto direto para nível específico
    if keyboard.just_pressed(KeyCode::KeyQ) { new_n = 1; }
    if keyboard.just_pressed(KeyCode::KeyW) { new_n = 2; }
    if keyboard.just_pressed(KeyCode::KeyR) { new_n = 4; }
    if keyboard.just_pressed(KeyCode::KeyT) { new_n = 5; }
    if keyboard.just_pressed(KeyCode::KeyY) { new_n = 6; }

    if new_n != old_n {
        state.current_n = new_n;

        // Se desceu (emissão), criar fóton
        if new_n < old_n {
            let wavelength_m = spectral::transition_wavelength(old_n, new_n);
            let wavelength_nm = wavelength_m * 1e9;
            let color = spectral::wavelength_to_color(wavelength_nm);

            // Posição do elétron no momento da transição
            let electron_pos = electron_query
                .iter()
                .next()
                .map(|t| t.translation.truncate())
                .unwrap_or(NUCLEUS_POS);

            // Emitir fóton em direção radial
            let dir = (electron_pos - NUCLEUS_POS).normalize_or_zero();
            let photon_speed = 200.0;

            commands.spawn((
                BohrEntity,
                Photon {
                    vel: dir * photon_speed,
                    lifetime: 3.0,
                },
                Mesh2d(meshes.add(Circle::new(4.0))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
                Transform::from_xyz(electron_pos.x, electron_pos.y, 4.0),
            ));

            // Adicionar linha espectral ao espectrógrafo
            let x_pos = wavelength_to_spectrum_x(wavelength_nm);
            commands.spawn((
                BohrEntity,
                SpectralLine,
                Mesh2d(meshes.add(Rectangle::new(2.0, 30.0))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
                Transform::from_xyz(x_pos, SPECTRUM_Y, 5.0),
            ));

            // Registrar transição
            state.transitions_recorded.push((old_n, new_n, wavelength_nm));
        }
    }
}

/// Mapeia comprimento de onda (nm) para posição X no espectrógrafo.
fn wavelength_to_spectrum_x(lambda_nm: f64) -> f32 {
    // Range do espectrógrafo: 50nm (UV) até 2000nm (IR)
    let min_nm = 50.0;
    let max_nm = 2000.0;
    let t = ((lambda_nm - min_nm) / (max_nm - min_nm)).clamp(0.0, 1.0);
    SPECTRUM_X + t as f32 * SPECTRUM_WIDTH
}

// ---------------------------------------------------------------------------
// HUD — energy diagram and info
// ---------------------------------------------------------------------------

fn update_bohr_hud(
    bohr_state: Option<Res<BohrState>>,
    mut energy_query: Query<&mut Text2d, (With<EnergyDiagramText>, Without<BohrInfoText>)>,
    mut info_query: Query<&mut Text2d, (With<BohrInfoText>, Without<EnergyDiagramText>)>,
) {
    let Some(state) = bohr_state else { return };

    // Diagrama de energia
    let mut energy_text = String::from("NIVEIS DE ENERGIA (Bohr)\n");
    energy_text.push_str("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n");
    for n in (1..=6u32).rev() {
        let e = spectral::bohr_energy(n);
        let marker = if n == state.current_n { " <-- e-" } else { "" };
        energy_text.push_str(&format!(
            "n={}: E = {:.3} eV  r = {:.3} A{}\n",
            n, e, spectral::bohr_radius(n) * 1e10, marker
        ));
    }
    energy_text.push_str(&format!(
        "\nR_H = {:.7} x 10^7 m^-1\n(derivado de constantes!)",
        spectral::RYDBERG_CONSTANT / 1e7
    ));

    for mut text in energy_query.iter_mut() {
        *text = Text2d::new(energy_text.clone());
    }

    // Info/controles + transições recentes
    let mut info = String::from(
        "CONTROLES\n\
         [Setas Cima/Baixo] Mudar orbita\n\
         [Q,W,R,T,Y] Saltar para n=1,2,4,5,6\n\n\
         TRANSICOES RECENTES:\n"
    );

    let start = if state.transitions_recorded.len() > 6 {
        state.transitions_recorded.len() - 6
    } else {
        0
    };

    for (n_up, n_low, lambda) in &state.transitions_recorded[start..] {
        let series = spectral::series_name(*n_low);
        info.push_str(&format!(
            "  {} -> {}: {:.1} nm ({})\n",
            n_up, n_low, lambda, series
        ));
    }

    if state.transitions_recorded.is_empty() {
        info.push_str("  (nenhuma ainda - desça de orbita!)");
    }

    for mut text in info_query.iter_mut() {
        *text = Text2d::new(info.clone());
    }
}
