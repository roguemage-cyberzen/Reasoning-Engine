# The Reasoning Engine: Structural Complexity Reduction Framework for Sudoku

A high-performance deductive framework and benchmark suite designed to solve NP-complete and exact-cover problems by prioritizing structural complexity reduction over brute-force combinatorial search.

1. Executive Overview & Core Architecture

Standard SAT, SMT, and depth-first search (DFS) solvers scale poorly against dense combinatorial spaces because raw search complexity asymptotically approaches $\Omega(2^n)$.

The core architectural thesis of the Reasoning Engine is:
*  Do not search faster; search less.

Rather than treating a problem as a black-box collection of boolean variables, the engine processes inputs through a 3-tier structural pipeline:

```mermaid
flowchart TD
    Input["Input Problem Instance (Matrix, Graph, or Grid)"]

    subgraph Tier1["Tier 1: Structural Analysis & Invariants"]
        direction TB
        T1_Desc["• Macro-Topology & Pre-Screening Validation<br/>• Instant Refutation Certificates (O(1) to O(V+E))"]
    end

    subgraph Tier2["Tier 2: Relational DSL & Reduction"]
        direction TB
        T2_Desc["• Candidate Domain Sifting (AC-3 / Forward Checking)<br/>• Automorphism Orbit Collapsing (Symmetry Breaking)<br/>• Compact Canonical CNF Compilation"]
    end

    subgraph Tier3["Tier 3: Deductive CDCL Core"]
        direction TB
        T3_Desc["• Event-Driven Two-Watched Literals (2WL)<br/>• 1UIP Conflict-Driven Clause Learning<br/>• Dynamic VSIDS Activity Decay & Non-Chronological Backjumping"]
    end

    Out_UNSAT["Instant Macro UNSAT Proof"]
    Out_Model["Exact Satisfying Model Assignment"]

    Input --> Tier1
    Tier1 -->|"Global Invariant Violated"| Out_UNSAT
    Tier1 -->|"Feasible Topology"| Tier2
    Tier2 -->|"Reduced Clause & Variable Database"| Tier3
    Tier3 --> Out_Model
```



