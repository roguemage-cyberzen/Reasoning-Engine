"""
Pattern: Assignment Symmetries
Use Case: Pigeonhole Principle (PHP)
Methodology: Relational Mapping + Symmetry Breaking
"""

from ReasoningKit import StructuralDSL, ReasoningCore
import time

def run_php_experiment(pigeons, holes):
    print(f"--- [PHP] Starting Experiment: {pigeons} pigeons, {holes} holes ---")
    
    # Initialize the Library Components
    dsl = StructuralDSL()
    
    # 1. Relational Mapping
    # Creating variables for "Pigeon i in Hole j"
    for i in range(pigeons):
        p_vars = [dsl.get_var("p", i, "h", j) for j in range(holes)]
        
        # Ensure every pigeon has exactly one hole
        dsl.add_exactly_one(p_vars)
        
    # 2. Symmetry Identification & Hole Conflict
    # No two pigeons can share a hole
    for j in range(holes):
        hole_occupants = [dsl.get_var("p", i, "h", j) for i in range(pigeons)]
        dsl.add_at_most_one(hole_occupants)

    # 3. Symmetry Breaking (Structural reduction)
    # Force Pigeon 1 into Hole 1 to remove N! permutations
    p1_h1 = dsl.get_var("p", 0, "h", 0)
    dsl.formula.append([p1_h1]) 

    # 4. Search Execution
    print(f"  Logic database size: {len(dsl.formula)} clauses.")
    solver = ReasoningCore(dsl.formula, dsl.var_count)
    
    start = time.time()
    result = solver.solve()
    duration = time.time() - start
    
    print(f"  Status: {'Satisfied' if result else 'Unsatisfiable'}")
    print(f"  Search Duration: {duration:.6f} seconds")

if __name__ == "__main__":
    run_php_experiment(10, 9)