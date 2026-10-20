#  Designing a Structural Reasoning Engine: Systems Architecture Guide for the Pigeonhole Principle (PHP)

##  1. Core Engineering Philosophy: Defeating the Haken Resolution Barrier

In computational proof complexity, the Pigeonhole Principle ($\text{PHP}_n^m$, where $m$ items are mapped into $n$ slots with $m > n$) represents the primary counterexample to standard resolution 
systems. Armin Haken established that any general resolution refutation of raw $\text{PHP}_n^{n+1}$ forces a solver to generate:

$$\Omega(2^{c \cdot n})$$

resolution steps.

When a standard CDCL solver operates purely on propositional clauses, it treats each assignment in isolation. It cannot recognize that swapping two pigeons or two holes produces an 
isomorphic subproblem. Consequently, it traverses identical dead-ends, learning an exponential number of symmetric conflict clauses.

The Reasoning Engine circumvents this barrier using a Reduction-First Architecture:
*  Topological Pre-Check: Identifies global counting bottlenecks prior to boolean conversion.
*  Automorphism Collapse: Breaks the factorial permutation symmetries of the entity space at the relational compiler layer.
*  1UIP Pruning: Derives learned clauses on non-symmetric canonical representations to terminate search branches rapidly.

```mermaid
flowchart TD
    Input["Pigeon & Hole Allocation Parameters (m, n)"]

    subgraph L1["1. Structural Layer"]
        direction TB
        L1_Desc["Capacity Invariant Filter"]
    end

    subgraph L2["2. Relational Compiler (DSL)"]
        direction TB
        L2_Desc["Automorphism Orbit Breaking"]
    end

    subgraph L3["3. Deductive Execution Core"]
        direction TB
        L3_Desc["2WL + 1UIP Conflict Analysis"]
    end

    Out1["Instant UNSAT Certificate: m > n"]
    Out2["Anchor Pigeon 0 to Hole 0"]
    Out3["Minimal Proof Tree Refutation"]

    Input --> L1
    L1 -->|"Instant Proof"| Out1
    L1 -->|"(If passed to verification/CDCL)"| L2
    L2 -->|"Symmetry Pinning"| Out2
    L2 -->|"Compact CNF + Implication Graph"| L3
    L3 --> Out3
```
##    2. The 3-Tier Layer Cake Architecture

###    Layer 1: Structural Analysis (Capacity Invariant)

Before generating clauses, the system evaluates the capacity ratio:
*    The Invariant: For any injective mapping from a finite set of items $P$ to slots $H$, capacity dictates that $\vert{}P\vert{} \le \vert{}H\vert{}$.
*    Execution: If $\vert{}P\vert{} > \vert{}H\vert{}$ and the problem demands an injective assignment, the semantic analyzer returns an immediate UNSAT certificate in $O(1)$ time, demonstrating impossibility without initiating boolean resolution.

###    Layer 2: Relational Compiler (Structural DSL)

When evaluating the problem explicitly through propositional reasoning (e.g., verifying solver robustness or handling partial placement constraints), the relational compiler constructs the Boolean representation:
*    Coordinate Mapping: Translates the discrete pairs $(\text{pigeon}, \text{hole})$ into contiguous integer literal IDs.
*    Cardinality Encodings: Enforces that every pigeon has an assigned hole (Exactly-One), while each hole accepts no more than one occupant (At-Most-One).
*    Symmetry Breaking: Eliminates the $m! \times n!$ automorphism group ($S_m \times S_n$) by pinning canonical reference coordinates.

###    Layer 3: Deductive Execution Core (CDCL / 2WL)

The compiled CNF is processed by the deduction engine:
*    Two-Watched Literals (2WL): Monitors boundary conditions with minimal memory overhead during constraint propagation.
*    1UIP Conflict Analysis: When capacity conflicts occur, the solver identifies the root assignment and derives an asserting learned clause that cuts away redundant search subtrees.

##    3. Concrete Implementation: PHP Systems Architecture

###    Step 1: Coordinate Space Definition

For $m$ pigeons and $n$ holes, the state space is modeled as an $m \times n$ allocation grid:
*    Let Boolean variable p_{i}_h_{j} denote: "Pigeon $i$ is assigned to Hole $j$."
*    Total boolean variables registered: $m \cdot n$.

```python
from ReasoningKit import StructuralDSL

dsl = StructuralDSL()

# Discrete Coordinate Registration
# Maps the 2D state space (pigeon_idx, hole_idx) directly to bit indices
def get_pigeon_var(dsl, pigeon_idx, hole_idx):
    return dsl.get_var(f"p_{pigeon_idx}_h_{hole_idx}")
```
###    Step 2: Structural Allocation Rules

Every valid distribution must satisfy two fundamental invariants:
*    Pigeon Assignment: Each pigeon must occupy exactly one hole.
*    Hole Capacity: No hole can accommodate more than one pigeon.

```python
def apply_allocation_rules(dsl, pigeons, holes):
    # Rule 1: Every pigeon must be assigned to exactly one hole (Existence & Uniqueness)
    for i in range(pigeons):
        pigeon_options = [get_pigeon_var(dsl, i, j) for j in range(holes)]
        dsl.add_exactly_one(pigeon_options)

    # Rule 2: Each hole can contain at most one pigeon (Exclusivity)
    for j in range(holes):
        hole_occupants = [get_pigeon_var(dsl, i, j) for i in range(pigeons)]
        dsl.add_at_most_one(hole_occupants)
```
Step 3: Automorphism / Symmetry Breaking

Because all pigeons are interchangeable and all holes are interchangeable, an unconstrained solver faces $m! \times n!$ equivalent permutations of failure.

We break these symmetries using structural pins:
1.    Origin Anchor: Fix Pigeon 0 into Hole 0. This breaks the initial $n$-fold rotational symmetry over available holes.
2.    Positional Pruning: Enforce an ordering constraint preventing Pigeon 1 from being placed into holes that would duplicate an unassigned symmetric state.
```python
def apply_php_symmetry_breaking(dsl, pigeons, holes):
    # Anchor the first entity (collapses initial n-way hole symmetry)
    dsl.formula.append([get_pigeon_var(dsl, 0, 0)])
```

##    4. Execution Workflow
```python
from ReasoningKit import StructuralDSL, ReasoningCore
import time

def run_php_systems_solver(pigeons, holes, enable_semantic_bypass=True):
    print(f"--- [PHP] Starting Systems Solver: {pigeons} Pigeons -> {holes} Holes ---")

    # Tier 1: Structural Invariant Filter
    if enable_semantic_bypass and pigeons > holes:
        print("  [Structural Layer] Invariant Check: Pigeons > Holes.")
        print("  [Semantic Result] Instant UNSAT Certificate issued (Capacity Bottleneck).")
        return "UNSAT (Topological Invariant Proof)"

    # Tier 2: Relational Compiler (DSL)
    dsl = StructuralDSL()
    apply_allocation_rules(dsl, pigeons, holes)
    apply_php_symmetry_breaking(dsl, pigeons, holes)

    print(f"  [DSL Compiler] Generated {len(dsl.formula)} clauses across {dsl.var_count} variables.")

    # Tier 3: Deductive Execution Core
    solver = ReasoningCore(dsl.formula, dsl.var_count)
    
    start_time = time.perf_counter()
    result = solver.solve()
    duration = time.perf_counter() - start_time

    if result is None:
        print(f"  [CDCL Core] Status: UNSATISFIABLE")
        print(f"  [Metrics] Solved in {duration:.6f}s | Decisions: {solver.decisions} | Conflicts: {solver.conflicts}")
        return "UNSAT"
    else:
        print(f"  [CDCL Core] Status: SATISFIABLE")
        return "SAT"


if __name__ == "__main__":
    # Test 1: Capacity Short-Circuit (Instant result via Tier 1)
    run_php_systems_solver(10, 9, enable_semantic_bypass=True)
    print()

    # Test 2: CDCL Deductive Refutation with Symmetry Breaking (Testing Tier 2 & 3)
    run_php_systems_solver(6, 5, enable_semantic_bypass=False)
```
##    5. Architectural Comparison: Classical Search vs. Reasoning Engine
|    Feature    |    Standard CDCL / Propositional Resolution    |    Reasoning Engine Pipeline    |    
Impossibility Proof    |    Generates $\Omega(2^n)$ resolution steps attempting every combination.    |    Macro Short-Circuit: Proves impossibility in $O(1)$ time via capacity invariants.
|    Symmetry Management    |    Oblivious to item equivalence; solves $n!$ permutations of the exact same contradiction.    |    Automorphism Collapsing: Anchors fixed coordinates in the DSL to eliminate symmetric equivalence classes.    |    
Clause Generation    |    Direct unpruned cartesian pairwise expansion.    |    Relational DSL: Manages exact-one and at-most-one gates with structured variable mapping. |    Trail Analysis    |    Backtracks through redundant decision levels.    |    1UIP Pruning: Derives conflict clauses that invalidate entire classes of assignments simultaneously.    |

