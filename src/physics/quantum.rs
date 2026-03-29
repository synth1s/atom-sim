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
/// (x,y) → (r = √(x²+y²), θ = atan2(√(x²),y), φ = 0)
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

    let theta = y.atan2(x); // Ângulo no plano de corte

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
fn factorial(n: u32) -> u64 {
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
