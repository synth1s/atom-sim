# CLAUDE.md — atom-sim

## Projeto

Simulacao atomica 2D interativa em Rust/Bevy 0.18 que percorre 9 eras do modelo atomico (Democrito ~400 a.C. a Dirac 1928). Cada commit = marco cientifico. Projeto educacional com rigor academico.

- **Repo:** https://github.com/synth1s/atom-sim
- **Stack:** Rust 1.94, Bevy 0.18.1, rand 0.9
- **LOC:** ~5700, 21 arquivos, 13 testes
- **Project board:** https://github.com/orgs/synth1s/projects/3

## Proprietario

- Exige rigor academico — nunca ser superficial, fundamentar com equacoes e evidencias
- Exige dados para decisoes — LOC, risco, score ROI, nao opiniao
- Prefere delegacao a agentes em vez de execucao direta
- Ultima palavra em tudo — nao commitar, contratar ou mudar escopo sem aprovacao

## Framework de Agentes

Specs compartilhados em https://github.com/synth1s/agency (clone local: F:/Users/goula/projects/lab/agency/).
Contexto do projeto em atom-sim/team/context/ (PROJECT.md, DECISIONS.md, CHANGELOG.md).

### Para invocar um agente:
1. Ler spec de agency/specs/[ID].spec.md
2. Instruir o agente a ler atom-sim/team/context/PROJECT.md e DECISIONS.md
3. Passar tarefa com criterios de aceite
4. Apos entrega: atualizar team/context/CHANGELOG.md e registro em agency/projects/atom-sim/

### Time ativo:
- SE-001 (Eng. Rust, 9.5/10) — implementacao e revisao de codigo
- PH-001 (Fisica, 9/10) — validacao cientifica e textos
- UX-001 (UX, 8.5/10) — validacao visual e interacao
- QM-001 (Qualidade, 9/10) — criterios e consolidacao
- CP-001 (Produto, 9.5/10) — ideacao e oportunidades
- AD-001 (Dados, 9.5/10) — quantificacao e ROI
- TL-001 (Tech Lead, 9/10) — decomposicao tecnica
- GP-001 (Gerente, pendente) — coordenacao de sprints

## Convencoes Tecnicas

- C-001: Texto sem acentos no codigo (ASCII, compatibilidade Bevy)
- C-002: Handles pre-alocados em XxxParticleAssets Resource (evitar leak)
- C-003: Cleanup completo no OnExit (despawnar XxxEntity + remover resources)
- C-004: Cada era e Plugin isolado (nao importa outras eras)
- C-005: UI com Text2d + posicoes absolutas (viewport 1280x720)
- C-006: Constantes CODATA/NIST (desvio < 0.001% em spectral.rs/quantum.rs)
- C-007: Testes para funcoes publicas de fisica (tolerancias explicitas)

## Padrao de Era (template)

1. XxxPlugin com OnEnter/OnExit/Update
2. XxxEntity marker para cleanup
3. XxxParticleAssets Resource com handles pre-alocados
4. setup_xxx: spawna entidades, insere EraControls + LimitationText
5. cleanup_xxx: despawna XxxEntity, remove todos os resources
6. Systems com .run_if(in_state(ActiveEra::Xxx)).run_if(in_state(SimulationState::Running))

## API Bevy 0.18 (gotchas)

- MessageReader em vez de EventReader
- .single() retorna Result (usar let Ok(x) = query.single() else { return })
- OrthographicProjection nao e Component — usar Projection enum
- WindowResolution aceita (u32, u32), nao (f32, f32)
- Text2d em vez de TextBundle (removido em 0.18)
- Justify em vez de JustifyText
