use bevy::prelude::*;

use crate::common::{ActiveEra, SimulationState};
use crate::common::ui::HudText;
use crate::physics::{quantum, spectral};

pub struct SchrodingerPlugin;

impl Plugin for SchrodingerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ActiveEra::Schrodinger), setup_schrodinger)
            .add_systems(OnExit(ActiveEra::Schrodinger), cleanup_schrodinger)
            .add_systems(
                Update,
                update_schrodinger_hud
                    .run_if(in_state(ActiveEra::Schrodinger))
                    .run_if(in_state(SimulationState::Running)),
            )
            .add_systems(
                Update,
                schrodinger_controls.run_if(in_state(ActiveEra::Schrodinger)),
            );
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct SchrodingerEntity;

/// Ponto na nuvem de probabilidade |ψ|².
#[derive(Component)]
struct CloudPoint;

/// Ponto de medição (colapso).
#[derive(Component)]
struct MeasurementDot;

#[derive(Component)]
struct SchrodingerInfoText;

#[derive(Component)]
struct OrbitalLabel;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct SchrodingerState {
    n: u32,
    l: u32,
    m: i32,
    measurements: usize,
    needs_rebuild: bool,
}

impl Default for SchrodingerState {
    fn default() -> Self {
        Self {
            n: 2,
            l: 1,
            m: 0,
            measurements: 0,
            needs_rebuild: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CENTER: Vec2 = Vec2::new(-100.0, 30.0);
const CLOUD_POINTS: usize = 3000;
// Escala: converter metros para pixels. a₀ ~ 5.3×10⁻¹¹ m → ~30 px
const SPATIAL_SCALE: f64 = 30.0 / spectral::BOHR_RADIUS_M;

// ---------------------------------------------------------------------------
// Setup & Cleanup
// ---------------------------------------------------------------------------

fn setup_schrodinger(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hud_query: Query<&mut Text2d, With<HudText>>,
) {
    for mut text in hud_query.iter_mut() {
        *text = Text2d::new(
            "SCHRODINGER (1926) \u{2014} Equacao de Onda\n\n\
             ih * dpsi/dt = [-h^2/(2m) * nabla^2 + V] * psi\n\n\
             Solucao para H: psi_nlm = R_nl(r) * Y_lm(t,f)\n\
             |psi|^2 = densidade de probabilidade (Born)\n\
             Numeros quanticos emergem das condicoes\n\
             de contorno, nao sao postulados!\n\n\
             LIMITACAO: Nao-relativistico.\n\
             Spin adicionado ad hoc (Pauli).\n\
             Nao preve antimateria."
        );
    }

    commands.insert_resource(SchrodingerState::default());

    // Núcleo
    commands.spawn((
        SchrodingerEntity,
        Mesh2d(meshes.add(Circle::new(3.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 0.3, 0.2, 0.8),
        ))),
        Transform::from_xyz(CENTER.x, CENTER.y, 5.0),
    ));

    // Info text
    commands.spawn((
        SchrodingerEntity,
        SchrodingerInfoText,
        Text2d::new(""),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgba(0.8, 0.8, 0.5, 0.9)),
        Transform::from_xyz(300.0, 250.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));

    // Orbital label
    commands.spawn((
        SchrodingerEntity,
        OrbitalLabel,
        Text2d::new(""),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgba(0.3, 0.7, 1.0, 0.9)),
        Transform::from_xyz(CENTER.x, CENTER.y - 200.0, 10.0),
    ));

    // A nuvem será criada no primeiro frame via needs_rebuild
}

fn cleanup_schrodinger(
    mut commands: Commands,
    query: Query<Entity, With<SchrodingerEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<SchrodingerState>();
}

// ---------------------------------------------------------------------------
// Rebuild cloud when quantum numbers change
// ---------------------------------------------------------------------------

fn rebuild_cloud(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    old_cloud: &Query<Entity, With<CloudPoint>>,
    old_measurements: &Query<Entity, With<MeasurementDot>>,
    state: &SchrodingerState,
) {
    // Limpar nuvem e medições anteriores
    for entity in old_cloud.iter() {
        commands.entity(entity).despawn();
    }
    for entity in old_measurements.iter() {
        commands.entity(entity).despawn();
    }

    // Gerar novos pontos amostrados de |ψ|²
    let points = quantum::sample_orbital_points(
        state.n, state.l, state.m,
        CLOUD_POINTS,
        SPATIAL_SCALE,
    );

    let point_mesh = meshes.add(Circle::new(1.5));

    // Cor baseada no orbital
    let base_color = match state.l {
        0 => Color::srgba(0.3, 0.6, 1.0, 0.4),   // s: azul
        1 => Color::srgba(0.3, 1.0, 0.5, 0.4),   // p: verde
        2 => Color::srgba(1.0, 0.6, 0.2, 0.4),   // d: laranja
        _ => Color::srgba(0.8, 0.3, 0.8, 0.4),   // f: roxo
    };
    let point_mat = materials.add(ColorMaterial::from_color(base_color));

    for (x, y) in points {
        commands.spawn((
            SchrodingerEntity,
            CloudPoint,
            Mesh2d(point_mesh.clone()),
            MeshMaterial2d(point_mat.clone()),
            Transform::from_xyz(CENTER.x + x, CENTER.y + y, 1.0),
        ));
    }
}

// ---------------------------------------------------------------------------
// Controls
// ---------------------------------------------------------------------------

fn schrodinger_controls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut state: Option<ResMut<SchrodingerState>>,
    cloud_query: Query<Entity, With<CloudPoint>>,
    measurement_query: Query<Entity, With<MeasurementDot>>,
) {
    let Some(ref mut state) = state else { return };

    let old_n = state.n;
    let old_l = state.l;
    let old_m = state.m;

    // n: principal
    if keyboard.just_pressed(KeyCode::ArrowUp) && state.n < 4 {
        state.n += 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) && state.n > 1 {
        state.n -= 1;
        state.l = state.l.min(state.n - 1);
        state.m = state.m.clamp(-(state.l as i32), state.l as i32);
    }

    // l: azimutal
    if keyboard.just_pressed(KeyCode::ArrowRight) && state.l < state.n - 1 {
        state.l += 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) && state.l > 0 {
        state.l -= 1;
        state.m = state.m.clamp(-(state.l as i32), state.l as i32);
    }

    // m: magnético
    if keyboard.just_pressed(KeyCode::KeyM) {
        state.m = if state.m < state.l as i32 {
            state.m + 1
        } else {
            -(state.l as i32)
        };
    }

    // Detectar mudança → rebuild
    if state.n != old_n || state.l != old_l || state.m != old_m || state.needs_rebuild {
        state.needs_rebuild = false;
        state.measurements = 0;
        rebuild_cloud(
            &mut commands,
            &mut meshes,
            &mut materials,
            &cloud_query,
            &measurement_query,
            state,
        );
    }

    // Medição (colapso): clique coloca ponto em posição amostrada de |ψ|²
    if mouse_button.just_pressed(MouseButton::Left) {
        let points = quantum::sample_orbital_points(
            state.n, state.l, state.m, 1, SPATIAL_SCALE,
        );
        if let Some((x, y)) = points.first() {
            let dot_mesh = meshes.add(Circle::new(3.0));
            let dot_mat = materials.add(ColorMaterial::from_color(
                Color::srgba(1.0, 1.0, 0.2, 0.9),
            ));
            commands.spawn((
                SchrodingerEntity,
                MeasurementDot,
                Mesh2d(dot_mesh),
                MeshMaterial2d(dot_mat),
                Transform::from_xyz(CENTER.x + x, CENTER.y + y, 6.0),
            ));
            state.measurements += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// HUD
// ---------------------------------------------------------------------------

fn update_schrodinger_hud(
    state: Option<Res<SchrodingerState>>,
    mut info_query: Query<&mut Text2d, (With<SchrodingerInfoText>, Without<OrbitalLabel>)>,
    mut label_query: Query<&mut Text2d, (With<OrbitalLabel>, Without<SchrodingerInfoText>)>,
) {
    let Some(state) = state else { return };

    let sublabel = match state.l {
        0 => "s",
        1 => "p",
        2 => "d",
        3 => "f",
        _ => "?",
    };

    let energy = spectral::bohr_energy(state.n);
    let radial_nodes = state.n - state.l - 1;
    let angular_nodes = state.l;

    let info = format!(
        "ORBITAL {}{}  (m={})\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         Energia: {:.4} eV\n\
         Nos radiais: {}\n\
         Nos angulares: {}\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         CONTROLES:\n\
         [Setas Cima/Baixo] n = {} (1-4)\n\
         [Setas Esq/Dir] l = {} (0 a n-1)\n\
         [M] m = {} (-l a +l)\n\
         [Clique] Medir posicao (colapso)\n\
         Medicoes: {}\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         NUMEROS QUANTICOS\n\
         (emergem das condicoes de contorno)\n\
         n = 1,2,3,...    (principal)\n\
         l = 0,...,n-1    (azimutal)\n\
         m = -l,...,+l    (magnetico)\n\n\
         INTERPRETACAO DE BORN (1926):\n\
         |psi|^2 d^3r = probabilidade\n\
         de encontrar o eletron em d^3r\n\n\
         PRINCIPIO DE INCERTEZA:\n\
         Dx * Dp >= hbar/2\n\
         O eletron NAO esta em lugar\n\
         nenhum ate ser medido.",
        state.n, sublabel, state.m,
        energy,
        radial_nodes,
        angular_nodes,
        state.n, state.l, state.m,
        state.measurements,
    );

    for mut text in info_query.iter_mut() {
        *text = Text2d::new(info.clone());
    }

    // Orbital label
    let label = format!("Orbital {}{} (n={}, l={}, m={})", state.n, sublabel, state.n, state.l, state.m);
    for mut text in label_query.iter_mut() {
        *text = Text2d::new(label.clone());
    }
}
