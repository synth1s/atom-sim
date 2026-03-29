# QM-001 — Critérios de Avaliação para Especialistas

**Projeto:** atom-sim | **Data:** 2026-03-28 | **Emissor:** QM-001 (Gestor de Qualidade)

---

## 1. PH-001 — Especialista em Física

### 1.1 Critérios de Avaliação

| # | Critério | Descrição | Peso |
|---|----------|-----------|------|
| 1 | **Correção das equações** | Verificar se CADA equação implementada corresponde à forma teórica aceita. Inclui: colisão elástica, oscilação Thomson, Rydberg/Bohr, Laguerre/Legendre, energia de Dirac. | 5 |
| 2 | **Precisão das constantes físicas** | Validar contra CODATA/NIST: a₀, R∞, E_Ry, h, c, mₑ, α. Reportar desvio percentual de cada uma. | 5 |
| 3 | **Fidelidade histórica** | Para cada era: texto HUD correto (datas, citações, atribuições), limitações historicamente precisas. | 4 |
| 4 | **Coerência da progressão** | Cada modelo resolve limitações do anterior e introduz novas. Limitações declaradas são superadas pela era seguinte. | 3 |
| 5 | **Rigor da simulação visual** | Comportamento visual corresponde à física: gás ideal em Demócrito, ω∝1/n³ em Bohr, |ψ|² correto em Schrödinger, E>2mₑc² em Dirac. | 4 |
| 6 | **Correção do espectro** | Séries espectrais, wavelength_to_color(), estrutura fina de Dirac vs dados experimentais. Mínimo 5 transições testadas. | 4 |
| 7 | **Tratamento dimensional** | Consistência de unidades (pixels vs SI), conversões corretas (SPATIAL_SCALE). | 3 |
| 8 | **Lacunas físicas relevantes** | Fenômenos ausentes que deveriam estar presentes. Distinguir omissões aceitáveis vs comprometedoras. | 2 |

### 1.2 Escala: 0-10 por critério
- 10: Completamente correto | 8-9: Correto com obs. menores | 6-7: Substancialmente correto | 4-5: Parcialmente correto | 2-3: Problemas sérios | 0-1: Fundamentalmente errado

### 1.3 Nota mínima: **7.0/10** ponderada. Peso 5 não pode ter nota <6.

### 1.4 Itens obrigatórios do relatório:
1. Tabela de validação de constantes (código vs CODATA/NIST vs desvio %)
2. Análise equação-por-equação de cada módulo `physics/`
3. Ficha histórica por era (9 fichas)
4. Lista de transições espectrais testadas (mín. 5, incluindo Lyman/Balmer/Paschen)
5. Parecer final com nota por critério e veredicto

### 1.5 Encaminhamentos:
- Bug de implementação → **SE-001**
- Resultado correto mas ilegível → **UX-001**
- Simplificação por limitação técnica → **SE-001**

---

## 2. SE-001 — Engenheiro de Software Rust

### 2.1 Critérios de Avaliação

| # | Critério | Descrição | Peso |
|---|----------|-----------|------|
| 1 | **Padrão Bevy ECS** | States, Plugins, Components vs Resources, system ordering, queries, lifecycle cleanup. | 5 |
| 2 | **Qualidade Rust** | Ownership/borrowing, Option/Result, pattern matching, nomenclatura, idiomaticidade. | 5 |
| 3 | **Arquitetura e modularidade** | Separação de módulos, acoplamento, DRY, extensibilidade, encapsulamento. | 4 |
| 4 | **Performance** | Complexidade algorítmica, alocações por frame, reutilização de handles, contagem de entidades. | 4 |
| 5 | **Tratamento de erros** | Option<Res<...>> com early return, single() com Ok/Err, guard clauses, prevenção de divisão por zero. | 3 |
| 6 | **Cobertura de testes** | Existência de testes ou lista prioritária de testes necessários (mín. 10 sugeridos). | 3 |
| 7 | **Documentação** | Doc comments, comentários de física, clareza de nomes, README. | 2 |
| 8 | **Build e toolchain** | Cargo.toml, profiles, cargo clippy limpo, compilação sem warnings. | 2 |

### 2.2 Escala: 0-10 por critério (mesma escala do PH-001)
### 2.3 Nota mínima: **7.0/10** ponderada. Critérios 1 e 2 não podem ter nota <6.

### 2.4 Itens obrigatórios:
1. Resultado de `cargo build --release` e `cargo clippy`
2. Mapa de dependências entre módulos
3. Revisão arquivo-por-arquivo (problemas, severidade, sugestão)
4. Análise de performance (top 3 gargalos)
5. Lista de testes recomendados (mín. 10)
6. Parecer final

### 2.5 Encaminhamentos:
- Equação Rust correta mas resultado fisicamente estranho → **PH-001**
- Performance afetando experiência visual → **UX-001**
- Decisão arquitetural motivada por UX → **UX-001**

---

## 3. UX-001 — Especialista em UX/Simulação

### 3.1 Critérios de Avaliação

| # | Critério | Descrição | Peso |
|---|----------|-----------|------|
| 1 | **Clareza visual** | Cada era compreensível: partículas distinguíveis, núcleo visível, órbitas claras, nuvens reconhecíveis, cores com contraste. | 5 |
| 2 | **Legibilidade de texto** | Tamanho, contraste, posicionamento, sobreposição, comprimento de linhas. | 5 |
| 3 | **Eficácia dos controles** | Mapeamento intuitivo, feedback imediato, consistência entre eras, descobribilidade. | 4 |
| 4 | **Transição entre eras** | Cleanup correto, sem artefatos, instantâneo, contexto muda imediatamente. | 4 |
| 5 | **Feedback ao usuário** | Indicadores visuais de estado, contadores, respostas a ações. | 4 |
| 6 | **Narrativa educacional** | Percorrer 1-9 em ordem transmite evolução histórica? Conexão limitação→solução? | 3 |
| 7 | **Acessibilidade** | Dependência de cor, tamanho de elementos, controles. Avaliação proporcional a protótipo. | 2 |
| 8 | **Layout e hierarquia** | Posicionamento, separação informação/interação, uso eficiente do espaço. | 3 |

### 3.2 Escala: 0-10 por critério
### 3.3 Nota mínima: **6.5/10** ponderada. Critérios 1 e 2 não podem ter nota <5.

### 3.4 Itens obrigatórios:
1. Descrição visual de cada era (9 eras)
2. Mapa de controles completo
3. Teste de fluxo completo (9 eras em sequência)
4. Lista de problemas de usabilidade por severidade
5. Avaliação de hierarquia de informação
6. Parecer final

### 3.5 Encaminhamentos:
- Simulação visualmente errada → **PH-001**
- Problema visual por performance → **SE-001**
- Texto científico possivelmente incorreto → **PH-001**

---

## Regras Transversais

1. Relatórios estruturados com seções numeradas, evidências com referência a arquivo:linha
2. Rejeição dupla → substituição do especialista
3. Prioridade em conflitos: correção física > clareza visual > elegância de código
4. Cada especialista avalia APENAS seu domínio
5. Afirmações sem evidência serão desconsideradas
