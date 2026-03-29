# Roadmap atom-sim — Abril a Setembro 2026

> Produzido por AD-001 | Data: 2026-03-29

## Resumo

| Mes | Milestone | LOC | Issues | Risco |
|-----|-----------|-----|--------|-------|
| 1 (Abr) | Vitrine & Navegacao | 260 | #14, #6 | 1.5 |
| 2 (Mai) | Transicoes & Polimento | 350 | #7, #9 | 4.5 |
| 3 (Jun) | Apresentacao Automatica | 300 | #8 | 3.0 |
| 4 (Jul) | Educacao Profunda | 400 | #10 | 3.0 |
| 5 (Ago) | Comparacao & Nova Fisica | 650 | #11, #12a | 5.5 |
| 6 (Set) | 3D & Lancamento | 1100 | #12b, #13 | 8.0 |

**LOC total projetado:** ~3060 (de ~5700 para ~8760)

## Mes 1 — Abril: Vitrine & Navegacao

- **#14 README** (Score 13.33): screenshots, GIF, tabela de eras, controles
- **#6 Timeline** (Score 8.00): barra navegavel no rodape

Entregavel: projeto apresentavel publicamente.
Decisoes: quais eras para screenshots, licenca MIT, design da timeline.

## Mes 2 — Maio: Transicoes & Polimento

- **#7 Transicao animada** (Score 6.00): fade 0.3s entre eras. Dep: #6
- **#9 Build WASM** (Score 4.44): rodar no browser. Risco 5/10

Entregavel: navegacao fluida + acessivel via web.
Decisoes: hospedagem WASM, duracao da transicao.

## Mes 3 — Junho: Apresentacao Automatica

- **#8 Auto-Tour** (Score 5.89): F5 inicia tour pelas 9 eras. Dep: #6, #7

Entregavel: demo autonomo para aulas e feiras.
Decisoes: duracao total, acoes por era, narrador textual ou nao.

## Mes 4 — Julho: Educacao Profunda

- **#10 Modo guiado** (Score 4.34): narrativa passo-a-passo com condicoes

Entregavel: ferramenta pedagogica completa.
Decisoes: nivel de profundidade, idioma, reutilizar textos de limitacoes.

## Mes 5 — Agosto: Comparacao & Nova Fisica

- **#11 Comparacao lado-a-lado** (Score 2.62): split view com 2 eras
- **#12 Era QED parte 1** (Score 2.31): diagramas de Feynman

Entregavel: comparacao entre modelos + nova era.
Decisoes: pares de eras default, nivel de detalhe QED.
Risco alto: refatoracao de coordenadas impacta 9 arquivos.

## Mes 6 — Setembro: 3D & Lancamento

- **#12 Era QED parte 2**: Lamb shift, g-2
- **#13 Transicao 2D para 3D** (Score 1.42): orbitais volumetricos

Entregavel: visualizacao 3D + QED completa + v1.0.
Decisoes: compatibilidade bevy_panorbit_camera, criterios de lancamento.
Risco muito alto (8/10): novo crate, integracao 2D/3D.

## Plano de Contingencia (Mes 6)

Se 3D falhar por incompatibilidade, substituir por:
- i18n (~200 LOC), painel de constantes (~150 LOC), temas de cor (~150 LOC), hotkeys (~100 LOC)
Total: ~600 LOC, risco 1.5. Perde wow factor visual, ganha polimento.

## Grafo de Dependencias

```
#14 README ─── independente
#6  Timeline ── independente
       |
       v
#7  Transicao ── dep #6
       |
       v
#8  Auto-Tour ── dep #6 + #7
#9  WASM ─────── independente
#10 Narrativa ── independente
#11 Comparacao ─ independente (refatora coordenadas)
#12 QED ──────── independente (dividida em 2 meses)
#13 3D ──────── independente (crate externo, risco 8)
```
