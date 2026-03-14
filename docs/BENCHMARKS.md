# Ogun Performance Benchmarks

> Last updated: 2026-03-14
> Hardware: Results are relative; run `cargo test --release --test perf -- --ignored --nocapture` for absolute numbers on your machine.

## Optimization History

Starting from the naive O(n²·S) implementation, 7 optimizations were applied sequentially. Each was measured independently before proceeding to the next.

### Baseline (naive implementation)

The naive algorithm evaluates every grid position for every node, scanning all edges and placed nodes per evaluation. Profiling showed **utility evaluation consumes 85–100% of runtime**, with routing at 1–4% and Boltzmann sampling negligible.

| Scenario | Nodes | Grid | Time |
|---|---|---|---|
| medium | 50 | 100×100 | 139ms |
| saltglass | 100 | 250×110 | 1.62s |
| large | 200 | 250×250 | 12.2s |
| stress | 500 | 500×500 | 280s |

### Optimization results

| # | Optimization | Technique | 50n | 100n | 200n | 500n |
|---|---|---|---|---|---|---|
| — | Baseline | — | 139ms | 1.62s | 12.2s | 280s |
| 1 | Pre-built adjacency list | O(degree) neighbor lookup instead of O(\|E\|) linear scan | 108ms | 1.02s | 7.75s | — |
| 2 | Vec\<Option\<Pos\>\> positions | Replace HashMap with indexed Vec for O(1) cache-friendly access | 95ms | 904ms | 6.55s | — |
| 3 | Fused overlap + repulsion | Single pass over placed nodes instead of two | 74ms | 698ms | 5.01s | — |
| 4 | Buffer reuse | Pre-allocated candidates Vec, flat blocked vec, reusable RouteBuf, allocation-free Boltzmann sampling | 64ms | 549ms | 4.40s | 98s |
| 5 | SoA placed nodes | Contiguous f32 arrays (xs, ys, radii) for cache locality; eliminates Option discriminant scanning | 56ms | 518ms | 4.26s | 99s |
| 6 | Rayon parallel utility eval | Parallel map-reduce over grid positions with adaptive threshold (sequential when grid×placed < 500k) | 62ms | 341ms | 1.39s | 13.0s |
| 7 | Spatial repulsion cutoff | Skip repulsion for nodes where contribution < 0.01 (dist² > k×100) | 44ms | 323ms | 1.28s | 11.3s |

### Cumulative speedup

| Scenario | Baseline | Final | Speedup |
|---|---|---|---|
| 50 nodes / 100×100 | 139ms | 44ms | **3.2×** |
| 100 nodes / 250×110 | 1.62s | 323ms | **5.0×** |
| 200 nodes / 250×250 | 12.2s | 1.28s | **9.5×** |
| 500 nodes / 500×500 | 280s | 11.3s | **24.8×** |

### Analysis

**Biggest individual wins:**
- Rayon parallel (opt 6): 7.6× on 500n — the S position evaluations per node are embarrassingly parallel
- Buffer reuse (opt 4): 2.9× on 500n — eliminated ~400MB of per-iteration allocation churn
- Adjacency list (opt 1): 1.6× — `edges_of()` was scanning all edges per position per node

**Diminishing returns:**
- SoA (opt 5): ~1.0× on 500n — the early-return overlap check prevents SIMD auto-vectorization; benefit is cache locality only
- Spatial cutoff (opt 7): 1.15× on 500n — repulsion division is cheap relative to the overlap check branch

**Scaling behavior:**
- Speedup increases with problem size (3.2× at 50n → 24.8× at 500n) because the O(n²·S) inner loop dominates more at scale
- Rayon's benefit scales with core count; the 500n result reflects the test machine's thread pool

## Running benchmarks

```bash
# All perf tests (includes 500n stress test — may take minutes on naive builds)
cargo test --release --test perf -- --ignored --nocapture

# Specific scenario
cargo test --release --test perf -- --ignored --exact perf_medium_50n_100x100 --nocapture

# Available scenarios
# perf_medium_50n_100x100      — 50 nodes, 100×100 grid
# perf_saltglass_100n_250x110  — 100 nodes, 250×110 grid (saltglass-steppe target scale)
# perf_large_200n_250x250      — 200 nodes, 250×250 grid
# perf_stress_500n_500x500     — 500 nodes, 500×500 grid
# perf_beta_sweep              — 100 nodes at β = 0.1, 1, 2, 5, 10, 50
# perf_dense_100n_6edges       — 100 nodes, 6 edges/node (high connectivity)
# perf_obstacle_heavy          — 100 nodes, 100 obstacles
```

## Remaining optimization targets

Documented in `docs/research/OGUN_ALGORITHM_RESEARCH.md`:

- **Weighted segment tree** for O(log S) Boltzmann sampling (currently O(S))
- **HPA\* routing** to replace Lee BFS (currently O(S) per route)
- **Separable kernels** for O(1) amortized repulsion via prefix sums
- **PathFinder negotiated routing** for multi-edge congestion resolution
