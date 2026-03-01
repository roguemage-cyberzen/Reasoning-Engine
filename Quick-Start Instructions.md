## 💻 Quick Start Instructions
1. System Preparation: Ensure Python 3.10+ and pip are installed:

``` Bash
sudo apt update
sudo apt install python3 python3-pip python3-venv -y
```
2. Repository Setup: Create a project directory and initialize a virtual environment to prevent dependency conflicts:

``` Bash
mkdir reasoning_engine_project && cd reasoning_engine_project
python3 -m venv venv
source venv/bin/activate
```

3. Library Deployment: Save the library code as ReasoningKit.py and your implementation scripts (e.g., php_experiment.py) in the same directory. No external heavy-weight dependencies (like PySAT) are required, as this is a "from-scratch" implementation.

## Architecture Review: The Semantic Compiler Logic Flow
The Reasoning Engine is structured as a hierarchical pipeline that progressively transforms human-readable constraints into a "pruned" deductive search.
Layer	Component	Function	

| Layer          | Component          | Function                                                                                                                                                                                                                                              |
|----------------|--------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| I. Conceptual  | The Case           | Defines raw problem parameters (e.g., pigeons, Sudoku clues, or graph edges).                                                                                                                                                                         |
| II. Semantic   | DSL Compiler       | "Maps relational objects (Node AColor Blue) to integer IDs via add_exactly_one(). Tags clauses with ""Structural Metadata"" for symmetry reduction."                                                                                                |  |
| III. Reduction | Structural Sifting | Builds an adjacency matrix for Topological Filtering and injects ""Canonical Constraints"" for Symmetry Breaking.                                                                                                                                     |
| IV. Execution  | CDCL Core          | Executes the O(2n−s) search. Uses 1UIP Conflict Analysis to learn new rules and dynamically increase the Structural Factor (s).                                                                                                                   |

## Implementation Walk-Through: $PHP_n$, Sudoku, and $K_6$

#### Implementation A: Pigeonhole Principle (Symmetry Breaking)
This script demonstrates how to defeat factorial growth by "folding" the search space.
* Mapping: Maps $N \times M$ variables.
* Symmetry Breaking: Uses a Canonical Injection (e.g., dsl.formula.append([dsl.get_var("p", 0, "h", 0)])) to force the first pigeon into the first hole.
* Impact: Collapses the search tree, as every other starting position is mathematically identical.

#### Implementation B: Sudoku $9 \times 9$ (Deduction Cascades)
The Sudoku solver treats the grid as a 3D Bit-Cube to trigger ripple-effect logic.
* 3D Mapping: var(r, c, v) represents Row, Column, and Value.
* Unit Injection: Pre-filled numbers are injected as "Unit Clauses".
* Cascade: The _unit_propagate function triggers a cascade across the 729 variables, solving the majority of the puzzle before the first branching decision.

#### Implementation C: $K_6$ Coloring (Topological Proof)
This highlights the power of the Semantic Layer to bypass search entirely.
* Graph Scan: The SemanticAnalyzer builds a dependency graph of the 6 nodes and detects a complete connectivity.
* The Shortcut: Compares clique_size (6) to available colors (5).
* Proof: Because $6 > 5$, the engine returns an Instant UNSAT, proving impossibility without ever starting the search core.

