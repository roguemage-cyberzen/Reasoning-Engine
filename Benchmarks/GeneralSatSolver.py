import itertools

def general_sat_solver(formula, num_variables):
    """
    Solves any SAT instance given in Conjunctive Normal Form (CNF).

    Args:
        formula: A list of lists of integers representing the CNF formula.
                 e.g., [[1, -2], [2, 3]] for (A or not B) and (B or C).
        num_variables: The total number of variables (e.g., 3 for A, B, C).

    Returns:
        A dictionary with the satisfying assignment if found, otherwise None.
    """
    # Create a list of variables from 1 to num_variables
    variables = range(1, num_variables + 1)
    
    # Generate all 2^n possible assignments of True/False
    for assignment in itertools.product([False, True], repeat=num_variables):
        
        # This flag will track if the current assignment satisfies all clauses
        formula_is_true = True
        
        # Check every clause in the formula
        for clause in formula:
            clause_is_true = False
            # Check every literal in the clause
            for literal in clause:
                # Get the variable number (e.g., -2 -> 2)
                var_index = abs(literal) - 1 
                
                # Check if the literal is satisfied by the assignment
                # Case 1: Positive literal (e.g., 2 for B)
                if literal > 0 and assignment[var_index] == True:
                    clause_is_true = True
                    break # This clause is satisfied, no need to check other literals
                # Case 2: Negative literal (e.g., -2 for not B)
                elif literal < 0 and assignment[var_index] == False:
                    clause_is_true = True
                    break # This clause is satisfied
            
            # If after checking all literals, the clause is still false,
            # then this entire assignment is wrong.
            if not clause_is_true:
                formula_is_true = False
                break # No need to check other clauses
        
        # If after checking all clauses, the formula is still true, we found a solution!
        if formula_is_true:
            # Format the solution nicely into a dictionary
            solution = {f"Var {i+1}": assignment[i] for i in range(num_variables)}
            print("\n--- Solution Found! ---")
            return solution
            
    print("\n--- No solution found. ---")
    return None

# --- Running Our Example ---

# Let's define our party problem: (A or B) and (not A or C) and (not B or not C)
# With A=1, B=2, C=3
party_formula = [[1, 2], [-1, 3], [-2, -3]]
number_of_vars = 3

# Call the general solver
solution = general_sat_solver(party_formula, number_of_vars)

if solution:
    print(solution)
