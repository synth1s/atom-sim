# Changelog — atom-sim

> Atualizado: 2026-03-29
> Registro do que mudou e por que. Agentes devem consultar para evitar retrabalho.

## 2026-03-29

### Sprint de Melhorias (commit 40a38d3)
- **Glow nos fotons** (SPRINT-01): Halos luminosos em Bohr e Dirac. 3x raio, alpha 0.15-0.18.
- **Tooltips** (SPRINT-02): Sistema generico em common/tooltip.rs. Aplicado em 3 eras.
- **CI/CD** (SPRINT-04): .github/workflows/ci.yml com fmt+clippy+test+build.
- **Exportar CSV** (SPRINT-05): common/export.rs, tecla X, 4 eras suportadas.
- **Limitacoes** (SPRINT-06): Painel overlay tecla L, 9 eras com textos do PH-001.

### Correcoes Criticas (commit 9d53945)
- **Bug theta corrigido** em quantum.rs:124 — `y.atan2(x)` → `x.atan2(y)`. Orbitais 2p agora com lobulos no eixo vertical correto. Teste de simetria adicionado.
- **Carbono visivel** em dalton.rs — cor de (0.25,0.25,0.25) para (0.45,0.45,0.45). Contraste ~4:1.
- **Textos reduzidos** em Schrodinger, Dirac, Sommerfeld — cabem na viewport.
- **Handles pre-alocados** em 6 eras — eliminado leak de mesh/material por frame.
- **Orbitas Sommerfeld redesenham** — OrbitSegment marker + redraw quando n muda.
- **Controles dinamicos** — EraControls Resource atualizado por era.
- **13 testes unitarios** adicionados (6 spectral + 7 quantum), todos passando.

### Infraestrutura
- Repositorio transferido para synth1s/atom-sim
- GitHub Project criado: https://github.com/orgs/synth1s/projects/3
- 14 issues criadas (5 fechadas/done, 9 abertas com descricao completa)

## 2026-03-28

### 9 Eras Implementadas (commits 1c0d0ca a 2275d8b)
- v0.1-democritus: Esferas identicas no vazio
- v0.2-dalton: Elementos com massa/cor, reacoes, conservacao
- v0.3-thomson: Eletron, pudim de passas, tubo catodico
- v0.4-rutherford: Nucleo, folha de ouro, histograma, colapso Larmor
- v0.5-bohr: Orbitas quantizadas, espectro H, Rydberg derivado
- v0.6-sommerfeld: Elipses, alfa=1/137, Zeeman normal
- v0.7-debroglie: Ondas estacionarias, fenda dupla
- v0.8-schrodinger: |psi|^2, orbitais s/p/d, colapso
- v0.9-dirac: Spin, positron, criacao de pares, estrutura fina exata

### Equipe Formada
- QM-001 (Gestor Qualidade): Definiu criterios de avaliacao
- PH-001 (Fisica): Avaliou — APROVADO 8.80/10
- SE-001 (Engenharia): Avaliou — REJEITADO 6.61/10 (zero testes, leak handles)
- UX-001 (UX): Avaliou — APROVADO condicional 6.67/10 (carbono invisivel, texto overflow)
- CP-001 (Produto): 20 ideias de evolucao
- AD-001 (Dados): Quantificacao das 20 ideias com Score ROI
- TL-001 (Tech Lead): Plano de execucao do sprint
- GP-001 (Gerente): Execucao do sprint + issues GitHub
