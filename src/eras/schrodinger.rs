use bevy::prelude::*;

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
    commands.insert_resource(EraControls(
        "[Setas] n/l  [M] m\n[Clique] Medir (colapso)".to_string()
    ));

    commands.insert_resource(EraParameters::from_tuples(&[
        ("n", 2.0, 1.0, 4.0, 1.0),
        ("l", 1.0, 0.0, 3.0, 1.0),
    ]));

    // Limitation text
    commands.insert_resource(LimitationText(
        "NAO-RELATIVISTICO".to_string(),
        "Schrodinger usa E = p^2/(2m) + V,\nvalida so para v << c. Para H(1s),\nv/c ~ alfa ~ 1/137: erro pequeno.\nMas para ouro (Z=79): v/c ~ 0.58!\nErro de ~20% na energia. Tambem\nnao preve spin. -> Dirac (1928).".to_string(),
    ));
    commands.insert_resource(LimitationVisible(false));

    commands.insert_resource(EraEquations(
        "Eq. de Schrodinger:\n  ih*dpsi/dt = [-h^2/(2m)*nabla^2 + V]*psi\n\nSolucao H:\n  psi_nlm = R_nl(r) * Y_lm(t,f)\n\nBorn: |psi|^2 = probabilidade".to_string(),
    ));
    commands.insert_resource(EquationsVisible(false));

    commands.insert_resource(EraEvidence {
        resolves: "Numeros quanticos de condicoes de contorno\nOrbitais s, p, d com formas corretas\nIntensidades via operador dipolo".to_string(),
        fails: "Nao-relativistico (E=p^2/2m)\nSpin adicionado ad hoc (Pauli)\nNao preve antimateria".to_string(),
    });
    commands.insert_resource(EvidenceVisible(false));

    commands.insert_resource(EraQuiz(vec![
        QuestionData {
            text: "O que |psi|^2 representa na interpretacao de Born?".to_string(),
            options: vec![
                "Energia da particula".to_string(),
                "Densidade de probabilidade de posicao".to_string(),
                "Carga eletrica distribuida".to_string(),
                "Frequencia de onda".to_string(),
            ],
            correct: 1,
        },
        QuestionData {
            text: "De onde vem os numeros quanticos n,l,m em Schrodinger?".to_string(),
            options: vec![
                "Postulados arbitrarios".to_string(),
                "Dados experimentais".to_string(),
                "Condicoes de contorno da eq. de onda".to_string(),
                "Principio de incerteza".to_string(),
            ],
            correct: 2,
        },
        QuestionData {
            text: "Qual a principal limitacao da eq. de Schrodinger?".to_string(),
            options: vec![
                "Nao funciona para H".to_string(),
                "E nao-relativistica (v << c)".to_string(),
                "Nao tem solucao analitica".to_string(),
                "Ignora interacoes nucleares".to_string(),
            ],
            correct: 1,
        },
    ]));
    commands.insert_resource(QuizState::default());

    commands.insert_resource(EraPredictions(vec![
        Prediction {
            question: "O orbital 2p tem quantos lobulos?".to_string(),
            options: vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
            ],
            correct: 1,
            explanation: "O orbital p (l=1) tem 2 lobulos simetricos\nseparados por um plano nodal. Cada orientacao\n(px, py, pz) tem 2 lobulos.".to_string(),
        },
        Prediction {
            question: "Se clicarmos para 'medir' a posicao do eletron,\na nuvem de probabilidade vai:".to_string(),
            options: vec![
                "Ficar mais difusa".to_string(),
                "Colapsar para um ponto".to_string(),
                "Nao mudar".to_string(),
                "Inverter simetria".to_string(),
            ],
            correct: 1,
            explanation: "Postulado da medicao: ao medir posicao,\n|psi|^2 colapsa para autovalor observado.\nIncerteza de Heisenberg: Delta x * Delta p >= h/2.".to_string(),
        },
    ]));
    commands.insert_resource(PredictState::default());

    commands.insert_resource(EraExperiment {
        name: "Nuvem de probabilidade".to_string(),
        year: "1926".to_string(),
        apparatus: "Modelo matematico — equacao de onda".to_string(),
        steps: vec![
            ExperimentStep {
                description: "Resolver a equacao de Schrodinger para H".to_string(),
                action: "Obter |psi|^2 como densidade de probabilidade".to_string(),
            },
            ExperimentStep {
                description: "Clicar para colapsar a funcao de onda".to_string(),
                action: "Medicao localiza o eletron — principio de incerteza".to_string(),
            },
        ],
    });

    commands.insert_resource(EraDerivation(vec![
        "Equacao de onda: ih*dpsi/dt = H*psi".to_string(),
        "H = -hbar^2/(2m)*nabla^2 + V(r)".to_string(),
        "Separacao: psi(r,t,f) = R(r)*Y(t,f)*T(t)".to_string(),
        "Solucao: R_nl com Laguerre, Y_lm com Legendre".to_string(),
    ]));

    commands.insert_resource(EraNarrative(vec![
        NarrativeStep {
            text: "Observe: nuvem de probabilidade |psi|^2 — o eletron nao tem orbita definida.".to_string(),
            action_hint: "Pressione [Setas] para mudar n, l, m".to_string(),
        },
        NarrativeStep {
            text: "Clique para colapsar a funcao de onda — medicao localiza o eletron!".to_string(),
            action_hint: "Clique na nuvem para medir a posicao".to_string(),
        },
        NarrativeStep {
            text: "Equacao nao-relativistica: ignora spin e criacao de pares.".to_string(),
            action_hint: "Avance para Dirac [->]".to_string(),
        },
    ]));

    // Núcleo
    commands.spawn((
        SchrodingerEntity,
        Tooltip("H: orbital nao-relativistico".to_string()),
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
    mut export_data: ResMut<ExportableData>,
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
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         E = {:.4} eV  Nos: {}r {}a\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         [Cima/Baixo] n={} [Esq/Dir] l={}\n\
         [M] m={}  [Clique] Medir ({})\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         n=1..4 (principal)\n\
         l=0..n-1 (azimutal)\n\
         m=-l..+l (magnetico)\n\n\
         |psi|^2 = prob. (Born 1926)\n\
         Dx*Dp >= hbar/2 (Heisenberg)",
        state.n, sublabel, state.m,
        energy, radial_nodes, angular_nodes,
        state.n, state.l,
        state.m, state.measurements,
    );

    for mut text in info_query.iter_mut() {
        *text = Text2d::new(info.clone());
    }

    // Orbital label
    let label = format!("Orbital {}{} (n={}, l={}, m={})", state.n, sublabel, state.n, state.l, state.m);
    for mut text in label_query.iter_mut() {
        *text = Text2d::new(label.clone());
    }

    // Populate export data with all allowed quantum numbers for current n
    export_data.era_name = "schrodinger".to_string();
    export_data.headers = vec![
        "n".to_string(),
        "l".to_string(),
        "m".to_string(),
        "energy_eV".to_string(),
    ];
    let mut rows = Vec::new();
    for l in 0..state.n {
        for m in -(l as i32)..=(l as i32) {
            let e = spectral::bohr_energy(state.n);
            rows.push(vec![
                state.n.to_string(),
                l.to_string(),
                m.to_string(),
                format!("{:.6}", e),
            ]);
        }
    }
    export_data.rows = rows;
}
