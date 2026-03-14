# ogun

Spatial layout generation via sequential logit dynamics on potential games.

Named after the Yoruba deity of iron, metalwork, and pathfinding.

## What it does

Ogun generates 2D spatial layouts (placed nodes, routed edges) at a **controllable optimization level**. Unlike "optimize then degrade" approaches, Ogun produces layouts with internal consistency at the target quality — the way real cities have coherent local structure despite global inefficiency.

## Core mechanism

1. Agents arrive **sequentially** and are placed **irrevocably**
2. Each agent selects a position via **logit choice** (noisy best-response) against a **potential function** encoding spatial utilities
3. Connections are **routed** with congestion-aware pathfinding
4. The inverse temperature **β** controls output character — from near-random (low β) to near-optimal (high β)
5. The **Price of Anarchy** emerges from agent heterogeneity and arrival order, not post-hoc degradation

## Status

Early research / pre-implementation. See `docs/` for:
- `docs/SCOPE.md` — project scope and algorithm specification
- `docs/research/OGUN_ALGORITHM_RESEARCH.md` — theoretical foundations, operator decomposition, complexity analysis
- `docs/research/research_notes.md` — raw research notes

## License

MIT
