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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_from_crossed_fields() {
        // v = E/B. Para E=1000 V/m e B=0.01 T: v = 100000 m/s
        let v = velocity_from_crossed_fields(1000.0, 0.01);
        assert!((v - 100000.0).abs() < 1e-2, "v = E/B = {}", v);
    }

    #[test]
    fn test_velocity_from_crossed_fields_zero_b() {
        // B=0 deve retornar 0 (proteção contra divisão por zero)
        let v = velocity_from_crossed_fields(1000.0, 0.0);
        assert!((v - 0.0).abs() < 1e-10, "B=0 deve retornar 0, got {}", v);
    }

    #[test]
    fn test_charge_to_mass_ratio() {
        // Cenário: Thomson mede e/m
        // e/m = 2·y·E / (B²·L²)
        // Com y=0.01m, E=10000 V/m, B=0.001 T, L=0.1 m:
        // e/m = 2 * 0.01 * 10000 / (0.001^2 * 0.1^2) = 200 / 1e-8 = 2e10
        let ratio = charge_to_mass_ratio(0.01, 10000.0, 0.001, 0.1);
        let expected = 2.0e10;
        let rel_err = ((ratio - expected) / expected).abs();
        assert!(rel_err < 1e-4, "e/m = {}, expected {}", ratio, expected);
    }

    #[test]
    fn test_electron_deflection() {
        // y = q·E·L² / (2·m·v²)
        // Com q=1.0, E=100.0, L=0.5, m=1.0, v=10.0:
        // y = 1.0 * 100.0 * 0.25 / (2.0 * 1.0 * 100.0) = 25 / 200 = 0.125
        let y = electron_deflection(100.0, 0.5, 1.0, 1.0, 10.0);
        assert!((y - 0.125).abs() < 1e-5, "deflection = {}", y);
    }

    #[test]
    fn test_electron_deflection_zero_velocity() {
        let y = electron_deflection(100.0, 0.5, 1.0, 1.0, 0.0);
        assert!((y - 0.0).abs() < 1e-10, "v=0 deve retornar 0, got {}", y);
    }

    #[test]
    fn test_thomson_oscillation_frequency() {
        // Para H (1 elétron, raio atômico ~1 Å):
        // ω ~ 4.4×10^16 rad/s (ordem de grandeza)
        let omega = thomson_oscillation_frequency(1.0, 1.0);
        assert!(omega > 1e15, "omega deve ser > 1e15, got {}", omega);
        assert!(omega < 1e18, "omega deve ser < 1e18, got {}", omega);
        // Valor mais preciso: ~4.4e16
        let order = (omega as f64).log10();
        assert!((order - 16.0).abs() < 1.0, "log10(omega) ~ 16, got {}", order);
    }

    #[test]
    fn test_charge_to_mass_ratio_zero_b() {
        let ratio = charge_to_mass_ratio(0.01, 10000.0, 0.0, 0.1);
        assert!((ratio - 0.0).abs() < 1e-10, "B=0 deve retornar 0");
    }

    #[test]
    fn test_charge_to_mass_ratio_zero_length() {
        let ratio = charge_to_mass_ratio(0.01, 10000.0, 0.001, 0.0);
        assert!((ratio - 0.0).abs() < 1e-10, "L=0 deve retornar 0");
    }
}
