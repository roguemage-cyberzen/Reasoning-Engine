# 1. Theoretical Bridge: Grounding PHP in ZFC Set Theory

In standard proof complexity, $\text{PHP}_n^{m}$ (asserting that $m$ pigeons placed into $n$ holes with $m > n$ 
requires at least one hole to contain more than one pigeon) serves as the canonical hard benchmark for 
resolution-based automated theorem provers.

```mermaid
flowchart LR
    subgraph ZFC["ZFC Domain"]
        direction TB
        P["P (m Pigeons)"]
        H["H (n Holes)"]
        Card["Card(P) > Card(H)"]
    end

    subgraph FOP["First-Order Invariants"]
        direction TB
        Rel["f ⊆ P × H<br/>Totality: ∀i ∃j f(i, j)<br/>Injectivity: ∀i₁≠i₂, ¬(f(i₁, j) ∧ f(i₂, j))"]
    end

    subgraph CNF["Propositional CNF"]
        direction TB
        Bool["p_{i,j} ∈ {0, 1}<br/>Haken Barrier: Ω(2^{cn}) Resolution Steps<br/>Structural Collapse via Orbit Breaking"]
    end

    ZFC -->|Cartesian Product| FOP
    FOP -->|Encoder| CNF
```
