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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bohr_energy_ground_state() {
        let e = bohr_energy(1);
        assert!((e - (-13.605693)).abs() < 0.001, "E(1) = {}", e);
    }

    #[test]
    fn test_bohr_energy_n2() {
        let e = bohr_energy(2);
        let expected = -13.605693 / 4.0;
        assert!((e - expected).abs() < 0.001, "E(2) = {}", e);
    }

    #[test]
    fn test_bohr_radius_ground_state() {
        let r = bohr_radius(1);
        assert!((r - 5.291_772_109e-11).abs() < 1e-15, "r(1) = {}", r);
    }

    #[test]
    fn test_hydrogen_balmer_alpha() {
        // Transição 3 -> 2: λ ≈ 656.3 nm
        let lambda = transition_wavelength(3, 2);
        let lambda_nm = lambda * 1e9;
        assert!((lambda_nm - 656.3).abs() < 1.0, "Balmer alpha = {} nm", lambda_nm);
    }

    #[test]
    fn test_hydrogen_lyman_alpha() {
        // Transição 2 -> 1: λ ≈ 121.5 nm
        let lambda = transition_wavelength(2, 1);
        let lambda_nm = lambda * 1e9;
        assert!((lambda_nm - 121.5).abs() < 1.0, "Lyman alpha = {} nm", lambda_nm);
    }

    #[test]
    fn test_rydberg_constant() {
        assert!((RYDBERG_CONSTANT - 1.0974e7).abs() < 1e4, "R_H = {}", RYDBERG_CONSTANT);
    }

    #[test]
    fn test_transition_energy_balmer_alpha() {
        // E(3->2) = 13.6*(1/4 - 1/9) = 13.6 * 5/36 ~ 1.889 eV
        let e = transition_energy(3, 2);
        assert!((e - 1.889).abs() < 0.05, "E(3->2) = {} eV, expected ~1.889", e);
    }

    #[test]
    fn test_wavelength_to_color_red() {
        // 656nm (Balmer alpha) deve ter componente R dominante
        let color = wavelength_to_color(656.0);
        let srgba = color.to_srgba();
        assert!(srgba.red > 0.5, "656nm R deve ser > 0.5, got {}", srgba.red);
        assert!(srgba.green < 0.3, "656nm G deve ser < 0.3, got {}", srgba.green);
        assert!(srgba.blue < 0.1, "656nm B deve ser < 0.1, got {}", srgba.blue);
    }

    #[test]
    fn test_wavelength_to_color_blue() {
        // 486nm (Balmer beta, H-beta) deve ter componente B dominante
        let color = wavelength_to_color(486.0);
        let srgba = color.to_srgba();
        assert!(srgba.blue > 0.5, "486nm B deve ser > 0.5, got {}", srgba.blue);
        // Nesta faixa (440-490), g cresce e b=1.0
        assert!(srgba.blue >= srgba.red, "486nm B >= R: B={} R={}", srgba.blue, srgba.red);
    }

    #[test]
    fn test_series_name_all() {
        assert_eq!(series_name(1), "Lyman (UV)");
        assert_eq!(series_name(2), "Balmer (visivel)");
        assert_eq!(series_name(3), "Paschen (IR)");
        assert_eq!(series_name(4), "Brackett (IR)");
        assert_eq!(series_name(5), "Pfund (IR)");
        assert_eq!(series_name(6), "?");
    }

    #[test]
    fn test_transition_frequency_positive() {
        // Frequência de Lyman alpha deve ser positiva e da ordem de 10^15 Hz
        let freq = transition_frequency(2, 1);
        assert!(freq > 1e14, "freq Lyman alpha > 1e14 Hz, got {}", freq);
        assert!(freq < 1e16, "freq Lyman alpha < 1e16 Hz, got {}", freq);
    }
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
