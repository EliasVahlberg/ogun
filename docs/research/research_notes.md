Excellent research results. Let me synthesize what I've found and present the different framings clearly.

Key findings:

1. **Potential Games + Price of Anarchy**: This is very promising. The Price of Anarchy in transportation networks has been studied extensively — it measures exactly the ratio between selfish (Nash equilibrium) routing and globally optimal routing. Real cities have been measured for their PoA. This is literally the "optimization gap" we're trying to produce.

2. **Online Facility Location**: The competitive ratio for online facility location is Θ(log n / log log n), and for incremental it's O(1). The Online Multi-Commodity Facility Location paper (arXiv:2005.08391) specifically addresses heterogeneity and shows it influences the competitive ratio.

3. **Agent-based urban morphology**: There's existing work on agent-based city generation (Springer 2019 paper on emergent urban morphologies, the Minecraft settlements paper, procedural urban environments for FPS). But these are mostly ad-hoc agent rules, not grounded in game theory.

4. **Spatial equilibria**: The paper "Potential Maximization and Stochastic Stability of Spatial Equilibria" (arXiv:2011.06778) is very relevant — it studies how spatial distribution of agents in equilibrium depends on initial conditions, with multiple locally stable equilibria.

5. **Urban Neighbourhood Games**: "Improving Equilibria in Urban Neighbourhood Games" (arXiv:2601.08642) — this is directly relevant! It formalizes urban development as a game.

Now let me think about the different framings:

**Framing 1: Spatial Congestion Game**
The layout is a congestion game on a spatial grid. Agents (builders) compete for spatial resources (grid cells). Each agent has a utility function based on:
- Proximity to desired neighbors (attraction)
- Distance from undesired neighbors (repulsion)
- Access to shared infrastructure (roads)
- Cost of the space (congestion — more agents wanting nearby space = higher cost)

The Nash equilibrium of this game IS the city layout. The Price of Anarchy measures how far the equilibrium is from the global optimum — and this IS the "optimization score."

Key insight: We don't need to TARGET a specific PoA. We can CONTROL it by adjusting the game parameters (agent heterogeneity, information availability, move order). Different game configurations produce different PoA values, which produce different city characters.

**Framing 2: Statistical Mechanics / Boltzmann Distribution**
The layout is a physical system. Each configuration has an energy (inverse of optimization score). At temperature T:
- T → 0: ground state (globally optimal layout)
- T → ∞: uniform random (maximum entropy)
- Specific T: Boltzmann distribution — configurations are sampled with probability proportional to exp(-E/kT)

The key insight: sampling from a Boltzmann distribution at temperature T doesn't give you "optimal + noise." It gives you configurations that are THERMODYNAMICALLY CONSISTENT at that energy level. They have internal structure and correlations that pure noise doesn't have.

This maps to: instead of simulated annealing (which cools to find the minimum), we anneal to a specific temperature and SAMPLE from the equilibrium distribution. This is called "Boltzmann sampling" or "equilibrium sampling."

**Framing 3: Online Multi-Commodity Facility Location**
Agents arrive sequentially. Each agent must be placed irrevocably. Different agent types have different facility requirements (multi-commodity). The competitive ratio depends on agent heterogeneity.

This is the most directly applicable existing framework. The paper by Fotakis (2005) on online/incremental facility location gives competitive ratio bounds. The multi-commodity extension (arXiv:2005.08391) shows heterogeneity matters.

**Framing 4: Morphogenetic Growth**
The PLOS ONE paper "On the Morphology of a Growing City" merges static equilibrium analysis with agent-based simulation. The Miskolc Method paper models city evolution with recursive algorithms using simulated morphogenesis.

This is the biological analogy: cities grow like organisms, following local rules that produce emergent global structure.

Now, which framing gives us the best core mechanism?

I think the most promising is the **Spatial Potential Game** framing because:
1. It has a well-defined mathematical foundation (potential games guarantee Nash equilibrium existence and convergence)
2. The Price of Anarchy is EXACTLY the metric we want — it measures the gap between selfish equilibrium and global optimum
3. We can control the PoA by adjusting game parameters (agent utility functions, information structure, move order)
4. It's been applied to transportation networks but NOT to procedural layout generation
5. It avoids the three-body problem because potential games have guaranteed convergence (unlike general games)

The key property of potential games: there exists a global potential function Φ such that any unilateral improvement by any agent also improves Φ. This means best-response dynamics always converge to a Nash equilibrium (local maximum of Φ). No chaos, no three-body problem.

The novel contribution would be:
- Formulating spatial layout as a potential game with heterogeneous agents
- Showing that the Price of Anarchy of the resulting equilibrium corresponds to the "realism" of the layout
- Providing a mechanism to control the PoA (and thus the layout character) through game parameter design
- Demonstrating this produces better procedural cities than existing methods

This is a much stronger paper than "optimize then degrade."

Let me present these framings to the user.

 - Completed in 0.0s

> Here are three framings of the same problem, each with different research lineage and different implications for what the core
mechanism would be:

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


Framing 1: Spatial Potential Game

The layout is a game. Each builder-agent has a utility function (market wants road access, temple wants isolation, house wants
proximity to services). Agents take turns placing themselves, each maximizing their own utility given the current state.

If we can formulate this as a potential game — where a single global function Φ exists such that any agent improving their own
utility also improves Φ — then:
- Best-response dynamics are guaranteed to converge to a Nash equilibrium (no chaos, no three-body problem)
- The Price of Anarchy (ratio of equilibrium cost to global optimum) is exactly the "optimization gap" we want to produce
- We control the PoA by tuning agent heterogeneity, information structure, and move order — not by post-hoc degradation

This is the cleanest framing because potential games have convergence guarantees by construction. The PoA has been measured on
real city road networks (Youn et al., 2008 — found significant waste from uncoordinated routing in Boston, New York, London). But
nobody has used it as a generative mechanism for procedural layout.

Key research:
- Monderer & Shapley (1996) — foundational potential games paper
- Youn et al. (2008) — "Price of Anarchy in Transportation Networks" — measured PoA on real cities
- Fotakis (2005) — "Online and incremental algorithms for facility location" — competitive ratio Θ(log n / log log n) for online,
O(1) for incremental
- arXiv:2005.08391 — "Online Multi-Commodity Facility Location" — shows heterogeneity directly influences competitive ratio
- arXiv:2601.08642 — "Improving Equilibria in Urban Neighbourhood Games" — formalizes urban development as a game with
equilibrium analysis

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


Framing 2: Boltzmann Sampling at Target Temperature

The layout is a physical system with an energy function (lower energy = more optimized). Instead of minimizing energy (simulated
annealing), we sample from the Boltzmann distribution at a specific temperature T.

At temperature T, configurations appear with probability ∝ exp(-E/kT). This is NOT "optimal + noise." Boltzmann samples at a
given temperature have internal thermodynamic consistency — correlations, structure, and local order that pure perturbation doesn
't produce. A layout sampled at T=0.3 looks fundamentally different from one optimized to score 0.7 and then randomly degraded.

Temperature IS the tuning parameter:
- T → 0: ground state (perfectly planned city)
- Low T: mostly ordered with thermal fluctuations (established city)
- Medium T: significant disorder but recognizable structure (organic settlement)
- High T: near-random (ruins, chaos)

Key research:
- Metropolis et al. (1953) — original Monte Carlo sampling
- arXiv:2409.18126 — "Boltzmann Sampling by Diabatic Quantum Annealing" — modern approaches to sampling at target energy
- The entire VLSI simulated annealing literature (TimberWolf, etc.) — but inverted: they cool to find minimum, we hold at target
temperature

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


Framing 3: Incremental Constrained Morphogenesis

The layout grows like an organism. Start with a seed (founding structures). At each step, a new structure is added following
local growth rules. Each addition constrains future growth. The final form emerges from the accumulation of local decisions.

This connects to:
- Reaction-diffusion models (Turing patterns)
- L-systems (Lindenmayer) for organic growth
- The "Miskolc Method" (2024) — models city evolution with recursive algorithms using simulated morphogenesis
- PLOS ONE (2015) — "On the Morphology of a Growing City" — merges equilibrium analysis with agent-based growth simulation
- Springer (2019) — "Emergent Urban Morphologies" — shows city-like agglomerates emerge from agent-based housing models

The growth rules ARE the algorithm. Different rule sets produce different morphologies. The "optimization level" emerges from how
much each growth step respects global vs. local objectives.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


Where I think the real mechanism lives:

Framing 1 (potential game) gives us the mathematical foundation and convergence guarantees. Framing 2 (Boltzmann) gives us the
sampling mechanism — a principled way to produce layouts at a specific "energy level" with internal consistency. Framing 3 (
morphogenesis) gives us the incremental growth model that produces temporal layering.

The core mechanism might be at the intersection: a spatial potential game where the equilibrium is sampled at a target temperature
via Boltzmann dynamics. The potential function encodes the agent utilities. The temperature controls how far from equilibrium
the layout sits. The incremental ordering produces temporal structure.

This avoids the three-body problem because:
- Potential games guarantee convergence (no chaotic orbits)
- Boltzmann sampling at fixed temperature has a well-defined stationary distribution
- We're not predicting trajectories — we're sampling from an equilibrium distribution

The novel contribution would be: formulating spatial layout generation as a potential game and using Boltzmann sampling at a
target temperature to produce layouts with a controllable, principled optimization gap — where the Price of Anarchy emerges from
the game structure rather than being imposed post-hoc.
