use bevy::prelude::*;
use rand::Rng;

use crate::common::{ActiveEra, Arena, SimulationState};
use crate::common::ui::{HudText, EraControls, LimitationText, LimitationVisible};
use crate::physics::classical::{self, Mass, Radius, Velocity};

pub struct DaltonPlugin;

impl Plugin for DaltonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ActiveEra::Dalton), setup_dalton)
            .add_systems(OnExit(ActiveEra::Dalton), cleanup_dalton)
            .add_systems(
                Update,
                (
                    classical::move_particles,
                    classical::bounce_walls(580.0, 320.0),
                    classical::collide_particles,
                    check_reactions,
                )
                    .chain()
                    .run_if(in_state(SimulationState::Running))
                    .run_if(in_state(ActiveEra::Dalton)),
            )
            .add_systems(
                Update,
                (spawn_atom_on_click, cycle_selected_element, update_dalton_hud)
                    .run_if(in_state(ActiveEra::Dalton)),
            );
    }
}

// ---------------------------------------------------------------------------
// Dalton's elements — each with distinct mass, radius, and color
// Dalton (1808): "Atoms of different elements have different masses."
// Masses relative to H=1 (using Dalton's ORIGINAL values, not modern ones)
// ---------------------------------------------------------------------------

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Element {
    Hydrogen,  // H — Dalton: 1, modern: 1.008
    Carbon,    // C — Dalton: 5.4, modern: 12.01
    Nitrogen,  // N — Dalton: 5, modern: 14.01
    Oxygen,    // O — Dalton: 7, modern: 16.00 (erro da "regra de máxima simplicidade": HO em vez de H2O)
}

impl Element {
    /// Peso atômico relativo (Dalton, 1808). Erros históricos preservados.
    pub fn dalton_mass(self) -> f32 {
        match self {
            Element::Hydrogen => 1.0,
            Element::Carbon => 5.4,
            Element::Nitrogen => 5.0,
            Element::Oxygen => 7.0, // Dalton achava HO, não H2O → O=7 em vez de 16
        }
    }

    /// Raio visual proporcional à massa (para visualização).
    pub fn radius(self) -> f32 {
        6.0 + self.dalton_mass() * 1.2
    }

    /// Cor característica do elemento.
    pub fn color(self) -> Color {
        match self {
            Element::Hydrogen => Color::srgba(0.9, 0.9, 0.95, 0.95),  // Branco azulado
            Element::Carbon => Color::srgba(0.45, 0.45, 0.45, 0.95),  // Cinza médio visível
            Element::Nitrogen => Color::srgba(0.3, 0.5, 0.9, 0.95),   // Azul
            Element::Oxygen => Color::srgba(0.9, 0.25, 0.2, 0.95),    // Vermelho
        }
    }

    pub fn next(self) -> Element {
        match self {
            Element::Hydrogen => Element::Carbon,
            Element::Carbon => Element::Nitrogen,
            Element::Nitrogen => Element::Oxygen,
            Element::Oxygen => Element::Hydrogen,
        }
    }
}

// ---------------------------------------------------------------------------
// Molecules — formed when compatible atoms collide
// Dalton's 4th postulate: "Compounds are formed by combination of atoms
// in ratios of small whole numbers."
// ---------------------------------------------------------------------------

/// Marcador para entidades pertencentes à era de Dalton.
#[derive(Component)]
struct DaltonEntity;

/// Uma molécula é um grupo de átomos ligados visualmente.
#[derive(Component)]
struct Molecule {
    #[allow(dead_code)]
    formula: String,
}

/// Qual elemento está selecionado para spawnar ao clicar.
#[derive(Resource)]
struct SelectedElement(Element);

impl Default for SelectedElement {
    fn default() -> Self {
        Self(Element::Hydrogen)
    }
}

/// Contagem de átomos por elemento (para demonstrar conservação).
#[derive(Resource, Default)]
struct AtomCounts {
    hydrogen: usize,
    carbon: usize,
    nitrogen: usize,
    oxygen: usize,
    molecules_formed: usize,
}

impl AtomCounts {
    fn add(&mut self, elem: Element) {
        match elem {
            Element::Hydrogen => self.hydrogen += 1,
            Element::Carbon => self.carbon += 1,
            Element::Nitrogen => self.nitrogen += 1,
            Element::Oxygen => self.oxygen += 1,
        }
    }
    fn total(&self) -> usize {
        self.hydrogen + self.carbon + self.nitrogen + self.oxygen
    }
}

/// Marcador para o texto de informações de Dalton.
#[derive(Component)]
struct DaltonInfoText;

/// Marcador para o texto do elemento selecionado.
#[derive(Component)]
struct SelectedElementText;

/// Cached mesh/material handles to avoid leaking new handles every frame.
#[derive(Resource)]
struct DaltonParticleAssets {
    /// Unit circle mesh (radius 1.0) — use `Transform::with_scale` to size.
    molecule_mesh: Handle<Mesh>,
    /// Green material for molecules.
    molecule_mat: Handle<ColorMaterial>,
    /// Per-element mesh (already sized to `element.radius()`).
    elem_mesh: std::collections::HashMap<Element, Handle<Mesh>>,
    /// Per-element material.
    elem_mat: std::collections::HashMap<Element, Handle<ColorMaterial>>,
}

const MAX_INITIAL_SPEED: f32 = 150.0;

// ---------------------------------------------------------------------------
// Setup & Cleanup
// ---------------------------------------------------------------------------

fn setup_dalton(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    arena: Res<Arena>,
    mut hud_query: Query<&mut Text2d, With<HudText>>,
) {
    // Atualizar HUD principal
    for mut text in hud_query.iter_mut() {
        *text = Text2d::new(
            "DALTON (1803) \u{2014} Teoria Atomica Quantitativa\n\n\
             Postulados: atomos de elementos diferentes tem massas\n\
             diferentes. Compostos formam-se em proporcoes inteiras.\n\
             Em reacoes, atomos sao rearranjados, nunca destruidos.\n\n\
             LIMITACAO: Atomos sao indivisiveis.\n\
             Nao explica eletricidade nem espectros de emissao."
        );
    }

    commands.init_resource::<SelectedElement>();
    commands.insert_resource(AtomCounts::default());
    commands.insert_resource(EraControls("[E] Trocar elemento\n[Clique] Adicionar atomo".to_string()));

    // Limitation text
    commands.insert_resource(LimitationText(
        "ATOMO INDIVISIVEL".to_string(),
        "Dalton assume atomos como esferas\nsolidas e indivisiveis. Mas em 1897,\nThomson descobre o eletron (e/m =\n1.76e11 C/kg): particula 1836x mais\nleve que H. O atomo TEM estrutura\ninterna. -> Thomson e o eletron.".to_string(),
    ));
    commands.insert_resource(LimitationVisible(false));

    // Arena walls
    let wall_color = Color::srgba(0.3, 0.3, 0.35, 0.6);
    let thickness = 2.0;
    let w = arena.half_width * 2.0;
    let h = arena.half_height * 2.0;

    for y_sign in [-1.0_f32, 1.0] {
        commands.spawn((
            DaltonEntity,
            Mesh2d(meshes.add(Rectangle::new(w + thickness * 2.0, thickness))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(wall_color))),
            Transform::from_xyz(0.0, y_sign * arena.half_height, 0.0),
        ));
    }
    for x_sign in [-1.0_f32, 1.0] {
        commands.spawn((
            DaltonEntity,
            Mesh2d(meshes.add(Rectangle::new(thickness, h + thickness * 2.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(wall_color))),
            Transform::from_xyz(x_sign * arena.half_width, 0.0, 0.0),
        ));
    }

    // Info panel (right side)
    commands.spawn((
        DaltonEntity,
        DaltonInfoText,
        Text2d::new(""),
        TextFont { font_size: 15.0, ..default() },
        TextColor(Color::srgba(0.8, 0.9, 0.8, 0.9)),
        Transform::from_xyz(430.0, 200.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));

    // Selected element indicator
    commands.spawn((
        DaltonEntity,
        SelectedElementText,
        Text2d::new("[E] Elemento: H (Hidrogenio)"),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgba(0.9, 0.9, 0.5, 0.9)),
        Transform::from_xyz(-500.0, -230.0, 10.0),
        TextLayout::new_with_justify(Justify::Left),
    ));

    // Build cached particle assets (avoids leaking handles each frame)
    let all_elements = [
        Element::Hydrogen,
        Element::Carbon,
        Element::Nitrogen,
        Element::Oxygen,
    ];
    let mut elem_mesh = std::collections::HashMap::new();
    let mut elem_mat = std::collections::HashMap::new();
    for &el in &all_elements {
        elem_mesh.insert(el, meshes.add(Circle::new(el.radius())));
        elem_mat.insert(el, materials.add(ColorMaterial::from_color(el.color())));
    }
    let particle_assets = DaltonParticleAssets {
        molecule_mesh: meshes.add(Circle::new(1.0)),
        molecule_mat: materials.add(ColorMaterial::from_color(Color::srgba(0.6, 0.8, 0.3, 0.9))),
        elem_mesh,
        elem_mat,
    };

    // Spawn initial atoms — a mix of elements
    let mut rng = rand::rng();
    let margin = 20.0;
    let mut counts = AtomCounts::default();

    let spawn_list: Vec<(Element, usize)> = vec![
        (Element::Hydrogen, 12),
        (Element::Oxygen, 6),
        (Element::Carbon, 4),
        (Element::Nitrogen, 3),
    ];

    for (element, count) in spawn_list {
        let r = element.radius();
        let mesh_handle = particle_assets.elem_mesh[&element].clone();
        let mat_handle = particle_assets.elem_mat[&element].clone();

        for _ in 0..count {
            let x = rng.random_range((-arena.half_width + margin)..(arena.half_width - margin));
            let y = rng.random_range((-arena.half_height + margin)..(arena.half_height - margin));
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let speed = rng.random_range(40.0..MAX_INITIAL_SPEED);

            commands.spawn((
                DaltonEntity,
                element,
                Mesh2d(mesh_handle.clone()),
                MeshMaterial2d(mat_handle.clone()),
                Transform::from_xyz(x, y, 1.0),
                Velocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
                Radius(r),
                Mass(element.dalton_mass()),
            ));
            counts.add(element);
        }
    }

    commands.insert_resource(particle_assets);
    commands.insert_resource(counts);
}

fn cleanup_dalton(
    mut commands: Commands,
    query: Query<Entity, With<DaltonEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<SelectedElement>();
    commands.remove_resource::<AtomCounts>();
    commands.remove_resource::<DaltonParticleAssets>();
    commands.remove_resource::<LimitationText>();
}

// ---------------------------------------------------------------------------
// Chemical reactions — Dalton's 5th postulate:
// "In chemical reactions, atoms are rearranged, never created nor destroyed."
//
// When two compatible atoms collide, they form a molecule (visual bond).
// The atoms are NOT destroyed — they become part of a molecule entity,
// demonstrating conservation of mass.
// ---------------------------------------------------------------------------

fn check_reactions(
    mut commands: Commands,
    assets: Option<Res<DaltonParticleAssets>>,
    query: Query<(Entity, &Transform, &Element, &Radius, &Velocity), Without<Molecule>>,
    mut counts: Option<ResMut<AtomCounts>>,
) {
    let Some(ref mut counts) = counts else { return };
    let Some(assets) = assets else { return };

    let atoms: Vec<_> = query.iter().collect();
    let n = atoms.len();
    let mut consumed: Vec<Entity> = Vec::new();

    for i in 0..n {
        let (e1, t1, el1, r1, v1) = atoms[i];
        if consumed.contains(&e1) { continue; }

        for j in (i + 1)..n {
            let (e2, t2, el2, r2, v2) = atoms[j];
            if consumed.contains(&e2) { continue; }

            let dist = t1.translation.truncate().distance(t2.translation.truncate());
            let min_dist = r1.0 + r2.0;

            if dist < min_dist * 1.1 {
                // Check for compatible reaction
                if let Some(formula) = reaction_formula(*el1, *el2) {
                    // Create molecule at midpoint
                    let midpoint = (t1.translation + t2.translation) / 2.0;
                    let avg_vel = (v1.0 + v2.0) / 2.0;
                    let mol_mass = el1.dalton_mass() + el2.dalton_mass();
                    let mol_radius = (r1.0 + r2.0) * 0.7;

                    // Reuse cached unit-circle mesh; scale transform to desired radius
                    commands.spawn((
                        DaltonEntity,
                        Molecule { formula: formula.to_string() },
                        Mesh2d(assets.molecule_mesh.clone()),
                        MeshMaterial2d(assets.molecule_mat.clone()),
                        Transform::from_xyz(midpoint.x, midpoint.y, 2.0)
                            .with_scale(Vec3::splat(mol_radius)),
                        Velocity(avg_vel),
                        Radius(mol_radius),
                        Mass(mol_mass),
                    ));

                    consumed.push(e1);
                    consumed.push(e2);
                    counts.molecules_formed += 1;
                    break;
                }
            }
        }
    }

    for entity in consumed {
        commands.entity(entity).despawn();
    }
}

/// Retorna a fórmula se dois elementos podem reagir.
/// Dalton pensava em proporções simples (regra de máxima simplicidade):
/// H + O → HO (ele achava água = HO, não H2O)
fn reaction_formula(a: Element, b: Element) -> Option<&'static str> {
    match (a, b) {
        (Element::Hydrogen, Element::Oxygen) | (Element::Oxygen, Element::Hydrogen) => Some("HO"),
        (Element::Carbon, Element::Oxygen) | (Element::Oxygen, Element::Carbon) => Some("CO"),
        (Element::Hydrogen, Element::Nitrogen) | (Element::Nitrogen, Element::Hydrogen) => Some("HN"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Interaction
// ---------------------------------------------------------------------------

fn cycle_selected_element(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected: Option<ResMut<SelectedElement>>,
    mut query: Query<&mut Text2d, With<SelectedElementText>>,
) {
    let Some(ref mut selected) = selected else { return };

    if keyboard.just_pressed(KeyCode::KeyE) {
        selected.0 = selected.0.next();
        let name = match selected.0 {
            Element::Hydrogen => "H (Hidrogenio)",
            Element::Carbon => "C (Carbono)",
            Element::Nitrogen => "N (Nitrogenio)",
            Element::Oxygen => "O (Oxigenio)",
        };
        for mut text in query.iter_mut() {
            *text = Text2d::new(format!("[E] Elemento: {}", name));
        }
    }
}

fn spawn_atom_on_click(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    arena: Res<Arena>,
    selected: Option<Res<SelectedElement>>,
    assets: Option<Res<DaltonParticleAssets>>,
    mut counts: Option<ResMut<AtomCounts>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) { return; }
    let Some(selected) = selected else { return };
    let Some(assets) = assets else { return };
    let Some(ref mut counts) = counts else { return };
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };
    let world_pos = ray.origin.truncate();

    let element = selected.0;
    let r = element.radius();

    if world_pos.x.abs() > arena.half_width - r
        || world_pos.y.abs() > arena.half_height - r
    {
        return;
    }

    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let speed = rng.random_range(40.0..MAX_INITIAL_SPEED);

    commands.spawn((
        DaltonEntity,
        element,
        Mesh2d(assets.elem_mesh[&element].clone()),
        MeshMaterial2d(assets.elem_mat[&element].clone()),
        Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
        Velocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
        Radius(r),
        Mass(element.dalton_mass()),
    ));
    counts.add(element);
}

// ---------------------------------------------------------------------------
// HUD update — conservation of mass display
// Dalton's 5th postulate: atoms are rearranged, never created nor destroyed
// ---------------------------------------------------------------------------

fn update_dalton_hud(
    counts: Option<Res<AtomCounts>>,
    mut query: Query<&mut Text2d, With<DaltonInfoText>>,
) {
    let Some(counts) = counts else { return };
    if !counts.is_changed() { return; }

    let info = format!(
        "CONTAGEM DE ATOMOS\n\
         (Conservacao de massa)\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         H: {}  (massa: {:.1})\n\
         C: {}  (massa: {:.1})\n\
         N: {}  (massa: {:.1})\n\
         O: {}  (massa: {:.1})\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         Total: {} atomos\n\
         Moleculas: {}\n\n\
         PESOS ATOMICOS\n\
         (Dalton vs Moderno)\n\
         H: 1 vs 1.008\n\
         C: 5.4 vs 12.01\n\
         N: 5 vs 14.01\n\
         O: 7 vs 16.00\n\
         (erro: regra da\n\
          maxima simplicidade)",
        counts.hydrogen, counts.hydrogen as f32 * Element::Hydrogen.dalton_mass(),
        counts.carbon, counts.carbon as f32 * Element::Carbon.dalton_mass(),
        counts.nitrogen, counts.nitrogen as f32 * Element::Nitrogen.dalton_mass(),
        counts.oxygen, counts.oxygen as f32 * Element::Oxygen.dalton_mass(),
        counts.total(),
        counts.molecules_formed,
    );

    for mut text in query.iter_mut() {
        *text = Text2d::new(info.clone());
    }
}
