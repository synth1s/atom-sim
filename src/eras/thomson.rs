use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::TAU;

use crate::common::{ActiveEra, SimulationState};
use crate::common::ui::{HudText, EraControls, LimitationText, LimitationVisible};
use crate::common::equations::{EraEquations, EquationsVisible};
use crate::common::evidence::{EraEvidence, EvidenceVisible};
use crate::common::quiz::{EraQuiz, QuestionData, QuizState};
use crate::common::sandbox::EraParameters;
use crate::common::narrative::{EraNarrative, NarrativeStep};
use crate::physics::coulomb;

pub struct ThomsonPlugin;

impl Plugin for ThomsonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ActiveEra::Thomson), setup_thomson)
            .add_systems(OnExit(ActiveEra::Thomson), cleanup_thomson)
            .add_systems(
                Update,
                (
                    electron_oscillation,
                    cathode_ray_simulation,
                    alpha_particle_test,
                    update_thomson_hud,
                    thomson_input,
                )
                    .run_if(in_state(ActiveEra::Thomson))
                    .run_if(in_state(SimulationState::Running)),
            )
            .add_systems(
                Update,
                thomson_controls.run_if(in_state(ActiveEra::Thomson)),
            );
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct ThomsonEntity;

/// Elétron embutido no "pudim" positivo de Thomson.
/// Oscila harmonicamente em torno de sua posição de equilíbrio.
#[derive(Component)]
struct PlumElectron {
    equilibrium: Vec2,  // posição de equilíbrio dentro do átomo
    displacement: Vec2,  // deslocamento atual
    velocity: Vec2,      // velocidade de oscilação
}

/// O "pudim" positivo — esfera de carga uniformemente distribuída.
#[derive(Component)]
struct PositivePudding;

/// Partícula no feixe de raios catódicos.
#[derive(Component)]
struct CathodeRayParticle {
    vel: Vec2,
}

/// Partícula alfa para teste de espalhamento.
#[derive(Component)]
struct AlphaParticle {
    vel: Vec2,
}

/// Configuração do tubo de raios catódicos.
#[derive(Resource)]
struct CathodeRayConfig {
    e_field: f32,     // Campo elétrico (V/m normalizado)
    b_field: f32,     // Campo magnético (T normalizado)
    firing: bool,     // Se está emitindo partículas
    #[allow(dead_code)]
    fire_timer: f32,  // Timer entre emissões
}

impl Default for CathodeRayConfig {
    fn default() -> Self {
        Self {
            e_field: 100.0,
            b_field: 1.0,
            firing: true,
            fire_timer: 0.0,
        }
    }
}

/// Estado do teste de partículas alfa.
#[derive(Resource)]
struct AlphaTestState {
    active: bool,
    fire_timer: f32,
    passed: usize,
    deflected: usize,
}

impl Default for AlphaTestState {
    fn default() -> Self {
        Self {
            active: false,
            fire_timer: 0.0,
            passed: 0,
            deflected: 0,
        }
    }
}

#[derive(Component)]
struct ThomsonInfoText;

#[derive(Component)]
struct CathodeRayInfoText;

/// Handles pré-alocados para partículas emitidas (evita leak de handles).
#[derive(Resource)]
struct ThomsonParticleAssets {
    cathode_mesh: Handle<Mesh>,
    cathode_mat: Handle<ColorMaterial>,
    alpha_mesh: Handle<Mesh>,
    alpha_mat: Handle<ColorMaterial>,
}

// ---------------------------------------------------------------------------
// Constants — Thomson model parameters
// ---------------------------------------------------------------------------

const PUDDING_RADIUS: f32 = 120.0;
const PUDDING_CENTER: Vec2 = Vec2::new(-250.0, 50.0);
const N_ELECTRONS: usize = 8;
const ELECTRON_RADIUS: f32 = 6.0;
// Oscilação harmônica: ω² = k/m, usando k proporcional a (e²·n)/(3ε₀·R³)
// Em unidades de simulação, usamos um spring_constant efetivo:
const SPRING_CONSTANT: f32 = 800.0; // ω² para oscilação visual

// Tubo de raios catódicos — posição na tela
const TUBE_LEFT: f32 = 100.0;
const TUBE_RIGHT: f32 = 550.0;
const TUBE_CENTER_Y: f32 = 50.0;
const PLATE_START_X: f32 = 250.0;
const PLATE_END_X: f32 = 400.0;
const PLATE_HALF_GAP: f32 = 60.0;

// ---------------------------------------------------------------------------
// Setup & Cleanup
// ---------------------------------------------------------------------------

fn setup_thomson(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hud_query: Query<&mut Text2d, With<HudText>>,
) {
    for mut text in hud_query.iter_mut() {
        *text = Text2d::new(
            "THOMSON (1897) \u{2014} Descoberta do Eletron\n\n\
             J.J. Thomson mediu e/m dos raios catodicos:\n\
             e/m = 1.759 x 10^11 C/kg (~1000x maior que H+).\n\
             Particulas universais: o ELETRON.\n\n\
             Modelo: esfera positiva uniforme com eletrons\n\
             embutidos (pudim de passas).\n\n\
             LIMITACAO: Particulas alfa passam direto.\n\
             Nenhum retroespalhamento previsto."
        );
    }

    commands.init_resource::<CathodeRayConfig>();
    commands.init_resource::<AlphaTestState>();
    commands.insert_resource(EraControls(
        "[Setas] Campos E/B\n[F] Toggle feixe\n[A] Teste alfa".to_string()
    ));

    commands.insert_resource(EraParameters::from_tuples(&[
        ("Campo E", 100.0, 0.0, 300.0, 10.0),
        ("Campo B", 1.0, 0.0, 10.0, 0.5),
    ]));

    // Limitation text
    commands.insert_resource(LimitationText(
        "PUDIM SEM NUCLEO".to_string(),
        "Thomson distribui carga + uniforme\ncom eletrons imersos (plum pudding).\nRutherford (1911): alfas a 5.5 MeV\nricocheteiam a >90 graus. Probabili-\ndade no modelo de Thomson: ~1e-3500.\nMassa esta concentrada. -> Rutherford.".to_string(),
    ));
    commands.insert_resource(LimitationVisible(false));

    commands.insert_resource(EraEquations(
        "Medicao e/m:\n  v = E/B\n  e/m = 2*y*E / (B^2 * L^2)\n\nOscilacao no pudim:\n  w = sqrt(n_e*e^2 / (3*eps0*m_e*R^3))".to_string(),
    ));
    commands.insert_resource(EquationsVisible(false));

    commands.insert_resource(EraEvidence {
        resolves: "Eletron universal: e/m = 1.76e11 C/kg\nRaios catodicos independentes do gas\nCarga subatomica".to_string(),
        fails: "Retroespalhamento alfa (~1/8000)\nEspectros discretos\nTeorema de Earnshaw".to_string(),
    });
    commands.insert_resource(EvidenceVisible(false));

    commands.insert_resource(EraQuiz(vec![
        QuestionData {
            text: "Como Thomson mediu e/m do eletron?".to_string(),
            options: vec![
                "Pesando atomos em balanca".to_string(),
                "Cruzando campos E e B no tubo catodico".to_string(),
                "Contando centelhas em tela".to_string(),
                "Medindo temperatura do gas".to_string(),
            ],
            correct: 1,
        },
        QuestionData {
            text: "Qual o modelo atomico de Thomson?".to_string(),
            options: vec![
                "Nucleo denso com eletrons orbitando".to_string(),
                "Esfera positiva com eletrons embutidos (pudim)".to_string(),
                "Onda de probabilidade".to_string(),
                "Esferas solidas identicas".to_string(),
            ],
            correct: 1,
        },
        QuestionData {
            text: "O que o valor e/m = 1.76e11 C/kg revelou?".to_string(),
            options: vec![
                "O atomo e eletricamente neutro".to_string(),
                "Particula ~1836x mais leve que H: subestrutura atomica".to_string(),
                "A luz e uma onda".to_string(),
                "O nucleo e positivo".to_string(),
            ],
            correct: 1,
        },
    ]));
    commands.insert_resource(QuizState::default());

    commands.insert_resource(EraNarrative(vec![
        NarrativeStep {
            text: "Observe: eletrons embutidos numa esfera positiva (pudim de passas).".to_string(),
            action_hint: "Veja os eletrons oscilando dentro do atomo".to_string(),
        },
        NarrativeStep {
            text: "Thomson mediu e/m no tubo catodico — eletron e universal!".to_string(),
            action_hint: "Pressione [A] para disparar alfas contra o atomo".to_string(),
        },
        NarrativeStep {
            text: "Alfas atravessam sem desvio: o modelo nao tem nucleo denso.".to_string(),
            action_hint: "Avance para Rutherford [->]".to_string(),
        },
    ]));

    // Pré-alocar handles para partículas emitidas
    commands.insert_resource(ThomsonParticleAssets {
        cathode_mesh: meshes.add(Circle::new(3.0)),
        cathode_mat: materials.add(ColorMaterial::from_color(
            Color::srgba(0.3, 0.6, 1.0, 0.9),
        )),
        alpha_mesh: meshes.add(Circle::new(5.0)),
        alpha_mat: materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 0.8, 0.2, 0.9),
        )),
    });

    // --- Pudim de passas (lado esquerdo) ---

    // Esfera positiva
    commands.spawn((
        ThomsonEntity,
        PositivePudding,
        Mesh2d(meshes.add(Circle::new(PUDDING_RADIUS))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(0.8, 0.7, 0.2, 0.25), // Amarelo semi-transparente
        ))),
        Transform::from_xyz(PUDDING_CENTER.x, PUDDING_CENTER.y, 0.5),
    ));

    // Label
    commands.spawn((
        ThomsonEntity,
        Text2d::new("+ Carga positiva\n  uniforme"),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgba(0.8, 0.7, 0.2, 0.7)),
        Transform::from_xyz(PUDDING_CENTER.x, PUDDING_CENTER.y - PUDDING_RADIUS - 20.0, 10.0),
    ));

    // Elétrons distribuídos dentro do pudim
    let electron_mesh = meshes.add(Circle::new(ELECTRON_RADIUS));
    let electron_mat = materials.add(ColorMaterial::from_color(
        Color::srgba(0.2, 0.5, 1.0, 0.95), // Azul brilhante
    ));

    let mut rng = rand::rng();
    for i in 0..N_ELECTRONS {
        let angle = (i as f32 / N_ELECTRONS as f32) * TAU;
        let r = PUDDING_RADIUS * 0.5;
        let eq = Vec2::new(angle.cos() * r, angle.sin() * r);

        // Deslocamento inicial aleatório
        let disp = Vec2::new(
            rng.random_range(-15.0..15.0),
            rng.random_range(-15.0..15.0),
        );

        commands.spawn((
            ThomsonEntity,
            PlumElectron {
                equilibrium: eq,
                displacement: disp,
                velocity: Vec2::ZERO,
            },
            Mesh2d(electron_mesh.clone()),
            MeshMaterial2d(electron_mat.clone()),
            Transform::from_xyz(
                PUDDING_CENTER.x + eq.x + disp.x,
                PUDDING_CENTER.y + eq.y + disp.y,
                1.0,
            ),
        ));
    }

    // --- Tubo de raios catódicos (lado direito) ---

    // Tubo outline
    let tube_color = Color::srgba(0.4, 0.4, 0.5, 0.4);
    // Topo e base do tubo
    for y_sign in [-1.0_f32, 1.0] {
        commands.spawn((
            ThomsonEntity,
            Mesh2d(meshes.add(Rectangle::new(TUBE_RIGHT - TUBE_LEFT, 2.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(tube_color))),
            Transform::from_xyz(
                (TUBE_LEFT + TUBE_RIGHT) / 2.0,
                TUBE_CENTER_Y + y_sign * PLATE_HALF_GAP * 1.5,
                0.0,
            ),
        ));
    }

    // Placas do campo elétrico
    let plate_color = Color::srgba(0.8, 0.3, 0.3, 0.6);
    for y_sign in [-1.0_f32, 1.0] {
        commands.spawn((
            ThomsonEntity,
            Mesh2d(meshes.add(Rectangle::new(PLATE_END_X - PLATE_START_X, 3.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(plate_color))),
            Transform::from_xyz(
                (PLATE_START_X + PLATE_END_X) / 2.0,
                TUBE_CENTER_Y + y_sign * PLATE_HALF_GAP,
                1.0,
            ),
        ));
    }

    // Labels
    commands.spawn((
        ThomsonEntity,
        Text2d::new("Catodo"),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.7)),
        Transform::from_xyz(TUBE_LEFT - 10.0, TUBE_CENTER_Y + PLATE_HALF_GAP * 1.5 + 15.0, 10.0),
    ));
    commands.spawn((
        ThomsonEntity,
        Text2d::new("Tela"),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.7)),
        Transform::from_xyz(TUBE_RIGHT + 15.0, TUBE_CENTER_Y + PLATE_HALF_GAP * 1.5 + 15.0, 10.0),
    ));
    commands.spawn((
        ThomsonEntity,
        Text2d::new("E"),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgba(0.8, 0.3, 0.3, 0.8)),
        Transform::from_xyz(PLATE_START_X - 15.0, TUBE_CENTER_Y, 10.0),
    ));

    // Info panel
    commands.spawn((
        ThomsonEntity,
        ThomsonInfoText,
        Text2d::new(""),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgba(0.7, 0.85, 0.7, 0.9)),
        Transform::from_xyz(200.0, -180.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));

    // Cathode ray info
    commands.spawn((
        ThomsonEntity,
        CathodeRayInfoText,
        Text2d::new(""),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgba(0.8, 0.8, 0.5, 0.9)),
        Transform::from_xyz(350.0, -80.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));
}

fn cleanup_thomson(
    mut commands: Commands,
    query: Query<Entity, With<ThomsonEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<CathodeRayConfig>();
    commands.remove_resource::<AlphaTestState>();
    commands.remove_resource::<ThomsonParticleAssets>();
    commands.remove_resource::<LimitationText>();
    commands.remove_resource::<EraEquations>();
    commands.remove_resource::<EraQuiz>();
    commands.remove_resource::<EraParameters>();
    commands.remove_resource::<EraNarrative>();
    commands.remove_resource::<EraEvidence>();
}

// ---------------------------------------------------------------------------
// Electron oscillation — harmonic motion in Thomson's positive sphere
//
// F = -(nₑe²)/(3ε₀R³) · r  →  mẍ = -k·x  →  ω = √(k/m)
// This is simple harmonic motion: the "plum pudding" acts as a spring.
// ---------------------------------------------------------------------------

fn electron_oscillation(
    time: Res<Time>,
    mut query: Query<(&mut PlumElectron, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (mut electron, mut transform) in query.iter_mut() {
        // F = -k * displacement (força restauradora harmônica)
        let disp = electron.displacement;
        let acceleration = -SPRING_CONSTANT * disp;

        // Velocity Verlet integration
        electron.velocity += acceleration * dt;
        let vel = electron.velocity;
        electron.displacement += vel * dt;

        // Damping leve para estabilidade visual
        electron.velocity *= 0.999;

        // Atualizar posição visual
        transform.translation.x = PUDDING_CENTER.x + electron.equilibrium.x + electron.displacement.x;
        transform.translation.y = PUDDING_CENTER.y + electron.equilibrium.y + electron.displacement.y;
    }
}

// ---------------------------------------------------------------------------
// Cathode ray tube — electrons deflected by E and B fields
//
// Thomson's key measurement:
//   1. Balance E and B: eE = evB → v = E/B
//   2. From E deflection alone: y = eEL²/(2mv²)
//   3. Therefore: e/m = 2yE/(B²L²)
//   Result: e/m ~ 1.759×10¹¹ C/kg (mass ~1/1836 of hydrogen)
// ---------------------------------------------------------------------------

fn cathode_ray_simulation(
    mut commands: Commands,
    time: Res<Time>,
    config: Option<Res<CathodeRayConfig>>,
    assets: Option<Res<ThomsonParticleAssets>>,
    mut particles: Query<(Entity, &mut CathodeRayParticle, &mut Transform), Without<AlphaParticle>>,
) {
    let Some(config) = config else { return };
    let Some(assets) = assets else { return };
    let dt = time.delta_secs();

    // Emitir novas partículas
    if config.firing {
        // Emitir a cada ~0.05s
        let rate = 0.05;
        let elapsed = time.elapsed_secs();
        if (elapsed % rate) < dt {
            let mut rng = rand::rng();
            let y_spread = rng.random_range(-3.0..3.0);

            commands.spawn((
                ThomsonEntity,
                CathodeRayParticle {
                    vel: Vec2::new(300.0, 0.0), // Velocidade horizontal
                },
                Mesh2d(assets.cathode_mesh.clone()),
                MeshMaterial2d(assets.cathode_mat.clone()),
                Transform::from_xyz(TUBE_LEFT + 10.0, TUBE_CENTER_Y + y_spread, 2.0),
            ));
        }
    }

    // Atualizar partículas existentes
    for (entity, mut particle, mut transform) in particles.iter_mut() {
        let x = transform.translation.x;

        // Dentro da região das placas: aplicar campo E (deflexão vertical)
        if x > PLATE_START_X && x < PLATE_END_X {
            // F = eE (normalizado para simulação)
            // Deflexão proporcional a e_field
            let e_deflection = config.e_field * 0.5 * dt;
            // Campo B cancela parcialmente (perpendicular)
            let b_cancellation = config.b_field * particle.vel.x * 0.005 * dt;

            particle.vel.y += e_deflection - b_cancellation;
        }

        // Mover
        transform.translation.x += particle.vel.x * dt;
        transform.translation.y += particle.vel.y * dt;

        // Remover se saiu da tela
        if transform.translation.x > TUBE_RIGHT + 50.0
            || transform.translation.y.abs() > 400.0
        {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Alpha particle scattering test — Thomson's model FAILS here
//
// In Thomson's plum pudding, the positive charge is spread uniformly.
// Alpha particles (~7000× electron mass) should pass through with
// minimal deflection: θ_rms ~ 1° for gold foil (~1000 atom layers).
// Backscattering (>90°) is essentially impossible.
// ---------------------------------------------------------------------------

fn alpha_particle_test(
    mut commands: Commands,
    time: Res<Time>,
    mut alpha_state: Option<ResMut<AlphaTestState>>,
    assets: Option<Res<ThomsonParticleAssets>>,
    mut alphas: Query<(Entity, &mut AlphaParticle, &mut Transform)>,
) {
    let Some(ref mut alpha_state) = alpha_state else { return };
    if !alpha_state.active { return; }
    let Some(assets) = assets else { return };

    let dt = time.delta_secs();
    alpha_state.fire_timer += dt;

    // Emitir partículas alfa da direita
    if alpha_state.fire_timer > 0.15 {
        alpha_state.fire_timer = 0.0;
        let mut rng = rand::rng();
        let y = PUDDING_CENTER.y + rng.random_range(-PUDDING_RADIUS..PUDDING_RADIUS);

        commands.spawn((
            ThomsonEntity,
            AlphaParticle {
                vel: Vec2::new(-250.0, 0.0), // Vindo da direita
            },
            Mesh2d(assets.alpha_mesh.clone()),
            MeshMaterial2d(assets.alpha_mat.clone()),
            Transform::from_xyz(PUDDING_CENTER.x + PUDDING_RADIUS + 200.0, y, 3.0),
        ));
    }

    // Atualizar alfas
    for (entity, mut alpha, mut transform) in alphas.iter_mut() {
        // No modelo de Thomson: deflexão mínima (carga positiva difusa)
        let pos = transform.translation.truncate();
        let to_center = PUDDING_CENTER - pos;
        let dist = to_center.length();

        if dist < PUDDING_RADIUS {
            // Dentro do pudim: força fraca e difusa (quase nenhuma deflexão)
            // Thomson prevê deflexão máxima de ~1° por camada atômica
            let weak_force = to_center.normalize_or_zero() * 5.0; // Muito fraco
            alpha.vel += weak_force * dt;
        }

        transform.translation.x += alpha.vel.x * dt;
        transform.translation.y += alpha.vel.y * dt;

        // Contabilizar resultado
        if transform.translation.x < PUDDING_CENTER.x - PUDDING_RADIUS - 200.0 {
            // Passou direto (saiu pelo lado esquerdo)
            alpha_state.passed += 1;
            commands.entity(entity).despawn();
        } else if transform.translation.x < -600.0
            || transform.translation.x > 700.0
            || transform.translation.y.abs() > 400.0
        {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Controls
// ---------------------------------------------------------------------------

fn thomson_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: Option<ResMut<CathodeRayConfig>>,
    mut alpha_state: Option<ResMut<AlphaTestState>>,
) {
    if let Some(ref mut config) = config {
        // Ajustar campo E
        if keyboard.pressed(KeyCode::ArrowUp) {
            config.e_field = (config.e_field + 2.0).min(300.0);
        }
        if keyboard.pressed(KeyCode::ArrowDown) {
            config.e_field = (config.e_field - 2.0).max(0.0);
        }
        // Ajustar campo B
        if keyboard.pressed(KeyCode::ArrowRight) {
            config.b_field = (config.b_field + 0.05).min(10.0);
        }
        if keyboard.pressed(KeyCode::ArrowLeft) {
            config.b_field = (config.b_field - 0.05).max(0.0);
        }
        // Toggle emissão
        if keyboard.just_pressed(KeyCode::KeyF) {
            config.firing = !config.firing;
        }
    }

    // Toggle teste alfa
    if let Some(ref mut alpha_state) = alpha_state {
        if keyboard.just_pressed(KeyCode::KeyA) {
            alpha_state.active = !alpha_state.active;
            if !alpha_state.active {
                alpha_state.passed = 0;
                alpha_state.deflected = 0;
            }
        }
    }
}

fn thomson_input(
    config: Option<Res<CathodeRayConfig>>,
    alpha_state: Option<Res<AlphaTestState>>,
    mut info_query: Query<&mut Text2d, (With<ThomsonInfoText>, Without<CathodeRayInfoText>)>,
    mut ray_query: Query<&mut Text2d, (With<CathodeRayInfoText>, Without<ThomsonInfoText>)>,
) {
    // Info panel
    if let Some(alpha_state) = &alpha_state {
        let alpha_text = if alpha_state.active {
            format!(
                "TESTE DE ESPALHAMENTO ALFA\n\
                 Passaram direto: {}\n\
                 Retroespalhadas: {} (sempre 0!)\n\n\
                 No modelo de Thomson, a carga positiva\n\
                 e difusa -- alfas passam com deflexao\n\
                 minima. NENHUMA retroespalhada.\n\
                 Geiger-Marsden (1909): 1 em 8000 volta!",
                alpha_state.passed, alpha_state.deflected
            )
        } else {
            "[A] Disparar particulas alfa\n\
             (testar modelo de Thomson)"
                .to_string()
        };

        for mut text in info_query.iter_mut() {
            *text = Text2d::new(alpha_text.clone());
        }
    }

    // Cathode ray info
    if let Some(config) = &config {
        let v = coulomb::velocity_from_crossed_fields(config.e_field, config.b_field);
        let plate_len = PLATE_END_X - PLATE_START_X;
        let deflection = coulomb::electron_deflection(
            config.e_field, plate_len, 1.0, 1.0, v.max(1.0),
        );
        let em_ratio = if config.b_field > 0.1 {
            coulomb::charge_to_mass_ratio(deflection, config.e_field, config.b_field, plate_len)
        } else {
            0.0
        };

        let ray_text = format!(
            "TUBO DE RAIOS CATODICOS\n\
             Campo E: {:.0}  [Setas Cima/Baixo]\n\
             Campo B: {:.2}  [Setas Esq/Dir]\n\
             v = E/B: {:.1}\n\
             e/m (relativo): {:.1}\n\
             [F] Toggle feixe  [A] Teste alfa\n\n\
             Valor real: e/m = 1.759 x 10^11 C/kg\n\
             ~1836x maior que para H+ (eletrolise)\n\
             Conclusao: massa ~1/1836 do Hidrogenio",
            config.e_field, config.b_field, v, em_ratio,
        );

        for mut text in ray_query.iter_mut() {
            *text = Text2d::new(ray_text.clone());
        }
    }
}

fn update_thomson_hud(
    // Placeholder para futuras atualizações dinâmicas
) {}
