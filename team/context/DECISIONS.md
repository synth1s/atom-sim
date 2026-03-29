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

## Historico de Mudancas

| Data | Decisao | Motivo |
|------|---------|--------|
| 2026-03-28 | D-001 Rigor academico | Proprietario rejeitou plano raso |
| 2026-03-28 | D-002 Aprovacao explicita | Definicao de governanca |
| 2026-03-29 | D-003 Delegacao | Proprietario quer time atuando |
| 2026-03-29 | D-004 Dados | Proprietario quer metricas |
| 2026-03-29 | C-001 a C-007 | Extraidos das avaliacoes SE-001/PH-001/UX-001 |
