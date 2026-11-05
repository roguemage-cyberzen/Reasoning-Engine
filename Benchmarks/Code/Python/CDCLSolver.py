import time

# ==============================================================================
#  Our New Intelligent Solver Engine (CDCL Algorithm - Final Corrected Version)
# ==============================================================================
# This is a robust, educational implementation of a CDCL SAT solver, rewritten
# from scratch with a correct main loop to fix the persistent infinite loop bug.

class CDCLSolver:
    def __init__(self, formula, num_vars):
        self.formula = formula
        self.num_vars = num_vars
        self.assignment = {}  # var -> (bool_value, decision_level, antecedent_clause_index)
        self.decision_level = 0
        self.trail = [] # Order of assignments

    def solve(self):
        """
        The main solver loop, rewritten with a standard, correct CDCL architecture.
        """
        # Initial propagation at level 0
        if self.unit_propagate() is not None:
            return None # UNSATISFIABLE at the root

        # Main solver loop
        while len(self.assignment) < self.num_vars:
            # 1. Make a new guess (a "decision")
            picked_var = self.pick_branching_variable()
            if picked_var is None:
                break # All variables assigned

            self.decision_level += 1
            self.assign(picked_var, False, None)

            # 2. Propagate consequences until a conflict or stable state
            while True:
                conflict_info = self.unit_propagate()
                
                if conflict_info is None:
                    # No conflict, break to make another decision
                    break

                # 3. If propagation leads to a conflict
                conflict_clause_index = conflict_info
                
                if self.decision_level == 0:
                    return None # UNSATISFIABLE

                # 4. Analyze, learn, and backjump
                learned_clause, backtrack_level = self.analyze_conflict(conflict_clause_index)
                
                self.formula.append(learned_clause)
                self.backtrack(backtrack_level)
                self.decision_level = backtrack_level
        
        # Format the final satisfying assignment
        final_assignment = {var: val[0] for var, val in self.assignment.items()}
        # Fill in any unassigned variables (can happen in satisfiable cases)
        for i in range(1, self.num_vars + 1):
            if i not in final_assignment:
                final_assignment[i] = True # Assign a default value
        return final_assignment

    def assign(self, var, value, antecedent_index):
        self.assignment[var] = (value, self.decision_level, antecedent_index)
        self.trail.append(var)

    def pick_branching_variable(self):
        # Simple heuristic: pick the first unassigned variable
        for i in range(1, self.num_vars + 1):
            if i not in self.assignment:
                return i
        return None

    def unit_propagate(self):
        # This is a more efficient implementation using a queue (the trail)
        head = 0
        while head < len(self.trail):
            var_to_propagate = self.trail[head]
            head += 1
            
            val = self.assignment[var_to_propagate][0]
            literal_to_check = -var_to_propagate if val else var_to_propagate

            for i, clause in enumerate(self.formula):
                if literal_to_check in clause:
                    unassigned_literals = []
                    is_satisfied = False
                    for literal in clause:
                        var = abs(literal)
                        val = True if literal > 0 else False
                        if var in self.assignment:
                            if self.assignment[var][0] == val:
                                is_satisfied = True
                                break
                        else:
                            unassigned_literals.append(literal)
                    
                    if is_satisfied:
                        continue

                    if len(unassigned_literals) == 1:
                        unit_literal = unassigned_literals[0]
                        var = abs(unit_literal)
                        val = True if unit_literal > 0 else False
                        self.assign(var, val, i)
                    elif len(unassigned_literals) == 0:
                        return i # Conflict
        return None

    def analyze_conflict(self, conflict_clause_index):
        conflict_clause = self.formula[conflict_clause_index]
        learned_clause = set(conflict_clause)
        
        while True:
            var_to_resolve = None
            lit_to_resolve = None
            
            # Find the most recent assignment in the learned clause that has a reason
            for var in reversed(self.trail):
                if var in {abs(l) for l in learned_clause}:
                    val, dl, ant_idx = self.assignment[var]
                    if dl == self.decision_level and ant_idx is not None:
                        lit = var if val else -var
                        if lit in learned_clause:
                            var_to_resolve = var
                            lit_to_resolve = lit
                            break
            
            if var_to_resolve is None:
                break # Reached the 1UIP (First Unique Implication Point)

            antecedent_clause = self.formula[self.assignment[var_to_resolve][2]]
            
            learned_clause.remove(lit_to_resolve)
            learned_clause.update(l for l in antecedent_clause if l != -lit_to_resolve)

        if not learned_clause:
            backtrack_level = 0
        else:
            other_levels = [self.assignment[abs(l)][1] for l in learned_clause if abs(l) in self.assignment and self.assignment[abs(l)][1] < self.decision_level]
            backtrack_level = max(other_levels, default=0)
            
        return list(learned_clause), backtrack_level

    def backtrack(self, level):
        vars_to_remove = [var for var, (_, dl, _) in self.assignment.items() if dl > level]
        for var in vars_to_remove:
            del self.assignment[var]
        
        self.trail = [v for v in self.trail if v not in vars_to_remove]

def cdcl_solver(formula, num_vars):
    """Public-facing function to call the solver class."""
    solver = CDCLSolver(list(formula), num_vars) # Pass a copy of the formula
    return solver.solve()

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
    print("--- Test Case 1: 3 Pigeons, 2 Holes (Unsatisfiable, should be fast) ---")
    formula_1, num_vars_1 = generate_pigeonhole_cnf(pigeons=3, holes=2)
    start_time_1 = time.time()
    solution_1 = cdcl_solver(formula_1, num_vars_1)
    duration_1 = time.time() - start_time_1
    print(f"Time taken: {duration_1:.6f} seconds")
    print(f"Result: {'SATISFIABLE' if solution_1 else 'UNSATISFIABLE'}\n")

    print("#"*60 + "\n")

    print("--- Test Case 2: 9 Pigeons, 8 Holes (Unsatisfiable, was slow for DPLL) ---")
    formula_2, num_vars_2 = generate_pigeonhole_cnf(pigeons=9, holes=8)
    start_time_2 = time.time()
    solution_2 = cdcl_solver(formula_2, num_vars_2)
    duration_2 = time.time() - start_time_2
    print(f"Time taken: {duration_2:.6f} seconds")
    print(f"Result: {'SATISFIABLE' if solution_2 else 'UNSATISFIABLE'}")
