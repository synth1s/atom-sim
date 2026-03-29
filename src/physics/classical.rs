use bevy::prelude::*;

use crate::common::Arena;

/// Velocidade de uma partícula (px/s).
#[derive(Component, Default, Clone)]
pub struct Velocity(pub Vec2);

/// Raio físico de uma partícula.
#[derive(Component, Clone)]
pub struct Radius(pub f32);

/// Massa de uma partícula.
#[derive(Component, Clone)]
pub struct Mass(pub f32);

/// Marcador genérico para entidades que participam da física clássica
/// (movimento, colisão com paredes, colisão entre partículas).
/// Cada era que usa esses systems deve adicionar este componente às suas entidades.
#[derive(Component)]
pub struct PhysicsBody;

/// Move partículas segundo suas velocidades (cinemática linear).
pub fn move_particles(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity), With<PhysicsBody>>,
) {
    let dt = time.delta_secs();
    for (mut transform, vel) in query.iter_mut() {
        transform.translation.x += vel.0.x * dt;
        transform.translation.y += vel.0.y * dt;
    }
}

/// Colisões elásticas com as paredes da arena.
/// Conservação de momento: a parede tem massa infinita, então a partícula
/// simplesmente inverte a componente normal da velocidade.
/// Lê as dimensões diretamente do resource `Arena`.
pub fn bounce_walls(
    arena: Res<Arena>,
    mut query: Query<(&mut Transform, &mut Velocity, &Radius), With<PhysicsBody>>,
) {
    let arena_half_w = arena.half_width;
    let arena_half_h = arena.half_height;
    for (mut transform, mut vel, radius) in query.iter_mut() {
        let r = radius.0;
        let pos = &mut transform.translation;

        if pos.x - r < -arena_half_w {
            pos.x = -arena_half_w + r;
            vel.0.x = vel.0.x.abs();
        } else if pos.x + r > arena_half_w {
            pos.x = arena_half_w - r;
            vel.0.x = -vel.0.x.abs();
        }

        if pos.y - r < -arena_half_h {
            pos.y = -arena_half_h + r;
            vel.0.y = vel.0.y.abs();
        } else if pos.y + r > arena_half_h {
            pos.y = arena_half_h - r;
            vel.0.y = -vel.0.y.abs();
        }
    }
}

/// Colisões elásticas entre pares de partículas.
/// Conservação de momento e energia cinética:
///   v1' = v1 - (2*m2/(m1+m2)) * dot(v1-v2, x1-x2) / |x1-x2|^2 * (x1-x2)
///   v2' = v2 - (2*m1/(m1+m2)) * dot(v2-v1, x2-x1) / |x2-x1|^2 * (x2-x1)
pub fn collide_particles(
    mut query: Query<(Entity, &Transform, &mut Velocity, &Radius, &Mass), With<PhysicsBody>>,
) {
    let mut pairs: Vec<(Entity, Vec2, Vec2, f32, f32)> = Vec::new();
    for (entity, transform, vel, radius, mass) in query.iter() {
        pairs.push((
            entity,
            transform.translation.truncate(),
            vel.0,
            radius.0,
            mass.0,
        ));
    }

    let n = pairs.len();
    let mut velocity_changes: Vec<(Entity, Vec2)> = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            let (e1, p1, v1, r1, m1) = pairs[i];
            let (e2, p2, v2, r2, m2) = pairs[j];

            let delta = p1 - p2;
            let dist_sq = delta.length_squared();
            let min_dist = r1 + r2;

            if dist_sq < min_dist * min_dist && dist_sq > 0.0 {
                let dist = dist_sq.sqrt();
                let normal = delta / dist;

                // Velocidades relativas ao longo da normal
                let rel_vel = v1 - v2;
                let vel_along_normal = rel_vel.dot(normal);

                // Só resolver se estão se aproximando
                if vel_along_normal > 0.0 {
                    continue;
                }

                let total_mass = m1 + m2;
                let impulse = 2.0 * vel_along_normal / total_mass;

                velocity_changes.push((e1, -impulse * m2 * normal));
                velocity_changes.push((e2, impulse * m1 * normal));
            }
        }
    }

    for (entity, dv) in velocity_changes {
        if let Ok((_, _, mut vel, _, _)) = query.get_mut(entity) {
            vel.0 += dv;
        }
    }
}
