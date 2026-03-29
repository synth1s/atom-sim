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

/// Calcula o impulso de colisão elástica 1D entre duas massas.
/// Retorna (dv1, dv2) — as mudanças de velocidade.
///
///   v1' = ((m1-m2)v1 + 2m2·v2) / (m1+m2)
///   v2' = ((m2-m1)v2 + 2m1·v1) / (m1+m2)
#[allow(dead_code)]
pub fn elastic_collision_1d(m1: f32, v1: f32, m2: f32, v2: f32) -> (f32, f32) {
    let total = m1 + m2;
    if total < 1e-10 {
        return (0.0, 0.0);
    }
    let v1_new = ((m1 - m2) * v1 + 2.0 * m2 * v2) / total;
    let v2_new = ((m2 - m1) * v2 + 2.0 * m1 * v1) / total;
    (v1_new, v2_new)
}

/// Verifica se uma posição está fora dos limites da arena e retorna
/// a posição corrigida e se a velocidade deve ser invertida.
#[allow(dead_code)]
pub fn wall_reflect(pos: f32, radius: f32, half_extent: f32) -> (f32, bool) {
    if pos - radius < -half_extent {
        (-half_extent + radius, true)
    } else if pos + radius > half_extent {
        (half_extent - radius, true)
    } else {
        (pos, false)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wall_reflect_right_boundary() {
        // Partícula com x + r > half_width deve ser refletida
        let (new_pos, reflected) = wall_reflect(580.0, 10.0, 580.0);
        assert!(reflected, "Deve refletir na parede direita");
        assert!((new_pos - 570.0).abs() < 1e-5, "Posicao corrigida: {}", new_pos);
    }

    #[test]
    fn test_wall_reflect_left_boundary() {
        let (new_pos, reflected) = wall_reflect(-575.0, 10.0, 580.0);
        assert!(reflected, "Deve refletir na parede esquerda");
        assert!((new_pos - (-570.0)).abs() < 1e-5, "Posicao corrigida: {}", new_pos);
    }

    #[test]
    fn test_wall_reflect_inside() {
        let (new_pos, reflected) = wall_reflect(100.0, 10.0, 580.0);
        assert!(!reflected, "Nao deve refletir dentro da arena");
        assert!((new_pos - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_elastic_collision_1d_equal_mass() {
        // Massas iguais trocam velocidades
        let (v1_new, v2_new) = elastic_collision_1d(1.0, 5.0, 1.0, -3.0);
        assert!((v1_new - (-3.0)).abs() < 1e-5, "v1' = {}", v1_new);
        assert!((v2_new - 5.0).abs() < 1e-5, "v2' = {}", v2_new);
    }

    #[test]
    fn test_elastic_collision_1d_conserves_momentum() {
        let m1 = 2.0;
        let v1 = 3.0;
        let m2 = 5.0;
        let v2 = -1.0;
        let p_before = m1 * v1 + m2 * v2;

        let (v1_new, v2_new) = elastic_collision_1d(m1, v1, m2, v2);
        let p_after = m1 * v1_new + m2 * v2_new;
        assert!((p_before - p_after).abs() < 1e-4, "Momento: {} vs {}", p_before, p_after);
    }

    #[test]
    fn test_elastic_collision_1d_conserves_energy() {
        let m1 = 2.0;
        let v1 = 3.0;
        let m2 = 5.0;
        let v2 = -1.0;
        let ke_before = 0.5 * m1 * v1 * v1 + 0.5 * m2 * v2 * v2;

        let (v1_new, v2_new) = elastic_collision_1d(m1, v1, m2, v2);
        let ke_after = 0.5 * m1 * v1_new * v1_new + 0.5 * m2 * v2_new * v2_new;
        assert!((ke_before - ke_after).abs() < 1e-3, "Energia: {} vs {}", ke_before, ke_after);
    }

    #[test]
    fn test_elastic_collision_1d_stationary_target() {
        // Bola atinge alvo parado com massa infinitamente maior -> quase para
        let m1 = 1.0;
        let v1 = 10.0;
        let m2 = 1000.0;
        let v2 = 0.0;

        let (v1_new, _v2_new) = elastic_collision_1d(m1, v1, m2, v2);
        // Bola leve deve inverter velocidade (quase -v1)
        assert!(v1_new < 0.0, "Bola leve deve quicar: v1' = {}", v1_new);
    }
}
