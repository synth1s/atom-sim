use bevy::prelude::*;
use rand::Rng;

use crate::common::{ActiveEra, Arena, HudPlugin, SimulationState};
use crate::common::ui::{HudText, EraControls, LimitationText, LimitationVisible};
use crate::physics::classical::{self, Mass, PhysicsBody, Radius, Velocity};

pub struct DemocritusPlugin;

impl Plugin for DemocritusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Arena>()
            .add_plugins(HudPlugin)
            .add_systems(OnEnter(ActiveEra::Democritus), setup_democritus)
            .add_systems(OnExit(ActiveEra::Democritus), cleanup_democritus)
            .add_systems(
                Update,
                (
                    classical::move_particles,
                    classical::bounce_walls,
                    classical::collide_particles,
                )
                    .chain()
                    .run_if(in_state(SimulationState::Running))
                    .run_if(in_state(ActiveEra::Democritus)),
            )
            .add_systems(
                Update,
                (spawn_atom_on_click, update_atom_count_text)
                    .run_if(in_state(ActiveEra::Democritus)),
            );
    }
}

/// Marcador para entidades pertencentes à era de Demócrito.
#[derive(Component)]
struct DemocritusEntity;

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

fn setup_democritus(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    arena: Res<Arena>,
    mut hud_query: Query<&mut Text2d, With<HudText>>,
) {
    // Atualizar HUD
    for mut text in hud_query.iter_mut() {
        *text = Text2d::new(
            "DEMOCRITO (~400 a.C.) \u{2014} Atomos indivisiveis no vazio\n\n\
             \"Nada existe exceto atomos e vazio; tudo o mais e opiniao.\"\n\n\
             Particulas identicas e eternas movendo-se no vazio,\n\
             colidindo mecanicamente. Sem forcas a distancia.\n\n\
             LIMITACAO: Todos os atomos sao identicos.\n\
             Como explicar a diversidade da materia?"
        );
    }

    commands.insert_resource(EraControls("[Clique] Adicionar atomo".to_string()));

    // Limitation text
    commands.insert_resource(LimitationText(
        "ATOMOS SEM DIVERSIDADE".to_string(),
        "Democrito propoe atomos identicos que\ndiferem apenas em forma e arranjo.\nSem diferencas de massa entre tipos,\nnao ha base para leis quantitativas.\nProporcoes definidas nas reacoes?\nImpossivel prever. -> Dalton (1803).".to_string(),
    ));
    commands.insert_resource(LimitationVisible(false));

    // Arena walls
    let wall_color = Color::srgba(0.3, 0.3, 0.35, 0.6);
    let thickness = 2.0;
    let w = arena.half_width * 2.0;
    let h = arena.half_height * 2.0;

    for y_sign in [-1.0_f32, 1.0] {
        commands.spawn((
            DemocritusEntity,
            Mesh2d(meshes.add(Rectangle::new(w + thickness * 2.0, thickness))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(wall_color))),
            Transform::from_xyz(0.0, y_sign * arena.half_height, 0.0),
        ));
    }
    for x_sign in [-1.0_f32, 1.0] {
        commands.spawn((
            DemocritusEntity,
            Mesh2d(meshes.add(Rectangle::new(thickness, h + thickness * 2.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(wall_color))),
            Transform::from_xyz(x_sign * arena.half_width, 0.0, 0.0),
        ));
    }

    // Atom count text
    commands.spawn((
        DemocritusEntity,
        AtomCountText,
        Text2d::new(format!("Atomos: {}", INITIAL_ATOM_COUNT)),
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
        Transform::from_xyz(480.0, 310.0, 10.0),
    ));

    // Shared atom assets
    let mesh = meshes.add(Circle::new(ATOM_RADIUS));
    let material = materials.add(ColorMaterial::from_color(
        Color::srgba(0.75, 0.75, 0.72, 0.9),
    ));
    commands.insert_resource(AtomAssets {
        mesh: mesh.clone(),
        material: material.clone(),
    });
    commands.insert_resource(AtomCount(INITIAL_ATOM_COUNT));

    // Spawn atoms
    let mut rng = rand::rng();
    let margin = ATOM_RADIUS + 5.0;
    for _ in 0..INITIAL_ATOM_COUNT {
        let x = rng.random_range((-arena.half_width + margin)..(arena.half_width - margin));
        let y = rng.random_range((-arena.half_height + margin)..(arena.half_height - margin));
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(50.0..MAX_INITIAL_SPEED);

        commands.spawn((
            DemocritusEntity,
            DemocritusAtom,
            PhysicsBody,
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(x, y, 1.0),
            Velocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
            Radius(ATOM_RADIUS),
            Mass(ATOM_MASS),
        ));
    }
}

fn cleanup_democritus(
    mut commands: Commands,
    query: Query<Entity, With<DemocritusEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<AtomAssets>();
    commands.remove_resource::<AtomCount>();
    commands.remove_resource::<LimitationText>();
}

fn spawn_atom_on_click(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    assets: Option<Res<AtomAssets>>,
    arena: Res<Arena>,
    atom_count: Option<ResMut<AtomCount>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(assets) = assets else { return };
    let Some(mut atom_count) = atom_count else { return };
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };
    let world_pos = ray.origin.truncate();

    if world_pos.x.abs() > arena.half_width - ATOM_RADIUS
        || world_pos.y.abs() > arena.half_height - ATOM_RADIUS
    {
        return;
    }

    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let speed = rng.random_range(50.0..MAX_INITIAL_SPEED);

    commands.spawn((
        DemocritusEntity,
        DemocritusAtom,
        PhysicsBody,
        Mesh2d(assets.mesh.clone()),
        MeshMaterial2d(assets.material.clone()),
        Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
        Velocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
        Radius(ATOM_RADIUS),
        Mass(ATOM_MASS),
    ));
    atom_count.0 += 1;
}

fn update_atom_count_text(
    atom_count: Option<Res<AtomCount>>,
    mut query: Query<&mut Text2d, With<AtomCountText>>,
) {
    let Some(atom_count) = atom_count else { return };
    if atom_count.is_changed() {
        for mut text in query.iter_mut() {
            *text = Text2d::new(format!("Atomos: {}", atom_count.0));
        }
    }
}
