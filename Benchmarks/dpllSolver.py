import time
import math
import copy

# ==============================================================================
#  Our New Intelligent Solver Engine (DPLL Algorithm)
# ==============================================================================

def dpll_solver(formula, num_vars):
    """
    The main entry point for the DPLL solver. It initializes the assignment
    and calls the recursive solving function.
    """
    # Start with an empty assignment (all variables are unassigned)
    assignment = {}
    
    # Call the recursive solver
    result = dpll_recursive(formula, assignment)
    
    if result is None:
        return None # Unsatisfiable
    else:
        # The recursive solver might only return a partial assignment.
        # Fill in any remaining unassigned variables (can be True or False).
        all_vars = set(range(1, num_vars + 1))
        assigned_vars = {abs(k) for k in result.keys()}
        unassigned_vars = all_vars - assigned_vars
        for var in unassigned_vars:
            result[var] = True # Assign a default value
        return result

def dpll_recursive(formula, assignment):
    """
    The recursive heart of the DPLL algorithm.
    """
    # --- 1. Unit Propagation (The "Forced Move") ---
    # Find all clauses with only one unassigned literal
    unit_clauses = [c for c in formula if len(c) == 1]
    while unit_clauses:
        unit_literal = unit_clauses[0][0]
        
        # Add the forced assignment
        var = abs(unit_literal)
        val = True if unit_literal > 0 else False
        if var in assignment and assignment[var] != val:
            return None # Contradiction found!
        assignment[var] = val
        
        # Simplify the formula
        new_formula = []
        for clause in formula:
            # If the unit literal is in the clause, the clause is satisfied
            if unit_literal in clause:
                continue
            # If the negation of the literal is in the clause, remove it
            new_clause = [l for l in clause if l != -unit_literal]
            # If the clause becomes empty, we have a contradiction
            if not new_clause:
                return None # Backtrack
            new_formula.append(new_clause)
        
        formula = new_formula
        unit_clauses = [c for c in formula if len(c) == 1]

    # --- 2. Check for Success or Failure ---
    if not formula:
        return assignment # Success! All clauses satisfied.
        
    # --- 3. Guess and Recurse ---
    # Pick the first unassigned variable to make a guess
    all_vars_in_formula = {abs(l) for c in formula for l in c}
    unassigned_vars = all_vars_in_formula - set(assignment.keys())
    if not unassigned_vars:
        return assignment # Should be caught by the "if not formula" check, but good for safety
        
    variable_to_guess = sorted(list(unassigned_vars))[0]

    # Try guessing the variable is TRUE
    solution = dpll_recursive(formula + [[variable_to_guess]], assignment.copy())
    if solution is not None:
        return solution
        
    # If that failed, backtrack and try guessing FALSE
    solution = dpll_recursive(formula + [[-variable_to_guess]], assignment.copy())
    return solution

# ==============================================================================
#  Helper Function to Generate the Pigeonhole Problem in CNF
# ==============================================================================

def generate_pigeonhole_cnf(pigeons, holes):
    """Generates the SAT formula (CNF) for the Pigeonhole Problem."""
    def var_num(p, h):
        return (p - 1) * holes + h
    
    num_total_variables = pigeons * holes
    formula = []

    # Rule A: Every pigeon must be in at least one hole.
    for p in range(1, pigeons + 1):
        formula.append([var_num(p, h) for h in range(1, holes + 1)])

    # Rule B: No two pigeons can be in the same hole.
    for h in range(1, holes + 1):
        for p1 in range(1, pigeons + 1):
            for p2 in range(p1 + 1, pigeons + 1):
                formula.append([-var_num(p1, h), -var_num(p2, h)])
        
    return formula, num_total_variables

# ==============================================================================
#  Main Program
# ==============================================================================

if __name__ == "__main__":
    # --- You can change these numbers to experiment! ---
    # The brute-force solver struggled with 8x8. Let's see how DPLL does.
    NUM_PIGEONS = 8
    NUM_HOLES = 8
    
    print(f"--- Solving Pigeonhole Problem for {NUM_PIGEONS}p, {NUM_HOLES}h using DPLL ---")
    
    formula, num_vars = generate_pigeonhole_cnf(NUM_PIGEONS, NUM_HOLES)
    print(f"Problem has {num_vars} variables and {len(formula)} clauses.")

    start_time = time.time()
    
    # Call the new DPLL solver
    solution = dpll_solver(formula, num_vars)
    
    end_time = time.time()
    duration = end_time - start_time
    
    print(f"\n--- Solver Finished ---")
    print(f"Time taken: {duration:.6f} seconds")

    if solution:
        print("Result: Formula is SATISFIABLE.")
        # Optional: Decode the solution for readability
        decoded_solution = {}
        for var, is_true in sorted(solution.items()):
            if is_true:
                p = (var - 1) // NUM_HOLES + 1
                h = (var - 1) % NUM_HOLES + 1
                decoded_solution[f"Pigeon_{p}"] = f"Hole_{h}"
        print("Decoded Solution (one of many possibilities):", decoded_solution)
    else:
        print("Result: Formula is UNSATISFIABLE.")
