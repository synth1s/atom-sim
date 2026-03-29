use bevy::prelude::*;
use std::f32::consts::TAU;

use crate::common::{ActiveEra, SimulationState};
use crate::common::ui::{HudText, EraControls, LimitationText, LimitationVisible};
use crate::common::equations::{EraEquations, EquationsVisible};
use crate::common::quiz::{EraQuiz, QuestionData, QuizState};

pub struct DeBrogliePlugin;

impl Plugin for DeBrogliePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ActiveEra::DeBroglie), setup_de_broglie)
            .add_systems(OnExit(ActiveEra::DeBroglie), cleanup_de_broglie)
            .add_systems(
                Update,
                (
                    animate_standing_wave,
                    double_slit_simulation,
                    update_de_broglie_hud,
                )
                    .run_if(in_state(ActiveEra::DeBroglie))
                    .run_if(in_state(SimulationState::Running)),
            )
            .add_systems(
                Update,
                de_broglie_controls.run_if(in_state(ActiveEra::DeBroglie)),
            );
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct DeBroglieEntity;

/// Ponto na onda estacionária ao redor da órbita.
#[derive(Component)]
struct WavePoint {
    orbit_angle: f32,     // Posição angular fixa na órbita
    #[allow(dead_code)]
    base_radius: f32,     // Raio da órbita
}

/// Partícula na simulação de fenda dupla.
#[derive(Component)]
struct SlitParticle {
    vel: Vec2,
    #[allow(dead_code)]
    phase: f32,
}

/// Ponto de detecção na tela (acumula padrão de difração).
#[derive(Component)]
struct DetectionDot;

#[derive(Component)]
struct DeBroglieInfoText;

#[derive(Component)]
struct SlitInfoText;

/// Handles pré-alocados para partículas emitidas (evita leak de handles).
#[derive(Resource)]
struct DeBroglieParticleAssets {
    slit_particle_mesh: Handle<Mesh>,
    slit_particle_mat: Handle<ColorMaterial>,
    detection_mesh: Handle<Mesh>,
    detection_mat: Handle<ColorMaterial>,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct DeBroglieState {
    n: f32,               // Número quântico (permite fracionário para demonstrar instabilidade)
    #[allow(dead_code)]
    wave_time: f32,       // Tempo para animação da onda
    slit_active: bool,    // Fenda dupla ativa
    slit_fire_timer: f32,
    detections: usize,
}

impl Default for DeBroglieState {
    fn default() -> Self {
        Self {
            n: 3.0,
            wave_time: 0.0,
            slit_active: false,
            slit_fire_timer: 0.0,
            detections: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const NUCLEUS_POS: Vec2 = Vec2::new(-250.0, 50.0);
const ORBIT_SCALE: f32 = 18.0;
const WAVE_POINTS: usize = 120; // Pontos na onda circular
const WAVE_AMPLITUDE: f32 = 15.0; // Amplitude da onda

// Fenda dupla
const SLIT_X: f32 = 200.0;
const SLIT_Y: f32 = 50.0;
const SLIT_GAP: f32 = 30.0;      // Distância entre fendas
const SLIT_WIDTH: f32 = 8.0;     // Largura de cada fenda
const SCREEN_X: f32 = 450.0;     // Posição da tela de detecção

// ---------------------------------------------------------------------------
// Setup & Cleanup
// ---------------------------------------------------------------------------

fn setup_de_broglie(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hud_query: Query<&mut Text2d, With<HudText>>,
) {
    for mut text in hud_query.iter_mut() {
        *text = Text2d::new(
            "DE BROGLIE (1924) \u{2014} Dualidade Onda-Particula\n\n\
             Tese de doutorado: lambda = h/p\n\
             Se luz (onda) tem propriedade de particula,\n\
             entao particulas tem propriedade de onda.\n\n\
             Quantizacao de Bohr reinterpretada:\n\
             2*pi*r = n*lambda (onda estacionaria)\n\n\
             Confirmado: Davisson-Germer (1927)\n\
             lambda medido = 1.65 A, previsto = 1.67 A\n\n\
             LIMITACAO: Nao fornece equacao de onda.\n\
             Nao define o que oscila."
        );
    }

    commands.insert_resource(DeBroglieState::default());
    commands.insert_resource(EraControls(
        "[Setas] n (0.5 em 0.5)\n[Q,W,R] n=1,2,4  [F] n=2.5\n[D] Fenda dupla".to_string()
    ));

    // Limitation text
    commands.insert_resource(LimitationText(
        "ONDA SEM EQUACAO".to_string(),
        "De Broglie: lambda = h/p = h/(m*v).\nPara H (n=1): lambda = 0.332 nm,\ncircunferencia da orbita = 0.332 nm.\nOnda estacionaria explica quantizacao!\nMas COMO a onda evolui no tempo?\nFalta equacao de movimento. -> Schrodinger.".to_string(),
    ));
    commands.insert_resource(LimitationVisible(false));

    commands.insert_resource(EraEquations(
        "Onda de materia:\n  lambda = h/p = h/(m*v)\n\nOnda estacionaria:\n  2*pi*r = n*lambda\n\nDispersao: w = hbar*k^2/(2m)".to_string(),
    ));
    commands.insert_resource(EquationsVisible(false));

    commands.insert_resource(EraQuiz(vec![
        QuestionData {
            text: "Qual a relacao de de Broglie?".to_string(),
            options: vec![
                "E = mc^2".to_string(),
                "lambda = h/p".to_string(),
                "F = ma".to_string(),
                "E = hv (Planck)".to_string(),
            ],
            correct: 1,
        },
        QuestionData {
            text: "O que Davisson-Germer confirmou em 1927?".to_string(),
            options: vec![
                "Eletrons tem massa".to_string(),
                "Difracao de eletrons: particulas tem propriedade de onda".to_string(),
                "Fotons tem massa".to_string(),
                "Nucleo e composto de protons".to_string(),
            ],
            correct: 1,
        },
        QuestionData {
            text: "Como de Broglie reinterpretou a quantizacao de Bohr?".to_string(),
            options: vec![
                "Eletrons saltam aleatoriamente".to_string(),
                "Orbitas sao elipticas".to_string(),
                "Onda estacionaria: 2*pi*r = n*lambda".to_string(),
                "Eletrons sao particulas puntuais".to_string(),
            ],
            correct: 2,
        },
    ]));
    commands.insert_resource(QuizState::default());

    // Pré-alocar handles para partículas da fenda dupla
    commands.insert_resource(DeBroglieParticleAssets {
        slit_particle_mesh: meshes.add(Circle::new(2.0)),
        slit_particle_mat: materials.add(ColorMaterial::from_color(
            Color::srgba(0.3, 0.6, 1.0, 0.8),
        )),
        detection_mesh: meshes.add(Circle::new(1.5)),
        detection_mat: materials.add(ColorMaterial::from_color(
            Color::srgba(0.4, 0.8, 1.0, 0.7),
        )),
    });

    // Núcleo
    commands.spawn((
        DeBroglieEntity,
        Mesh2d(meshes.add(Circle::new(5.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 0.3, 0.2, 1.0),
        ))),
        Transform::from_xyz(NUCLEUS_POS.x, NUCLEUS_POS.y, 2.0),
    ));

    // Pontos da onda estacionária
    let point_mesh = meshes.add(Circle::new(2.5));
    let point_mat = materials.add(ColorMaterial::from_color(
        Color::srgba(0.3, 0.7, 1.0, 0.9),
    ));

    let n = 3.0_f32;
    let r = n * n * ORBIT_SCALE;

    for i in 0..WAVE_POINTS {
        let angle = (i as f32 / WAVE_POINTS as f32) * TAU;
        commands.spawn((
            DeBroglieEntity,
            WavePoint {
                orbit_angle: angle,
                base_radius: r,
            },
            Mesh2d(point_mesh.clone()),
            MeshMaterial2d(point_mat.clone()),
            Transform::from_xyz(
                NUCLEUS_POS.x + angle.cos() * r,
                NUCLEUS_POS.y + angle.sin() * r,
                1.0,
            ),
        ));
    }

    // Fenda dupla — barreira
    let wall_color = Color::srgba(0.5, 0.5, 0.6, 0.7);
    let wall_height = 200.0;
    let slit_center = SLIT_Y;

    // Parede superior (acima da fenda superior)
    commands.spawn((
        DeBroglieEntity,
        Mesh2d(meshes.add(Rectangle::new(4.0, wall_height / 2.0 - SLIT_GAP - SLIT_WIDTH))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(wall_color))),
        Transform::from_xyz(SLIT_X, slit_center + wall_height / 4.0 + SLIT_GAP / 2.0 + SLIT_WIDTH / 2.0, 1.0),
    ));
    // Parede do meio (entre fendas)
    commands.spawn((
        DeBroglieEntity,
        Mesh2d(meshes.add(Rectangle::new(4.0, SLIT_GAP - SLIT_WIDTH))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(wall_color))),
        Transform::from_xyz(SLIT_X, slit_center, 1.0),
    ));
    // Parede inferior
    commands.spawn((
        DeBroglieEntity,
        Mesh2d(meshes.add(Rectangle::new(4.0, wall_height / 2.0 - SLIT_GAP - SLIT_WIDTH))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(wall_color))),
        Transform::from_xyz(SLIT_X, slit_center - wall_height / 4.0 - SLIT_GAP / 2.0 - SLIT_WIDTH / 2.0, 1.0),
    ));

    // Tela de detecção
    commands.spawn((
        DeBroglieEntity,
        Mesh2d(meshes.add(Rectangle::new(2.0, wall_height))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(0.3, 0.3, 0.3, 0.4),
        ))),
        Transform::from_xyz(SCREEN_X, slit_center, 0.5),
    ));

    // Labels
    commands.spawn((
        DeBroglieEntity,
        Text2d::new("Fenda dupla"),
        TextFont { font_size: 11.0, ..default() },
        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.7)),
        Transform::from_xyz(SLIT_X, slit_center + wall_height / 2.0 + 15.0, 10.0),
    ));
    commands.spawn((
        DeBroglieEntity,
        Text2d::new("Tela"),
        TextFont { font_size: 11.0, ..default() },
        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.7)),
        Transform::from_xyz(SCREEN_X, slit_center + wall_height / 2.0 + 15.0, 10.0),
    ));

    // Info texts
    commands.spawn((
        DeBroglieEntity,
        DeBroglieInfoText,
        Text2d::new(""),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgba(0.8, 0.8, 0.5, 0.9)),
        Transform::from_xyz(-250.0, -200.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));
    commands.spawn((
        DeBroglieEntity,
        SlitInfoText,
        Text2d::new("[D] Iniciar fenda dupla"),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgba(0.7, 0.8, 0.7, 0.9)),
        Transform::from_xyz(300.0, -200.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));
}

fn cleanup_de_broglie(
    mut commands: Commands,
    query: Query<Entity, With<DeBroglieEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<DeBroglieState>();
    commands.remove_resource::<DeBroglieParticleAssets>();
    commands.remove_resource::<LimitationText>();
    commands.remove_resource::<EraEquations>();
    commands.remove_resource::<EraQuiz>();
}

// ---------------------------------------------------------------------------
// Standing wave animation
//
// De Broglie's key insight: 2πr = nλ
// The circumference of Bohr's orbit contains exactly n wavelengths.
// For non-integer n, the wave doesn't close → destructive interference.
//
// λ = h/p = h/(mₑv). For Bohr orbit: λ = 2πa₀n/1 = 2πr/n
// ---------------------------------------------------------------------------

fn animate_standing_wave(
    time: Res<Time>,
    state: Option<Res<DeBroglieState>>,
    mut query: Query<(&WavePoint, &mut Transform)>,
) {
    let Some(state) = state else { return };
    let t = time.elapsed_secs();

    let n = state.n;
    let r = n * n * ORBIT_SCALE;

    for (point, mut transform) in query.iter_mut() {
        let angle = point.orbit_angle;

        // Onda estacionária: amplitude = A·sin(n·θ)·cos(ωt)
        // Para n inteiro, a onda fecha sobre si mesma.
        // Para n fracionário, há interferência destrutiva.
        let wave_value = (n * angle).sin() * (t * 3.0).cos();

        // Amplitude normal à órbita (radial)
        let radial_offset = wave_value * WAVE_AMPLITUDE;
        // Fade para n fracionário (interferência destrutiva)
        let n_frac = n - n.floor();
        let stability = if n_frac < 0.05 || n_frac > 0.95 {
            1.0 // Estável (n quase inteiro)
        } else {
            // Interferência destrutiva: amplitude reduzida
            0.3 + 0.7 * (1.0 - (0.5 - n_frac).abs() * 2.0).max(0.0)
        };

        let final_r = r + radial_offset * stability;

        transform.translation.x = NUCLEUS_POS.x + angle.cos() * final_r;
        transform.translation.y = NUCLEUS_POS.y + angle.sin() * final_r;
    }
}

// ---------------------------------------------------------------------------
// Double slit simulation
//
// Electrons fired one at a time through double slit.
// Each electron is detected at a single point on the screen,
// but over many electrons, an interference pattern emerges.
// (Tonomura et al., 1989 — single electron interference)
//
// Pattern: I(y) = cos²(π·d·y/(λ·L)) where d = slit spacing,
//          L = slit-to-screen distance
// ---------------------------------------------------------------------------

fn double_slit_simulation(
    mut commands: Commands,
    time: Res<Time>,
    mut state: Option<ResMut<DeBroglieState>>,
    assets: Option<Res<DeBroglieParticleAssets>>,
    mut particles: Query<(Entity, &mut SlitParticle, &mut Transform)>,
) {
    let Some(ref mut state) = state else { return };
    if !state.slit_active { return; }
    let Some(assets) = assets else { return };

    let dt = time.delta_secs();
    state.slit_fire_timer += dt;

    // Emitir elétrons
    if state.slit_fire_timer > 0.08 {
        state.slit_fire_timer = 0.0;

        let mut rng = rand::rng();
        let y_start = SLIT_Y + rng.random_range(-5.0_f32..5.0);

        commands.spawn((
            DeBroglieEntity,
            SlitParticle {
                vel: Vec2::new(200.0, 0.0),
                phase: rng.random_range(0.0_f32..TAU),
            },
            Mesh2d(assets.slit_particle_mesh.clone()),
            MeshMaterial2d(assets.slit_particle_mat.clone()),
            Transform::from_xyz(SLIT_X - 100.0, y_start, 3.0),
        ));
    }

    // Atualizar partículas
    for (entity, particle, mut transform) in particles.iter_mut() {
        transform.translation.x += particle.vel.x * dt;
        transform.translation.y += particle.vel.y * dt;

        // Ao passar pela fenda, aplicar difração
        if transform.translation.x > SLIT_X - 2.0 && transform.translation.x < SLIT_X + 2.0 {
            let y = transform.translation.y - SLIT_Y;
            // Checar se passa por uma das fendas
            let in_upper = (y - SLIT_GAP / 2.0).abs() < SLIT_WIDTH / 2.0;
            let in_lower = (y + SLIT_GAP / 2.0).abs() < SLIT_WIDTH / 2.0;
            if !in_upper && !in_lower {
                commands.entity(entity).despawn();
                continue;
            }
        }

        // Ao chegar na tela: detectar com padrão de interferência
        if transform.translation.x >= SCREEN_X {
            // Padrão de interferência: probabilidade ∝ cos²(π·d·y/(λ·L))
            let y_screen = transform.translation.y - SLIT_Y;
            let lambda_eff = 15.0; // λ efetivo para visualização
            let l_dist = SCREEN_X - SLIT_X;
            let phase = std::f32::consts::PI * SLIT_GAP * y_screen / (lambda_eff * l_dist);
            let prob = phase.cos().powi(2);

            // Aceitar/rejeitar baseado na probabilidade
            let mut rng = rand::rng();
            if rng.random_range(0.0_f32..1.0) < prob {
                // Detectado — criar ponto permanente
                commands.spawn((
                    DeBroglieEntity,
                    DetectionDot,
                    Mesh2d(assets.detection_mesh.clone()),
                    MeshMaterial2d(assets.detection_mat.clone()),
                    Transform::from_xyz(
                        SCREEN_X + rng.random_range(1.0_f32..8.0),
                        transform.translation.y,
                        2.0,
                    ),
                ));
                state.detections += 1;
            }

            commands.entity(entity).despawn();
        }

        // Timeout
        if transform.translation.x > SCREEN_X + 50.0 {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Controls
// ---------------------------------------------------------------------------

fn de_broglie_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: Option<ResMut<DeBroglieState>>,
) {
    let Some(ref mut state) = state else { return };

    // n contínuo (permite fracionário)
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        state.n = (state.n + 0.5).min(6.0);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        state.n = (state.n - 0.5).max(0.5);
    }

    // Inteiros exatos
    if keyboard.just_pressed(KeyCode::KeyQ) { state.n = 1.0; }
    if keyboard.just_pressed(KeyCode::KeyW) { state.n = 2.0; }
    if keyboard.just_pressed(KeyCode::KeyR) { state.n = 4.0; }

    // Fracionários para demonstrar instabilidade
    if keyboard.just_pressed(KeyCode::KeyF) { state.n = 2.5; }

    // Toggle fenda dupla
    if keyboard.just_pressed(KeyCode::KeyD) {
        state.slit_active = !state.slit_active;
    }
}

use rand::Rng;

// ---------------------------------------------------------------------------
// HUD
// ---------------------------------------------------------------------------

fn update_de_broglie_hud(
    state: Option<Res<DeBroglieState>>,
    mut info_query: Query<&mut Text2d, (With<DeBroglieInfoText>, Without<SlitInfoText>)>,
    mut slit_query: Query<&mut Text2d, (With<SlitInfoText>, Without<DeBroglieInfoText>)>,
) {
    let Some(state) = state else { return };

    let n = state.n;
    let is_integer = (n - n.round()).abs() < 0.05;
    let stability = if is_integer { "ESTAVEL (onda fecha)" } else { "INSTAVEL (interferencia destrutiva)" };

    let info = format!(
        "ONDA DE MATERIA\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         lambda = h/p = h/(m*v)\n\
         n = {:.1} comprimentos de onda\n\
         2*pi*r = n*lambda\n\n\
         Estado: {}\n\n\
         [Setas] Ajustar n (0.5 em 0.5)\n\
         [Q,W,R] n = 1, 2, 4 (inteiros)\n\
         [F] n = 2.5 (fracionario)\n\n\
         Davisson-Germer (1927):\n\
         e- de 54 eV em cristal Ni\n\
         lambda medido: 1.65 A\n\
         lambda previsto: 1.67 A",
        n, stability,
    );

    for mut text in info_query.iter_mut() {
        *text = Text2d::new(info.clone());
    }

    let slit_info = if state.slit_active {
        format!(
            "FENDA DUPLA (Tonomura 1989)\n\
             Eletrons um a um formam padrao\n\
             de interferencia!\n\
             Deteccoes: {}\n\n\
             [D] Parar",
            state.detections
        )
    } else {
        "[D] Iniciar fenda dupla\n\
         (eletrons um a um)"
            .to_string()
    };

    for mut text in slit_query.iter_mut() {
        *text = Text2d::new(slit_info.clone());
    }
}
