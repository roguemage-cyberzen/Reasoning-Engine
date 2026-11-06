#  Solving the Traveling Salesperson Problem (TSP): Structural Complexity Reduction

The Traveling Salesperson Problem (TSP) is the canonical NP-hard challenge of combinatorial optimization. Classical approaches model TSP using Integer Linear 
Programming (ILP) with lazy cutting-plane sub-tour generation (Dantzig-Fulkerson-Johnson / DFJ) or Branch-and-Bound over continuous LP relaxations. When edge 
densities increase or tight cost bottlenecks are introduced, linear relaxations weaken, forcing solvers into heavy branch-and-cut exploration.

Below is the complete engineering walkthrough, architectural flow, and standalone benchmark suite demonstrating 
how the 3-Tier Reasoning Engine solves bounded and optimal TSP instances by prioritizing topological invariants, 
discrete space-time permutation grids, and dihedral orbit collapsing over brute-force search.

1. Executive Overview & Pipeline Architecture
A naive search for an $n$-city Hamiltonian cycle confronts an $(n-1)! / 2$ search space. Furthermore, unguided propositional solvers face severe factorial symmetry: a single valid circular tour of $n$ cities possesses $2n$ identical representations under the Dihedral group $D_n$ ($n$ cyclic time shifts and 2 traversal directions).The Reasoning Engine structures TSP through a 3-tier pipeline:

```mermaid
flowchart TD
    Input["Input: N-City Distance Matrix D_ij & Candidate Cost Bound W"]

    subgraph Tier1["Tier 1: Structural Analysis (Topological Invariants)"]
        direction TB
        T1_Desc["• Filter Edge Subgraph: E_W = {(u, v) : D_uv ≤ W}<br/>• Degree Check: deg(v) ≥ 2 ∀ v ∈ V<br/>• Global Connectivity: Single Connected Component (BFS in O(V + E))<br/>• If disconnected or deg < 2: Instant UNSAT Certificate"]
    end

    subgraph Tier2["Tier 2: Relational DSL Compiler (Space-Time Embedding)"]
        direction TB
        T2_Desc["• Space-Time Grid: Exactly-1 City per Step & Exactly-1 Step per City<br/>• Dihedral Orbit Pinning: Fix City 0 at Step 0 (Breaks n-fold cyclic shifts)<br/>• Directional Ordering: City at Step 1 < City at Step n-1 (Breaks 2-fold reversal)<br/>• Metric Boundary Filter: Forbid (u at t ∧ v at t+1) for D_uv > W"]
    end

    subgraph Tier3["Tier 3: Deductive Execution Core (CDCL / 2WL)"]
        direction TB
        T3_Desc["• 2-Watched Literals (2WL) Event-Driven Incompatibility Propagation<br/>• 1UIP Conflict-Driven Clause Learning & Non-Chronological Backjumping<br/>• Binary Search Sweep over Edge Weights to Converge to Exact Optimum"]
    end

    Out_UNSAT["Instant Topological Proof (UNSAT in O(V + E))"]
    Out_Tour["Exact Hamiltonian Tour & Minimax Bottleneck Proof"]

    Input --> Tier1
    Tier1 -->|"Isolated Node / Disconnected Cut"| Out_UNSAT
    Tier1 -->|"Feasible Connected Graph"| Tier2
    Tier2 -->|"Compact CNF + Implication Graph"| Tier3
    Tier3 --> Out_Tour
```
