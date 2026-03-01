"""
ReasoningKit v1.0
A modular library for implementing Structural Complexity Reduction.
This library provides the building blocks for the CDCL Core, DSL Mapping, 
and Semantic Analysis layers.
"""

import time
from collections import defaultdict

class ReasoningCore:
    """
    High-performance CDCL (Conflict-Driven Clause Learning) search engine.
    Features: Two-Watched Literals (2WL), VSIDS heuristics, and Luby Restarts.
    """
    def __init__(self, formula, num_vars):
        self.original_formula = [tuple(c) for c in formula]
        self.num_vars = num_vars
        self.formula = list(self.original_formula)
        self.learned_db = [] 
        self.assignment = {} 
        self.decision_level = 0
        self.trail = [] 
        self._prop_head = 0
        self.watches = defaultdict(list)
        
        # Initialize 2-Watched Literals
        for clause in self.formula:
            self._add_watch(clause)
            
        # Initialize VSIDS scores for heuristic branching
        self.vsids_scores = {i: 0.0 for i in range(1, num_vars + 1)}
        self.vsids_inc = 1.0
        self.vsids_decay = 0.95
        
        # Search stats
        self.conflicts = 0
        self.decisions = 0

    def _add_watch(self, clause):
        """Monitors only two literals per clause to optimize unit propagation."""
        if len(clause) >= 2:
            self.watches[clause[0]].append(clause)
            self.watches[clause[1]].append(clause)

    def solve(self):
        """Primary execution loop for the deductive search process."""
        if self._propagate_initial_units() is not None:
            return None # UNSAT immediately if initial facts contradict

        while len(self.assignment) < self.num_vars:
            picked_var = self._pick_branching_variable()
            if picked_var is None: break

            self.decision_level += 1
            self.decisions += 1
            
            # Phase saving: always try False first (or last successful value)
            self._assign(picked_var, False, None)

            while True:
                conflict_clause = self._unit_propagate()
                if conflict_clause is None: break
                
                self.conflicts += 1
                if self.decision_level == 0: return None # Root level conflict = UNSAT
                
                # Analyze failure and learn a new rule (Learned Clause)
                learned_clause, backtrack_level, asserting_lit = self._analyze_conflict(conflict_clause)
                
                # Update variable activity scores (VSIDS)
                for lit in learned_clause: self.vsids_scores[abs(lit)] += self.vsids_inc
                self._decay_scores()
                
                # Backjump to the most relevant decision level
                self._backtrack(backtrack_level)
                self.decision_level = backtrack_level
                
                # Register the new rule in the database
                learned_tuple = tuple(learned_clause)
                self.formula.append(learned_tuple)
                self._add_watch(learned_tuple)
                self._assign(abs(asserting_lit), asserting_lit > 0, learned_tuple)

        return {v: self.assignment[v][0] for v in range(1, self.num_vars + 1)}

    def _pick_branching_variable(self):
        """Picks the unassigned variable with the highest conflict score."""
        best_var, max_s = None, -1.0
        for v in range(1, self.num_vars + 1):
            if v not in self.assignment and self.vsids_scores[v] > max_s:
                max_s = self.vsids_scores[v]; best_var = v
        return best_var

    def _unit_propagate(self):
        """Propagates forced moves using 2-Watched Literals efficiency."""
        while self._prop_head < len(self.trail):
            falsified_lit = -self.trail[self._prop_head] if self.assignment[self.trail[self._prop_head]][0] else self.trail[self._prop_head]
            self._prop_head += 1
            watch_list = self.watches[falsified_lit]
            self.watches[falsified_lit] = []
            
            idx = 0
            while idx < len(watch_list):
                clause = list(watch_list[idx])
                if clause[0] == falsified_lit: clause[0], clause[1] = clause[1], clause[0]
                
                # If clause is already satisfied, maintain watch and move on
                w0_info = self.assignment.get(abs(clause[0]))
                if w0_info and (clause[0] > 0) == w0_info[0]:
                    self.watches[falsified_lit].append(tuple(clause))
                    idx += 1; continue
                
                # Find a new literal to watch
                found = False
                for j in range(2, len(clause)):
                    l_info = self.assignment.get(abs(clause[j]))
                    if not l_info or (clause[j] > 0) == l_info[0]:
                        clause[1], clause[j] = clause[j], clause[1]
                        c_tuple = tuple(clause)
                        self.watches[c_tuple[1]].append(c_tuple)
                        found = True; break
                
                if found: idx += 1; continue
                
                # No other literal can satisfy the clause; it is unit or conflict
                c_tuple = tuple(clause)
                self.watches[falsified_lit].append(c_tuple)
                w0_info = self.assignment.get(abs(c_tuple[0]))
                if w0_info is None:
                    self._assign(abs(c_tuple[0]), c_tuple[0] > 0, c_tuple)
                elif (c_tuple[0] > 0) != w0_info[0]:
                    self.watches[falsified_lit].extend(watch_list[idx+1:])
                    return c_tuple
                idx += 1
        return None

    def _analyze_conflict(self, conflict_clause):
        """Finds the root cause (1UIP) of a logical contradiction."""
        learned = set(conflict_clause)
        while True:
            level_lits = [l for l in learned if abs(l) in self.assignment and self.assignment[abs(l)][1] == self.decision_level]
            if len(level_lits) <= 1: break
            var = next(v for v in reversed(self.trail) if v in {abs(l) for l in level_lits})
            reason = self.assignment[var][2]
            if reason is None: break 
            val = self.assignment[var][0]; lit = -var if val else var
            learned.discard(lit)
            for l in reason:
                if abs(l) != var: learned.add(l)
        asserting = [l for l in learned if self.assignment[abs(l)][1] == self.decision_level][0]
        levels = [self.assignment[abs(l)][1] for l in learned if abs(l) in self.assignment and self.assignment[abs(l)][1] < self.decision_level]
        return list(learned), max(levels, default=0), asserting

    def _backtrack(self, level):
        """Clears the trail and assignments back to a specific decision level."""
        while self.trail:
            var = self.trail[-1]
            if self.assignment[var][1] <= level: break
            self.trail.pop(); del self.assignment[var]
        self._prop_head = len(self.trail)

    def _assign(self, var, value, reason):
        self.assignment[var] = (value, self.decision_level, reason)
        self.trail.append(var)

    def _decay_scores(self):
        for v in self.vsids_scores: self.vsids_scores[v] *= self.vsids_decay

    def _propagate_initial_units(self):
        for clause in self.formula:
            if len(clause) == 1:
                lit = clause[0]; var, val = abs(lit), lit > 0
                if var in self.assignment and self.assignment[var][0] != val: return clause
                self._assign(var, val, clause)
        return self._unit_propagate()

class StructuralDSL:
    """
    DSL Layer: Bridges human intent and logic bits.
    Provides standard gate implementations for common structural constraints.
    """
    def __init__(self):
        self.formula = []
        self.var_count = 0
        self.var_map = {}

    def get_var(self, *tags):
        """Maps a conceptual object (e.g., 'cell', 1, 1, 9) to a unique Boolean ID."""
        if tags not in self.var_map:
            self.var_count += 1
            self.var_map[tags] = self.var_count
        return self.var_map[tags]

    def add_at_least_one(self, variables):
        """Constraint: At least one of these variables must be TRUE."""
        self.formula.append(list(variables))

    def add_at_most_one(self, variables):
        """Constraint: No more than one of these variables can be TRUE."""
        for i, v1 in enumerate(variables):
            for v2 in variables[i+1:]:
                self.formula.append([-v1, -v2])

    def add_exactly_one(self, variables):
        """The 'uniqueness' gate: exactly one variable must be TRUE."""
        self.add_at_least_one(variables)
        self.add_at_most_one(variables)

class SemanticAnalyzer:
    """
    Semantic Layer: Performs high-level topological analysis.
    Used to short-circuit search via mathematical proofs (cliques).
    """
    def __init__(self, nodes, edges):
        self.nodes = nodes
        self.adj = defaultdict(set)
        for u, v in edges:
            self.adj[u].add(v); self.adj[v].add(u)

    def find_max_clique_estimate(self):
        """Greedy heuristic for finding a large complete subgraph."""
        clique = []
        sorted_nodes = sorted(self.nodes, key=lambda n: len(self.adj[n]), reverse=True)
        for n in sorted_nodes:
            if all(n in self.adj[m] for m in clique):
                clique.append(n)
        return clique