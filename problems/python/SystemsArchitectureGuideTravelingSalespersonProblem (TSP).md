#  Designing a Structural Reasoning Engine: Systems Architecture Guide for the Traveling Salesperson Problem (TSP)

```mermaid
flowchart TD
    Input["Distance Matrix D_ij & Candidate Cost Bound W"]

    subgraph L1["1. Structural Layer"]
        direction TB
        L1_Desc["Connectivity & Degree Bottleneck Pre-Screening"]
    end

    subgraph L2["2. Relational Compiler (DSL)"]
        direction TB
        L2_Desc["Space-Time Permutation Grid, Metric Filters & Dihedral Orbit Collapsing"]
    end

    subgraph L3["3. Deductive Execution Core"]
        direction TB
        L3_Desc["2WL Edge Exclusion Cascades + 1UIP Path Pruning"]
    end

    Out1["Instant UNSAT Certificate: Disconnected Cut-Set / Infeasible Node Degree"]
    Out2["Symmetry Pins: City 0 -> Time 0 & Direction Anchor"]
    Out3["Exact Hamiltonian Cycle Model"]

    Input --> L1
    L1 -->|"Degree < 2 or Isolated Cut"| Out1
    L1 -->|"(Connected Component: Pass to Core)"| L2
    L2 -->|"Break Dihedral Group D_n"| Out2
    L2 -->|"Compact CNF + Implication Graph"| L3
    L3 --> Out3
```











|  |  |  |
| :---         |     :---:      |          ---: |
|  |  |  |
|  |  |  |
|  |  |  |
|  |  |  |
