# PCB-Inspired Procedural City Generation

> Status: Scoping
> Last updated: 2026-03-14

---

## Core Idea

Borrow classical algorithms from PCB design (Electronic Design Automation) and adapt them for procedural town and city generation. The hypothesis is that the spatial optimization problems solved by EDA — component placement, trace routing, congestion management, constraint satisfaction on a grid — map directly onto urban layout problems: building placement, road networks, infrastructure routing, and zoning constraints.

The goal is not to simulate PCB design, but to use its algorithmic toolkit as a generation engine that produces city layouts with a distinctive structured-yet-organic quality — particularly suited to the post-industrial, glass-fused aesthetic of Saltglass Steppe.

---

## Scope

### In scope
- Classical deterministic EDA algorithms adapted for city generation
- Pure Rust implementation, compatible with terrain-forge's `Algorithm` trait (`Send + Sync`)
- Deterministic output via seeded RNG (ChaCha8Rng)
- Integration path for both terrain-forge (as new algorithms) and saltglass-steppe (as city/town generator)
- Grid-based and graph-based representations

### Out of scope
- Machine learning / neural network approaches (GNNs, DRL, transformers)
- Electrical simulation (signal integrity, impedance, thermal analysis)
- Real-world urban planning accuracy — this is game procgen, not civil engineering
- 3D building generation (output is 2D layout; vertical detail is a rendering concern)

---

## The PCB → City Mapping

### Direct parallels

| PCB Concept | City Analogue | Notes |
|-------------|---------------|-------|
| Component | Building / structure | Placed entity with a footprint, connection requirements, and keep-out zones |
| Net / netlist | Infrastructure demand | Which buildings need to connect (roads, utilities, trade routes) |
| Trace | Road / path | Physical connection between placed components |
| Via | Intersection / bridge / tunnel | Point where routes change layer or cross |
| Layer | Infrastructure stratum | Ground-level roads, underground utilities, elevated walkways |
| Keep-out zone | Zoning restriction | Areas where certain structures can't be placed (terrain, sacred ground, ruins) |
| Design Rule Check (DRC) | Layout validation | Minimum road width, building spacing, accessibility constraints |
| Board outline | City boundary | The bounding region for generation |
| Pad / pin | Building entrance / connection point | Where a structure interfaces with the road network |

### Where the analogy diverges

| PCB property | City reality | Implication |
|--------------|-------------|-------------|
| Optimal routing is the goal | Organic inefficiency is desirable | Need a post-processing pass that introduces irregularity, dead ends, winding paths |
| Flat hierarchy (components + traces) | Deep hierarchy (districts → blocks → buildings) | Need to layer algorithms at multiple scales |
| Components are pre-defined | Buildings vary wildly in size/shape | Component library needs to be generative or template-based |
| All connections are equally important | Roads have hierarchy (highway → street → alley) | Trace width / priority mapping to road importance |
| Static design | Cities grow over time | Optional: iterative generation that simulates growth phases |

---

## Algorithms to Adapt

### Tier 1 — Core (implement first)

**Force-Directed Placement**
- PCB use: Components modeled as masses, nets as springs. Repulsive forces prevent overlap, attractive forces minimize trace length.
- City adaptation: Buildings as masses, connection demands as springs. Produces clustered layouts where related structures (market + warehouses, temple + housing) naturally group together.
- Properties: Deterministic with fixed iteration count, tunable spring constants, naturally handles variable-size components.
- Existing reference: KiCad's force-directed autoplacement, yworks force-directed graph layout.

**Lee Algorithm (Maze Routing)**
- PCB use: BFS on a grid to find shortest path between two points, respecting obstacles.
- City adaptation: Route roads between placed buildings on a tile grid, respecting terrain and existing structures.
- Properties: Guarantees shortest path if one exists. Memory-heavy on large grids but straightforward to implement.
- Enhancement: Hadlock's algorithm (A*-like detour heuristic) for better performance on large maps.

### Tier 2 — Enhancement (implement after core works)

**Negotiation-Based Routing (PathFinder)**
- PCB use: Multiple traces compete for grid cells. Iteratively penalizes shared resources until each trace has a unique path.
- City adaptation: Multiple road demands compete for limited space. Produces road networks where major routes "win" prime real estate and minor routes detour — naturally creating road hierarchy.
- Properties: Iterative, converges to valid solution, naturally produces primary/secondary/tertiary road structure.

**Congestion-Aware Placement**
- PCB use: Predict routing congestion from placement and adjust component positions to reduce bottlenecks.
- City adaptation: After initial placement, identify areas where too many roads would need to converge and redistribute buildings to reduce chokepoints.

### Tier 3 — Polish (optional, for character)

**Entropy / Decay Pass → Functional Erosion**
- Not from PCB — this is the "anti-optimization" layer.
- Evolved concept: rather than random perturbation, simulate **functional erosion** — entropy overtaking human maintenance effort.
- Each structure and route has a *function* (shelter, transit, trade, defense). Erosion degrades function first, and physical collapse follows as a consequence.
- Erosion is iterative and cascading:
  1. A road segment degrades → reduced connectivity
  2. Buildings served only by that road lose accessibility → accelerated decay
  3. Collapsed buildings produce rubble → new obstacles that block adjacent roads
  4. Isolation propagates outward from the initial failure point
- This produces ruins that *make sense* — you can read the history of collapse in the layout. The player sees a blocked main road and understands why an entire district is dead.
- Erosion rate is tunable per structure type (stone walls last longer than wooden roofs, main roads resist longer than alleys) and feeds into the target optimization score.
- For saltglass-steppe: glass storms act as erosion accelerators on exposed structures while simultaneously *fusing* others into permanent obstacles.

**Hierarchical Generation**
- Run the pipeline at multiple scales:
  1. District placement (force-directed, large components = districts with type: residential, commercial, industrial, sacred)
  2. Block layout within districts (force-directed, medium components = building clusters)
  3. Road routing between districts (PathFinder for major roads)
  4. Street routing within districts (Lee/Hadlock for local streets)
  5. Building placement within blocks (force-directed or grid-snap)

---

## Prior Work & References

### PCB / EDA algorithms (primary sources)
- Lee, C.Y. (1961) — "An Algorithm for Path Connections and Its Applications" (original maze routing)
- Hadlock, F.O. (1977) — "A Shortest Path Algorithm for Grid Graphs" (detour-number improvement)
- McMurchie & Ebeling (1995) — "PathFinder: A Negotiation-Based Performance-Driven Router" (negotiation routing)
- Force-directed placement — standard in EDA since the 1980s, well-documented in VLSI textbooks

### PCB ↔ urban planning crossover
- "From Electronic Design Automation to Building Design Automation" (2023, arXiv:2305.06380)
- Density penalty and overlap constraint methods shared between IC placement and land-use optimization (arXiv:2210.14259)
- Network topology studies comparing transport and circuit network efficiency (FBK Magazine)

### Existing procgen approaches (for contrast)
- WFC (Wave Function Collapse) — tile-based, good for local coherence but no global structure
- BSP (Binary Space Partitioning) — recursive subdivision, produces grid-like layouts
- Voronoi-based — organic district boundaries but no road routing
- L-systems — good for organic growth patterns but hard to constrain
- Agent-based — simulates growth but non-deterministic without careful seeding

### Game references
- SYNTHETIK 1 & 2 — mechanical, circuit-board-like level layouts
- Saltglass Steppe — target integration, post-apocalyptic glass-fused cities

---

## Research Angle

Following the terrain-forge / Glass Seam Bridging precedent: this crate should be anchored by a novel algorithm with an accompanying research paper (LaTeX).

### What's novel

The individual techniques (force-directed placement, maze routing, negotiation routing) are established. The novelty is in their combination and reframing:

1. **Formal PCB → urban primitive mapping** with defined semantics (not just analogy — a concrete translation layer with rules)
2. **Target Optimization Score (TOS)** as a governing parameter — the idea that believable procedural cities exist in a measurable band between over-optimized and chaotic, and that generation should aim for a specific point in that band
3. **Functional Erosion** as a principled, cascading degradation model that preserves readable structural history in the output
4. **The unified pipeline** — placement → routing → scoring → erosion as a single generation framework where TOS controls the character of the output

The publishable algorithm is the pipeline itself: an EDA-derived procedural city generator governed by a target optimization score, with functional erosion as its entropy mechanism.

### Paper structure (rough)
1. Introduction — the PCB/urban planning parallel, motivation for procgen
2. Related work — EDA algorithms, existing city procgen (WFC, BSP, L-systems, agent-based), the crossover literature
3. Method — the formal mapping, adapted algorithms, TOS framework, functional erosion model
4. Results — generated outputs at various TOS values, comparison with existing procgen methods
5. Discussion — limitations, tuning, applicability beyond games

### Name
**Ogun** — from the Yoruba deity of iron, metalwork, and pathfinding. Ogun cleared the first roads through primordial wilderness for the other gods to follow. A god who routes paths through uncharted space — the core operation of this algorithm.

- Algorithm name: Ogun
- Core crate: `ogun`
- Application crate (city generator): TBD

---

## The Ogun Algorithm

### Definition

Ogun is a **target-convergent spatial layout algorithm**. Given a weighted graph and spatial constraints, it produces a 2D layout at a specified optimization level — not maximum, not minimum, but a target.

### Input
- **Graph**: Nodes (with sizes/footprints) and edges (with weights/priorities)
- **Space**: 2D boundary, obstacle map, keep-out zones
- **Target Optimization Score (TOS)**: Float in [0.0, 1.0] — where 1.0 is perfectly optimized and 0.0 is fully degraded
- **Seed**: For deterministic RNG

### Output
- **Layout**: Node positions within the space, edge paths as sequences of grid cells
- **Score**: The achieved optimization score
- **Metadata**: Per-node accessibility, per-edge efficiency, congestion map

### Process (convergence loop)

```
1. PLACE   — Force-directed node placement
             Nodes repel each other (prevent overlap)
             Connected nodes attract (minimize edge length)
             Obstacles and keep-outs exert repulsive force
             Iterate until stable

2. ROUTE   — Edge path routing
             For each edge, find path between connected nodes
             Phase A: Maze routing (Lee/Hadlock) for initial paths
             Phase B: Negotiation routing (PathFinder) to resolve conflicts
             Higher-priority edges win contested space

3. SCORE   — Measure composite optimization score
             Weighted combination of metrics:
               - Path efficiency (actual vs. optimal length)
               - Accessibility (reachable nodes from any node)
               - Congestion (max edges sharing a region)
               - Void ratio (unused space)
               - Dead-end ratio
             Normalize to [0.0, 1.0]

4. CONVERGE — Adjust toward target
             IF score > TOS + tolerance:
               - Sever lowest-priority edges
               - Perturb node positions (displacement proportional to overshoot)
               - Inject obstacles (dead zones, blockages)
               → Re-route affected edges (step 2, partial)
             IF score < TOS - tolerance:
               - Re-place displaced nodes closer to optimal
               - Re-route severed edges via longer paths
               → Tighten toward target from below
             REPEAT from step 3 until |score - TOS| < tolerance
```

### What's novel

Existing layout algorithms optimize toward a minimum cost. Ogun optimizes toward a **target cost** — it can produce layouts anywhere on the spectrum from perfectly efficient to heavily degraded, and it converges on the requested point. The convergence loop (steps 3–4) is the novel contribution. Steps 1–2 are adapted from established EDA techniques.

### Properties
- **Deterministic**: Same graph + space + TOS + seed = same output
- **Domain-agnostic**: Operates on abstract nodes and edges, not buildings and roads
- **Tunable**: TOS is a single parameter that controls output character
- **Composable**: Can be run at multiple scales (coarse placement, then fine-grained within regions)
- **Thread-safe**: `Send + Sync` — no interior mutability, pure function from input to output

---

## Crate Architecture

Two crates, following the `itoa`/`ryu` model — core algorithm separate from application.

### `ogun` — Core algorithm crate
The target-convergent spatial layout algorithm. Domain-agnostic.

Responsibilities:
- Force-directed node placement
- Maze and negotiation-based edge routing
- Composite optimization scoring
- Target convergence loop (the novel part)

Does NOT know about: cities, buildings, roads, erosion, game engines, tile maps.

API surface (conceptual):
```rust
pub struct Graph { nodes: Vec<Node>, edges: Vec<Edge> }
pub struct Space { width: u32, height: u32, obstacles: Vec<Rect> }
pub struct OgunConfig { target_score: f32, tolerance: f32, seed: u64 }
pub struct Layout { positions: Vec<Position>, paths: Vec<Path>, score: f32 }

pub fn generate(graph: &Graph, space: &Space, config: &OgunConfig) -> Layout
```

### Application crate (TBD name) — City generator
Urban generation built on top of Ogun.

Responsibilities:
- Define urban node/edge types (building footprints, road demands, zoning)
- Configure Ogun with city-specific constraints and TOS targets
- Functional erosion (cascading degradation with urban semantics)
- Hierarchical generation (run Ogun at district → block → building scales)
- Output as tile map / semantic grid

Depends on: `ogun`, `serde`, `rand_chacha`

---

## Project Form

Standalone Rust crate (library), following the same model as terrain-forge: published to crates.io, consumed as a dependency by saltglass-steppe and potentially other projects.

The crate is a generation-time tool — it produces city layouts, it doesn't run them. Game-specific concerns (rendering, real-time simulation, theming) belong to the consumer.

### Relationship to terrain-forge
Two options to resolve:
- **Option A — Separate crate**: Independent library with its own API. terrain-forge and this crate are siblings, both consumed by saltglass-steppe.
- **Option B — terrain-forge module**: New algorithm(s) added to terrain-forge, extending its existing `Algorithm` trait and pipeline system.

Option A gives more freedom (custom data structures, city-specific API) but means maintaining two crates. Option B leverages existing infrastructure (pipeline, serializable configs, semantic layers) but constrains the API to terrain-forge's grid-based model.

---

## Integration Path

### terrain-forge
- New algorithm(s) implementing the `Algorithm` trait
- Could be a single `PcbCity` algorithm with config for which phases to run, or separate algorithms (`ForceDirectedPlacement`, `MazeRouting`, `NegotiationRouting`) composable via terrain-forge's pipeline system
- Output: semantic layer grid (roads, buildings, open space, walls, etc.)
- Config: serializable (serde), matching terrain-forge's pattern

### saltglass-steppe
- Consumes terrain-forge output as tile map data
- Applies game-specific theming: glass-fused structures, storm damage, crystal formations
- POI integration: place quest-relevant locations as "high-priority components" in the netlist
- Biome influence: desert terrain constraints feed into keep-out zones and road routing costs

---

## Evaluation: Target Optimization Score

A core design principle: believable cities are **semi-optimized**. They sit in a band between two failure modes:

| Extreme | What it looks like | Why it fails |
|---------|--------------------|--------------|
| Over-optimized | Perfect grid, shortest paths everywhere, no wasted space | Feels sterile, mechanical, inhuman — no city looks like a PCB |
| Under-optimized | Random sprawl, dead ends everywhere, no coherent structure | Feels broken, unnavigable — no city is truly random either |
| **Target band** | **Mostly logical with local inefficiencies, organic deviations, historical accidents** | **Reflects how humans actually build: pragmatic but imperfect** |

This means the generation pipeline should:
1. **Start optimized** — let the PCB algorithms do their job (placement, routing, congestion resolution)
2. **Measure the optimization score** — quantify how "perfect" the layout is
3. **Degrade toward a target band** — introduce controlled entropy until the score falls within a desired range

### Candidate metrics for the optimization score

| Metric | Optimal (PCB) | Realistic (city) | Chaotic |
|--------|---------------|-------------------|---------|
| Path efficiency (actual vs. shortest route) | ~1.0 | 1.2–1.6 | >2.0 |
| Dead-end ratio (dead ends / total road segments) | 0% | 10–25% | >50% |
| Building accessibility (% of buildings reachable from main road within N steps) | 100% | 90–98% | <70% |
| Road hierarchy variance (std dev of road widths/importance) | Low (uniform) | High (highways + alleys) | Low (all same) |
| Clustering coefficient (how much related buildings group) | Very high | Moderate–high | Random |
| Void ratio (unused space / total area) | Near 0% | 15–30% | >50% |

The target band is tunable per generation context:
- A **planned capital city** would score closer to the optimized end
- A **frontier outpost** would score toward the middle
- A **post-catastrophe ruin** (saltglass-steppe) would score toward the chaotic end, but with visible traces of former optimization — the ghost of a plan

### Implementation approach

The optimization score could work as a feedback loop:
1. Generate layout using PCB algorithms (score ≈ 0.95)
2. Apply entropy pass (perturbation, path breaking, gap insertion)
3. Measure score after each entropy step
4. Stop when score enters the target band for the city type
5. Validate minimum constraints (main road connectivity, building accessibility threshold)

This gives a single tunable parameter — the **target optimization score** — that controls the character of the output. A slider from "circuit board" to "organic ruin" with believable cities in the middle.

---

## Open Questions

1. **Grid resolution**: What tile size works for both force-directed placement and maze routing? PCB grids are uniform; city grids might need variable resolution (coarse for districts, fine for streets).

2. **Component library**: How are buildings defined? Fixed footprints from JSON data files (matching saltglass-steppe's data-driven approach)? Parameterized templates? Procedurally sized?

3. **Connection semantics**: In PCBs, the netlist explicitly defines what connects to what. For cities, what drives connection demand? Building type adjacency rules? Population flow? Trade routes?

4. **Multi-layer**: Is the layer concept useful beyond a single ground plane? Underground (sewers, tunnels, catacombs) and elevated (bridges, walkways) could use separate routing layers with vias as vertical connections.

5. **Scale**: What map sizes are we targeting? Saltglass-steppe's tile maps are 250×110. A single town might be 40×40 to 80×80 tiles. Does Lee routing scale acceptably at that size?

6. **Composability**: Should this be one monolithic algorithm or a pipeline of composable steps? terrain-forge's pipeline system could chain placement → routing → decoration.

7. **Evaluation**: How do we judge output quality? PCBs have DRC. What's the city equivalent — connectivity checks, dead-end limits, building accessibility, aesthetic metrics?

8. **Functional erosion model**: What properties does each structure/route need to carry for erosion to work? At minimum: material durability, functional type, connectivity degree, maintenance dependency. How much simulation state is too much for a procgen algorithm vs. a full game system?

9. **Erosion as generation vs. erosion as game mechanic**: Functional erosion is a generation-time algorithm within the crate. If a consumer wants real-time erosion, they can call the erosion function iteratively on their own schedule — the crate doesn't need to care about runtime context.

10. **Crate scope**: Where does this crate's responsibility end? It generates a city layout (placed buildings, routed roads, optional erosion). Does it also handle interior generation? NPC placement? Or is that strictly consumer-side?

11. **Separate crate vs. terrain-forge module**: Independent crate gives API freedom but means a second crate to maintain. terrain-forge integration reuses existing infrastructure but constrains the data model. Which path?
