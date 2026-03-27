use bevy::prelude::*;
use rand::Rng;

use crate::common::{Arena, HudPlugin, SimulationState};
use crate::physics::classical::{self, Mass, Radius, Velocity};

pub struct DemocritusPlugin;

impl Plugin for DemocritusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Arena>()
            .init_resource::<AtomCount>()
            .add_plugins(HudPlugin)
            .add_systems(Startup, (spawn_arena, spawn_initial_atoms))
            .add_systems(
                Update,
                (
                    classical::move_particles,
                    classical::bounce_walls(580.0, 320.0),
                    classical::collide_particles,
                )
                    .chain()
                    .run_if(in_state(SimulationState::Running)),
            )
            .add_systems(Update, (spawn_atom_on_click, update_atom_count_text));
    }
}

/// Marcador para átomos de Demócrito — partículas indivisíveis idênticas.
#[derive(Component)]
pub struct DemocritusAtom;

/// Contador global de átomos para exibição no HUD.
#[derive(Resource, Default)]
struct AtomCount(usize);

/// Marcador para o texto do contador de átomos.
#[derive(Component)]
struct AtomCountText;

/// Handles compartilhados para o mesh e material dos átomos (todos idênticos).
#[derive(Resource)]
struct AtomAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

const ATOM_RADIUS: f32 = 10.0;
const ATOM_MASS: f32 = 1.0;
const INITIAL_ATOM_COUNT: usize = 30;
const MAX_INITIAL_SPEED: f32 = 200.0;

/// Desenha as bordas da arena como retângulo visível.
fn spawn_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    arena: Res<Arena>,
) {
    let wall_color = Color::srgba(0.3, 0.3, 0.35, 0.6);
    let thickness = 2.0;
    let w = arena.half_width * 2.0;
    let h = arena.half_height * 2.0;

    // Paredes horizontais (topo e base)
    for y_sign in [-1.0_f32, 1.0] {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(w + thickness * 2.0, thickness))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(wall_color))),
            Transform::from_xyz(0.0, y_sign * arena.half_height, 0.0),
        ));
    }

    // Paredes verticais (esquerda e direita)
    for x_sign in [-1.0_f32, 1.0] {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(thickness, h + thickness * 2.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(wall_color))),
            Transform::from_xyz(x_sign * arena.half_width, 0.0, 0.0),
        ));
    }

    // Texto do contador de átomos
    commands.spawn((
        AtomCountText,
        Text2d::new("Atomos: 0"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
        Transform::from_xyz(480.0, 310.0, 10.0),
    ));
}

/// Gera os átomos iniciais com posições e velocidades aleatórias.
fn spawn_initial_atoms(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    arena: Res<Arena>,
    mut atom_count: ResMut<AtomCount>,
) {
    // Criar assets compartilhados
    let mesh = meshes.add(Circle::new(ATOM_RADIUS));
    let material = materials.add(ColorMaterial::from_color(
        Color::srgba(0.75, 0.75, 0.72, 0.9), // Cinza claro — todos idênticos (Demócrito)
    ));

    commands.insert_resource(AtomAssets {
        mesh: mesh.clone(),
        material: material.clone(),
    });

    let mut rng = rand::rng();
    let margin = ATOM_RADIUS + 5.0;

    for _ in 0..INITIAL_ATOM_COUNT {
        let x = rng.random_range((-arena.half_width + margin)..(arena.half_width - margin));
        let y = rng.random_range((-arena.half_height + margin)..(arena.half_height - margin));
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(50.0..MAX_INITIAL_SPEED);
        let vx = angle.cos() * speed;
        let vy = angle.sin() * speed;

        commands.spawn((
            DemocritusAtom,
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(x, y, 1.0),
            Velocity(Vec2::new(vx, vy)),
            Radius(ATOM_RADIUS),
            Mass(ATOM_MASS),
        ));
    }

    atom_count.0 = INITIAL_ATOM_COUNT;
}

/// Adiciona um novo átomo na posição do clique do mouse.
fn spawn_atom_on_click(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    assets: Option<Res<AtomAssets>>,
    arena: Res<Arena>,
    mut atom_count: ResMut<AtomCount>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(assets) = assets else { return };

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };
    let world_pos = ray.origin.truncate();

    // Só criar se dentro da arena
    if world_pos.x.abs() > arena.half_width - ATOM_RADIUS
        || world_pos.y.abs() > arena.half_height - ATOM_RADIUS
    {
        return;
    }

    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let speed = rng.random_range(50.0..MAX_INITIAL_SPEED);

    commands.spawn((
        DemocritusAtom,
        Mesh2d(assets.mesh.clone()),
        MeshMaterial2d(assets.material.clone()),
        Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
        Velocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
        Radius(ATOM_RADIUS),
        Mass(ATOM_MASS),
    ));

    atom_count.0 += 1;
}

/// Atualiza o texto do contador de átomos no HUD.
fn update_atom_count_text(
    atom_count: Res<AtomCount>,
    mut query: Query<&mut Text2d, With<AtomCountText>>,
) {
    if atom_count.is_changed() {
        for mut text in query.iter_mut() {
            *text = Text2d::new(format!("Atomos: {}", atom_count.0));
        }
    }
}
