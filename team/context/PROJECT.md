# Estado do Projeto — atom-sim

> Atualizado: 2026-03-29
> Este arquivo deve ser lido por TODOS os agentes antes de iniciar qualquer tarefa.

## Visao Geral

Simulacao atomica 2D interativa em Rust/Bevy que percorre a historia do modelo atomico em 9 eras (Democrito ~400 a.C. a Dirac 1928). Cada commit no git representa um marco cientifico. Projeto educacional com rigor academico.

## Stack

- **Linguagem:** Rust 1.94, edition 2024
- **Engine:** Bevy 0.18.1
- **Dependencias:** bevy, rand 0.9 (apenas 2 crates)
- **Plataforma alvo:** Windows (primario), Linux/WASM (futuro)
- **Repositorio:** https://github.com/synth1s/atom-sim

## Metricas do Codigo

- **LOC total:** ~5600 (apos sprint de melhorias)
- **Arquivos fonte:** 21 (19 originais + tooltip.rs + export.rs)
- **Testes unitarios:** 13 (6 spectral + 7 quantum, todos passando)
- **Warnings:** 0 (cargo check limpo)
- **Eras implementadas:** 9

## Arquitetura

```
src/
├── main.rs              # Entry point, registra plugins e states
├── common/
│   ├── mod.rs           # SimulationState, ActiveEra (9 variantes), Arena
│   ├── camera.rs        # Camera2d com zoom (scroll), MainCamera marker
│   ├── ui.rs            # HudText, EraControls, LimitationText/Visible, StatusText
│   ├── tooltip.rs       # Tooltip component + TooltipPlugin (hover 25px)
│   └── export.rs        # ExportableData resource + ExportPlugin (tecla X → CSV)
├── eras/                # Cada era = Plugin Bevy independente
│   ├── mod.rs           # Re-exporta os 9 modulos
│   ├── democritus.rs    # Esferas identicas, colisoes elasticas
│   ├── dalton.rs        # Elementos com massa/cor, moleculas, DaltonParticleAssets
│   ├── thomson.rs       # Pudim de passas, tubo catodico, ThomsonParticleAssets
│   ├── rutherford.rs    # Folha de ouro, histograma, colapso, RutherfordParticleAssets
│   ├── bohr.rs          # Orbitas quantizadas, espectro, fotons, BohrParticleAssets
│   ├── sommerfeld.rs    # Elipses, estrutura fina, Zeeman, OrbitSegment redraw
│   ├── de_broglie.rs    # Ondas estacionarias, fenda dupla, DeBroglieParticleAssets
│   ├── schrodinger.rs   # Nuvem |psi|^2, 3000 pontos, colapso por clique
│   └── dirac.rs         # Criacao de pares, spin, estrutura fina, DiracParticleAssets
└── physics/
    ├── mod.rs
    ├── classical.rs     # move_particles, bounce_walls, collide_particles (O(n^2))
    ├── coulomb.rs       # thomson_oscillation_frequency, electron_deflection, e/m ratio
    ├── spectral.rs      # Bohr: energia, raio, Rydberg, wavelength_to_color (+6 testes)
    └── quantum.rs       # Laguerre, Legendre, hydrogen_radial, probability_density_2d (+7 testes)
```

## Padrao de Era (template para novas)

Cada era segue este padrao:
1. `XxxPlugin` implementa `Plugin` com OnEnter/OnExit/Update systems
2. `XxxEntity` marker component para cleanup via despawn
3. `XxxParticleAssets` Resource com handles pre-alocados (mesh + material)
4. `setup_xxx`: spawna entidades, insere resources (EraControls, LimitationText, etc)
5. `cleanup_xxx`: despawna XxxEntity, remove resources
6. Systems condicionais: `.run_if(in_state(ActiveEra::Xxx)).run_if(in_state(SimulationState::Running))`
7. HudText atualizado no setup com contexto historico

## Features Implementadas (Sprint)

- Glow nos fotons/particulas (Bohr, Dirac)
- Tooltips interativos (Bohr, Rutherford, Schrodinger)
- Exportar CSV (tecla X, 4 eras)
- Painel de limitacoes (tecla L, 9 eras)
- CI/CD GitHub Actions (fmt + clippy + test + build)
- Controles dinamicos por era (EraControls resource)

## Problemas Conhecidos

- collide_particles em classical.rs nao filtra por era (query global)
- bounce_walls usa valores hardcoded (580, 320) em vez de Arena resource
- Orbitas de Bohr n=5/n=6 saem da viewport (raio 648px vs viewport 640px)
- HUD usa Text2d com posicoes absolutas (nao responsivo)

## CI/CD

- `.github/workflows/ci.yml`: fmt, clippy, test, build --release
- Runner: ubuntu-latest com deps Bevy (libasound2-dev, libudev-dev, etc)
- Cache: Cargo registry + target
