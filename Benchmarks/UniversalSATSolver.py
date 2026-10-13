import time


# ==============================================================================
#  Project Architecture: A Universal SAT Solver (Fresh Start)
# ==============================================================================
# This script implements our high-level plan with a clean, modular structure.
# 1. Translate a problem into CNF.
# 2. Optimize the formula using pre-processing techniques.
# 3. Solve the optimized formula with a reliable backend engine.
# ==============================================================================


# ==============================================================================
#  Step 1: Problem Translator (Generator)
# ==============================================================================

def generate_pigeonhole_cnf(pigeons, holes):
    """
    Translates the Pigeonhole Problem into the universal CNF language.
    """
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
#  Step 2: Formula Optimizer (Preprocessor)
# ==============================================================================

class ProblemOptimizer:
    def __init__(self, formula, num_vars):
        self.formula = formula
        self.num_vars = num_vars

    def apply_symmetry_breaking(self, problem_params):
        """
        Applies pre-processing techniques to add simplifying clauses.
        """
        print("  [Optimizer] Applying symmetry-breaking constraints...")
        
        if problem_params.get('type') == 'pigeonhole':
            if problem_params['pigeons'] > 1 and problem_params['holes'] > 1:
                # Add a clause to force Pigeon 1 into Hole 1
                self.formula.append([1])
                print("    - Constraint added: Pigeon 1 must be in Hole 1.")
                return True
        
        print("    - No applicable optimizations found.")
        return False


# ==============================================================================
#  Step 3: Backend Solver Engines
# ==============================================================================

class DpllSolver:
    """A simple, reliable, recursive implementation of the DPLL algorithm."""
    def solve(self, formula, num_vars):
        """Public entry point to the solver."""
        self.num_vars = num_vars
        return self._dpll(formula, {})

    def _dpll(self, formula, assignment):
        """The recursive heart of the DPLL algorithm."""
        
        # --- Unit Propagation ---
        while True:
            unit_clauses = [c for c in formula if len(c) == 1]
            if not unit_clauses:
                break
            
            unit_literal = unit_clauses[0][0]
            var = abs(unit_literal)
            val = True if unit_literal > 0 else False
            
            if var in assignment and assignment[var] != val:
                return None # Contradiction

            if var not in assignment:
                assignment[var] = val
            
            # Simplify the formula
            new_formula = []
            for clause in formula:
                if unit_literal in clause:
                    continue
                new_clause = [l for l in clause if l != -unit_literal]
                if not new_clause:
                    return None # Contradiction
                new_formula.append(new_clause)
            formula = new_formula

        # --- Check for Success or Failure ---
        if not formula:
            return assignment

        # --- Pick a variable and Recurse ---
        unassigned_vars = [v for v in range(1, self.num_vars + 1) if v not in assignment]
        if not unassigned_vars:
             return assignment

        var_to_guess = unassigned_vars[0]

        solution = self._dpll(formula + [[var_to_guess]], assignment.copy())
        if solution is not None:
            return solution
        
        return self._dpll(formula + [[-var_to_guess]], assignment.copy())

class CDCLSolver:
    """
    A Conflict-Driven Clause Learning (CDCL) SAT solver.
    This is a robust, educational implementation, rewritten to be correct.
    """
    def __init__(self):
        self.formula = []
        self.num_vars = 0
        self.assignment = {}  # var -> (bool_value, decision_level, antecedent_clause_index)
        self.decision_level = 0
        self.trail = [] # An ordered list of assigned variables

    def solve(self, formula, num_vars):
        """Main solver entry point. Executes the CDCL algorithm."""
        self.formula = formula
        self.num_vars = num_vars
        self.assignment = {}
        self.decision_level = 0
        self.trail = []

        # Initial unit propagation at the base level (decision_level 0)
        if self._unit_propagate() is not None:
            return None # Formula is unsatisfiable from the start

        # Main solver loop
        while len(self.assignment) < self.num_vars:
            picked_var = self._pick_branching_variable()
            if picked_var is None:
                break # All variables are assigned

            # Start a new decision level and make a guess
            self.decision_level += 1
            self._assign(picked_var, False, None)

            # Deduce consequences of the guess
            while True:
                conflict_clause_index = self._unit_propagate()
                if conflict_clause_index is None:
                    # No conflict, we can make another decision
                    break
                
                # Conflict occurred
                if self.decision_level == 0:
                    return None # Conflict at base level means UNSAT

                # Learn from the conflict and backjump
                learned_clause, backtrack_level = self._analyze_conflict(conflict_clause_index)
                self.formula.append(learned_clause)
                self._backtrack(backtrack_level)
                self.decision_level = backtrack_level

        return self._format_solution()

    def _assign(self, var, value, antecedent_index):
        """Assigns a value to a variable and adds it to the trail."""
        self.assignment[var] = (value, self.decision_level, antecedent_index)
        self.trail.append(var)

    def _pick_branching_variable(self):
        """Selects the next unassigned variable to make a guess on."""
        for i in range(1, self.num_vars + 1):
            if i not in self.assignment:
                return i
        return None

    def _unit_propagate(self):
        """
        Performs unit propagation. Returns a conflict clause index or None.
        """
        # This is a more efficient implementation using a queue (the trail)
        head = 0
        while head < len(self.trail):
            var_to_propagate = self.trail[head]
            head += 1
            
            val = self.assignment[var_to_propagate][0]
            literal_to_check = -var_to_propagate if val else var_to_propagate

            for i, clause in enumerate(self.formula):
                # This check is an optimization to only check clauses affected by the new assignment
                if literal_to_check not in clause:
                    continue

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
                    if var not in self.assignment:
                        self._assign(var, val, i)
                elif len(unassigned_literals) == 0:
                    return i # Conflict
        return None

    def _analyze_conflict(self, conflict_clause_index):
        """Analyzes a conflict to produce a learned clause and a backjump level."""
        conflict_clause = self.formula[conflict_clause_index]
        learned_clause = set(conflict_clause)
        
        while True:
            var_to_resolve = None
            # Find the most recent assignment in the learned clause that has a reason
            for var in reversed(self.trail):
                if var in {abs(l) for l in learned_clause}:
                    val, dl, ant_idx = self.assignment[var]
                    if dl == self.decision_level and ant_idx is not None:
                        var_to_resolve = var
                        break
            
            if var_to_resolve is None:
                break # Reached the 1UIP (First Unique Implication Point)

            antecedent_clause = self.formula[self.assignment[var_to_resolve][2]]
            val = self.assignment[var_to_resolve][0]
            lit_to_resolve = var_to_resolve if val else -var_to_resolve
            
            learned_clause.remove(lit_to_resolve)
            learned_clause.update(l for l in antecedent_clause if l != -lit_to_resolve)

        if not learned_clause:
            backtrack_level = 0
        else:
            other_levels = [self.assignment[abs(l)][1] for l in learned_clause if abs(l) in self.assignment and self.assignment[abs(l)][1] < self.decision_level]
            backtrack_level = max(other_levels, default=0)
            
        return list(learned_clause), backtrack_level

    def _backtrack(self, level):
        """Undoes assignments and trail entries above a certain decision level."""
        vars_to_remove = {var for var, (_, dl, _) in self.assignment.items() if dl > level}
        for var in vars_to_remove:
            del self.assignment[var]
        
        self.trail = [v for v in self.trail if v not in vars_to_remove]

    def _format_solution(self):
        """Formats the final assignment."""
        solution = {var: val[0] for var, val in self.assignment.items()}
        for i in range(1, self.num_vars + 1):
            if i not in solution:
                solution[i] = True
        return solution

# ==============================================================================
#  The Main Controller: nn_solve
# ==============================================================================

def nn_solve(problem_type, params, solver_engine):
    """
    The main controller that orchestrates the solving process.
    """
    print(f"--- Initiating nn_solve for a '{problem_type}' problem ---")
    print(f"--- Using '{solver_engine.__class__.__name__}' engine ---")

    # Step 1: Translate
    print("\n[Step 1] Translating problem into CNF...")
    formula, num_vars = generate_pigeonhole_cnf(params['pigeons'], params['holes'])
    print(f"Translation complete: {num_vars} variables, {len(formula)} clauses.")

    # Step 2: Optimize
    print("\n[Step 2] Optimizing formula...")
    optimizer = ProblemOptimizer(formula, num_vars)
    optimizer.apply_symmetry_breaking({'type': problem_type, **params})
    optimized_formula = optimizer.formula

    # Step 3: Solve
    print("\n[Step 3] Handing off to backend solver engine...")
    start_time = time.time()
    solution = solver_engine.solve(optimized_formula, num_vars)
    duration = time.time() - start_time

    # --- Final Result ---
    print("\n" + "="*50)
    print("--- SOLVER FINISHED ---")
    print(f"Time taken: {duration:.6f} seconds")
    if solution:
        print("Result: SATISFIABLE")
    else:
        print("Result: UNSATISFIABLE")
    print("="*50)


# ==============================================================================
#  Main Program Execution
# ==============================================================================

if __name__ == "__main__":
    
    # --- CHOOSE YOUR SOLVER ENGINE ---
    solver_to_use = "dpll" # Options: "dpll", "cdcl"
    
    if solver_to_use == "dpll":
        solver_engine = DpllSolver()
    elif solver_to_use == "cdcl":
        solver_engine = CDCLSolver()
    else:
        raise ValueError("Unknown solver type specified.")

    # --- Test Case 1: 9 Pigeons, 8 Holes (Unsatisfiable) ---
    nn_solve(problem_type="pigeonhole", params={'pigeons': 9, 'holes': 8}, solver_engine=solver_engine)
    
    print("\n\n" + "#"*60 + "\n")

    # --- Test Case 2: 8 Pigeons, 8 Holes (Satisfiable) ---
    nn_solve(problem_type="pigeonhole", params={'pigeons': 8, 'holes': 8}, solver_engine=solver_engine)
