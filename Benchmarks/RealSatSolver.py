import time
import itertools
import random
import matplotlib.pyplot as plt

# ==============================================================================
#  The SAT Solver Algorithm
# ==============================================================================
# This is the function you will work on improving.
# The goal is to make this faster than the current brute-force method.

def real_sat_solver(formula, num_variables):
    """
    Solves any SAT instance given in Conjunctive Normal Form (CNF).
    
    This current implementation uses a brute-force, exponential-time search.
    It checks every single possible assignment of truth values.

    Args:
        formula: A list of lists of integers representing the CNF formula.
                 e.g., [[1, -2], [2, 3]] for (A or not B) and (B or C).
        num_variables: The total number of variables in the formula.

    Returns:
        A dictionary with the satisfying assignment if found, otherwise None.
    """
    # Generate all 2^n possible assignments of True/False
    for assignment_tuple in itertools.product([False, True], repeat=num_variables):
        
        formula_is_satisfied = True
        
        # Check every clause in the formula against the current assignment
        for clause in formula:
            clause_is_satisfied = False
            for literal in clause:
                var_index = abs(literal) - 1  # Convert variable number (1-based) to list index (0-based)
                
                # Check if the literal's truth value matches the assignment
                if (literal > 0 and assignment_tuple[var_index] is True) or \
                   (literal < 0 and assignment_tuple[var_index] is False):
                    clause_is_satisfied = True
                    break  # This clause is true, move to the next clause
            
            # If a clause is not satisfied, this entire assignment is invalid
            if not clause_is_satisfied:
                formula_is_satisfied = False
                break # No need to check other clauses for this assignment
        
        # If we got through all the clauses and they were all satisfied, we found a solution!
        if formula_is_satisfied:
            # Format the solution into a readable dictionary and return it
            solution = {f"Var_{i+1}": assignment_tuple[i] for i in range(num_variables)}
            return solution
            
    # If the loops complete without finding a solution, the formula is unsatisfiable
    return None

# ==============================================================================
#  Helper Function to Generate Test Data
# ==============================================================================

def create_random_sat_problem(num_vars, num_clauses, clause_length=3):
    """
    Generates a random SAT problem in CNF format for testing.
    """
    formula = []
    variables = range(1, num_vars + 1)
    for _ in range(num_clauses):
        # Choose unique variables for the clause
        clause_vars = random.sample(variables, clause_length)
        # Randomly negate each one
        clause = [v if random.choice([True, False]) else -v for v in clause_vars]
        formula.append(clause)
    return formula

# ==============================================================================
#  Performance Testing Framework
# ==============================================================================
# This part of the code will remain the same, even if you change the solver.

def test_solver_performance():
    """
    Measures and plots the execution time of the real_sat_solver.
    """
    # Test for n variables, from 8 up to 20. 
    # Warning: Exponential growth means times get very long quickly!
    input_sizes = range(8, 20) 
    execution_times = []

    print("Starting performance test for real_sat_solver...")
    for n in input_sizes:
        # For a hard problem, the number of clauses is often around 4.2 times the number of variables
        num_clauses = int(n * 4.2)
        problem_formula = create_random_sat_problem(n, num_clauses)

        start_time = time.time()
        
        # --- THIS IS THE CALL TO YOUR SOLVER ---
        real_sat_solver(problem_formula, n)
        # -----------------------------------------
        
        end_time = time.time()
        
        duration = end_time - start_time
        execution_times.append(duration)
        print(f"Input size n={n}, Time={duration:.6f} seconds")

    # Plotting the results
    plt.figure(figsize=(12, 6))

    plt.subplot(1, 2, 1)
    plt.plot(input_sizes, execution_times, 'o-', label="Execution Time")
    plt.xlabel("Input Size (n variables)")
    plt.ylabel("Time (seconds)")
    plt.title("Performance (Linear Scale)")
    plt.grid(True)

    plt.subplot(1, 2, 2)
    plt.semilogy(input_sizes, execution_times, 'o-', color='red', label="Execution Time")
    plt.xlabel("Input Size (n variables)")
    plt.ylabel("Time (Logarithmic Scale)")
    plt.title("Performance (Semi-Log Scale)")
    plt.grid(True)

    plt.tight_layout()
    plt.show()

# --- Main entry point of the script ---
if __name__ == "__main__":
    # You can either test performance or solve a single problem
    
    # Option 1: Run the performance test
    test_solver_performance()
    
    # Option 2: Solve a single, specific problem
    # print("\nSolving a specific problem:")
    # my_formula = [[1, 2], [-1, 3], [-2, -3]] # (A or B) and (-A or C) and (-B or -C)
    # my_solution = real_sat_solver(my_formula, 3)
    # if my_solution:
    #     print("Solution found:", my_solution)
    # else:
    #     print("Formula is unsatisfiable.")
