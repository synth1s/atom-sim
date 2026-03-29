use bevy::prelude::*;
use rand::Rng;
use crate::common::{ActiveEra, SimulationState};
use crate::common::ui::{HudText, EraControls, LimitationText, LimitationVisible};
use crate::common::equations::{EraEquations, EquationsVisible};
use crate::common::evidence::{EraEvidence, EvidenceVisible};
use crate::common::quiz::{EraQuiz, QuestionData, QuizState};
use crate::common::predict::{EraPredictions, Prediction, PredictState};
use crate::common::sandbox::EraParameters;
use crate::common::narrative::{EraNarrative, NarrativeStep};
use crate::common::experiment::{EraExperiment, ExperimentStep};
use crate::common::walkthrough::EraDerivation;
use crate::common::tooltip::Tooltip;
use crate::common::export::ExportableData;

pub struct RutherfordPlugin;

impl Plugin for RutherfordPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ActiveEra::Rutherford), setup_rutherford)
            .add_systems(OnExit(ActiveEra::Rutherford), cleanup_rutherford)
            .add_systems(
                Update,
                (
                    update_alpha_particles,
                    fire_alpha_particles,
                    classical_collapse_system,
                    update_rutherford_hud,
                )
                    .run_if(in_state(ActiveEra::Rutherford))
                    .run_if(in_state(SimulationState::Running)),
            )
            .add_systems(
                Update,
                rutherford_controls.run_if(in_state(ActiveEra::Rutherford)),
            );
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct RutherfordEntity;

/// O núcleo atômico — ponto denso de carga positiva.
/// Rutherford provou: raio ~10⁻¹⁵ m vs átomo ~10⁻¹⁰ m (100.000× menor).
#[derive(Component)]
struct Nucleus {
    #[allow(dead_code)]
    charge_z: f32, // Número atômico (Au = 79)
}

/// Partícula alfa no experimento de Geiger-Marsden.
/// He²⁺: carga +2e, massa ~4u, energia ~5.5 MeV.
#[derive(Component)]
struct AlphaParticle {
    vel: Vec2,
    trail: Vec<Vec2>, // Rastro da trajetória
}

/// Elétron orbitando no modelo de Rutherford.
#[derive(Component)]
struct OrbitalElectron {
    angle: f32,
    radius: f32,
    angular_vel: f32,
    // Para demonstração de colapso:
    #[allow(dead_code)]
    radiating: bool,
    #[allow(dead_code)]
    energy: f32,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct ExperimentConfig {
    alpha_speed: f32,
    firing: bool,
    fire_timer: f32,
    show_collapse: bool,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            alpha_speed: 400.0,
            firing: true,
            fire_timer: 0.0,
            show_collapse: false,
        }
    }
}

/// Histograma de deflexões (buckets de 10° de 0° a 180°).
#[derive(Resource)]
struct DeflectionHistogram {
    bins: [usize; 18], // 0-10, 10-20, ..., 170-180
    total: usize,
    backscattered: usize, // >90°
}

impl Default for DeflectionHistogram {
    fn default() -> Self {
        Self {
            bins: [0; 18],
            total: 0,
            backscattered: 0,
        }
    }
}

#[derive(Component)]
struct RutherfordInfoText;

#[derive(Component)]
struct HistogramBar(usize); // index do bin

/// Handles pré-alocados para partículas emitidas (evita leak de handles).
#[derive(Resource)]
struct RutherfordParticleAssets {
    alpha_mesh: Handle<Mesh>,
    alpha_mat: Handle<ColorMaterial>,
    trail_mesh: Handle<Mesh>,
    trail_mat_gray: Handle<ColorMaterial>,
    trail_mat_orange: Handle<ColorMaterial>,
    trail_mat_red: Handle<ColorMaterial>,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const NUCLEUS_POS: Vec2 = Vec2::new(-100.0, 0.0);
const NUCLEUS_VISUAL_RADIUS: f32 = 5.0; // Visualmente pequeno (real: 10⁵× menor que átomo)
const ATOM_VISUAL_RADIUS: f32 = 150.0;  // Raio do "átomo" (nuvem de elétrons)

// Coulomb scattering: k = Z₁Z₂e²/(4πε₀)
// Em unidades de simulação, usamos um fator de força efetivo:
const COULOMB_K: f32 = 50000.0; // Z₁=2 (alfa) × Z₂=79 (Au), normalizado

const ALPHA_SPAWN_X: f32 = 350.0;
const ALPHA_DESPAWN_X: f32 = -450.0;

const HISTOGRAM_X: f32 = 300.0;
const HISTOGRAM_Y: f32 = -50.0;

// ---------------------------------------------------------------------------
// Setup & Cleanup
// ---------------------------------------------------------------------------

fn setup_rutherford(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hud_query: Query<&mut Text2d, With<HudText>>,
) {
    for mut text in hud_query.iter_mut() {
        *text = Text2d::new(
            "RUTHERFORD (1911) \u{2014} O Nucleo Atomico\n\n\
             Geiger-Marsden dispararam alfas contra folha\n\
             de ouro. Maioria passou, mas ~1/8000 voltou!\n\
             Conclusao: nucleo denso e positivo, 10^5x\n\
             menor que o atomo.\n\n\
             LIMITACAO: Eletrons orbitando radiam energia\n\
             (Larmor). Colapso em ~16 picossegundos!"
        );
    }

    commands.init_resource::<ExperimentConfig>();
    commands.insert_resource(DeflectionHistogram::default());
    commands.insert_resource(EraControls(
        "[Setas] Velocidade alfa\n[F] Toggle disparos\n[C] Colapso classico".to_string()
    ));

    commands.insert_resource(EraParameters::from_tuples(&[
        ("Vel. alfa", 400.0, 100.0, 800.0, 20.0),
        ("Forca Coulomb", 50000.0, 10000.0, 100000.0, 5000.0),
    ]));

    // Pré-alocar handles para partículas emitidas
    commands.insert_resource(RutherfordParticleAssets {
        alpha_mesh: meshes.add(Circle::new(3.5)),
        alpha_mat: materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 0.85, 0.2, 0.9),
        )),
        trail_mesh: meshes.add(Rectangle::new(6.0, 1.0)),
        trail_mat_gray: materials.add(ColorMaterial::from_color(
            Color::srgba(0.5, 0.5, 0.5, 0.15),
        )),
        trail_mat_orange: materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 0.7, 0.2, 0.3),
        )),
        trail_mat_red: materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 0.2, 0.2, 0.4),
        )),
    });

    // Limitation text
    commands.insert_resource(LimitationText(
        "PARADOXO DO COLAPSO".to_string(),
        "Pela eletrodinamica classica (Larmor),\num eletron orbitando irradia energia a\nP = e^2*a^2/(6*pi*eps0*c^3).\nPara o Hidrogenio: colapso em ~16 ps.\nO atomo NAO deveria existir.\n-> Bohr postulou orbitas estaveis (1913).".to_string(),
    ));
    commands.insert_resource(LimitationVisible(false));

    commands.insert_resource(EraEquations(
        "Espalhamento Coulomb:\n  ds/dO = (Z1*Z2*e^2/(16*pi*e0*E))^2\n         * 1/sin^4(t/2)\n\nRadiacao de Larmor:\n  P = e^2*a^2 / (6*pi*eps0*c^3)".to_string(),
    ));
    commands.insert_resource(EquationsVisible(false));

    commands.insert_resource(EraEvidence {
        resolves: "Retroespalhamento: nucleo denso\nds/dO proporcional 1/sin^4(t/2)\nTamanho nuclear: ~10^-15 m".to_string(),
        fails: "Colapso em ~16 ps (Larmor)\nEspectro continuo previsto\nEstabilidade atomica".to_string(),
    });
    commands.insert_resource(EvidenceVisible(false));

    commands.insert_resource(EraQuiz(vec![
        QuestionData {
            text: "Por que ~1/8000 alfas retroespalham?".to_string(),
            options: vec![
                "Eletrons as repelem".to_string(),
                "Nucleo denso e positivo".to_string(),
                "Campo magnetico".to_string(),
                "Folha muito espessa".to_string(),
            ],
            correct: 1,
        },
        QuestionData {
            text: "Qual o tamanho do nucleo vs atomo?".to_string(),
            options: vec![
                "Metade do atomo".to_string(),
                "1/10 do atomo".to_string(),
                "~10^-5 do atomo (1 fm vs 0.1 nm)".to_string(),
                "Mesmo tamanho".to_string(),
            ],
            correct: 2,
        },
        QuestionData {
            text: "Qual o paradoxo classico deste modelo?".to_string(),
            options: vec![
                "Eletrons nao existem".to_string(),
                "Nucleo deveria se desintegrar".to_string(),
                "Eletron irradiaria e colapsaria em ~16 ps".to_string(),
                "Atomos deveriam ser transparentes".to_string(),
            ],
            correct: 2,
        },
    ]));
    commands.insert_resource(QuizState::default());

    commands.insert_resource(EraPredictions(vec![
        Prediction {
            question: "Se dispararmos alfa a 45 graus do nucleo,\no angulo de saida sera:".to_string(),
            options: vec![
                "~45 graus".to_string(),
                "~90 graus".to_string(),
                "~135 graus".to_string(),
                "Depende da distancia ao nucleo".to_string(),
            ],
            correct: 3,
            explanation: "O espalhamento Coulombiano depende do\nparametro de impacto b, nao do angulo incidente.".to_string(),
        },
        Prediction {
            question: "Se substituirmos ouro (Z=79) por aluminio\n(Z=13), os retroespalhamentos vao:".to_string(),
            options: vec![
                "Aumentar".to_string(),
                "Diminuir muito".to_string(),
                "Ficar iguais".to_string(),
                "Desaparecer completamente".to_string(),
            ],
            correct: 1,
            explanation: "sigma ~ Z^2. Aluminio tem Z=13 vs Au Z=79.\nRetroespalhamento cai por fator ~(13/79)^2 = ~2.7%.".to_string(),
        },
    ]));
    commands.insert_resource(PredictState::default());

    commands.insert_resource(EraExperiment {
        name: "Folha de Ouro de Geiger-Marsden".to_string(),
        year: "1909-1913".to_string(),
        apparatus: "Fonte de Ra-222 + folha Au + detector ZnS".to_string(),
        steps: vec![
            ExperimentStep {
                description: "Observar as alfas sendo disparadas".to_string(),
                action: "Maioria passa com deflexao minima".to_string(),
            },
            ExperimentStep {
                description: "Contar retroespalhamentos no histograma".to_string(),
                action: "~1/8000 volta a >90 graus".to_string(),
            },
            ExperimentStep {
                description: "Ativar colapso classico [C]".to_string(),
                action: "O atomo colapsa em ~16 ps -- impossivel!".to_string(),
            },
        ],
    });

    commands.insert_resource(EraDerivation(vec![
        "Secao de choque Rutherford: dsigma/dOmega = (Z1*Z2*e^2/(4E))^2 / sin^4(theta/2)".to_string(),
        "Distancia de maxima aproximacao: d = Z1*Z2*e^2/(4*pi*eps0*E)".to_string(),
        "Fracao retroespalhada ~ (Z*e^2/(4E))^2 * pi * n * t".to_string(),
    ]));

    commands.insert_resource(EraNarrative(vec![
        NarrativeStep {
            text: "Observe: alfas sendo disparadas contra folha de ouro (Au Z=79).".to_string(),
            action_hint: "Veja a maioria atravessando sem desvio".to_string(),
        },
        NarrativeStep {
            text: "~1/8000 alfas retroespalham! Deve haver algo denso e positivo.".to_string(),
            action_hint: "Pressione [C] para ver o colapso classico".to_string(),
        },
        NarrativeStep {
            text: "Eletron orbitando irradia energia (Larmor): colapso em ~16 ps!".to_string(),
            action_hint: "Avance para Bohr [->]".to_string(),
        },
    ]));

    // Núcleo — ponto vermelho minúsculo
    commands.spawn((
        RutherfordEntity,
        Nucleus { charge_z: 79.0 },
        Tooltip("Nucleo Au (Z=79), r~7 fm".to_string()),
        Mesh2d(meshes.add(Circle::new(NUCLEUS_VISUAL_RADIUS))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 0.2, 0.15, 1.0),
        ))),
        Transform::from_xyz(NUCLEUS_POS.x, NUCLEUS_POS.y, 2.0),
    ));

    // Atom outline (nuvem de elétrons)
    commands.spawn((
        RutherfordEntity,
        Mesh2d(meshes.add(Circle::new(ATOM_VISUAL_RADIUS))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(0.3, 0.3, 0.5, 0.08),
        ))),
        Transform::from_xyz(NUCLEUS_POS.x, NUCLEUS_POS.y, 0.1),
    ));

    // Labels
    commands.spawn((
        RutherfordEntity,
        Text2d::new("Nucleo (+79e)"),
        TextFont { font_size: 11.0, ..default() },
        TextColor(Color::srgba(1.0, 0.3, 0.2, 0.8)),
        Transform::from_xyz(NUCLEUS_POS.x, NUCLEUS_POS.y - 15.0, 10.0),
    ));

    // Orbiting electrons (3 elétrons)
    let electron_mesh = meshes.add(Circle::new(4.0));
    let electron_mat = materials.add(ColorMaterial::from_color(
        Color::srgba(0.3, 0.6, 1.0, 0.9),
    ));

    for i in 0..3 {
        let angle = (i as f32 / 3.0) * std::f32::consts::TAU;
        let radius = 60.0 + i as f32 * 30.0;
        commands.spawn((
            RutherfordEntity,
            OrbitalElectron {
                angle,
                radius,
                angular_vel: 3.0 / (radius / 60.0), // Mais rápido perto do núcleo
                radiating: false,
                energy: -13.6 / ((i + 1) as f32 * (i + 1) as f32), // eV (Bohr preview)
            },
            Mesh2d(electron_mesh.clone()),
            MeshMaterial2d(electron_mat.clone()),
            Transform::from_xyz(
                NUCLEUS_POS.x + angle.cos() * radius,
                NUCLEUS_POS.y + angle.sin() * radius,
                1.5,
            ),
        ));
    }

    // Histogram bars
    let bar_mesh = meshes.add(Rectangle::new(8.0, 1.0));
    let bar_mat = materials.add(ColorMaterial::from_color(
        Color::srgba(0.2, 0.8, 0.3, 0.8),
    ));
    for i in 0..18 {
        commands.spawn((
            RutherfordEntity,
            HistogramBar(i),
            Tooltip(format!("Bin {}-{}deg", i * 10, (i + 1) * 10)),
            Mesh2d(bar_mesh.clone()),
            MeshMaterial2d(bar_mat.clone()),
            Transform::from_xyz(
                HISTOGRAM_X + i as f32 * 12.0,
                HISTOGRAM_Y,
                5.0,
            ),
        ));
    }

    // Histogram label
    commands.spawn((
        RutherfordEntity,
        Text2d::new("Angulo de deflexao (0-180)"),
        TextFont { font_size: 11.0, ..default() },
        TextColor(Color::srgba(0.6, 0.8, 0.6, 0.7)),
        Transform::from_xyz(HISTOGRAM_X + 90.0, HISTOGRAM_Y - 20.0, 10.0),
    ));

    // Info text
    commands.spawn((
        RutherfordEntity,
        RutherfordInfoText,
        Text2d::new(""),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgba(0.8, 0.8, 0.6, 0.9)),
        Transform::from_xyz(300.0, 200.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));
}

fn cleanup_rutherford(
    mut commands: Commands,
    query: Query<Entity, With<RutherfordEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<ExperimentConfig>();
    commands.remove_resource::<DeflectionHistogram>();
    commands.remove_resource::<RutherfordParticleAssets>();
    commands.remove_resource::<LimitationText>();
    commands.remove_resource::<EraEquations>();
    commands.remove_resource::<EraQuiz>();
    commands.remove_resource::<EraPredictions>();
    commands.remove_resource::<EraParameters>();
    commands.remove_resource::<EraNarrative>();
    commands.remove_resource::<EraEvidence>();
    commands.remove_resource::<EraExperiment>();
    commands.remove_resource::<EraDerivation>();
}

// ---------------------------------------------------------------------------
// Alpha particle scattering — Coulomb repulsion from nucleus
//
// Rutherford scattering formula:
//   dσ/dΩ = (Z₁Z₂e²/(16πε₀E))² · 1/sin⁴(θ/2)
//
// Impact parameter vs angle:
//   b = (a/2)·cot(θ/2)
//   where a = Z₁Z₂e²/(4πε₀E) = distance of closest approach (head-on)
//
// For 5.5 MeV α on Au (Z=79): d_min ~ 41 fm ≈ 10⁻⁴ × atomic radius
// ---------------------------------------------------------------------------

fn fire_alpha_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut config: Option<ResMut<ExperimentConfig>>,
    assets: Option<Res<RutherfordParticleAssets>>,
) {
    let Some(ref mut config) = config else { return };
    if !config.firing { return; }
    let Some(assets) = assets else { return };

    let dt = time.delta_secs();
    config.fire_timer += dt;

    if config.fire_timer > 0.12 {
        config.fire_timer = 0.0;

        let mut rng = rand::rng();
        let impact_param = rng.random_range(-ATOM_VISUAL_RADIUS..ATOM_VISUAL_RADIUS);

        commands.spawn((
            RutherfordEntity,
            AlphaParticle {
                vel: Vec2::new(-config.alpha_speed, 0.0),
                trail: Vec::new(),
            },
            Mesh2d(assets.alpha_mesh.clone()),
            MeshMaterial2d(assets.alpha_mat.clone()),
            Transform::from_xyz(ALPHA_SPAWN_X, NUCLEUS_POS.y + impact_param, 3.0),
        ));
    }
}

fn update_alpha_particles(
    mut commands: Commands,
    time: Res<Time>,
    assets: Option<Res<RutherfordParticleAssets>>,
    mut histogram: Option<ResMut<DeflectionHistogram>>,
    mut query: Query<(Entity, &mut AlphaParticle, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let Some(ref mut histogram) = histogram else { return };

    for (entity, mut alpha, mut transform) in query.iter_mut() {
        let pos = transform.translation.truncate();

        // Força de Coulomb: F = k·Z₁Z₂/(r²) na direção radial (repulsiva)
        let to_nucleus = NUCLEUS_POS - pos;
        let dist_sq = to_nucleus.length_squared().max(100.0); // Evitar singularidade
        let dist = dist_sq.sqrt();
        let force_dir = to_nucleus / dist;

        // F = -k/r² (repulsivo: alfa e núcleo são ambos positivos)
        let force_magnitude = -COULOMB_K / dist_sq;
        let acceleration = force_dir * force_magnitude;

        // Velocity Verlet
        alpha.vel += acceleration * dt;
        transform.translation.x += alpha.vel.x * dt;
        transform.translation.y += alpha.vel.y * dt;

        // Trail (salvar posição a cada N frames)
        if alpha.trail.len() < 200 {
            alpha.trail.push(pos);
        }

        // Detectar se saiu da área
        let out_of_bounds = transform.translation.x < ALPHA_DESPAWN_X
            || transform.translation.x > ALPHA_SPAWN_X + 100.0
            || transform.translation.y.abs() > 400.0;

        if out_of_bounds {
            // Calcular ângulo de deflexão
            let initial_dir = Vec2::new(-1.0, 0.0); // Direção inicial (vindo da direita)
            let final_dir = alpha.vel.normalize_or_zero();
            let dot = initial_dir.dot(final_dir).clamp(-1.0, 1.0);
            let angle_rad = dot.acos();
            let angle_deg = angle_rad.to_degrees();

            // Registrar no histograma
            let bin = (angle_deg / 10.0).min(17.0) as usize;
            histogram.bins[bin] += 1;
            histogram.total += 1;
            if angle_deg > 90.0 {
                histogram.backscattered += 1;
            }

            // Desenhar trail antes de remover (usando handles pré-alocados)
            if let Some(ref assets) = assets {
                let trail_mat = if angle_deg > 90.0 {
                    assets.trail_mat_red.clone()
                } else if angle_deg > 30.0 {
                    assets.trail_mat_orange.clone()
                } else {
                    assets.trail_mat_gray.clone()
                };

                for window in alpha.trail.windows(3) {
                    let mid = (window[0] + window[2]) / 2.0;
                    let dir = window[2] - window[0];
                    let len = dir.length();
                    if len > 1.0 {
                        let angle = dir.y.atan2(dir.x);
                        commands.spawn((
                            RutherfordEntity,
                            Mesh2d(assets.trail_mesh.clone()),
                            MeshMaterial2d(trail_mat.clone()),
                            Transform::from_xyz(mid.x, mid.y, 0.5)
                                .with_rotation(Quat::from_rotation_z(angle))
                                .with_scale(Vec3::new(len / 6.0, 1.0, 1.0)),
                        ));
                    }
                }
            }

            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Classical collapse — Larmor radiation
//
// An orbiting electron is an accelerated charge. By Maxwell's
// electrodynamics, it radiates at rate:
//   P = e²a²/(6πε₀c³)
//
// For electron at Bohr radius: P ~ 4.6×10⁻⁸ W
// Total energy: E ~ -13.6 eV
// Collapse time: t ~ 1.6×10⁻¹¹ s = 16 picoseconds
//
// The atom should NOT exist. This contradiction is fatal.
// ---------------------------------------------------------------------------

fn classical_collapse_system(
    time: Res<Time>,
    config: Option<Res<ExperimentConfig>>,
    mut query: Query<(&mut OrbitalElectron, &mut Transform)>,
) {
    let Some(config) = config else { return };
    let dt = time.delta_secs();

    for (mut electron, mut transform) in query.iter_mut() {
        if config.show_collapse {
            // Elétron espirala para o núcleo perdendo energia (Larmor radiation)
            electron.radiating = true;
            // Taxa de perda de raio proporcional à potência irradiada
            // Em escala de simulação: raio diminui visivelmente
            let decay_rate = 30.0; // px/s (acelerado para visualização)
            electron.radius = (electron.radius - decay_rate * dt).max(NUCLEUS_VISUAL_RADIUS);

            // Frequência orbital aumenta à medida que raio diminui
            electron.angular_vel = 3.0 * (60.0 / electron.radius.max(5.0));
        }

        // Atualizar posição orbital
        electron.angle += electron.angular_vel * dt;
        transform.translation.x = NUCLEUS_POS.x + electron.angle.cos() * electron.radius;
        transform.translation.y = NUCLEUS_POS.y + electron.angle.sin() * electron.radius;
    }
}

// ---------------------------------------------------------------------------
// Controls & HUD
// ---------------------------------------------------------------------------

fn rutherford_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: Option<ResMut<ExperimentConfig>>,
) {
    let Some(ref mut config) = config else { return };

    if keyboard.just_pressed(KeyCode::KeyF) {
        config.firing = !config.firing;
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        config.show_collapse = !config.show_collapse;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        config.alpha_speed = (config.alpha_speed + 5.0).min(800.0);
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        config.alpha_speed = (config.alpha_speed - 5.0).max(100.0);
    }
}

fn update_rutherford_hud(
    histogram: Option<Res<DeflectionHistogram>>,
    config: Option<Res<ExperimentConfig>>,
    mut info_query: Query<&mut Text2d, With<RutherfordInfoText>>,
    mut bar_query: Query<(&HistogramBar, &mut Transform), Without<RutherfordInfoText>>,
    mut export_data: ResMut<ExportableData>,
) {
    let Some(histogram) = histogram else { return };
    let Some(config) = config else { return };

    // Atualizar barras do histograma
    let max_count = histogram.bins.iter().copied().max().unwrap_or(1).max(1);
    for (bar, mut transform) in bar_query.iter_mut() {
        let count = histogram.bins[bar.0];
        let height = (count as f32 / max_count as f32) * 100.0;
        transform.scale.y = height.max(1.0);
        transform.translation.y = HISTOGRAM_Y + height / 2.0;
    }

    // Atualizar texto
    let backscatter_pct = if histogram.total > 0 {
        histogram.backscattered as f32 / histogram.total as f32 * 100.0
    } else {
        0.0
    };

    let collapse_text = if config.show_collapse {
        "[C] Colapso classico: ATIVO\n\
         Eletrons espiralam para o nucleo!\n\
         P = e^2*a^2/(6*pi*eps0*c^3)\n\
         Tempo de colapso: ~16 picossegundos"
    } else {
        "[C] Ativar colapso classico (Larmor)"
    };

    let info = format!(
        "EXPERIMENTO DE GEIGER-MARSDEN\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         Alfas disparadas: {}\n\
         Retroespalhadas (>90): {} ({:.2}%)\n\
         Real (Geiger-Marsden): ~0.0125%\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         Velocidade alfa: {:.0} [Setas]\n\
         [F] Toggle disparos\n\n\
         Formula de Rutherford:\n\
         ds/dO = (Z1*Z2*e^2/(16*pi*e0*E))^2\n\
                 * 1/sin^4(t/2)\n\n\
         Distancia minima (frontal):\n\
         d_min ~ 41 fm (5.5 MeV, Au)\n\
         = 10^-4 x raio atomico\n\n\
         {}\n",
        histogram.total,
        histogram.backscattered, backscatter_pct,
        config.alpha_speed,
        collapse_text,
    );

    for mut text in info_query.iter_mut() {
        *text = Text2d::new(info.clone());
    }

    // Populate export data
    export_data.era_name = "rutherford".to_string();
    export_data.headers = vec![
        "bin_start_deg".to_string(),
        "bin_end_deg".to_string(),
        "count".to_string(),
    ];
    export_data.rows = (0..18)
        .map(|i| {
            vec![
                (i * 10).to_string(),
                ((i + 1) * 10).to_string(),
                histogram.bins[i].to_string(),
            ]
        })
        .collect();
}
