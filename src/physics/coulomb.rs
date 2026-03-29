use bevy::prelude::*;

/// Carga elétrica de uma partícula (unidades arbitrárias para visualização).
#[derive(Component, Clone, Copy)]
#[allow(dead_code)]
pub struct Charge(pub f32);

/// Calcula a frequência de oscilação de um elétron no modelo de Thomson.
///
/// No pudim de passas, um elétron deslocado de sua posição de equilíbrio
/// dentro de uma esfera de carga positiva uniforme experimenta uma força
/// restauradora linear (como mola):
///
///   F = -(n_e · e²) / (3ε₀ · R³) · r
///   ω = √(n_e · e² / (3ε₀ · mₑ · R³))
///
/// Para H (n_e=1, R~10⁻¹⁰ m): ω ~ 4.4×10¹⁶ rad/s → λ ~ 43 nm (UV)
/// Isto já indica problemas com as previsões espectrais de Thomson.
#[allow(dead_code)]
pub fn thomson_oscillation_frequency(n_electrons: f32, atom_radius: f32) -> f32 {
    // Constantes em unidades SI
    let e: f64 = 1.602e-19;         // Coulombs
    let m_e: f64 = 9.109e-31;       // kg
    let eps0: f64 = 8.854e-12;      // F/m
    let r: f64 = atom_radius as f64 * 1e-10; // converter de Å para metros

    let omega = ((n_electrons as f64 * e * e) / (3.0 * eps0 * m_e * r.powi(3))).sqrt();
    omega as f32
}

/// Calcula a deflexão de um elétron num campo elétrico uniforme.
///
/// Thomson (1897) usou esta deflexão para medir e/m:
///   y = (e·E·L²) / (2·m·v²)
///
/// Onde L é o comprimento das placas e v = E/B quando os campos se cancelam.
pub fn electron_deflection(
    e_field: f32,
    plate_length: f32,
    mass: f32,
    charge: f32,
    velocity: f32,
) -> f32 {
    if velocity.abs() < 1e-10 {
        return 0.0;
    }
    charge * e_field * plate_length * plate_length / (2.0 * mass * velocity * velocity)
}

/// Quando campos E e B se cancelam: eE = evB → v = E/B
pub fn velocity_from_crossed_fields(e_field: f32, b_field: f32) -> f32 {
    if b_field.abs() < 1e-10 {
        return 0.0;
    }
    e_field / b_field
}

/// e/m calculado a partir da deflexão:
///   e/m = 2·y·v² / (E·L²) = 2·y·E / (B²·L²)
pub fn charge_to_mass_ratio(deflection_y: f32, e_field: f32, b_field: f32, plate_length: f32) -> f32 {
    if b_field.abs() < 1e-10 || plate_length.abs() < 1e-10 {
        return 0.0;
    }
    2.0 * deflection_y * e_field / (b_field * b_field * plate_length * plate_length)
}
