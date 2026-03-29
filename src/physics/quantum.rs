/// Funções quânticas para o átomo de Hidrogênio.
///
/// Solução da equação de Schrödinger independente do tempo:
///   Ĥψ = Eψ → [-ℏ²/(2m)∇² + V(r)]ψ = Eψ
///
/// Para V = -e²/(4πε₀r):
///   ψₙₗₘ(r,θ,φ) = Rₙₗ(r) · Yₗᵐ(θ,φ)
///
/// Números quânticos emergem naturalmente das condições de contorno:
///   n = 1,2,3,...      (principal — de r)
///   l = 0,1,...,n-1    (azimutal — de θ)
///   m = -l,...,0,...,+l (magnético — de φ)

use super::spectral::BOHR_RADIUS_M;

/// Polinômio de Laguerre generalizado L_n^α(x) por recorrência.
///
/// L_0^α(x) = 1
/// L_1^α(x) = 1 + α - x
/// (n+1)L_{n+1}^α(x) = (2n+1+α-x)L_n^α(x) - (n+α)L_{n-1}^α(x)
pub fn laguerre(n: u32, alpha: u32, x: f64) -> f64 {
    if n == 0 {
        return 1.0;
    }
    if n == 1 {
        return 1.0 + alpha as f64 - x;
    }

    let mut l_prev = 1.0;
    let mut l_curr = 1.0 + alpha as f64 - x;
    let a = alpha as f64;

    for k in 1..n {
        let kf = k as f64;
        let l_next = ((2.0 * kf + 1.0 + a - x) * l_curr - (kf + a) * l_prev) / (kf + 1.0);
        l_prev = l_curr;
        l_curr = l_next;
    }

    l_curr
}

/// Polinômio de Legendre associado P_l^m(x) por recorrência.
///
/// P_m^m(x) = (-1)^m · (2m-1)!! · (1-x²)^(m/2)
/// P_{m+1}^m(x) = x · (2m+1) · P_m^m(x)
/// (l-m)P_l^m(x) = x(2l-1)P_{l-1}^m(x) - (l+m-1)P_{l-2}^m(x)
pub fn associated_legendre(l: u32, m_abs: u32, x: f64) -> f64 {
    if m_abs > l {
        return 0.0;
    }

    // P_m^m
    let mut pmm = 1.0;
    if m_abs > 0 {
        let somx2 = (1.0 - x * x).sqrt();
        let mut fact = 1.0;
        for _ in 1..=m_abs {
            pmm *= -fact * somx2;
            fact += 2.0;
        }
    }

    if l == m_abs {
        return pmm;
    }

    // P_{m+1}^m
    let pmm1 = x * (2 * m_abs + 1) as f64 * pmm;
    if l == m_abs + 1 {
        return pmm1;
    }

    // Recorrência geral
    let mut p_prev = pmm;
    let mut p_curr = pmm1;
    for ll in (m_abs + 2)..=l {
        let llf = ll as f64;
        let mf = m_abs as f64;
        let p_next = (x * (2.0 * llf - 1.0) * p_curr - (llf + mf - 1.0) * p_prev)
            / (llf - mf);
        p_prev = p_curr;
        p_curr = p_next;
    }

    p_curr
}

/// Parte radial da função de onda: Rₙₗ(r)
/// Rₙₗ(r) = Nₙₗ · ρˡ · e^(-ρ/2) · L_{n-l-1}^{2l+1}(ρ)
/// onde ρ = 2r/(na₀)
pub fn hydrogen_radial(n: u32, l: u32, r_meters: f64) -> f64 {
    let a0 = BOHR_RADIUS_M;
    let rho = 2.0 * r_meters / (n as f64 * a0);

    // Normalização (simplificada para visualização)
    let norm = ((2.0 / (n as f64 * a0)).powi(3)
        * factorial(n - l - 1) as f64
        / (2.0 * n as f64 * factorial(n + l) as f64))
    .sqrt();

    let lag = laguerre(n - l - 1, 2 * l + 1, rho);

    norm * rho.powi(l as i32) * (-rho / 2.0).exp() * lag
}

/// Densidade de probabilidade no plano 2D (corte z=0):
/// P(x,y) = |ψ(r,θ,φ=0)|² para corte no plano xz
///
/// Para visualização 2D, usamos corte no plano que passa pelo eixo z:
/// (x,y) → (r = √(x²+y²), θ = x.atan2(y), φ = 0)
///
/// θ é o ângulo polar medido a partir do eixo z (que corresponde ao eixo y
/// no display 2D). Assim, `x.atan2(y)` retorna 0 quando o ponto está no
/// eixo y positivo e π/2 quando está no eixo x positivo.
pub fn probability_density_2d(n: u32, l: u32, m: i32, x: f64, y: f64) -> f64 {
    let r = (x * x + y * y).sqrt();
    if r < 1e-15 {
        if l == 0 {
            // Para s orbitais, |ψ(0)|² é finito
            let r_small = 1e-13;
            let radial = hydrogen_radial(n, l, r_small);
            return radial * radial; // Y_0^0 = 1/√(4π), constante
        }
        return 0.0; // Para l>0, ψ(0)=0
    }

    let theta = x.atan2(y); // Ângulo polar medido do eixo z (y no display)

    // Parte radial
    let radial = hydrogen_radial(n, l, r);

    // Parte angular simplificada (corte φ=0):
    // |Yₗᵐ|² ∝ |Pₗᵐ(cos θ)|²
    let m_abs = m.unsigned_abs();
    let plm = associated_legendre(l, m_abs, theta.cos());

    // Normalização angular
    let angular_norm = (2.0 * l as f64 + 1.0) / (4.0 * std::f64::consts::PI)
        * factorial(l - m_abs) as f64
        / factorial(l + m_abs) as f64;

    radial * radial * angular_norm * plm * plm
}

/// Fatorial simples (suficiente para n ≤ 20).
pub(crate) fn factorial(n: u32) -> u64 {
    (1..=n as u64).product::<u64>().max(1)
}

/// Gera pontos amostrados de |ψ|² usando rejection sampling.
/// Retorna lista de (x, y) em metros.
pub fn sample_orbital_points(
    n: u32,
    l: u32,
    m: i32,
    num_points: usize,
    scale: f64,
) -> Vec<(f32, f32)> {
    use rand::Rng;
    let mut rng = rand::rng();
    let mut points = Vec::with_capacity(num_points);

    // Encontrar máximo da densidade para rejection sampling
    let max_r = (n as f64).powi(2) * BOHR_RADIUS_M * 3.0;
    let mut max_density = 0.0_f64;
    for _ in 0..500 {
        let x = rng.random_range(-max_r..max_r);
        let y = rng.random_range(-max_r..max_r);
        let d = probability_density_2d(n, l, m, x, y);
        if d > max_density {
            max_density = d;
        }
    }
    max_density *= 1.2; // Margem de segurança

    if max_density < 1e-30 {
        return points;
    }

    let mut attempts = 0;
    while points.len() < num_points && attempts < num_points * 100 {
        let x = rng.random_range(-max_r..max_r);
        let y = rng.random_range(-max_r..max_r);
        let d = probability_density_2d(n, l, m, x, y);

        if rng.random_range(0.0..max_density) < d {
            points.push(((x * scale) as f32, (y * scale) as f32));
        }
        attempts += 1;
    }

    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_laguerre_l0() {
        // L_0^a(x) = 1 para qualquer a, x
        assert!((laguerre(0, 0, 5.0) - 1.0).abs() < 1e-10);
        assert!((laguerre(0, 3, 2.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_laguerre_l1() {
        // L_1^2(3) = 1 + 2 - 3 = 0
        assert!((laguerre(1, 2, 3.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_associated_legendre_p10() {
        // P_1^0(x) = x → P_1^0(0.5) = 0.5
        let val = associated_legendre(1, 0, 0.5);
        assert!((val - 0.5).abs() < 1e-10, "P_1^0(0.5) = {}", val);
    }

    #[test]
    fn test_associated_legendre_p11() {
        // P_1^1(x) = -(1-x^2)^{1/2} → P_1^1(0.5) = -sqrt(0.75)
        let val = associated_legendre(1, 1, 0.5);
        let expected = -(1.0 - 0.25_f64).sqrt();
        assert!((val - expected).abs() < 1e-10, "P_1^1(0.5) = {}", val);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(10), 3628800);
    }

    #[test]
    fn test_probability_density_1s_origin() {
        // |ψ_1s(0)|² > 0 (orbital s tem densidade finita na origem)
        let density = probability_density_2d(1, 0, 0, 0.0, 0.0);
        assert!(density > 0.0, "|psi_1s(0)|^2 = {}", density);
    }

    #[test]
    fn test_2p_orbital_symmetry() {
        // 2p orbital (l=1, m=0): lóbulos ao longo do eixo z (y no display)
        // Densidade no eixo y deve ser MAIOR que no eixo x para mesma distância
        let d = 2.0 * BOHR_RADIUS_M;
        let on_y = probability_density_2d(2, 1, 0, 0.0, d);
        let on_x = probability_density_2d(2, 1, 0, d, 0.0);
        assert!(on_y > on_x * 5.0, "2p lobe should be along y axis: on_y={} on_x={}", on_y, on_x);
    }
}
