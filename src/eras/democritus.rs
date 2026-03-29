use bevy::prelude::*;
use rand::Rng;

use crate::common::{ActiveEra, Arena, HudPlugin, SimulationState};
use crate::common::ui::{HudText, EraControls, LimitationText, LimitationVisible};
use crate::common::equations::{EraEquations, EquationsVisible};
use crate::common::evidence::{EraEvidence, EvidenceVisible};
use crate::common::quiz::{EraQuiz, QuestionData, QuizState};
use crate::common::predict::{EraPredictions, Prediction, PredictState};
use crate::common::sandbox::EraParameters;
use crate::common::narrative::{EraNarrative, NarrativeStep};
use crate::common::experiment::{EraExperiment, ExperimentStep};
use crate::common::walkthrough::EraDerivation;
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

    commands.insert_resource(EraParameters::from_tuples(&[
        ("Velocidade max", 200.0, 50.0, 500.0, 10.0),
        ("Raio atomo", 10.0, 5.0, 20.0, 1.0),
    ]));

    // Limitation text
    commands.insert_resource(LimitationText(
        "ATOMOS SEM DIVERSIDADE".to_string(),
        "Democrito propoe atomos identicos que\ndiferem apenas em forma e arranjo.\nSem diferencas de massa entre tipos,\nnao ha base para leis quantitativas.\nProporcoes definidas nas reacoes?\nImpossivel prever. -> Dalton (1803).".to_string(),
    ));
    commands.insert_resource(LimitationVisible(false));

    commands.insert_resource(EraEquations(
        "Cinematica:\n  p = m * v\n  E_k = (1/2) * m * v^2\n\nColisao elastica:\n  m1*v1 + m2*v2 = m1*v1' + m2*v2'".to_string(),
    ));
    commands.insert_resource(EquationsVisible(false));

    commands.insert_resource(EraEvidence {
        resolves: "Diversidade aparente da materia\npor forma e arranjo dos atomos".to_string(),
        fails: "Nenhuma previsao quantitativa\nNao explica proporcoes quimicas\nNao explica eletricidade".to_string(),
    });
    commands.insert_resource(EvidenceVisible(false));

    commands.insert_resource(EraQuiz(vec![
        QuestionData {
            text: "O que diferencia atomos neste modelo?".to_string(),
            options: vec![
                "Massa".to_string(),
                "Carga".to_string(),
                "Forma e arranjo".to_string(),
                "Cor".to_string(),
            ],
            correct: 2,
        },
        QuestionData {
            text: "Por que Democrito diz que atomos sao indivisiveis?".to_string(),
            options: vec![
                "Experimento com martelo".to_string(),
                "Argumento logico: divisao infinita e absurda".to_string(),
                "Observacao ao microscopio".to_string(),
                "Medicao de massa".to_string(),
            ],
            correct: 1,
        },
        QuestionData {
            text: "No modelo de Democrito, o que existe entre atomos?".to_string(),
            options: vec![
                "Eter luminifero".to_string(),
                "Ar comprimido".to_string(),
                "Vazio (kenon)".to_string(),
                "Fluido continuo".to_string(),
            ],
            correct: 2,
        },
    ]));
    commands.insert_resource(QuizState::default());

    commands.insert_resource(EraPredictions(vec![
        Prediction {
            question: "Se adicionarmos mais atomos, a temperatura\n(velocidade media) vai:".to_string(),
            options: vec![
                "Aumentar".to_string(),
                "Diminuir".to_string(),
                "Ficar igual".to_string(),
                "Oscilar".to_string(),
            ],
            correct: 2,
            explanation: "Conservacao de energia: novos atomos sao\nadicionados com vel. media do sistema.".to_string(),
        },
        Prediction {
            question: "Se dobrarmos o tamanho da arena mantendo\nos atomos, a frequencia de colisoes vai:".to_string(),
            options: vec![
                "Dobrar".to_string(),
                "Cair pela metade".to_string(),
                "Cair a 1/4".to_string(),
                "Ficar igual".to_string(),
            ],
            correct: 2,
            explanation: "Frequencia de colisao ~ n/V^2 em 2D.\nDobrar area = quadruplicar volume efetivo.".to_string(),
        },
    ]));
    commands.insert_resource(PredictState::default());

    commands.insert_resource(EraExperiment {
        name: "Experimento conceitual".to_string(),
        year: "~400 a.C.".to_string(),
        apparatus: "Argumento filosofico — divisao infinita da materia".to_string(),
        steps: vec![
            ExperimentStep {
                description: "Dividir materia repetidamente".to_string(),
                action: "Em algum ponto, a divisao deve parar: atomos".to_string(),
            },
            ExperimentStep {
                description: "Observar colisoes entre atomos".to_string(),
                action: "Atomos sao indivisiveis e identicos".to_string(),
            },
        ],
    });

    commands.insert_resource(EraDerivation(vec![
        "Hipotese: materia nao e infinitamente divisivel".to_string(),
        "Conclusao: existem particulas fundamentais — atomos".to_string(),
    ]));

    commands.insert_resource(EraNarrative(vec![
        NarrativeStep {
            text: "Observe: particulas identicas colidindo no vazio.".to_string(),
            action_hint: "Clique para adicionar atomos".to_string(),
        },
        NarrativeStep {
            text: "Todas sao iguais — mesma cor, mesmo tamanho.".to_string(),
            action_hint: "Pressione [L] para ver a limitacao".to_string(),
        },
        NarrativeStep {
            text: "Como explicar a diversidade da materia?".to_string(),
            action_hint: "Avance para Dalton [->]".to_string(),
        },
    ]));

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
    commands.remove_resource::<EraEquations>();
    commands.remove_resource::<EraQuiz>();
    commands.remove_resource::<EraPredictions>();
    commands.remove_resource::<EraParameters>();
    commands.remove_resource::<EraNarrative>();
    commands.remove_resource::<EraEvidence>();
    commands.remove_resource::<EraExperiment>();
    commands.remove_resource::<EraDerivation>();
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
