
import time
import sys
import math

# ==============================================================================
#  CDCL Solver: High-Performance Edition (Index-Independent Version)
#  Tactics: VSIDS, Phase Saving, Luby Restarts, LBD Clause Deletion
# ==============================================================================

class CDCLSolver:
    def __init__(self, formula, num_vars):
        # We store clauses as tuples for hashability/safety
        self.original_formula = [tuple(c) for c in formula]
        self.learned_db = [] # List of [clause_tuple, lbd_score]
        self.num_vars = num_vars
        
        # Current working formula (Original + Learned)
        self.formula = list(self.original_formula)
        
        # Core State
        # var -> (bool_value, decision_level, antecedent_clause_object)
        self.assignment = {}  
        self.decision_level = 0
        self.trail = [] 

        # --- Heuristics ---
        self.vsids_scores = {i: 0.0 for i in range(1, num_vars + 1)}
        self.vsids_inc = 1.0
        self.vsids_decay = 0.95
        self.phase_saving = {i: False for i in range(1, num_vars + 1)}
        
        # --- Restart Policy (Luby) ---
        self.restart_base = 100
        self.restart_count = 0
        self.next_restart = self.restart_base
        
        # --- Memory Management (LBD) ---
        self.reduce_db_limit = 1000 # Reduce more frequently for stress tests
        self.conflicts_since_reduce = 0
        
        # Diagnostic
        self.conflicts = 0
        self.decisions = 0

    def solve(self):
        print(f"  [Solver] Commencing Index-Independent Search for {self.num_vars} variables...", flush=True)
        sys.stdout.flush()
        
        if self._unit_propagate() is not None:
            return None

        while len(self.assignment) < self.num_vars:
            # 1. Check for Restart
            if self.conflicts >= self.next_restart:
                self._perform_restart()
            
            # 2. Check for Database Reduction
            if self.conflicts_since_reduce >= self.reduce_db_limit:
                self._reduce_clause_database()

            picked_var = self._pick_branching_variable()
            if picked_var is None: break

            self.decision_level += 1
            self.decisions += 1
            
            if self.decisions % 100 == 0:
                print(f"  [Heartbeat] Decisions: {self.decisions} | Conflicts: {self.conflicts} | DL: {self.decision_level}", flush=True)
                sys.stdout.flush()
            
            preferred_val = self.phase_saving.get(picked_var, False)
            self._assign(picked_var, preferred_val, None)

            while True:
                conflict_clause = self._unit_propagate()
                if conflict_clause is None:
                    break
                
                self.conflicts += 1
                self.conflicts_since_reduce += 1
                
                if self.conflicts % 200 == 0:
                    print(f"  [Heartbeat] Conflict #{self.conflicts} | Trail: {len(self.trail)}/{self.num_vars}", flush=True)
                    sys.stdout.flush()

                if self.decision_level == 0:
                    return None # UNSAT

                learned_clause, backtrack_level, asserting_literal = self._analyze_conflict(conflict_clause)
                
                # Calculate LBD (Literal Block Distance)
                lbd = len(set(self.assignment[abs(lit)][1] for lit in learned_clause if abs(lit) in self.assignment))
                
                # Update VSIDS
                for lit in learned_clause:
                    self.vsids_scores[abs(lit)] += self.vsids_inc
                self._decay_scores()

                # Backtrack
                self._backtrack(backtrack_level)
                self.decision_level = backtrack_level
                
                # Add to formula and learned DB
                learned_tuple = tuple(learned_clause)
                self.formula.append(learned_tuple)
                self.learned_db.append([learned_tuple, lbd])
                
                # Force asserting literal
                self._assign(abs(asserting_literal), asserting_literal > 0, learned_tuple)

        return self._format_solution()

    def _reduce_clause_database(self):
        """Removes high-LBD clauses. Pointers remain safe because we store objects."""
        print(f"  [Memory] Reducing database. Learned count: {len(self.learned_db)}", flush=True)
        
        # Sort by LBD
        self.learned_db.sort(key=lambda x: x[1])
        
        # Keep best 50% OR anything with LBD <= 2
        mid = len(self.learned_db) // 2
        new_db = []
        for i, (clause, lbd) in enumerate(self.learned_db):
            # Never delete clauses currently being used as a reason for an assignment
            is_reason = any(info[2] == clause for info in self.assignment.values())
            if i < mid or lbd <= 2 or is_reason:
                new_db.append([clause, lbd])
        
        self.learned_db = new_db
        self.formula = list(self.original_formula) + [item[0] for item in self.learned_db]
        self.conflicts_since_reduce = 0
        print(f"  [Memory] Done. Formula size: {len(self.formula)}", flush=True)

    def _perform_restart(self):
        self.restart_count += 1
        self.next_restart = self.conflicts + (self.restart_base * (1.5 ** self.restart_count))
        self._backtrack(0)
        self.decision_level = 0

    def _assign(self, var, value, antecedent_clause):
        self.assignment[var] = (value, self.decision_level, antecedent_clause)
        self.trail.append(var)
        self.phase_saving[var] = value

    def _pick_branching_variable(self):
        best_var = None
        max_score = -1.0
        for v in range(1, self.num_vars + 1):
            if v not in self.assignment:
                if self.vsids_scores[v] > max_score:
                    max_score = self.vsids_scores[v]
                    best_var = v
        return best_var

    def _decay_scores(self):
        for var in self.vsids_scores:
            self.vsids_scores[var] *= self.vsids_decay

    def _unit_propagate(self):
        head = 0
        while head < len(self.trail):
            var_to_propagate = self.trail[head]
            head += 1
            val, dl, _ = self.assignment[var_to_propagate]
            lit_is_true = var_to_propagate if val else -var_to_propagate
            
            for clause in self.formula:
                if -lit_is_true not in clause: continue
                
                unassigned, satisfied = [], False
                for lit in clause:
                    v = abs(lit)
                    if v in self.assignment:
                        if (lit > 0) == self.assignment[v][0]:
                            satisfied = True; break
                    else: unassigned.append(lit)
                
                if satisfied: continue
                if len(unassigned) == 1:
                    u_lit = unassigned[0]
                    if abs(u_lit) not in self.assignment:
                        self._assign(abs(u_lit), u_lit > 0, clause)
                elif len(unassigned) == 0:
                    return clause 
        return None

    def _analyze_conflict(self, conflict_clause):
        learned_clause = set(conflict_clause)
        while True:
            curr_level_lits = [l for l in learned_clause 
                               if abs(l) in self.assignment and 
                               self.assignment[abs(l)][1] == self.decision_level]
            if len(curr_level_lits) <= 1: break
            
            var_to_resolve = None
            for var in reversed(self.trail):
                if var in {abs(l) for l in curr_level_lits}:
                    var_to_resolve = var; break
            
            # The reason (antecedent) is now the clause object itself
            reason_clause = self.assignment[var_to_resolve][2]
            if reason_clause is None: break 
            
            self.vsids_scores[var_to_resolve] += self.vsids_inc
            val = self.assignment[var_to_resolve][0]
            lit_in_clause = -var_to_resolve if val else var_to_resolve
            learned_clause.discard(lit_in_clause)
            for l in reason_clause:
                if abs(l) != var_to_resolve: learned_clause.add(l)

        # Asserting literal (1UIP)
        asserting_literal = [l for l in learned_clause if self.assignment[abs(l)][1] == self.decision_level][0]
        levels = [self.assignment[abs(l)][1] for l in learned_clause if abs(l) in self.assignment]
        levels = [lv for lv in levels if lv < self.decision_level]
        backtrack_level = max(levels, default=0)
            
        return list(learned_clause), backtrack_level, asserting_literal

    def _backtrack(self, level):
        while self.trail:
            var = self.trail[-1]
            if self.assignment[var][1] <= level: break
            self.trail.pop()
            del self.assignment[var]

    def _format_solution(self):
        return {v: self.assignment[v][0] for v in range(1, self.num_vars + 1)}

# ==============================================================================
#  Problem Generator
# ==============================================================================

def generate_pigeonhole_cnf(pigeons, holes):
    def var_num(p, h): return (p - 1) * holes + h
    num_vars = pigeons * holes
    formula = []
    for p in range(1, pigeons + 1):
        formula.append([var_num(p, h) for h in range(1, holes + 1)])
    for h in range(1, holes + 1):
        for p1 in range(1, pigeons + 1):
            for p2 in range(p1 + 1, pigeons + 1):
                formula.append([-var_num(p1, h), -var_num(p2, h)])
    return formula, num_vars

if __name__ == "__main__":
    print("--- ROBUST STRESS TEST START ---", flush=True)
    p, h = 10, 9
    print(f"Solving {p} pigeons into {h} holes...", flush=True)
    formula, num_vars = generate_pigeonhole_cnf(p, h)
    
    solver = CDCLSolver(formula, num_vars)
    start = time.time()
    result = solver.solve()
    duration = time.time() - start
    
    print(f"\n--- Result ---", flush=True)
    print(f"Conflicts: {solver.conflicts} | Time: {duration:.4f}s | Result: {'SAT' if result else 'UNSAT'}", flush=True)
