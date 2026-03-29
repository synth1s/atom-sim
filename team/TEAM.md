# Catalogo de Agentes — atom-sim

## Visao Geral

Cada agente e uma instancia de IA (Claude) configurada por um **spec file** que define seu papel, competencias, padrao de output e anti-patterns. A "expertise" do agente esta no spec, nao em memoria persistente.

**Revisor final:** Proprietario do projeto (ultima palavra em todas as decisoes).

## Estrutura de Memoria

**Specs (compartilhados):** https://github.com/synth1s/agency
**Contexto (local):** atom-sim/team/context/

```
synth1s/agency/          ← repositorio centralizado (specs compartilhados)
├── specs/               ← CANONICAL: specs dos agentes
│   ├── SE-001.spec.md
│   └── ...
├── TEAM.md
└── projects/atom-sim/   ← registros de entregas deste projeto

atom-sim/team/           ← contexto LOCAL deste projeto
├── TEAM.md              ← este arquivo
├── specs/               ← DEPRECATED (ver specs/DEPRECATED.md)
│   ├── SE-001.spec.md   ← copia historica, nao editar
│   ├── PH-001.spec.md
│   ├── UX-001.spec.md
│   ├── QM-001.spec.md
│   ├── CP-001.spec.md
│   ├── AD-001.spec.md
│   ├── TL-001.spec.md
│   └── GP-001.spec.md
├── context/             ← MEMORIA do projeto (injetada em todo agente)
│   ├── PROJECT.md       ← estado atual do codigo, arquitetura, metricas
│   ├── DECISIONS.md     ← decisoes do proprietario + convencoes tecnicas
│   └── CHANGELOG.md     ← historico do que mudou e por que
├── QM-001-criterios.md  ← criterios de avaliacao vigentes
├── QM-001.md            ← registro individual
├── PH-001.md
├── SE-001.md
├── UX-001.md
├── CP-001.md
├── AD-001.md
├── TL-001.md
└── GP-001.md
```

### Como funciona

1. **Spec file** = o "DNA" do agente. Define comportamento, padrao de output, anti-patterns
2. **Context files** = memoria do projeto. Todo agente le PROJECT.md e DECISIONS.md antes de atuar
3. **Registro individual** = historico de entregas e notas (curriculo real, nao descricao de cargo)
4. **CHANGELOG.md** = evita retrabalho (agente sabe o que ja foi feito)

### Para invocar um agente

O coordenador (Claude principal) deve:
1. Ler o spec file do agente
2. Instruir o agente a ler context/PROJECT.md e context/DECISIONS.md
3. Passar a tarefa especifica com criterios de aceite
4. Apos entrega: atualizar registro individual + CHANGELOG.md

---

## Quadro de Agentes

| ID | Cargo | Spec | Entregas | Nota | Status |
|----|-------|------|----------|------|--------|
| QM-001 | Gestor de Qualidade | [spec](specs/QM-001.spec.md) | 1 | 9/10 | Ativo |
| PH-001 | Especialista em Fisica | [spec](specs/PH-001.spec.md) | 3 | 9/10 | Ativo |
| SE-001 | Engenheiro Rust | [spec](specs/SE-001.spec.md) | 5 | 9.5/10 | Ativo |
| UX-001 | Especialista UX | [spec](specs/UX-001.spec.md) | 2 | 8.5/10 | Ativo |
| CP-001 | Consultor Produto | [spec](specs/CP-001.spec.md) | 1 | 9.5/10 | Ativo |
| AD-001 | Analista Dados | [spec](specs/AD-001.spec.md) | 1 | 9.5/10 | Ativo |
| TL-001 | Tech Lead | [spec](specs/TL-001.spec.md) | 1 | 9/10 | Ativo |
| GP-001 | Gerente Projetos | [spec](specs/GP-001.spec.md) | 2 | Pendente | Ativo |

---

## Regras

- **Specs sao versionados.** Mudancas no spec devem ser justificadas (melhoria de output observada ou anti-pattern novo).
- **Context files sao atualizados apos cada entrega.** GP-001 ou coordenador e responsavel.
- **Toda avaliacao deve citar evidencias** (arquivo:linha, valor numerico, referencia).
- **Encaminhamentos cross-domain sao obrigatorios** quando a questao esta fora do escopo do agente.
- **Promocoes e dispensas requerem aprovacao do proprietario.**
