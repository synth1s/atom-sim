# Decisoes e Convencoes — atom-sim

> Atualizado: 2026-03-29
> Este arquivo deve ser lido por TODOS os agentes antes de iniciar qualquer tarefa.
> Decisoes aqui registradas tem forca de lei ate serem explicitamente revogadas pelo proprietario.

## Decisoes do Proprietario

### D-001: Rigor academico obrigatorio
**Data:** 2026-03-28
**Contexto:** Proprietario rejeitou o primeiro planejamento por ser raso e sem fundamento.
**Decisao:** Todo conteudo cientifico deve ser pautado em justificativas academicas — citar experimentos, equacoes, evidencias historicas, contradicoes teoricas concretas. Nunca ser superficial.
**Aplica-se a:** PH-001, qualquer agente que produza texto cientifico.

### D-002: Aprovacao explicita para todas as acoes
**Data:** 2026-03-28
**Contexto:** Proprietario definiu que a ultima palavra e sempre dele.
**Decisao:** Nenhuma acao de escopo (contratar, dispensar, mudar prioridade, commitar) deve ser executada sem aprovacao explicita. Autonomia sera concedida gradualmente e explicitamente.
**Aplica-se a:** Todos os agentes.

### D-003: Delegacao antes de execucao direta
**Data:** 2026-03-29
**Contexto:** Proprietario solicitou que tarefas sejam delegadas a profissionais do time em vez de executadas diretamente.
**Decisao:** Quando existe um profissional competente no time para uma tarefa, delegar a ele. O coordenador (eu/Claude) atua como gestor, nao como executor.
**Aplica-se a:** Fluxo de trabalho geral.

### D-004: Decisoes baseadas em dados
**Data:** 2026-03-29
**Contexto:** Proprietario solicitou dados quantitativos antes de decidir prioridades.
**Decisao:** Toda priorizacao deve vir acompanhada de metricas objetivas (LOC, risco, score ROI, dependencias). Opiniao sem dados nao e suficiente.
**Aplica-se a:** CP-001, AD-001, TL-001, GP-001.

### D-005: Sincronizacao de memoria obrigatoria
**Data:** 2026-03-29
**Contexto:** Existem dois silos de memoria isolados — a memoria do Claude (~/.claude/memory/) e os context files dos agentes (team/context/). Subagentes nao acessam a memoria do Claude. O Claude nao le team/context/ automaticamente.
**Decisao:** Apos cada entrega significativa (sprint, correcao, nova feature, nova decisao), o coordenador (Claude principal) DEVE executar o protocolo de sincronizacao definido abaixo.
**Aplica-se a:** Coordenador (Claude principal) em toda conversa sobre atom-sim.

### D-006: Ponto de encontro no inicio de cada sessao
**Data:** 2026-03-29
**Contexto:** Nao havia ritual de alinhamento — trabalho acontecia ad hoc sem visibilidade do que esta planejado.
**Decisao:** No inicio de cada sessao de trabalho, o coordenador deve apresentar um briefing com: (1) o que foi feito na ultima sessao, (2) o que esta planejado para esta sessao, (3) decisoes pendentes do proprietario, (4) bloqueios. O proprietario aprova ou redireciona antes de iniciar execucao.
**Aplica-se a:** Coordenador (Claude principal), inicio de toda conversa.

### D-007: Cadencia de ideacao — backlog nunca esvazia
**Data:** 2026-03-29
**Contexto:** Nao havia cadencia para propor novas ideias. Quando a fila esvaziar, o time para.
**Decisao:** A cada sprint concluido, o CP-001 deve:
1. Avaliar o que foi entregue e o que resta na fila
2. Se a fila estiver abaixo de 5 issues abertas, propor 3-5 novas ideias
3. AD-001 quantifica as novas ideias (Score ROI, LOC, risco)
4. Novas issues criadas no GitHub Project
5. Proprietario prioriza antes do proximo sprint
**Aplica-se a:** CP-001, AD-001, GP-001 (dispara o ciclo).

## Protocolo de Sincronizacao de Memoria

O coordenador e a UNICA ponte entre os dois silos. Apos cada entrega significativa:

### Passo 1: Atualizar memoria dos agentes (team/context/)
- `PROJECT.md`: atualizar LOC, arquivos, testes, features, problemas conhecidos
- `DECISIONS.md`: registrar novas decisoes do proprietario ou convencoes tecnicas
- `CHANGELOG.md`: registrar o que mudou e por que

### Passo 2: Atualizar memoria do Claude (~/.claude/memory/)
- Se houve nova decisao do proprietario → atualizar feedback memories
- Se o estado do projeto mudou significativamente → atualizar project memory
- Se novo agente foi criado ou dispensado → atualizar reference memory

### Passo 3: Verificar consistencia
- A informacao em `~/.claude/memory/project_atom_sim.md` deve ser coerente com `team/context/PROJECT.md`
- As decisoes em `~/.claude/memory/feedback_*.md` devem estar refletidas em `team/context/DECISIONS.md`

### Quando sincronizar
- Apos cada commit
- Apos cada decisao do proprietario
- No inicio de cada nova conversa (ler ambos os silos e reconciliar se houver divergencia)
- Quando o proprietario solicitar

## Convencoes Tecnicas

### C-001: Texto sem acentos no codigo
Todos os textos exibidos na simulacao usam ASCII (sem acentos, cedilha). Motivo: compatibilidade com fontes padrao do Bevy.

### C-002: Handles pre-alocados
Toda era que emite particulas deve pre-alocar mesh/material handles em um Resource `XxxParticleAssets` no setup. Spawns usam `.clone()` do handle. Motivo: evitar leak de handles por frame (identificado por SE-001).

### C-003: Cleanup completo no OnExit
Toda era deve despawnar TODAS as suas entidades (via marker component XxxEntity) e remover TODOS os seus resources no cleanup. Motivo: evitar artefatos entre eras.

### C-004: Cada era e um Plugin isolado
Eras nao importam outras eras. Dependencias vao para `common/` ou `physics/`. Motivo: isolamento e extensibilidade.

### C-005: UI com Text2d + posicoes absolutas
O projeto usa Text2d com Transform para todo o HUD (nao usa bevy_ui). Posicoes sao absolutas para viewport 1280x720. Limitacao conhecida: nao responsivo.

### C-006: Constantes fisicas CODATA/NIST
Constantes fundamentais devem usar valores CODATA 2018 ou mais recentes. Desvio aceitavel: < 0.1% para constantes em modulos de demonstracao (coulomb.rs), < 0.001% para constantes em modulos de calculo (spectral.rs, quantum.rs).

### C-007: Testes para funcoes de fisica
Toda funcao publica em `physics/` deve ter pelo menos 1 teste unitario validando contra valor de referencia. Tolerancias devem ser explicitas nos asserts.

### C-008: Cobertura de testes como gate de aceite
**Data:** 2026-03-29
**Contexto:** Projeto cresceu a 9500 LOC com apenas 16 testes (todos em physics/). Nenhum teste para eras, UI ou integracao. Identificado quando #11 exigiu refatoracao sem rede de seguranca.
**Regra:** Toda entrega de feature ou refatoracao DEVE incluir testes que cubram o codigo alterado:
(a) Toda funcao publica nova deve ter ao menos 1 teste
(b) Todo setup/cleanup de era deve ter teste de spawn/despawn
(c) Toda feature de UI deve ter ao menos 1 smoke test
Entregas sem testes serao rejeitadas independente da qualidade do codigo.
O criterio #6 de SE-001 e reclassificado de peso 3 para peso 5.

---

## Convencoes Globais (herdadas de agency/DECISIONS.md)

> Definidas no repositorio `agency/` e aplicaveis a todos os projetos.
> Para detalhes completos, consultar `agency/DECISIONS.md`.
> Atualizado: 2026-04-03.
> Nota: D-006 local (Ponto de Encontro) e distinto de D-006 global (Process Freeze).

### D-006 Global: Congelamento de Novos Processos (Process Freeze)
Nenhum novo PROC ate primeiro cliente pagante. Processos existentes continuam vigentes.

### C-022: Research Gate (PILOTO)
Tarefas Medium/High uncertainty requerem Research Brief antes de decomposicao.

### C-023: Rastreabilidade (PILOTO)
Cadeia: Decisao → Story (TG-NNN) → Commit (`[TG-NNN]`) → Teste → Validacao.

### C-024: Security Baseline (PILOTO)
Checklist OWASP Top 10 para entregas com endpoints ou input de usuario.

### C-025: Workflow Tiers (PILOTO)
Tier 1 (trivial, 3 passos) / Tier 2 (padrao, 7 passos) / Tier 3 (complexo, 12+ passos).

### C-026: Trust Tiers (PILOTO)
Standard (<5 entregas) / Trusted (>=5, nota >=9.0) / Autonomous (>=10, nota >=9.5).

### C-027: Telemetria por Self-Reporting (PILOTO)
Todo output de agente inclui bloco `_telemetry` com: agent_id, tokens estimados, confidence, etc.

### C-028: Learning Spike (PILOTO)
Nova ferramenta/processo → explorar em sandbox → codificar em spec update → validar A/B.

---

## Historico de Mudancas

| Data | Decisao | Motivo |
|------|---------|--------|
| 2026-03-28 | D-001 Rigor academico | Proprietario rejeitou plano raso |
| 2026-03-28 | D-002 Aprovacao explicita | Definicao de governanca |
| 2026-03-29 | D-003 Delegacao | Proprietario quer time atuando |
| 2026-03-29 | D-004 Dados | Proprietario quer metricas |
| 2026-03-29 | C-001 a C-007 | Extraidos das avaliacoes SE-001/PH-001/UX-001 |
| 2026-03-29 | D-005 Sincronizacao | Dois silos de memoria isolados precisam de ponte manual |
| 2026-03-29 | D-006 Ponto de encontro | Briefing no inicio de cada sessao antes de executar |
| 2026-03-29 | D-007 Cadencia de ideacao | CP-001 propoe ideias a cada sprint se fila < 5 issues |
| 2026-03-29 | D-008 Autonomia temporaria | Coordenador decide por proprietario ate retorno (sessao noturna) |
| 2026-03-29 | C-008 Testes como gate | Toda entrega DEVE incluir testes. Sem testes = rejeicao automatica |
| 2026-03-30 | D-009 Integracao Taiga | Projeto atom-sim criado no Taiga (id=4). Backlog e sprints gerenciados la. |
