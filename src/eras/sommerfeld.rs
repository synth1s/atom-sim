use bevy::prelude::*;

use crate::common::{ActiveEra, SimulationState};
use crate::common::ui::{HudText, EraControls, LimitationText, LimitationVisible};
use crate::physics::spectral;

pub struct SommerfeldPlugin;

impl Plugin for SommerfeldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ActiveEra::Sommerfeld), setup_sommerfeld)
            .add_systems(OnExit(ActiveEra::Sommerfeld), cleanup_sommerfeld)
            .add_systems(
                Update,
                (orbit_electron_elliptical, update_sommerfeld_hud)
                    .run_if(in_state(ActiveEra::Sommerfeld))
                    .run_if(in_state(SimulationState::Running)),
            )
            .add_systems(
                Update,
                sommerfeld_controls.run_if(in_state(ActiveEra::Sommerfeld)),
            );
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct SommerfeldEntity;

/// Elétron numa órbita elíptica de Sommerfeld.
/// Quantização de Bohr-Sommerfeld (integrais de ação):
///   ∮ p_φ dφ = k·h  (k = 1,...,n: número quântico azimutal)
///   ∮ p_r dr = n_r·h (n_r = n - k: número quântico radial)
///
/// Excentricidade: e = √(1 - (k/n)²)
/// k = n → circular (Bohr), k < n → elipse
#[derive(Component)]
struct SommerfeldElectron {
    angle: f32,         // Ângulo atual (true anomaly)
    precession: f32,    // Precessão relativística acumulada
}

#[derive(Component)]
struct SommerfeldInfoText;

#[derive(Component)]
struct EnergyDetailText;

/// Marcador para segmentos de órbita (redesenhados quando n/k mudam).
#[derive(Component)]
struct OrbitSegment;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct SommerfeldState {
    n: u32,      // Número quântico principal (1-6)
    k: u32,      // Número quântico azimutal (1..n)
    m: i32,      // Número quântico magnético (-k..+k)
    b_field: f32, // Campo magnético externo (para Zeeman)
}

impl Default for SommerfeldState {
    fn default() -> Self {
        Self {
            n: 3,
            k: 2,
            m: 0,
            b_field: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const NUCLEUS_POS: Vec2 = Vec2::new(-150.0, 30.0);
const ORBIT_SCALE: f32 = 15.0;

/// Constante de estrutura fina: α = e²/(4πε₀ℏc) ≈ 1/137.036
const FINE_STRUCTURE_ALPHA: f64 = 1.0 / 137.036;

/// Energia de Sommerfeld com correção relativística.
/// E_{n,k} = -13.6/n² · [1 + α²/n² · (n/k - 3/4)] eV
///
/// O desdobramento é ΔE ~ α² · E_n / n ~ 10⁻⁴ eV para n=2
/// correspondendo a ~0.3 cm⁻¹ — concorda com Michelson (1891).
///
/// NOTA HISTÓRICA: Esta fórmula dá o resultado correto pelos motivos
/// errados! Dirac (1928) mostrou que k deve ser substituído por j+½.
/// A coincidência numérica vem da degenerescência acidental entre
/// estados com mesmo j mas diferente l.
fn sommerfeld_energy(n: u32, k: u32) -> f64 {
    let e_bohr = spectral::bohr_energy(n);
    let alpha2 = FINE_STRUCTURE_ALPHA * FINE_STRUCTURE_ALPHA;
    let n_f = n as f64;
    let k_f = k as f64;
    let correction = alpha2 / (n_f * n_f) * (n_f / k_f - 0.75);
    e_bohr * (1.0 + correction)
}

/// Efeito Zeeman normal: ΔE = m · ℏω_L = m · eB/(2mₑ)
/// Em unidades de simulação: ΔE proporcional a m × B
fn zeeman_shift(m: i32, b_field: f32) -> f64 {
    // ω_L = eB/(2mₑ) ≈ 8.794×10¹⁰ × B rad/s
    // ΔE = m·ℏ·ω_L. Em eV: ΔE ≈ 5.788×10⁻⁵ × m × B eV/T
    m as f64 * 5.788e-5 * b_field as f64
}

/// Parâmetros da elipse para dados (n, k).
fn ellipse_params(n: u32, k: u32) -> (f32, f32, f32) {
    let a = (n as f32).powi(2) * ORBIT_SCALE; // Semi-eixo maior
    let eccentricity = (1.0 - (k as f32 / n as f32).powi(2)).sqrt();
    let b = a * (1.0 - eccentricity * eccentricity).sqrt(); // Semi-eixo menor
    (a, b, eccentricity)
}

/// Posição na elipse Kepler (equação da órbita em coordenadas polares):
/// r(θ) = a(1-e²) / (1 + e·cos(θ))
fn ellipse_position(a: f32, eccentricity: f32, theta: f32) -> Vec2 {
    let r = a * (1.0 - eccentricity * eccentricity) / (1.0 + eccentricity * theta.cos());
    Vec2::new(r * theta.cos(), r * theta.sin())
}

// ---------------------------------------------------------------------------
// Setup & Cleanup
// ---------------------------------------------------------------------------

fn setup_sommerfeld(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hud_query: Query<&mut Text2d, With<HudText>>,
) {
    for mut text in hud_query.iter_mut() {
        *text = Text2d::new(
            "SOMMERFELD (1916) \u{2014} Orbitas Elipticas\n\n\
             Generalizacao de Bohr com mecanica analitica:\n\
             quantizacao por integrais de acao (Bohr-Sommerfeld).\n\
             Numeros quanticos: n (principal), k (azimutal),\n\
             m (magnetico). Correcao relativistica introduz\n\
             a constante de estrutura fina: a = 1/137.036\n\n\
             LIMITACAO: Efeito Zeeman anomalo inexplicavel.\n\
             Formula de estrutura fina correta, mas pelos\n\
             motivos errados! (coincidencia numerica)"
        );
    }

    commands.insert_resource(SommerfeldState::default());
    commands.insert_resource(EraControls(
        "[Setas] n/k  [M] m\n[B/V] Campo magnetico".to_string()
    ));

    // Limitation text
    commands.insert_resource(LimitationText(
        "ORBITAS SEM ONDAS".to_string(),
        "Sommerfeld adiciona orbitas elipticas\ne correcao relativistica (alfa~1/137).\nMas regras de quantizacao sao ad hoc:\npor que L = n*hbar? Nenhuma razao\nfisica. Em 1924, de Broglie responde:\no eletron e uma ONDA. -> De Broglie.".to_string(),
    ));
    commands.insert_resource(LimitationVisible(false));

    // Núcleo
    commands.spawn((
        SommerfeldEntity,
        Mesh2d(meshes.add(Circle::new(5.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 0.3, 0.2, 1.0),
        ))),
        Transform::from_xyz(NUCLEUS_POS.x, NUCLEUS_POS.y, 2.0),
    ));

    // Desenhar todas as órbitas possíveis para n=3 (k=1,2,3)
    draw_orbits(&mut commands, &mut meshes, &mut materials, 3);

    // Elétron
    commands.spawn((
        SommerfeldEntity,
        SommerfeldElectron {
            angle: 0.0,
            precession: 0.0,
        },
        Mesh2d(meshes.add(Circle::new(6.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(0.2, 0.6, 1.0, 1.0),
        ))),
        Transform::from_xyz(NUCLEUS_POS.x, NUCLEUS_POS.y, 3.0),
    ));

    // Info text
    commands.spawn((
        SommerfeldEntity,
        SommerfeldInfoText,
        Text2d::new(""),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgba(0.8, 0.8, 0.5, 0.9)),
        Transform::from_xyz(300.0, 250.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));

    // Energy detail text
    commands.spawn((
        SommerfeldEntity,
        EnergyDetailText,
        Text2d::new(""),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgba(0.7, 0.8, 0.9, 0.9)),
        Transform::from_xyz(300.0, -30.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));
}

fn draw_orbits(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    n: u32,
) {
    // Sub-shell colors: s=vermelho, p=azul, d=verde
    let colors = [
        Color::srgba(0.8, 0.3, 0.3, 0.3), // k=1 (s-like, muito excêntrica)
        Color::srgba(0.3, 0.5, 0.8, 0.3), // k=2 (p-like)
        Color::srgba(0.3, 0.8, 0.4, 0.3), // k=3 (d-like)
        Color::srgba(0.8, 0.7, 0.3, 0.3), // k=4
        Color::srgba(0.6, 0.3, 0.8, 0.3), // k=5
        Color::srgba(0.3, 0.8, 0.8, 0.3), // k=6
    ];

    for k in 1..=n {
        let (a, _b, ecc) = ellipse_params(n, k);
        let color = colors.get((k - 1) as usize).copied().unwrap_or(colors[0]);

        // Desenhar elipse como sequência de segmentos
        let segments = 64;
        for i in 0..segments {
            let theta1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let theta2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let p1 = ellipse_position(a, ecc, theta1);
            let p2 = ellipse_position(a, ecc, theta2);

            let mid = (p1 + p2) / 2.0 + NUCLEUS_POS;
            let dir = p2 - p1;
            let len = dir.length();
            let angle = dir.y.atan2(dir.x);

            commands.spawn((
                SommerfeldEntity,
                OrbitSegment,
                Mesh2d(meshes.add(Rectangle::new(len, 1.0))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
                Transform::from_xyz(mid.x, mid.y, 0.5)
                    .with_rotation(Quat::from_rotation_z(angle)),
            ));
        }

        // Label
        let label_pos = ellipse_position(a, ecc, 0.0) + NUCLEUS_POS;
        let sublabel = match k {
            1 => "s",
            2 => "p",
            3 => "d",
            4 => "f",
            _ => "?",
        };
        commands.spawn((
            SommerfeldEntity,
            OrbitSegment,
            Text2d::new(format!("k={} ({})", k, sublabel)),
            TextFont { font_size: 10.0, ..default() },
            TextColor(color.with_alpha(0.8)),
            Transform::from_xyz(label_pos.x + 10.0, label_pos.y + 8.0, 10.0),
        ));
    }
}

fn cleanup_sommerfeld(
    mut commands: Commands,
    query: Query<Entity, With<SommerfeldEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<SommerfeldState>();
    commands.remove_resource::<LimitationText>();
}

// ---------------------------------------------------------------------------
// Elliptical orbit — Kepler motion with relativistic precession
//
// The electron follows r(θ) = a(1-e²)/(1 + e·cos(θ))
// Velocity is VARIABLE: faster near nucleus (2nd Kepler law).
// Relativistic precession: the ellipse rotates slowly, rate ∝ α²/k
// ---------------------------------------------------------------------------

fn orbit_electron_elliptical(
    time: Res<Time>,
    state: Option<Res<SommerfeldState>>,
    mut query: Query<(&mut SommerfeldElectron, &mut Transform)>,
) {
    let Some(state) = state else { return };
    let dt = time.delta_secs();

    let (a, _b, ecc) = ellipse_params(state.n, state.k);

    for (mut electron, mut transform) in query.iter_mut() {
        // Velocidade angular variável (2ª lei de Kepler: r²dθ/dt = const)
        let current_pos = ellipse_position(a, ecc, electron.angle);
        let r = current_pos.length().max(1.0);
        let angular_vel = 3.0 * a * a / (r * r * (state.n as f32).powi(2));

        electron.angle += angular_vel * dt;

        // Precessão relativística: δφ = πα²/(k) por órbita
        // Acelerado para visualização
        let precession_rate = 0.3 * FINE_STRUCTURE_ALPHA as f32 / (state.k as f32);
        electron.precession += precession_rate * dt;

        // Posição com precessão
        let pos = ellipse_position(a, ecc, electron.angle);
        let cos_p = electron.precession.cos();
        let sin_p = electron.precession.sin();
        let rotated = Vec2::new(
            pos.x * cos_p - pos.y * sin_p,
            pos.x * sin_p + pos.y * cos_p,
        );

        transform.translation.x = NUCLEUS_POS.x + rotated.x;
        transform.translation.y = NUCLEUS_POS.y + rotated.y;
    }
}

// ---------------------------------------------------------------------------
// Controls
// ---------------------------------------------------------------------------

fn sommerfeld_controls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Option<ResMut<SommerfeldState>>,
    orbit_segments: Query<Entity, With<OrbitSegment>>,
) {
    let Some(mut state) = state else { return };

    let old_n = state.n;

    // n: principal quantum number
    if keyboard.just_pressed(KeyCode::ArrowUp) && state.n < 6 {
        state.n += 1;
        state.k = state.k.min(state.n);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) && state.n > 1 {
        state.n -= 1;
        state.k = state.k.min(state.n);
        state.m = state.m.clamp(-(state.k as i32), state.k as i32);
    }

    // k: azimuthal quantum number
    if keyboard.just_pressed(KeyCode::ArrowRight) && state.k < state.n {
        state.k += 1;
        state.m = state.m.clamp(-(state.k as i32), state.k as i32);
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) && state.k > 1 {
        state.k -= 1;
        state.m = state.m.clamp(-(state.k as i32), state.k as i32);
    }

    // m: magnetic quantum number
    if keyboard.just_pressed(KeyCode::KeyM) {
        state.m = if state.m < state.k as i32 { state.m + 1 } else { -(state.k as i32) };
    }

    // B field
    if keyboard.pressed(KeyCode::KeyB) {
        state.b_field = (state.b_field + 0.5).min(20.0);
    }
    if keyboard.pressed(KeyCode::KeyV) {
        state.b_field = (state.b_field - 0.5).max(0.0);
    }

    // Redesenhar órbitas quando n muda
    if state.n != old_n {
        for entity in orbit_segments.iter() {
            commands.entity(entity).despawn();
        }
        draw_orbits(&mut commands, &mut meshes, &mut materials, state.n);
    }
}

// ---------------------------------------------------------------------------
// HUD
// ---------------------------------------------------------------------------

fn update_sommerfeld_hud(
    state: Option<Res<SommerfeldState>>,
    mut info_query: Query<&mut Text2d, (With<SommerfeldInfoText>, Without<EnergyDetailText>)>,
    mut energy_query: Query<&mut Text2d, (With<EnergyDetailText>, Without<SommerfeldInfoText>)>,
) {
    let Some(state) = state else { return };

    let (a, b, ecc) = ellipse_params(state.n, state.k);
    let energy = sommerfeld_energy(state.n, state.k);
    let bohr_e = spectral::bohr_energy(state.n);
    let fine_split = (energy - bohr_e) * 1000.0; // meV
    let zeeman = zeeman_shift(state.m, state.b_field);

    let info = format!(
        "CONTROLES\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         [Cima/Baixo] n={}  [Esq/Dir] k={}\n\
         [M] m={} (-{}..+{})  [B/V] B={:.1}T\n\n\
         ORBITA: e={:.3} a={:.1}a0 b={:.1}a0\n\
         k=n: circular | k<n: elipse",
        state.n, state.k, state.m, state.k, state.k,
        state.b_field,
        ecc, a / ORBIT_SCALE, b / ORBIT_SCALE,
    );

    for mut text in info_query.iter_mut() {
        *text = Text2d::new(info.clone());
    }

    // Energy details
    let mut energy_text = format!(
        "ENERGIA (Sommerfeld)\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         Bohr:       E = {:.6} eV\n\
         Sommerfeld: E = {:.6} eV\n\
         Estrutura fina: {:.3} meV\n",
        bohr_e, energy, fine_split,
    );

    if state.b_field > 0.0 {
        energy_text.push_str(&format!(
            "Zeeman (m={}): dE = {:.6} eV\n\
             E total = {:.6} eV\n\n\
             Niveis m desdobrados: {} componentes\n\
             (Zeeman normal: Dm = 0, +/-1 -> 3 linhas)",
            state.m, zeeman,
            energy + zeeman,
            2 * state.k + 1,
        ));
    }

    energy_text.push_str(&format!(
        "\na = 1/{:.3} (a^2={:.2e})\n\
         Coincidencia: k -> j+1/2 (Dirac)",
        1.0 / FINE_STRUCTURE_ALPHA,
        FINE_STRUCTURE_ALPHA * FINE_STRUCTURE_ALPHA,
    ));

    for mut text in energy_query.iter_mut() {
        *text = Text2d::new(energy_text.clone());
    }
}
