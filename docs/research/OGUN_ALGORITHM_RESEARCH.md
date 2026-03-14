# Ogun — Theoretical Foundations & Research Context

> Status: Active research
> Last updated: 2026-03-14

---

## Problem Statement

We want a core algorithm that produces 2D spatial layouts (placed nodes, routed edges) at a controllable optimization level. The output should not be "globally optimized + noise" — it should have internal consistency at the target optimization level, the way real cities have coherent local structure despite global inefficiency.

The challenge: find a mechanism that goes beyond simulating agent interactions without falling into inherently chaotic dynamics (three-body-problem territory).

---

## Formal Decomposition

### Input Space
- Graph G = (V, E, w) — nodes with footprints, weighted edges representing connection demand
- 2D domain S — boundary, obstacle map, keep-out zones
- Target optimization parameter (formulation depends on framing)
- Seed for deterministic RNG

### Output Space
- Placement P: V → S (node positions, non-overlapping, within boundary)
- Routing R: E → Path(S) (edge paths through the space)
- Optimization score s ∈ [0, 1]

### Atomic Operations

| Operation | What it does | Maps to (solved problem) |
|-----------|-------------|--------------------------|
| PLACE | Assign a node a position given constraints | VLSI Placement / Facility Layout Problem (NP-hard, force-directed heuristics well-studied) |
| ROUTE | Find a path between two placed nodes given existing paths | Multi-net Steiner routing (NP-hard, Lee/PathFinder well-studied) |
| SCORE | Evaluate layout quality | Multi-objective evaluation (standard) |
| DEGRADE | Perturb layout to reduce score | Adding noise to an optimized solution (trivial) |

### Why "optimize then degrade" is insufficient

Targeting a specific optimization level by degrading an optimal solution is mathematically trivial — it reduces to `|score - target|` as the loss function. This is a paragraph in a paper, not a paper. The resulting layouts lack internal consistency: a degraded optimal layout looks like a good plan that was vandalized, not like something that was built organically.

---

## Framing 1: Spatial Potential Game

### Concept

The layout is a game. Each builder-agent has a utility function (market wants road access, temple wants isolation, house wants proximity to services). Agents take turns placing themselves, each maximizing their own utility given the current state.

If formulated as a **potential game** — where a single global function Φ exists such that any agent improving their own utility also improves Φ — then:

- Best-response dynamics are **guaranteed to converge** to a Nash equilibrium (no chaos)
- The **Price of Anarchy** (PoA = ratio of equilibrium cost to global optimum) is exactly the "optimization gap" we want to produce
- We control the PoA by tuning agent heterogeneity, information structure, and move order — not by post-hoc degradation

### Why it fits

- Potential games have convergence guarantees by construction — avoids chaotic dynamics
- The PoA has been empirically measured on real city road networks, showing significant waste from uncoordinated routing
- Nobody has used PoA as a *generative* mechanism for procedural layout
- Different agent utility functions produce different equilibria with different PoA values
- The equilibrium IS the layout — no separate optimization + degradation steps

### Key properties

| Property | Implication |
|----------|-------------|
| Convergence guaranteed | No three-body problem; best-response dynamics always terminate |
| PoA is measurable | The optimization gap has a theoretical bound, not just an empirical score |
| Agent heterogeneity controls PoA | More diverse agents → higher PoA → more "organic" layout |
| Move order matters | Sequential play produces different equilibria than simultaneous — models historical layering |
| Multiple equilibria possible | Same agents can produce different layouts depending on initial conditions / ordering |

### Key research

- Monderer & Shapley (1996) — foundational potential games paper, proved existence of pure Nash equilibria via potential function
- Rosenthal (1973) — congestion games are potential games; agents sharing spatial resources is a congestion game
- Youn, Jeong & Gastner (2008) — "Price of Anarchy in Transportation Networks" — measured PoA on real road networks of Boston, New York, London; found significant inefficiency from uncoordinated routing
- Fotakis (2005) — "Online and incremental algorithms for facility location" — competitive ratio Θ(log n / log log n) for online variant, O(1) for incremental
- Feldman & Fiat (2020) — "The Online Multi-Commodity Facility Location Problem" (arXiv:2005.08391) — shows agent heterogeneity directly influences competitive ratio; lower bound Ω(√|S| + log n / log log n) for heterogeneous agents
- arXiv:2601.08642 — "Improving Equilibria in Urban Neighbourhood Games" — formalizes urban development as a game, shows targeted interventions can transform equilibrium quality
- arXiv:2011.06778 — "Potential Maximization and Stochastic Stability of Spatial Equilibria" — spatial distribution of agents in equilibrium depends on initial conditions; multiple locally stable equilibria arise

---

## Framing 2: Boltzmann Sampling at Target Temperature

### Concept

The layout is a physical system with an energy function E (lower energy = more optimized). Instead of minimizing energy (simulated annealing → cooling to ground state), we **sample from the Boltzmann distribution at a specific temperature T**.

At temperature T, configurations appear with probability ∝ exp(-E/kT).

This is fundamentally different from "optimal + noise":
- Boltzmann samples at a given temperature have **thermodynamic consistency** — internal correlations, local order, and structure that pure perturbation doesn't produce
- A layout sampled at T=0.3 looks different from one optimized to score 0.7 and then randomly degraded
- The former has coherent local neighborhoods with global inefficiency; the latter has a broken global plan

### Temperature as tuning parameter

| Temperature | Layout character | City analogue |
|-------------|-----------------|---------------|
| T → 0 | Ground state, globally optimal | Perfectly planned capital |
| Low T | Mostly ordered, thermal fluctuations | Established city with minor organic deviations |
| Medium T | Significant disorder, recognizable structure | Organic settlement, historical town |
| High T | Near-random | Ruins, abandoned settlement |

### Why it fits

- Principled mechanism for producing layouts at a specific energy level with internal consistency
- Well-studied mathematics (statistical mechanics, Markov chain Monte Carlo)
- The stationary distribution is known analytically — we can reason about what configurations are typical at each temperature
- Avoids chaos: the Boltzmann distribution at fixed T is a well-defined equilibrium — we're sampling from it, not predicting trajectories

### Key research

- Metropolis et al. (1953) — original Monte Carlo sampling algorithm
- Kirkpatrick, Gelatt & Vecchi (1983) — simulated annealing for optimization (we invert this: hold temperature instead of cooling)
- The VLSI simulated annealing literature (TimberWolf placer, etc.) — established energy functions for spatial layout
- arXiv:2409.18126 — "Boltzmann Sampling by Diabatic Quantum Annealing" — modern approaches to efficient sampling at target energy levels
- arXiv:2501.19077 — "Temperature-Annealed Boltzmann Generators" — efficient generation of samples at specific temperatures

---

## Framing 3: Incremental Constrained Morphogenesis

### Concept

The layout grows like an organism. Start with a seed (founding structures). At each time step, a new structure is added following local growth rules. Each addition constrains future growth. The final form emerges from the accumulation of local decisions over simulated time.

This produces temporal layering: early structures occupy prime locations, later structures fill gaps and work around existing infrastructure. The layout carries readable history — you can infer what was built first by its position.

### Why it fits

- Matches how real cities actually form — incremental growth, not global optimization
- Produces temporal structure that pure optimization lacks
- Local rules are simple; global complexity is emergent
- The growth rule set IS the tuning parameter — different rules produce different morphologies

### Where it's limited

- Agent-based growth models are typically ad-hoc — rules are hand-crafted, not derived from a formal framework
- Hard to provide theoretical guarantees about output quality
- Risk of chaotic sensitivity to initial conditions without careful design

### Key research

- Lechner, Watson & Wilensky (2003) — "Procedural Modeling of Land Use in Cities" — agent-based city generation with land-use zoning
- Plos ONE (2015) — "On the Morphology of a Growing City" — merges static equilibrium analysis with agent-based growth simulation; shows monocentric configurations emerge under strong agglomeration
- Springer (2019) — "Emergent Urban Morphologies in an Agent-Based Model of Housing" — city-like agglomerates emerge from agent-based housing models with income distribution and transportation
- ResearchGate (2024) — "The Miskolc Method: Modelling the Evolution of a Natural City with Recursive Algorithms Using Simulated Morphogenesis" — recursive algorithms for urban fabric generation
- arXiv:2309.10871 — "Believable Minecraft Settlements by Means of Decentralised Iterative Planning" — decentralized agents producing believable settlements in constrained environments

---

## Synthesis: Where the Core Mechanism Lives

The three framings are not competing — they address different aspects of the same problem:

| Framing | Provides | Doesn't provide |
|---------|----------|-----------------|
| Potential game | Mathematical foundation, convergence guarantees, PoA as optimization metric | Sampling mechanism, temporal structure |
| Boltzmann sampling | Principled way to produce internally consistent layouts at a target energy level | Agent semantics, growth narrative |
| Morphogenesis | Incremental growth, temporal layering, readable history | Formal guarantees, controllable optimization level |

### Proposed core mechanism

**A spatial potential game where the equilibrium is sampled at a target temperature via Boltzmann dynamics, with incremental agent arrival modeling temporal growth.**

Concretely:
1. The **potential function** Φ encodes agent utilities (attraction, repulsion, access costs) on the spatial domain
2. Agents arrive **incrementally** (morphogenesis ordering), each playing a best-response given current state
3. The layout converges to a **Nash equilibrium** of the potential game
4. **Boltzmann sampling at temperature T** perturbs the equilibrium — agents don't always play perfect best-response; they play ε-best-response where ε is controlled by T
5. The **Price of Anarchy** emerges from the combination of agent heterogeneity, arrival order, and temperature

### Why this avoids the three-body problem

- Potential games guarantee convergence of best-response dynamics — no chaotic orbits
- Boltzmann sampling at fixed T has a well-defined stationary distribution — we sample from equilibrium, not trajectories
- Incremental arrival is sequential, not simultaneous — each step is a single-agent optimization against a fixed environment
- The system is dissipative (potential function decreases or stays constant), not conservative

### What's novel

No existing work combines all three:
- Potential games have been applied to spatial resource allocation (sensor networks, vehicle routing) but not procedural layout generation
- Boltzmann sampling is standard in physics and VLSI but hasn't been used with a game-theoretic potential function for layout
- Agent-based city generation exists but lacks the formal foundation of potential games and the principled sampling of Boltzmann dynamics
- The Price of Anarchy has been measured on real cities but never used as a generative target

The contribution: a procedural layout algorithm grounded in potential game theory, where the optimization gap is a controllable, theoretically bounded emergent property rather than an imposed post-hoc degradation.

---

## Open Research Questions

1. **Can spatial layout be formulated as a potential game?** Need to define utility functions for heterogeneous agents such that a global potential Φ exists. Congestion games (Rosenthal 1973) are a natural starting point — agents sharing spatial cells incur congestion costs.

2. **What is the PoA of the resulting game?** Depends on the utility functions and network structure. Need to derive or empirically measure bounds for our specific formulation.

3. **Is Boltzmann sampling at target T computationally tractable?** Mixing time of the Markov chain determines how many steps are needed to reach the stationary distribution. For spatial layouts on grids, this needs investigation.

4. **How does incremental arrival order interact with the potential function?** Different orderings produce different equilibria. Can we characterize which orderings produce which PoA values?

5. **What energy function produces "believable" layouts?** The energy function must capture enough urban structure (clustering, road hierarchy, accessibility) without becoming intractable. Need to validate against real city morphologies or human judgment.

6. **Determinism**: Boltzmann sampling is inherently stochastic. For deterministic output (required for saltglass-steppe), we need seeded RNG for the MCMC chain. Same seed + same input = same layout. This is standard but worth noting.

---

## Critical Prior Art Discovery: Logit Dynamics

Before proceeding to operator decomposition, an honest reckoning.

The proposed synthesis — "Boltzmann sampling on a potential game" — is a well-studied mechanism called **logit dynamics** (Blume 1993, McKelvey & Palfrey 1995). In logit dynamics:

- At each step, a player is selected uniformly at random
- That player plays a **noisy best response**: P(strategy s) ∝ exp(β · uᵢ(s, s₋ᵢ))
- β is the inverse noise level (= inverse temperature)
- For potential games, the **stationary distribution** of this Markov chain is exactly the Boltzmann distribution weighted by the potential function: π(s) ∝ exp(β · Φ(s))

This is well-studied:
- Convergence: Auletta et al. (2012) — mixing time is exponential in β and max potential difference for potential games
- PoA under logit dynamics: Penna et al. (2015) — "The price of anarchy and stability in general noisy best-response dynamics"
- Stability: Auletta et al. (2013) — "Mixing Time and Stationary Expected Social Welfare of Logit Dynamics"

### What logit dynamics has been applied to
- Abstract game theory (convergence analysis, equilibrium selection)
- Evolutionary game theory (population dynamics)
- Network congestion / routing games
- Sensor network planning (potential games for spatial sensor placement — but for **optimization**, not generation)

### What logit dynamics has NOT been applied to
- Procedural spatial layout generation
- Online/incremental settings with irrevocable placement
- Integrated placement + routing
- Using PoA as a controllable generative target (rather than an analytical measure)

### How Ogun differs from standard logit dynamics

In standard logit dynamics, the system reaches a stationary distribution through **repeated play** — agents are selected again and again, changing strategies, until the Markov chain mixes. The output is a sample from π(s) ∝ exp(β · Φ(s)).

In Ogun, each agent plays **once** and is **irrevocably committed**. This means:
- Ogun does NOT converge to the Boltzmann distribution over configurations
- It produces a single configuration from sequential one-shot noisy best-responses
- The output distribution (across seeds) depends on **arrival order**, not just Φ and β
- The process is a **sequential game with commitment**, not a repeated game

This is a fundamentally different object. Ogun uses the logit choice rule as a **local decision mechanism** within a one-pass sequential process, not as a dynamics that converges to equilibrium.

### Honest novelty assessment

| Component | Known? | Novel application? |
|-----------|--------|-------------------|
| Logit choice rule (noisy best response) | Yes (Blume 1993) | — |
| Potential games for spatial allocation | Yes (sensor networks) | Yes — for layout **generation** at target quality |
| PoA as analytical measure | Yes (Youn et al. 2008) | Yes — as a **generative** parameter |
| Online facility location with irrevocable placement | Yes (Fotakis 2005) | Yes — combined with logit choice + potential game |
| Sequential one-shot logit in spatial domain with routing | **No known prior work** | **Novel combination** |

The contribution is not a new primitive — it's a new **composition** of known primitives applied to a new domain, producing a property (controllable optimization level with internal consistency) that no existing method achieves.

---

## Abstract Operator Decomposition

### Why this section exists

There is no standard formalism for structurally comparing spatial layout algorithms. The closest precedents are:
- **Algorithmic skeletons** (Cole 1989, Skillicorn & Talia 1998) — for parallel computation patterns; too low-level
- **Metaheuristic framework** (Sörensen 2015) — decomposes optimization heuristics into (initialize, generate, evaluate, accept, update); good but doesn't capture spatial-specific operations like routing or commitment models
- **Search behavior analysis** (arXiv:2507.01668) — compares algorithms empirically by solution distributions; doesn't expose structural differences

We define a custom operator vocabulary for spatial layout algorithms. The goal: make the structural differences between Ogun and existing algorithms visible at a glance.

### State

All algorithms operate on a configuration:

```
C = (P, R, A, Θ)
```

- **P**: placement map — node → position (or ∅ if unplaced)
- **R**: routing map — edge → path (or ∅ if unrouted)
- **A**: available space / constraint state (what positions remain valid)
- **Θ**: algorithm parameters (temperature, congestion costs, etc.)

### Operator Vocabulary

Ten operators. Every spatial layout algorithm we examine can be expressed as a composition of these.

| # | Operator | Signature | What it does |
|---|----------|-----------|-------------|
| 1 | **INIT** | (nodes, domain) → C₀ | Initialize state |
| 2 | **SELECT** | C → agent | Choose which agent to process next |
| 3 | **CANDIDATES** | (agent, C) → {positions} | Generate candidate positions |
| 4 | **EVAL** | (agent, position, C) → score | Score a candidate position |
| 5 | **CHOOSE** | (scores, Θ) → position | Pick a position from scored candidates |
| 6 | **COMMIT** | (agent, position, C) → C' | Place the agent, update state |
| 7 | **ROUTE** | (edge, C) → C' | Connect two placed nodes with a path |
| 8 | **PROPAGATE** | C → C' | Update constraints after a commit |
| 9 | **UPDATE** | (Θ, C, t) → Θ' | Modify algorithm parameters |
| 10 | **TERMINATE** | (C, t) → bool | Check stopping condition |

Each operator has **variants** — the specific implementation chosen. The combination of variants across all operators defines an algorithm's **signature**.

### Operator Variants

**INIT variants**:
- `INIT_EMPTY` — no nodes placed (WFC-style: all cells uncollapsed)
- `INIT_RANDOM` — all nodes placed at random positions
- `INIT_SEED` — subset of founding nodes placed

**SELECT variants**:
- `SELECT_ALL` — process all agents simultaneously
- `SELECT_RANDOM` — pick one uniformly at random
- `SELECT_ORDERED` — follow a predetermined arrival sequence
- `SELECT_MIN_ENTROPY` — pick the most constrained agent/cell
- `SELECT_PRIORITY` — pick by importance/weight

**CANDIDATES variants**:
- `CAND_ALL` — all valid positions in domain
- `CAND_NEIGHBOR` — positions near agent's current position
- `CAND_RANDOM` — random subset of valid positions
- `CAND_RULE` — positions satisfying hand-crafted spatial rules

**EVAL variants**:
- `EVAL_ENERGY` — global energy function E(C) (physics-based)
- `EVAL_UTILITY` — agent-specific utility uᵢ(p, C) from potential function Φ
- `EVAL_FORCE` — force vector from pairwise interactions
- `EVAL_CONSTRAINT` — count of satisfied/violated constraints
- `EVAL_HEURISTIC` — hand-crafted scoring rules

**CHOOSE variants**:
- `CHOOSE_ARGMAX` — pick highest-scoring position (greedy / best-response)
- `CHOOSE_BOLTZMANN(β)` — sample with P(p) ∝ exp(β · score(p))
- `CHOOSE_METROPOLIS(T)` — accept new position if ΔE < 0, else with prob exp(-ΔE/T)
- `CHOOSE_WEIGHTED` — sample proportional to weights (no temperature)
- `CHOOSE_RULE` — hand-crafted selection logic

**COMMIT variants**:
- `COMMIT_IRREVOCABLE` — placement is permanent
- `COMMIT_TENTATIVE` — agent can be moved in future iterations
- `COMMIT_BATCH` — multiple agents committed simultaneously

**ROUTE variants**:
- `ROUTE_SHORTEST` — shortest available path (A*, Lee)
- `ROUTE_NEGOTIATED` — path considering shared-resource congestion costs
- `ROUTE_RULE` — hand-crafted routing heuristics
- `ROUTE_NONE` — no routing (placement-only algorithm)

**PROPAGATE variants**:
- `PROP_CONSTRAINTS` — reduce possibility space of neighbors (WFC arc consistency)
- `PROP_CONGESTION` — update congestion cost map from committed routes
- `PROP_NONE` — no propagation (constraints encoded in EVAL)

**UPDATE variants**:
- `UPDATE_COOL` — decrease temperature: T ← α·T
- `UPDATE_INCREASE_COST` — raise penalties on congested resources
- `UPDATE_NONE` — parameters are fixed for the entire run

**TERMINATE variants**:
- `TERM_ALL_PLACED` — every agent has been committed
- `TERM_CONVERGED` — state change below threshold
- `TERM_THRESHOLD` — parameter crossed a boundary (T < T_min)
- `TERM_COLLAPSED` — all cells resolved (WFC)

---

## Algorithm Decompositions

### Simulated Annealing (VLSI Placement)

```
C = INIT_RANDOM(all nodes)
loop:
    a = SELECT_RANDOM(C)
    cands = CAND_NEIGHBOR(a, C)
    scores = EVAL_ENERGY(a, cands, C)
    p = CHOOSE_METROPOLIS(T, scores)
    C = COMMIT_TENTATIVE(a, p, C)
    T = UPDATE_COOL(T)
    if TERM_THRESHOLD(T < T_min): break
// ROUTE in separate phase
```

**Signature**: `INIT_RANDOM → (SELECT_RANDOM · CAND_NEIGHBOR · EVAL_ENERGY · CHOOSE_METROPOLIS · COMMIT_TENTATIVE · UPDATE_COOL)* → TERM_THRESHOLD`

### Force-Directed Placement

```
C = INIT_RANDOM(all nodes)
loop:
    for a in SELECT_ALL(C):
        forces = EVAL_FORCE(a, *, C)
        p = CHOOSE_ARGMAX(gradient direction)
        C = COMMIT_TENTATIVE(a, p, C)
    if TERM_CONVERGED(max_force < ε): break
```

**Signature**: `INIT_RANDOM → (SELECT_ALL · EVAL_FORCE · CHOOSE_ARGMAX · COMMIT_TENTATIVE)* → TERM_CONVERGED`

### Wave Function Collapse

```
C = INIT_EMPTY(all cells uncollapsed)
loop:
    cell = SELECT_MIN_ENTROPY(C)
    cands = CAND_ALL(cell, C)
    scores = EVAL_CONSTRAINT(cell, cands, C)  // tile weights
    v = CHOOSE_WEIGHTED(scores)
    C = COMMIT_IRREVOCABLE(cell, v, C)
    C = PROP_CONSTRAINTS(C)
    if TERM_COLLAPSED(all resolved): break
```

**Signature**: `INIT_EMPTY → (SELECT_MIN_ENTROPY · CAND_ALL · EVAL_CONSTRAINT · CHOOSE_WEIGHTED · COMMIT_IRREVOCABLE · PROP_CONSTRAINTS)* → TERM_COLLAPSED`

### Agent-Based City Generation (typical)

```
C = INIT_SEED(founding structures)
for a in SELECT_ORDERED(arrival):
    cands = CAND_RULE(a, C)
    scores = EVAL_HEURISTIC(a, cands, C)
    p = CHOOSE_RULE(scores)
    C = COMMIT_IRREVOCABLE(a, p, C)
    C = ROUTE_RULE(a.edges, C)
TERM_ALL_PLACED
```

**Signature**: `INIT_SEED → (SELECT_ORDERED · CAND_RULE · EVAL_HEURISTIC · CHOOSE_RULE · COMMIT_IRREVOCABLE · ROUTE_RULE)* → TERM_ALL_PLACED`

### PathFinder (VLSI Negotiated Routing)

```
C = INIT(all nodes pre-placed)
loop:
    for net in SELECT_ALL(C):
        path = ROUTE_NEGOTIATED(net, C)
        C = COMMIT_TENTATIVE(net, path, C)
    C = PROP_CONGESTION(C)
    Θ = UPDATE_INCREASE_COST(congested resources)
    if TERM_CONVERGED(no shared resources): break
```

**Signature**: `INIT_PLACED → (SELECT_ALL · ROUTE_NEGOTIATED · COMMIT_TENTATIVE · PROP_CONGESTION · UPDATE_INCREASE_COST)* → TERM_CONVERGED`

### Ogun

```
C = INIT_SEED(founding structures)
for a in SELECT_ORDERED(arrival):
    cands = CAND_ALL(a, C)
    scores = EVAL_UTILITY(a, cands, C)       // from potential function Φ
    p = CHOOSE_BOLTZMANN(β, scores)           // P(p) ∝ exp(β · uᵢ(p))
    C = COMMIT_IRREVOCABLE(a, p, C)
    for e in a.edges:
        C = ROUTE_NEGOTIATED(e, C)
TERM_ALL_PLACED
```

**Signature**: `INIT_SEED → (SELECT_ORDERED · CAND_ALL · EVAL_UTILITY · CHOOSE_BOLTZMANN · COMMIT_IRREVOCABLE · ROUTE_NEGOTIATED)* → TERM_ALL_PLACED`

---

## Comparison Matrix

| Operator | SA | Force-Dir | WFC | Agent-City | PathFinder | **Ogun** |
|----------|-----|-----------|-----|------------|------------|----------|
| INIT | random | random | empty | seed | pre-placed | **seed** |
| SELECT | random | all | min-entropy | ordered | all | **ordered** |
| CANDIDATES | neighbor | all (cont.) | all (remaining) | rule-filtered | N/A | **all (valid)** |
| EVAL | energy (global) | force (pairwise) | constraint (local) | heuristic (ad-hoc) | congestion cost | **utility (Φ)** |
| CHOOSE | Metropolis (cooling T) | argmax | weighted random | rule-based | shortest path | **Boltzmann (fixed β)** |
| COMMIT | tentative | tentative | irrevocable | irrevocable | tentative | **irrevocable** |
| ROUTE | separate phase | none | none | post-hoc rules | core operation | **integrated** |
| PROPAGATE | none | none | constraints | none | congestion | **none (in Φ)** |
| UPDATE | cool T | none | none | none | increase costs | **none (fixed β)** |
| TERMINATE | T < T_min | forces < ε | all collapsed | all placed | no congestion | **all placed** |

### Reading the matrix

Each column is an algorithm's signature. Algorithms that share a row value share that structural property. Algorithms that differ in a row differ in that dimension.

**Ogun's closest structural neighbor** is Agent-Based City Generation — both use `SELECT_ORDERED`, `COMMIT_IRREVOCABLE`, and `TERM_ALL_PLACED`. The differences are:

| Dimension | Agent-City | Ogun | What changes |
|-----------|------------|------|-------------|
| EVAL | heuristic (ad-hoc) | utility (potential Φ) | Formal foundation → convergence guarantees, PoA bounds |
| CHOOSE | rule-based | Boltzmann (fixed β) | Principled stochastic selection → thermodynamic consistency |
| ROUTE | post-hoc rules | integrated negotiated | Routing quality feeds back into layout quality |
| CANDIDATES | rule-filtered | all valid | Exhaustive search over position space |

**Ogun's closest mathematical neighbor** is logit dynamics on a potential game. The differences are:

| Dimension | Logit Dynamics | Ogun | What changes |
|-----------|---------------|------|-------------|
| SELECT | random (repeated) | ordered (one-pass) | Temporal structure; arrival order matters |
| COMMIT | tentative (can change) | irrevocable | No convergence to stationary distribution; single-pass output |
| ROUTE | none (abstract strategies) | integrated negotiated | Spatial domain with geometric routing |
| TERMINATE | mixing time | all placed | Finite, deterministic termination |

---

## Structural Novelty Assessment

### What Ogun shares with existing algorithms

- **Logit choice rule** (CHOOSE_BOLTZMANN): from logit dynamics / statistical mechanics
- **Potential function evaluation** (EVAL_UTILITY): from potential game theory
- **Sequential irrevocable placement** (SELECT_ORDERED + COMMIT_IRREVOCABLE): from online facility location
- **Negotiated routing** (ROUTE_NEGOTIATED): from PathFinder / VLSI CAD

### What no existing algorithm does

The specific composition: `EVAL_UTILITY · CHOOSE_BOLTZMANN(fixed β) · COMMIT_IRREVOCABLE · ROUTE_NEGOTIATED` in a `SELECT_ORDERED` loop.

Unpacked, this means:
1. Agents arrive in order and cannot be moved once placed — producing temporal layering
2. Each agent evaluates positions using a formal potential function — providing convergence guarantees and PoA bounds
3. Position selection is Boltzmann-sampled at fixed temperature — producing thermodynamically consistent local structure without cooling to optimum
4. Routing is negotiated and integrated with placement — connection quality is part of the layout, not an afterthought

No existing algorithm combines all four. The closest pairs:
- (1) + (2) without (3): greedy sequential potential game (deterministic, always picks best response — produces a single Nash equilibrium)
- (2) + (3) without (1): standard logit dynamics (repeated play, converges to Boltzmann distribution — no temporal structure)
- (1) + (3) without (2): stochastic sequential placement (random selection without game-theoretic foundation — no quality guarantees)
- (1) + (2) + (3) without (4): exists in abstract game theory but not with spatial routing

### The β parameter

The fixed inverse temperature β is the key design parameter:
- β → ∞: deterministic best-response (greedy sequential potential game)
- β → 0: uniform random placement (ignores utility entirely)
- Finite β: controlled stochasticity — the "optimization level" of the output

This is NOT the same as SA's cooling schedule. SA uses temperature to escape local minima during optimization. Ogun uses β to control the **character** of the output — it's a generative parameter, not an optimization parameter.

### What this means for the paper

The publishable contribution is:
1. **Formulation**: spatial layout as a potential game with heterogeneous agents
2. **Mechanism**: one-pass sequential logit placement with irrevocable commitment and integrated routing
3. **Property**: β controls output optimization level with internal consistency (not optimize-then-degrade)
4. **Analysis**: relationship between β, agent heterogeneity, arrival order, and emergent PoA
5. **Application**: first use of game-theoretic potential functions for procedural city/layout generation

---

## Time & Space Complexity Analysis

### Variables

| Symbol | Meaning |
|--------|---------|
| n | Number of agents (nodes to place) |
| m | Number of edges (connections to route); m ≈ n·d/2 |
| d | Average node degree |
| S | Domain size (total grid cells); S = W × H |
| r | Interaction radius for local utility |

### Per-operator costs

| Operator | Cost | Notes |
|----------|------|-------|
| SELECT_ORDERED | O(1) | Next in queue |
| CAND_ALL | O(S) | Enumerate valid positions |
| EVAL_UTILITY (local, radius r) | O(r²) per position | Check neighborhood for congestion, attraction/repulsion |
| EVAL_UTILITY (global) | O(k) per position | k = agents placed so far; sum over all pairwise interactions |
| EVAL_UTILITY (precomputed field) | O(1) per position | Read from cached potential field; field update O(S) after each COMMIT |
| CHOOSE_BOLTZMANN | O(S) | Compute weights, normalize, sample via inverse CDF |
| COMMIT_IRREVOCABLE | O(1) | Update placement map + spatial index |
| ROUTE_NEGOTIATED (per edge) | O(S · log S) | A* on grid with congestion costs |

### Total complexity by EVAL strategy

**Local utility (radius r)**:
```
Per agent:  O(S · r²)  [eval]  +  O(S)  [choose]  +  O(d · S · log S)  [route]
Total:      O(n · S · (r² + d · log S))
```

**Global utility (naive)**:
```
Per agent k: O(S · k)  [eval]  +  O(S)  [choose]  +  O(d · S · log S)  [route]
Total:       O(S · (n² + n · d · log S))
```

**Precomputed potential field** (recommended):
```
Per agent:  O(S)  [eval: read field]  +  O(S)  [choose]  +  O(S)  [field update]  +  O(d · S · log S)  [route]
Total:      O(n · d · S · log S)
```

Routing dominates. For placement-only (no routing): **O(n · S)** — linear in both agents and domain.

### Comparison to other algorithms

| Algorithm | Complexity | Iterations | Notes |
|-----------|-----------|------------|-------|
| **SA (VLSI)** | O(T · d) | T >> n (millions) | T is cooling schedule length; routing separate |
| **Force-Directed** | O(I · n log n) | I until convergence | Barnes-Hut approximation; routing separate |
| **WFC** | O(S · log S) | S collapses | With heap for min-entropy; no routing |
| **Agent-City** | O(n · S · log S) | 1 pass | Similar to Ogun but ad-hoc eval |
| **PathFinder** | O(I · m · S · log S) | I negotiation rounds | Routing only; I typically 10–50 |
| **Ogun** | O(n · d · S · log S) | **1 pass** | Single pass, no iteration |

Ogun's key efficiency property: **single-pass, no iteration**. SA requires millions of perturbation steps. Force-directed requires convergence iterations. PathFinder requires negotiation rounds. Ogun processes each agent exactly once.

The tradeoff: iterative algorithms can improve their solution over time. Ogun's output quality is determined by the potential function design and β — there's no iterative refinement. This is by design (irrevocable commitment models real settlement growth), but it means the potential function Φ must be well-designed upfront.

### Space complexity

| Component | Cost |
|-----------|------|
| Placement map P | O(S) |
| Potential field cache | O(S) |
| Congestion map (for routing) | O(S) |
| Routing paths | O(m · √S) average path length |
| Spatial index | O(S) |
| **Total** | **O(S + m · √S)** |

### Numerical considerations

- **Boltzmann overflow**: exp(β · score) overflows for large β. Use log-sum-exp trick: subtract max score before exponentiating. Standard, O(S) extra pass.
- **Candidate pruning**: CAND_ALL is O(S) but most positions may be invalid (occupied, out of footprint). Maintaining a free-cell set reduces this to O(|free cells|), which shrinks as agents are placed.
- **Potential field updates**: if Φ is decomposable (sum of pairwise terms), the field update after placing agent i is O(S) in the worst case but O(r²) if interactions are local. With local interactions, total field maintenance is O(n · r²).

### Practical estimate for target use case

For saltglass-steppe scale (250×110 grid, ~100 structures, d ≈ 3, r ≈ 10):
- S = 27,500
- n = 100
- Placement: 100 × 27,500 = 2.75M eval calls (with field cache: 2.75M lookups)
- Routing: 150 edges × 27,500 × log(27,500) ≈ 60M operations
- Total: ~60M operations → **well under 1 second** on modern hardware
- Memory: ~200 KB for grids + ~1 MB for routing paths

---

## Complexity Reduction: Mathematical Identities & Reformulations

The naive analysis gives O(n · d · S · log S). Three operations dominate:

1. **EVAL over all positions**: O(S) per agent
2. **CHOOSE_BOLTZMANN**: O(S) per agent (compute weights, normalize, sample)
3. **ROUTE**: O(S · log S) per edge

Each can be reduced by rephrasing the problem in terms that expose exploitable structure.

### Reduction 1: Merge EVAL + CHOOSE via Weighted Segment Tree

**The O(S) problem**: We evaluate u_i(p) at every cell, compute exp(β · u_i(p)) for each, normalize, then sample. Three O(S) passes.

**The rephrasing**: Don't think of EVAL and CHOOSE as separate steps. Think of the grid as a **weighted probability structure** that we maintain incrementally.

Maintain a segment tree (or 2D Fenwick tree / quadtree) over grid cells where each leaf stores w(p) = exp(β · u(p)) and each internal node stores the sum of its children's weights.

- **Sampling**: Walk down the tree, at each node go left with probability w_left / (w_left + w_right). Cost: **O(log S)**.
- **Update after COMMIT**: When agent j is placed at q, only cells within interaction radius r have their utility changed. Update those leaves and propagate up. Cost: **O(r² · log S)**.

**Total for all placements**: O(n · r² · log S + n · log S) = **O(n · r² · log S)**

vs. naive O(n · S). For r = 10, S = 10⁶: that's 100 · 20 = 2000 vs. 1,000,000 — a **500× speedup**.

**Mathematical basis**: The segment tree exploits the fact that Boltzmann sampling is equivalent to sampling from a categorical distribution, and categorical distributions support O(log n) sampling with O(log n) weight updates via augmented balanced trees.

### Reduction 2: Gumbel-Max Trick for Lazy Boltzmann Sampling

**The identity**: Sampling p with probability P(p) ∝ exp(β · u(p)) is equivalent to:

```
For each p: g(p) = β · u(p) + Gumbel(0,1)
Return argmax_p g(p)
```

where Gumbel(0,1) = -ln(-ln(Uniform(0,1))).

**Why this helps**: Argmax can be computed lazily with branch-and-bound. Maintain a quadtree where each node stores an upper bound on u(p) in its subtree. Then:

1. Start at root. Compute upper bound on g(p) for each child: β · max_u(child) + max_possible_Gumbel
2. Expand the most promising child first (priority queue)
3. Prune children whose upper bound < current best g(p)
4. At leaf: compute exact g(p), update best

**Expected cost**: O(log S) when the distribution is concentrated (high β). O(S) worst case (low β, near-uniform). This is formalized as **A\* Sampling** (Maddison, Tarlow & Adams, 2014).

**When to use**: High β (low temperature) — exactly the regime where the naive approach wastes the most time evaluating low-probability positions. Complements the segment tree approach, which has uniform O(log S) regardless of β.

### Reduction 3: Separable Kernels for Field Updates

**The structure**: The utility field is a sum of interaction kernels:

```
u(p) = Σ_j K(p - q_j)
```

When agent j is placed at q_j, the field update is: for all p in range, u(p) += K(p - q_j). Naively O(r²) per placement.

**The identity**: If K is separable — K(x, y) = f(x) · g(y) — then the 2D update decomposes into two 1D updates:

```
// Instead of r² cell updates:
for dx in -r..r:
    for dy in -r..r:
        field[qx+dx][qy+dy] += f(dx) · g(dy)

// Do r row updates + r column updates:
row_acc[qy+dy] += g(dy)  for dy in -r..r     // O(r)
col_acc[qx+dx] += f(dx)  for dx in -r..r     // O(r)
// Reconstruct: field[x][y] = Σ col_acc[x] · row_acc[y]
```

**Cost**: O(r) per placement instead of O(r²).

**Which kernels are separable?**
- Gaussian: K(x,y) = exp(-(x²+y²)/2σ²) = exp(-x²/2σ²) · exp(-y²/2σ²) ✓
- Box (uniform within radius): separable as product of 1D box functions ✓
- Inverse distance 1/√(x²+y²): NOT separable ✗ — but can be approximated as sum of separable terms (fast multipole method)

**Practical impact**: For r = 20, this is 40 vs 1600 operations per placement — **40× speedup** on field updates.

### Reduction 4: Barnes-Hut for Global Interactions

If the utility includes long-range terms (e.g., "distance to nearest market," "total accessibility"), evaluating these naively is O(k) per position where k = agents placed so far.

**The rephrasing**: This is the N-body problem. Each placed agent exerts a "gravitational" influence on all positions. Barnes-Hut (1986) solves this:

- Build a quadtree over placed agents
- For a query position p, traverse the tree
- If a node's agents are far enough away (opening angle < θ), approximate their combined influence as a single source
- Cost: **O(log k)** per position instead of O(k)

**When it matters**: Only when utility has global (non-local) interaction terms. For purely local interactions (radius r), this isn't needed.

### Reduction 5: Hierarchical Pathfinding for ROUTE

**The O(S log S) problem**: A* on the full grid for each edge.

**The rephrasing**: Decompose routing into coarse planning + local refinement.

**HPA\* (Hierarchical Pathfinding A\*)**:
1. **Preprocess**: Divide grid into R regions of size S/R each. Build a region graph with edge weights = shortest path between region borders. Cost: O(S) one-time.
2. **Route**: Find path on region graph: O(R · log R). Refine within each region on the path: O((S/R) · log(S/R)) per region.
3. **Total per edge**: O(R · log R + L · (S/R) · log(S/R)) where L = path length in regions.

For S = 10⁶, R = 1000 (regions of ~1000 cells): routing drops from O(10⁶ · 20) to O(1000 · 10 + L · 1000 · 10) ≈ O(10⁴ · L).

**Incremental update**: When an agent is placed, only the region it occupies needs re-preprocessing. Cost: O(S/R) per placement.

### Reduction 6: Spectral Potential Field (Fourier Domain)

**The rephrasing**: The utility field u(p) = Σ_j K(p - q_j) is a convolution. In the Fourier domain:

```
û(k) = K̂(k) · Σ_j exp(-2πi k · q_j)
```

The sum S(k) = Σ_j exp(-2πi k · q_j) is a **structure factor** that can be updated incrementally:

```
S(k) ← S(k) + exp(-2πi k · q_new)    // O(1) per frequency
```

If we truncate to F low-frequency components (the field is spatially smooth), the full field can be reconstructed via inverse FFT in O(F log F), and point queries are O(F).

**When it matters**: When the interaction kernel is smooth and long-range (not sharp cutoff). For sharp local kernels, the spatial-domain approach (segment tree) is better.

**Hybrid**: Use spectral methods for the smooth long-range component of utility, spatial methods for the sharp local component. This is standard in molecular dynamics (Ewald summation / particle-mesh Ewald).

---

## Revised Complexity with Optimizations

### Recommended approach: Segment tree + separable kernels + HPA*

| Operation | Naive | Optimized | Method |
|-----------|-------|-----------|--------|
| EVAL + CHOOSE (per agent) | O(S) | **O(log S)** | Weighted segment tree sampling |
| Field update (per COMMIT) | O(r²) | **O(r · log S)** | Separable kernel + tree update |
| ROUTE (per edge) | O(S · log S) | **O(R log R + L · (S/R) log(S/R))** | HPA* |
| Total (all agents) | O(n · d · S · log S) | **O(n · r · log S + m · R log R)** | Combined |

For saltglass-steppe scale (S = 27,500, n = 100, r = 10, d = 3):
- Naive: ~60M operations
- Optimized: ~100 · 10 · 15 + 150 · 30 · 5 ≈ 37K operations

For large scale (S = 10⁶, n = 1000, r = 20, d = 4):
- Naive: ~80B operations (minutes)
- Optimized: ~1000 · 20 · 20 + 2000 · 1000 · 10 ≈ 20M operations (milliseconds)

The segment tree reduction is the single biggest win — it eliminates the O(S) sweep entirely.
