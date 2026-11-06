Solving the Traveling Salesperson Problem (TSP): Structural Complexity Reduction

The Traveling Salesperson Problem (TSP) is the canonical NP-hard challenge of combinatorial optimization. Classical approaches model TSP using Integer Linear 
Programming (ILP) with lazy cutting-plane sub-tour generation (Dantzig-Fulkerson-Johnson / DFJ) or Branch-and-Bound over continuous LP relaxations. When edge 
densities increase or tight cost bottlenecks are introduced, linear relaxations weaken, forcing solvers into heavy branch-and-cut exploration.

Below is the complete engineering walkthrough, architectural flow, and standalone benchmark suite demonstrating 
how the 3-Tier Reasoning Engine solves bounded and optimal TSP instances by prioritizing topological invariants, 
discrete space-time permutation grids, and dihedral orbit collapsing over brute-force search.
