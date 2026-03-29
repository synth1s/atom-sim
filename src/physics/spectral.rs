use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Bohr model constants and equations
//
// Postulate 1: L = mₑvr = nℏ (quantized angular momentum)
// Postulate 2: hν = Eₙ₁ - Eₙ₂ (frequency condition)
//
// Derived:
//   rₙ = n²a₀, where a₀ = 4πε₀ℏ²/(mₑe²) = 0.529 Å
//   Eₙ = -13.6/n² eV
//   R_H = mₑe⁴/(8ε₀²h³c) = 1.0974×10⁷ m⁻¹
//
// Triumph: R_H from first principles matches empirical value to 4 decimals.
// ---------------------------------------------------------------------------

/// Bohr radius: a₀ = 0.529 Å = 5.291×10⁻¹¹ m
pub const BOHR_RADIUS_M: f64 = 5.291_772_109e-11;

/// Rydberg energy: 13.605693 eV
pub const RYDBERG_ENERGY_EV: f64 = 13.605_693;

/// Rydberg constant: R_H = 1.0974×10⁷ m⁻¹
pub const RYDBERG_CONSTANT: f64 = 1.097_373_157e7;

/// Planck constant: h = 6.626×10⁻³⁴ J·s
#[allow(dead_code)]
pub const H_PLANCK: f64 = 6.626_070_15e-34;

/// Speed of light: c = 2.998×10⁸ m/s
#[allow(dead_code)]
pub const C_LIGHT: f64 = 2.997_924_58e8;

/// Raio da n-ésima órbita de Bohr (metros).
/// rₙ = n²·a₀
pub fn bohr_radius(n: u32) -> f64 {
    (n as f64).powi(2) * BOHR_RADIUS_M
}

/// Energia do n-ésimo nível de Bohr (eV).
/// Eₙ = -13.6/n² eV
pub fn bohr_energy(n: u32) -> f64 {
    -RYDBERG_ENERGY_EV / (n as f64).powi(2)
}

/// Comprimento de onda emitido na transição n₂ → n₁ (metros).
/// 1/λ = R_H·(1/n₁² - 1/n₂²)
pub fn transition_wavelength(n_upper: u32, n_lower: u32) -> f64 {
    let inv_lambda = RYDBERG_CONSTANT
        * (1.0 / (n_lower as f64).powi(2) - 1.0 / (n_upper as f64).powi(2));
    1.0 / inv_lambda
}

/// Energia do fóton emitido na transição (eV).
#[allow(dead_code)]
pub fn transition_energy(n_upper: u32, n_lower: u32) -> f64 {
    (bohr_energy(n_lower) - bohr_energy(n_upper)).abs()
}

/// Frequência do fóton emitido (Hz).
#[allow(dead_code)]
pub fn transition_frequency(n_upper: u32, n_lower: u32) -> f64 {
    let wavelength = transition_wavelength(n_upper, n_lower);
    C_LIGHT / wavelength
}

/// Converte comprimento de onda (nm) para cor RGB aproximada.
/// Baseado no espectro visível (380-780 nm).
pub fn wavelength_to_color(lambda_nm: f64) -> Color {
    let (r, g, b) = if lambda_nm < 380.0 {
        (0.4, 0.0, 0.6) // UV → violeta escuro
    } else if lambda_nm < 440.0 {
        let t = (lambda_nm - 380.0) / 60.0;
        (0.4 - 0.4 * t, 0.0, 0.6 + 0.4 * t)
    } else if lambda_nm < 490.0 {
        let t = (lambda_nm - 440.0) / 50.0;
        (0.0, t, 1.0)
    } else if lambda_nm < 510.0 {
        let t = (lambda_nm - 490.0) / 20.0;
        (0.0, 1.0, 1.0 - t)
    } else if lambda_nm < 580.0 {
        let t = (lambda_nm - 510.0) / 70.0;
        (t, 1.0, 0.0)
    } else if lambda_nm < 645.0 {
        let t = (lambda_nm - 580.0) / 65.0;
        (1.0, 1.0 - t, 0.0)
    } else if lambda_nm < 780.0 {
        let t = (lambda_nm - 645.0) / 135.0;
        (1.0 - 0.5 * t, 0.0, 0.0)
    } else {
        (0.5, 0.0, 0.0) // IR → vermelho escuro
    };

    Color::srgba(r as f32, g as f32, b as f32, 0.95)
}

/// Nome da série espectral baseado no nível inferior.
pub fn series_name(n_lower: u32) -> &'static str {
    match n_lower {
        1 => "Lyman (UV)",
        2 => "Balmer (visivel)",
        3 => "Paschen (IR)",
        4 => "Brackett (IR)",
        5 => "Pfund (IR)",
        _ => "?",
    }
}
