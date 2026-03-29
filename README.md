# atom-sim

**An interactive journey through the history of the atomic model**

[![CI](https://github.com/synth1s/atom-sim/actions/workflows/ci.yml/badge.svg)](https://github.com/synth1s/atom-sim/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.94-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## About

atom-sim is a 2D interactive atomic simulation built with Rust and Bevy that walks you through nine eras of atomic theory, from Democritus (~400 BC) to Dirac (1928). Each era is a self-contained Bevy plugin that visualizes the key concepts, experiments, and limitations of that historical model.

The simulation is designed as an educational tool with academic rigor: physical constants follow CODATA/NIST reference values, spectral calculations match known hydrogen emission lines, and quantum probability densities are computed from actual Laguerre/Legendre wavefunctions. Every commit in the repository represents a scientific milestone.

Whether you are a student exploring how our understanding of the atom evolved, or a Rust/Bevy developer looking for an ECS-driven physics simulation, atom-sim offers an engaging way to see two centuries of atomic physics come to life on screen.

## Eras

| # | Era | Year | Scientist | Key Concept |
|---|-----|------|-----------|-------------|
| 1 | Democritus | ~400 BC | Democritus | Indivisible spheres, elastic collisions |
| 2 | Dalton | 1803 | John Dalton | Elements with distinct mass/color, molecules |
| 3 | Thomson | 1897 | J.J. Thomson | Plum pudding model, cathode ray tube, e/m ratio |
| 4 | Rutherford | 1911 | Ernest Rutherford | Gold foil experiment, scattering histogram, orbital collapse |
| 5 | Bohr | 1913 | Niels Bohr | Quantized orbits, emission spectrum, photon transitions |
| 6 | Sommerfeld | 1916 | Arnold Sommerfeld | Elliptical orbits, fine structure, Zeeman effect |
| 7 | de Broglie | 1924 | Louis de Broglie | Standing waves, wave-particle duality, double slit |
| 8 | Schrodinger | 1926 | Erwin Schrodinger | Probability cloud \|psi\|^2, wavefunction collapse by click |
| 9 | Dirac | 1928 | Paul Dirac | Pair creation, spin, relativistic fine structure |

## Screenshots

<!-- TODO: adicionar screenshots -->

*Screenshots will be added soon. Run the simulation to explore each era visually.*

## Controls

### Global

| Key | Action |
|-----|--------|
| `1` - `9` | Switch to era 1 through 9 |
| `Space` | Pause / Resume simulation |
| `L` | Toggle limitations panel for the current era |
| `X` | Export simulation data to CSV |
| `Scroll` | Zoom in / out (camera) |

### Era-Specific

| Key | Era(s) | Action |
|-----|--------|--------|
| `Arrow Up/Down` | Bohr, Sommerfeld, de Broglie, Schrodinger, Dirac, Thomson, Rutherford | Change quantum number n (or adjust parameters) |
| `Arrow Left/Right` | Sommerfeld, Schrodinger, Dirac | Change secondary quantum number (k, l, or j) |
| `Q`, `W`, `R`, `T`, `Y` | Bohr, de Broglie | Jump to specific quantum levels (n=1,2,4,5,6) |
| `E` | Dalton | Cycle through elements |
| `F` | Thomson, Rutherford, de Broglie | Fire particle / toggle mode |
| `A` | Thomson | Toggle animation |
| `D` | de Broglie | Toggle double-slit experiment |
| `M` | Sommerfeld, Schrodinger | Cycle magnetic quantum number m |
| `B` / `V` | Sommerfeld | Increase / decrease magnetic field (Zeeman) |
| `P` | Dirac | Toggle pair creation |
| `C` | Rutherford | Toggle collapse visualization |
| `Left Click` | Democritus, Dalton, Schrodinger | Spawn particle / collapse wavefunction |

## Physics Implemented

The simulation implements progressively more sophisticated physics across its nine eras. Classical mechanics handle elastic collisions with momentum and energy conservation (Democritus, Dalton). Coulomb electrostatics drive Thomson's cathode ray deflection and Rutherford's scattering, computing the experimental e/m ratio. Bohr's model uses the Rydberg formula to produce accurate hydrogen emission wavelengths, converting them to visible-spectrum RGB colors. Sommerfeld extends this with relativistic fine-structure corrections and Zeeman splitting under external magnetic fields. The quantum eras compute actual hydrogen wavefunctions: radial components via associated Laguerre polynomials, angular components via associated Legendre polynomials, and probability densities |psi(r,theta)|^2 rendered as a 3000-point cloud (Schrodinger). Dirac's era introduces relativistic energy levels with spin-orbit coupling. All fundamental constants (electron mass, elementary charge, Bohr radius, Rydberg constant, fine-structure constant) follow CODATA/NIST reference values.

## Build

### Requirements

- **Rust** 1.85+ (edition 2024)
- **Bevy** 0.18 (pulled automatically via Cargo)

### Run

```bash
cargo run
```

### Release Build

```bash
cargo run --release
```

### Tests

```bash
cargo test
```

The test suite validates spectral calculations (Bohr energy levels, orbital radii, Rydberg wavelengths) and quantum mechanics functions (Laguerre/Legendre polynomials, radial wavefunctions, probability densities) against known reference values with explicit tolerances.

## Project Structure

```
src/
├── main.rs              # Entry point, registers plugins and states
├── common/
│   ├── mod.rs           # SimulationState, ActiveEra (9 variants), Arena
│   ├── camera.rs        # Camera2d with zoom (scroll)
│   ├── ui.rs            # HUD text, era controls, limitations panel
│   ├── tooltip.rs       # Interactive tooltip system (hover)
│   └── export.rs        # CSV export (key X)
├── eras/                # Each era = independent Bevy Plugin
│   ├── democritus.rs    # Identical spheres, elastic collisions
│   ├── dalton.rs        # Elements with mass/color, molecules
│   ├── thomson.rs       # Plum pudding, cathode ray tube
│   ├── rutherford.rs    # Gold foil, scattering histogram
│   ├── bohr.rs          # Quantized orbits, emission spectrum
│   ├── sommerfeld.rs    # Ellipses, fine structure, Zeeman
│   ├── de_broglie.rs    # Standing waves, double slit
│   ├── schrodinger.rs   # |psi|^2 cloud, wavefunction collapse
│   └── dirac.rs         # Pair creation, spin, fine structure
└── physics/
    ├── classical.rs     # Movement, wall bouncing, collisions
    ├── coulomb.rs       # Electrostatics, e/m ratio
    ├── spectral.rs      # Bohr model, Rydberg formula
    └── quantum.rs       # Laguerre, Legendre, wavefunctions
```

## License

[MIT](LICENSE)
