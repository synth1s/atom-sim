# Organograma — atom-sim

> Atualizado: 2026-03-29

## Estrutura Hierarquica

```
                    ┌─────────────────────┐
                    │    PROPRIETARIO      │
                    │    (Revisor Final)   │
                    │                     │
                    │  Aprova/rejeita     │
                    │  Ultima palavra     │
                    │  Concede autonomia  │
                    └─────────┬───────────┘
                              │
                              │ reporta
                              │
                    ┌─────────┴───────────┐
                    │    COORDENADOR       │
                    │    (Claude Principal)│
                    │                     │
                    │  Ponte entre silos  │
                    │  Invoca agentes     │
                    │  Sincroniza memoria │
                    └─────────┬───────────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
     ┌────────┴──────┐ ┌─────┴──────┐ ┌──────┴───────┐
     │   GESTAO      │ │ ESTRATEGIA │ │  QUALIDADE   │
     │               │ │            │ │              │
     │  GP-001       │ │  TL-001    │ │  QM-001      │
     │  Ger. Projeto │ │  Tech Lead │ │  Ger. Qual.  │
     │  9/10(pend.)  │ │  9/10      │ │  9/10        │
     └───────┬───────┘ └─────┬──────┘ └──────┬───────┘
             │               │               │
             │ coordena      │ planeja        │ avalia
             │               │               │
     ┌───────┴───────────────┴───────────────┴───────┐
     │                 EXECUCAO                       │
     │                                                │
     │  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
     │  │ SE-001   │  │ PH-001   │  │ UX-001   │    │
     │  │ Eng.Rust │  │ Fisica   │  │ UX/Sim   │    │
     │  │ 9.5/10   │  │ 9/10     │  │ 8.5/10   │    │
     │  └──────────┘  └──────────┘  └──────────┘    │
     │                                                │
     └────────────────────────────────────────────────┘
             │
             │ informa
             │
     ┌───────┴────────────────────────────────────────┐
     │                CONSULTORIA                      │
     │                                                 │
     │  ┌──────────┐  ┌──────────┐                    │
     │  │ CP-001   │  │ AD-001   │                    │
     │  │ Produto  │  │ Dados    │                    │
     │  │ 9.5/10   │  │ 9.5/10   │                    │
     │  └──────────┘  └──────────┘                    │
     │                                                 │
     └─────────────────────────────────────────────────┘
```

## Papeis e Responsabilidades

### Proprietario (Voce)
- **Autoridade:** Maxima. Toda decisao de escopo, priorizacao, contratacao e dispensa passa por ele.
- **Atua como:** Revisor final de entregas, aprovador de commits, definidor de direcao.
- **Nao faz:** Implementacao direta. Delega ao time.

### Coordenador (Claude Principal)
- **Autoridade:** Operacional. Invoca agentes, sincroniza memoria, reporta ao proprietario.
- **Atua como:** Ponte entre o proprietario e os agentes. Unico que acessa ambos os silos de memoria (~/.claude/memory/ e team/context/).
- **Nao faz:** Decisoes de escopo sem aprovacao. Nao implementa diretamente quando ha agente disponivel (D-003).
- **Responsabilidade unica:** Protocolo de sincronizacao (D-005) — manter ambos os silos coerentes.

### Camada de Gestao

| Agente | Papel | Quando atua |
|--------|-------|-------------|
| **GP-001** (Ger. Projetos) | Dispara tarefas, cobra entregas, reporta progresso, integra com GitHub | Durante sprints — coordena execucao |
| **TL-001** (Tech Lead) | Decompoe roadmap em fichas tecnicas, define criterios de aceite, planeja paralelismo | Antes de sprints — planeja execucao |
| **QM-001** (Ger. Qualidade) | Define criterios de avaliacao, revisa relatorios, consolida pareceres, arbitra conflitos | Apos entregas — avalia qualidade |

### Camada de Execucao

| Agente | Papel | Quando atua | Validado por |
|--------|-------|-------------|--------------|
| **SE-001** (Eng. Rust) | Implementa codigo, corrige bugs, refatora, cria testes | Implementacao de features e correcoes | PH-001 (fisica), UX-001 (visual) |
| **PH-001** (Fisica) | Valida equacoes, produz textos cientificos, verifica constantes | Validacao pos-implementacao, producao de conteudo | QM-001 (completude) |
| **UX-001** (UX/Sim) | Valida visual, analisa layout, testa fluxo, avalia acessibilidade | Validacao pos-implementacao, design de interacao | QM-001 (completude) |

### Camada de Consultoria

| Agente | Papel | Quando atua |
|--------|-------|-------------|
| **CP-001** (Produto) | Propoe ideias, analisa oportunidades, define roadmap qualitativo | Quando o proprietario pede direcao |
| **AD-001** (Dados) | Quantifica propostas, calcula ROI, estima esforco, prioriza por dados | Quando o proprietario quer dados para decidir |

## Fluxo de Trabalho Padrao

```
1. Proprietario define direcao
         │
         v
2. CP-001 propoe ideias (se necessario)
         │
         v
3. AD-001 quantifica e prioriza (se necessario)
         │
         v
4. Proprietario aprova prioridade
         │
         v
5. TL-001 decompoe em tarefas tecnicas
         │
         v
6. Proprietario aprova plano
         │
         v
7. GP-001 dispara execucao
         │
    ┌────┴────┐
    v         v
8. SE-001   PH-001 (textos, se necessario)
   implementa
    │
    ├──> PH-001 valida fisica
    ├──> UX-001 valida visual
    │
    v
9. QM-001 consolida (se necessario)
         │
         v
10. Coordenador reporta ao Proprietario
         │
         v
11. Proprietario aprova → commit + push
         │
         v
12. Coordenador sincroniza memoria (D-005)
```

## Regras de Interacao

### Encaminhamento Cross-Domain
- SE-001 encontra equacao duvidosa → encaminha para PH-001
- SE-001 identifica impacto visual → encaminha para UX-001
- PH-001 encontra bug de codigo → encaminha para SE-001
- PH-001 identifica resultado ilegivel → encaminha para UX-001
- UX-001 ve simulacao estranha → encaminha para PH-001
- UX-001 identifica problema de performance → encaminha para SE-001

### Arbitragem de Conflitos
Quando dois agentes discordam, QM-001 arbitra com prioridade:
1. **Correcao fisica** (PH-001 tem precedencia)
2. **Clareza visual** (UX-001 tem precedencia)
3. **Elegancia de codigo** (SE-001 tem precedencia)

### Isolamento
- Agentes NAO se comunicam diretamente entre si
- Toda comunicacao passa pelo Coordenador (Claude Principal)
- Agentes NAO acessam memoria do Claude (~/.claude/)
- Agentes NAO leem specs de outros agentes (a menos que instruidos)

## Natureza dos Agentes

Cada agente e uma instancia do mesmo modelo de IA (Claude), diferenciada por:
- **Spec file** (`team/specs/[ID].spec.md`): define comportamento, formato, anti-patterns
- **Context files** (`team/context/`): injetados no prompt de invocacao
- **Nao ha memoria persistente** entre invocacoes — toda "experiencia" esta nos spec files

A qualidade de cada agente depende da qualidade do seu spec. Specs evoluem com base no desempenho observado:
- Entrega boa → extrair trecho como exemplo no spec
- Entrega ruim → adicionar como anti-pattern no spec

## Metricas do Time

| Metrica | Valor |
|---------|-------|
| Agentes ativos | 8 |
| Nota media | 9.2/10 (excluindo GP-001 pendente) |
| Entregas totais | 16 |
| Specs documentados | 8/8 |
| Taxa de aprovacao na 1a entrega | 7/8 (SE-001 rejeitou projeto na 1a avaliacao → corrigido e reaprovado) |
