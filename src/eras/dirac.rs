use bevy::prelude::*;
use rand::Rng;

use crate::common::{ActiveEra, SimulationState};
use crate::common::ui::{HudText, EraControls, LimitationText, LimitationVisible};
use crate::common::equations::{EraEquations, EquationsVisible};
use crate::common::quiz::{EraQuiz, QuestionData, QuizState};
use crate::common::export::ExportableData;
use crate::physics::spectral;

pub struct DiracPlugin;

impl Plugin for DiracPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ActiveEra::Dirac), setup_dirac)
            .add_systems(OnExit(ActiveEra::Dirac), cleanup_dirac)
            .add_systems(
                Update,
                (
                    pair_creation_system,
                    annihilation_system,
                    update_particles,
                    update_dirac_hud,
                )
                    .run_if(in_state(ActiveEra::Dirac))
                    .run_if(in_state(SimulationState::Running)),
            )
            .add_systems(
                Update,
                dirac_controls.run_if(in_state(ActiveEra::Dirac)),
            );
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct DiracEntity;

#[derive(Component)]
struct Electron {
    vel: Vec2,
    #[allow(dead_code)]
    spin_up: bool,
}

#[derive(Component)]
struct Positron {
    vel: Vec2,
    #[allow(dead_code)]
    spin_up: bool,
}

/// Fóton gama (da aniquilação ou para criação de pares).
#[derive(Component)]
struct GammaPhoton {
    vel: Vec2,
    energy_mev: f32,
    lifetime: f32,
}

/// Indicador visual de spin.
#[derive(Component)]
#[allow(dead_code)]
struct SpinArrow {
    follows: Entity,
    is_up: bool,
}

#[derive(Component)]
struct DiracInfoText;

#[derive(Component)]
struct EnergySpectrumText;

/// Handles pré-alocados para partículas emitidas (evita leak de handles).
#[derive(Resource)]
struct DiracParticleAssets {
    electron_mesh: Handle<Mesh>,
    electron_mat: Handle<ColorMaterial>,
    positron_mesh: Handle<Mesh>,
    positron_mat: Handle<ColorMaterial>,
    gamma_mesh: Handle<Mesh>,
    gamma_mat: Handle<ColorMaterial>,
    gamma_high_mesh: Handle<Mesh>,
    gamma_high_mat: Handle<ColorMaterial>,
    // Glow (halo) assets
    electron_glow_mesh: Handle<Mesh>,
    electron_glow_mat: Handle<ColorMaterial>,
    positron_glow_mesh: Handle<Mesh>,
    positron_glow_mat: Handle<ColorMaterial>,
    gamma_glow_mesh: Handle<Mesh>,
    gamma_glow_mat: Handle<ColorMaterial>,
    gamma_high_glow_mesh: Handle<Mesh>,
    gamma_high_glow_mat: Handle<ColorMaterial>,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct DiracState {
    pairs_created: usize,
    annihilations: usize,
    #[allow(dead_code)]
    show_sea: bool,        // Mostrar "mar de Dirac"
    selected_n: u32,
    selected_j_half: u32,  // 2j (para evitar frações): j=1/2 → 1, j=3/2 → 3
}

impl Default for DiracState {
    fn default() -> Self {
        Self {
            pairs_created: 0,
            annihilations: 0,
            show_sea: false,
            selected_n: 2,
            selected_j_half: 1, // j = 1/2
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CENTER: Vec2 = Vec2::new(-150.0, 50.0);
const ELECTRON_MASS_MEV: f32 = 0.511;
const PAIR_THRESHOLD_MEV: f32 = 2.0 * ELECTRON_MASS_MEV; // 1.022 MeV

/// Energia exata de Dirac para átomo de hidrogênio.
///
/// E_{n,j} = mc² · [1 + (Zα/(n - (j+½) + √((j+½)² - Z²α²)))²]^{-1/2}
///
/// Expandindo em potências de α²:
/// E ≈ mc² - 13.6/n² · [1 + α²/n · (1/(j+½) - 3/(4n))] eV
fn dirac_energy(n: u32, j_half: u32, z: u32) -> f64 {
    let alpha = 1.0 / 137.036_f64;
    let za = z as f64 * alpha;
    let j_plus_half = j_half as f64 / 2.0 + 0.5;
    let gamma = (j_plus_half * j_plus_half - za * za).sqrt();
    let n_prime = n as f64 - j_plus_half + gamma;
    let m_e_c2 = 0.511e6_f64; // eV

    m_e_c2 * (1.0 + (za / n_prime).powi(2)).powf(-0.5) - m_e_c2
}

// ---------------------------------------------------------------------------
// Setup & Cleanup
// ---------------------------------------------------------------------------

fn setup_dirac(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hud_query: Query<&mut Text2d, With<HudText>>,
) {
    for mut text in hud_query.iter_mut() {
        *text = Text2d::new(
            "DIRAC (1928) \u{2014} Mecanica Quantica Relativistica\n\n\
             (i*gamma^u * d_u - mc/hbar) * psi = 0\n\n\
             O spin EMERGE da algebra (nao e postulado).\n\
             Fator giromagnetico g=2 automaticamente.\n\
             Solucoes de energia negativa → POSITRON.\n\n\
             Anderson (1932): positron descoberto em\n\
             raios cosmicos. Confirmacao espetacular.\n\n\
             Estrutura fina exata: depende de j,\n\
             nao de l (coincidencia de Sommerfeld\n\
             finalmente explicada)."
        );
    }

    commands.insert_resource(DiracState::default());
    commands.insert_resource(EraControls(
        "[P] Foton gamma\n[Setas] n/j (estrutura fina)".to_string()
    ));

    // Limitation text
    commands.insert_resource(LimitationText(
        "ALEM DE DIRAC: QED".to_string(),
        "Dirac preve spin, antimateria e\nestrutura fina EXATA. Mas em 1947,\nLamb mede desvio de +1057 MHz entre\n2S(1/2) e 2P(1/2) do H: Lamb shift.\nDirac preve zero. A causa: flutuacoes\ndo vacuo quantico. -> QED (Feynman).".to_string(),
    ));
    commands.insert_resource(LimitationVisible(false));

    commands.insert_resource(EraEquations(
        "Eq. de Dirac:\n  (i*gamma^u * d_u - mc/hbar)*psi = 0\n\nSpin: J = L + S, s = 1/2\n\nEstrutura fina exata:\n  E = mc^2*[1+(Za/n')^2]^(-1/2) - mc^2".to_string(),
    ));
    commands.insert_resource(EquationsVisible(false));

    commands.insert_resource(EraQuiz(vec![
        QuestionData {
            text: "O que emerge NATURALMENTE da equacao de Dirac?".to_string(),
            options: vec![
                "Massa do eletron".to_string(),
                "Spin 1/2 e fator g=2".to_string(),
                "Numeros quanticos n,l,m".to_string(),
                "Constante de Planck".to_string(),
            ],
            correct: 1,
        },
        QuestionData {
            text: "O que as solucoes de energia negativa previram?".to_string(),
            options: vec![
                "Neutrinos".to_string(),
                "Neutrons".to_string(),
                "Positrons (antimateria)".to_string(),
                "Quarks".to_string(),
            ],
            correct: 2,
        },
        QuestionData {
            text: "Quem descobriu o positron e quando?".to_string(),
            options: vec![
                "Rutherford, 1911".to_string(),
                "Bohr, 1920".to_string(),
                "Anderson, 1932 (raios cosmicos)".to_string(),
                "Feynman, 1948".to_string(),
            ],
            correct: 2,
        },
    ]));
    commands.insert_resource(QuizState::default());

    // Pré-alocar handles para partículas emitidas
    commands.insert_resource(DiracParticleAssets {
        electron_mesh: meshes.add(Circle::new(6.0)),
        electron_mat: materials.add(ColorMaterial::from_color(
            Color::srgba(0.2, 0.5, 1.0, 0.9),
        )),
        positron_mesh: meshes.add(Circle::new(6.0)),
        positron_mat: materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 0.3, 0.2, 0.9),
        )),
        gamma_mesh: meshes.add(Circle::new(4.0)),
        gamma_mat: materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 1.0, 0.3, 0.9),
        )),
        gamma_high_mesh: meshes.add(Circle::new(5.0)),
        gamma_high_mat: materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 1.0, 0.0, 0.9),
        )),
        // Glow (halo) assets
        electron_glow_mesh: meshes.add(Circle::new(16.0)),
        electron_glow_mat: materials.add(ColorMaterial::from_color(
            Color::srgba(0.2, 0.5, 1.0, 0.15),
        )),
        positron_glow_mesh: meshes.add(Circle::new(16.0)),
        positron_glow_mat: materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 0.3, 0.2, 0.15),
        )),
        gamma_glow_mesh: meshes.add(Circle::new(12.0)),
        gamma_glow_mat: materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 1.0, 0.3, 0.18),
        )),
        gamma_high_glow_mesh: meshes.add(Circle::new(15.0)),
        gamma_high_glow_mat: materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 1.0, 0.0, 0.18),
        )),
    });

    // Diagrama de energia de Dirac (visual)
    // Gap de 2mc² entre estados positivos e negativos
    let gap_y = CENTER.y;
    let gap_height = 80.0;

    // Região de energia positiva (elétrons)
    commands.spawn((
        DiracEntity,
        Mesh2d(meshes.add(Rectangle::new(200.0, 100.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(0.2, 0.3, 0.6, 0.15),
        ))),
        Transform::from_xyz(CENTER.x, gap_y + gap_height / 2.0 + 50.0, 0.1),
    ));

    // Gap 2mc²
    commands.spawn((
        DiracEntity,
        Mesh2d(meshes.add(Rectangle::new(200.0, gap_height))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(0.1, 0.1, 0.1, 0.3),
        ))),
        Transform::from_xyz(CENTER.x, gap_y, 0.1),
    ));
    commands.spawn((
        DiracEntity,
        Text2d::new(format!("Gap: 2mc^2 = {:.3} MeV", PAIR_THRESHOLD_MEV)),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgba(0.8, 0.5, 0.5, 0.8)),
        Transform::from_xyz(CENTER.x, gap_y, 10.0),
    ));

    // Região de energia negativa (mar de Dirac / pósitrons)
    commands.spawn((
        DiracEntity,
        Mesh2d(meshes.add(Rectangle::new(200.0, 100.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(0.5, 0.2, 0.2, 0.1),
        ))),
        Transform::from_xyz(CENTER.x, gap_y - gap_height / 2.0 - 50.0, 0.1),
    ));

    // Labels
    commands.spawn((
        DiracEntity,
        Text2d::new("E > 0 (eletrons)"),
        TextFont { font_size: 11.0, ..default() },
        TextColor(Color::srgba(0.3, 0.5, 0.9, 0.7)),
        Transform::from_xyz(CENTER.x, gap_y + gap_height / 2.0 + 105.0, 10.0),
    ));
    commands.spawn((
        DiracEntity,
        Text2d::new("E < 0 (mar de Dirac / positrons)"),
        TextFont { font_size: 11.0, ..default() },
        TextColor(Color::srgba(0.9, 0.3, 0.3, 0.7)),
        Transform::from_xyz(CENTER.x, gap_y - gap_height / 2.0 - 105.0, 10.0),
    ));

    // Spin demonstration: two electrons in 1s orbital
    let e_mesh = meshes.add(Circle::new(8.0));
    let e_mat = materials.add(ColorMaterial::from_color(
        Color::srgba(0.2, 0.5, 1.0, 0.9),
    ));

    // Electron spin up
    commands.spawn((
        DiracEntity,
        Mesh2d(e_mesh.clone()),
        MeshMaterial2d(e_mat.clone()),
        Transform::from_xyz(CENTER.x - 20.0, gap_y + gap_height / 2.0 + 30.0, 2.0),
    ));
    commands.spawn((
        DiracEntity,
        Text2d::new("\u{2191} spin +1/2"),
        TextFont { font_size: 10.0, ..default() },
        TextColor(Color::srgba(0.3, 0.6, 1.0, 0.8)),
        Transform::from_xyz(CENTER.x - 20.0, gap_y + gap_height / 2.0 + 45.0, 10.0),
    ));

    // Electron spin down
    commands.spawn((
        DiracEntity,
        Mesh2d(e_mesh),
        MeshMaterial2d(e_mat),
        Transform::from_xyz(CENTER.x + 20.0, gap_y + gap_height / 2.0 + 30.0, 2.0),
    ));
    commands.spawn((
        DiracEntity,
        Text2d::new("\u{2193} spin -1/2"),
        TextFont { font_size: 10.0, ..default() },
        TextColor(Color::srgba(0.3, 0.6, 1.0, 0.8)),
        Transform::from_xyz(CENTER.x + 20.0, gap_y + gap_height / 2.0 + 45.0, 10.0),
    ));
    commands.spawn((
        DiracEntity,
        Text2d::new("Pauli: max 2 e- por orbital"),
        TextFont { font_size: 10.0, ..default() },
        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.7)),
        Transform::from_xyz(CENTER.x, gap_y + gap_height / 2.0 + 60.0, 10.0),
    ));

    // Info text
    commands.spawn((
        DiracEntity,
        DiracInfoText,
        Text2d::new(""),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgba(0.8, 0.8, 0.5, 0.9)),
        Transform::from_xyz(300.0, 250.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));

    // Energy spectrum text
    commands.spawn((
        DiracEntity,
        EnergySpectrumText,
        Text2d::new(""),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgba(0.7, 0.8, 0.9, 0.9)),
        Transform::from_xyz(300.0, -50.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));
}

fn cleanup_dirac(
    mut commands: Commands,
    query: Query<Entity, With<DiracEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<DiracState>();
    commands.remove_resource::<DiracParticleAssets>();
    commands.remove_resource::<LimitationText>();
    commands.remove_resource::<EraEquations>();
    commands.remove_resource::<EraQuiz>();
}

// ---------------------------------------------------------------------------
// Pair creation: γ → e⁻ + e⁺
// Requires photon energy > 2mₑc² = 1.022 MeV
// ---------------------------------------------------------------------------

fn pair_creation_system(
    mut commands: Commands,
    mut state: Option<ResMut<DiracState>>,
    assets: Option<Res<DiracParticleAssets>>,
    gammas: Query<(Entity, &GammaPhoton, &Transform), (Without<Electron>, Without<Positron>)>,
) {
    let Some(ref mut state) = state else { return };
    let Some(assets) = assets else { return };

    for (entity, gamma, transform) in gammas.iter() {
        if gamma.energy_mev >= PAIR_THRESHOLD_MEV && gamma.lifetime < 1.5 {
            let pos = transform.translation.truncate();
            let mut rng = rand::rng();
            let spread = rng.random_range(30.0_f32..80.0);

            let e_vel = Vec2::new(80.0, spread);
            let e_spin = rng.random_bool(0.5);
            commands.spawn((
                DiracEntity,
                Electron {
                    vel: e_vel,
                    spin_up: e_spin,
                },
                Mesh2d(assets.electron_mesh.clone()),
                MeshMaterial2d(assets.electron_mat.clone()),
                Transform::from_xyz(pos.x, pos.y, 3.0),
            ));
            // Electron glow
            commands.spawn((
                DiracEntity,
                Electron {
                    vel: e_vel,
                    spin_up: e_spin,
                },
                Mesh2d(assets.electron_glow_mesh.clone()),
                MeshMaterial2d(assets.electron_glow_mat.clone()),
                Transform::from_xyz(pos.x, pos.y, 2.5),
            ));

            let p_vel = Vec2::new(80.0, -spread);
            let p_spin = !rng.random_bool(0.5);
            commands.spawn((
                DiracEntity,
                Positron {
                    vel: p_vel,
                    spin_up: p_spin,
                },
                Mesh2d(assets.positron_mesh.clone()),
                MeshMaterial2d(assets.positron_mat.clone()),
                Transform::from_xyz(pos.x, pos.y, 3.0),
            ));
            // Positron glow
            commands.spawn((
                DiracEntity,
                Positron {
                    vel: p_vel,
                    spin_up: p_spin,
                },
                Mesh2d(assets.positron_glow_mesh.clone()),
                MeshMaterial2d(assets.positron_glow_mat.clone()),
                Transform::from_xyz(pos.x, pos.y, 2.5),
            ));

            commands.entity(entity).despawn();
            state.pairs_created += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Annihilation: e⁻ + e⁺ → 2γ
// ---------------------------------------------------------------------------

fn annihilation_system(
    mut commands: Commands,
    mut state: Option<ResMut<DiracState>>,
    assets: Option<Res<DiracParticleAssets>>,
    electrons: Query<(Entity, &Transform), (With<Electron>, Without<Positron>)>,
    positrons: Query<(Entity, &Transform), (With<Positron>, Without<Electron>)>,
) {
    let Some(ref mut state) = state else { return };
    let Some(assets) = assets else { return };

    for (e_entity, e_transform) in electrons.iter() {
        for (p_entity, p_transform) in positrons.iter() {
            let dist = e_transform
                .translation
                .truncate()
                .distance(p_transform.translation.truncate());

            if dist < 15.0 {
                let mid = (e_transform.translation + p_transform.translation) / 2.0;

                for dir in [Vec2::new(150.0, 0.0), Vec2::new(-150.0, 0.0)] {
                    commands.spawn((
                        DiracEntity,
                        GammaPhoton {
                            vel: dir,
                            energy_mev: ELECTRON_MASS_MEV,
                            lifetime: 3.0,
                        },
                        Mesh2d(assets.gamma_mesh.clone()),
                        MeshMaterial2d(assets.gamma_mat.clone()),
                        Transform::from_xyz(mid.x, mid.y, 4.0),
                    ));
                    // Gamma glow
                    commands.spawn((
                        DiracEntity,
                        GammaPhoton {
                            vel: dir,
                            energy_mev: ELECTRON_MASS_MEV,
                            lifetime: 3.0,
                        },
                        Mesh2d(assets.gamma_glow_mesh.clone()),
                        MeshMaterial2d(assets.gamma_glow_mat.clone()),
                        Transform::from_xyz(mid.x, mid.y, 3.5),
                    ));
                }

                commands.entity(e_entity).despawn();
                commands.entity(p_entity).despawn();
                state.annihilations += 1;
                return;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Update all moving particles
// ---------------------------------------------------------------------------

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut electrons: Query<(Entity, &Electron, &mut Transform), Without<Positron>>,
    mut positrons: Query<(Entity, &Positron, &mut Transform), Without<Electron>>,
    mut gammas: Query<(Entity, &mut GammaPhoton, &mut Transform), (Without<Electron>, Without<Positron>)>,
) {
    let dt = time.delta_secs();

    for (entity, electron, mut transform) in electrons.iter_mut() {
        transform.translation.x += electron.vel.x * dt;
        transform.translation.y += electron.vel.y * dt;
        if transform.translation.x.abs() > 600.0 || transform.translation.y.abs() > 400.0 {
            commands.entity(entity).despawn();
        }
    }

    for (entity, positron, mut transform) in positrons.iter_mut() {
        transform.translation.x += positron.vel.x * dt;
        transform.translation.y += positron.vel.y * dt;
        if transform.translation.x.abs() > 600.0 || transform.translation.y.abs() > 400.0 {
            commands.entity(entity).despawn();
        }
    }

    for (entity, mut gamma, mut transform) in gammas.iter_mut() {
        transform.translation.x += gamma.vel.x * dt;
        transform.translation.y += gamma.vel.y * dt;
        gamma.lifetime -= dt;
        if gamma.lifetime <= 0.0 || transform.translation.x.abs() > 600.0 {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Controls
// ---------------------------------------------------------------------------

fn dirac_controls(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: Option<ResMut<DiracState>>,
    assets: Option<Res<DiracParticleAssets>>,
) {
    let Some(ref mut state) = state else { return };

    // Criar fóton gama de alta energia (para criação de pares)
    if keyboard.just_pressed(KeyCode::KeyP) {
        if let Some(ref assets) = assets {
            commands.spawn((
                DiracEntity,
                GammaPhoton {
                    vel: Vec2::new(120.0, 0.0),
                    energy_mev: 2.0, // > 1.022 MeV threshold
                    lifetime: 3.0,
                },
                Mesh2d(assets.gamma_high_mesh.clone()),
                MeshMaterial2d(assets.gamma_high_mat.clone()),
                Transform::from_xyz(-400.0, CENTER.y, 4.0),
            ));
            // Gamma high-energy glow
            commands.spawn((
                DiracEntity,
                GammaPhoton {
                    vel: Vec2::new(120.0, 0.0),
                    energy_mev: 2.0,
                    lifetime: 3.0,
                },
                Mesh2d(assets.gamma_high_glow_mesh.clone()),
                MeshMaterial2d(assets.gamma_high_glow_mat.clone()),
                Transform::from_xyz(-400.0, CENTER.y, 3.5),
            ));
        }
    }

    // Ajustar n para comparação de estrutura fina
    if keyboard.just_pressed(KeyCode::ArrowUp) && state.selected_n < 4 {
        state.selected_n += 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) && state.selected_n > 1 {
        state.selected_n -= 1;
    }

    // Ajustar j
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        let max_j = 2 * state.selected_n - 1; // 2j max = 2n-1
        if state.selected_j_half < max_j {
            state.selected_j_half += 2;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) && state.selected_j_half > 1 {
        state.selected_j_half -= 2;
    }
}

// ---------------------------------------------------------------------------
// HUD
// ---------------------------------------------------------------------------

fn update_dirac_hud(
    state: Option<Res<DiracState>>,
    mut info_query: Query<&mut Text2d, (With<DiracInfoText>, Without<EnergySpectrumText>)>,
    mut spectrum_query: Query<&mut Text2d, (With<EnergySpectrumText>, Without<DiracInfoText>)>,
    mut export_data: ResMut<ExportableData>,
) {
    let Some(state) = state else { return };

    let info = format!(
        "CONTROLES\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         [P] Foton gamma (>1.022 MeV)\n\
         [Setas] n={}, j={}/2\n\n\
         Pares: {}  Aniquilacoes: {}\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         g -> e- + e+ (E>1.022 MeV)\n\
         e- + e+ -> 2g (0.511 MeV cada)\n\n\
         Spin emerge da equacao: g=2\n\
         QED: g=2.002319 (Schwinger 48)",
        state.selected_n, state.selected_j_half,
        state.pairs_created, state.annihilations,
    );

    for mut text in info_query.iter_mut() {
        *text = Text2d::new(info.clone());
    }

    // Energy spectrum: Dirac vs Bohr vs Sommerfeld
    let n = state.selected_n;
    let j_half = state.selected_j_half.min(2 * n - 1);
    let j_str = format!("{}/2", j_half);

    let e_bohr = spectral::bohr_energy(n);
    let e_dirac = dirac_energy(n, j_half, 1); // Z=1 (Hydrogen)

    let fine_split_mev = (e_dirac - e_bohr) * 1000.0;

    let mut spectrum = format!(
        "ESTRUTURA FINA (n={}, j={})\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         Bohr:       {:.6} eV\n\
         Dirac:      {:.6} eV\n\
         Diferenca:  {:.4} meV\n\n",
        n, j_str,
        e_bohr, e_dirac, fine_split_mev,
    );

    // Comparação de todos os sub-níveis para este n
    spectrum.push_str("Sub-niveis de Dirac:\n");
    let mut j_h = 1u32;
    while j_h <= 2 * n - 1 {
        let e = dirac_energy(n, j_h, 1);
        let marker = if j_h == j_half { " <--" } else { "" };
        spectrum.push_str(&format!(
            "  j={}/2: {:.6} eV{}\n",
            j_h, e, marker
        ));
        j_h += 2;
    }

    spectrum.push_str(
        "\nDesdobra por j (nao l).\n\
         Lamb shift ~1057 MHz: requer QED"
    );

    for mut text in spectrum_query.iter_mut() {
        *text = Text2d::new(spectrum.clone());
    }

    // Populate export data with fine structure for all sub-levels up to n=4
    export_data.era_name = "dirac".to_string();
    export_data.headers = vec![
        "n".to_string(),
        "j".to_string(),
        "energy_eV".to_string(),
    ];
    let mut rows = Vec::new();
    for n_val in 1..=4u32 {
        let mut j_h = 1u32;
        while j_h <= 2 * n_val - 1 {
            let e = dirac_energy(n_val, j_h, 1);
            rows.push(vec![
                n_val.to_string(),
                format!("{}/2", j_h),
                format!("{:.6}", e),
            ]);
            j_h += 2;
        }
    }
    export_data.rows = rows;
}
