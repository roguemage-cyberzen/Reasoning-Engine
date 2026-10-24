# Designing a Structural Reasoning Engine: Systems Architecture Guide for 9x9 Sudoku

```mermaid
flowchart TD
    Input["9x9 Grid Input & Clues (Givens)"]

    subgraph L1["1. Structural Layer"]
        direction TB
        L1_Desc["Grid Integrity & Conflict Pre-Screening"]
    end

    subgraph L2["2. Relational Compiler (DSL)"]
        direction TB
        L2_Desc["3D Bit-Cube Exact-1 Embedding & Clue Unit Injections"]
    end

    subgraph L3["3. Deductive Execution Core"]
        direction TB
        L3_Desc["2WL Cascades (Naked Singles) + 1UIP Conflict Resolution"]
    end

    Out1["Instant UNSAT Certificate: Clue Collision"]
    Out2["Axiomatic Unit Cluses [X_{r,c,v}]"]
    Out3["Exact Latin-Square Completion"]

    Input --> L1
    L1 -->|"Duplicate Clue in Row/Col/Box"| Out1
    L1 -->|"(Valid Board Configuration)"| L2
    L2 -->|"Direct Fact Injection"| Out2
    L2 -->|"Compact Exact-1 CNF System"| L3
    L3 --> Out3
```
##    1. Core Engineering Philosophy: The 3D Bit-Cube Exact-Cover Model

Standard backtracking solvers treat 9x9 Sudoku as a trial-and-error recursive search, evaluating empty cells chronologically and rewinding when a constraint is violated. This approach scales poorly on adversarial instances ("evil" or minimal 17-clue puzzles) where depth-first paths dead-end after hundreds of speculative branch decisions.

The Reasoning Engine reformulates Sudoku as a 4-Way Dual Orthogonal Exact-Cover System:
*    The 3D Bit-Cube: Rather than storing integers $1..9$ in a 2D array, the state space is modeled as a 3D coordinate system of dimensions $9 \times 9 \times 9 = 729$ binary indicator variables.
*    Structural Intersection: Every valid state must simultaneously satisfy four orthogonal partition invariants: cell occupancy, row Latin coverage, column Latin coverage, and 3x3 block subgrid coverage.
*    Deterministic Unit Cascades: Injecting board clues directly as boolean unit facts triggers a deterministic propagation wave via Two-Watched Literals (2WL), resolving simple and intermediate puzzles with zero search decisions.

