//! ReasoningKit v1.0 (Rust Edition)
//! A modular library for implementing Structural Complexity Reduction.
//! Provides the building blocks for the CDCL Core, DSL Mapping, and Semantic Analysis.

use std::collections::{HashMap, HashSet};

pub type Var = usize;
pub type Lit = i32;
pub type Clause = Vec<Lit>;

#[inline]
pub fn lit_to_var(lit: Lit) -> Var {
    lit.unsigned_abs() as Var
}

#[inline]
pub fn lit_is_pos(lit: Lit) -> bool {
    lit > 0
}

#[derive(Clone, Debug)]
pub struct VarAssignment {
    pub value: bool,
    pub level: usize,
    pub reason: Option<usize>, // Index into `ReasoningCore::formula`
}

/// High-performance CDCL (Conflict-Driven Clause Learning) search engine.
/// Features: Two-Watched Literals (2WL), 1UIP Conflict Resolution, and VSIDS heuristics.
pub struct ReasoningCore {
    pub num_vars: usize,
    pub formula: Vec<Clause>,
    pub assignment: HashMap<Var, VarAssignment>,
    pub decision_level: usize,
    pub trail: Vec<Var>,
    prop_head: usize,
    pub watches: HashMap<Lit, Vec<usize>>,
    pub vsids_scores: Vec<f64>,
    pub vsids_inc: f64,
    pub vsids_decay: f64,
    pub conflicts: usize,
    pub decisions: usize,
}

impl ReasoningCore {
    pub fn new(formula: Vec<Clause>, num_vars: usize) -> Self {
        let mut watches: HashMap<Lit, Vec<usize>> = HashMap::new();

        for (idx, clause) in formula.iter().enumerate() {
            if clause.len() >= 2 {
                watches.entry(clause[0]).or_default().push(idx);
                watches.entry(clause[1]).or_default().push(idx);
            }
        }

        let vsids_scores = vec![0.0; num_vars + 1];

        Self {
            num_vars,
            formula,
            assignment: HashMap::with_capacity(num_vars),
            decision_level: 0,
            trail: Vec::with_capacity(num_vars),
            prop_head: 0,
            watches,
            vsids_scores,
            vsids_inc: 1.0,
            vsids_decay: 0.95,
            conflicts: 0,
            decisions: 0,
        }
    }

    fn add_watch(&mut self, clause_idx: usize) {
        let clause = &self.formula[clause_idx];
        if clause.len() >= 2 {
            self.watches.entry(clause[0]).or_default().push(clause_idx);
            self.watches.entry(clause[1]).or_default().push(clause_idx);
        }
    }

    pub fn solve(&mut self) -> Option<HashMap<Var, bool>> {
        // Root-level unit propagation
        if self.propagate_initial_units().is_some() {
            return None; // UNSAT immediately if initial facts contradict
        }

        while self.assignment.len() < self.num_vars {
            let picked_var = match self.pick_branching_variable() {
                Some(v) => v,
                None => break, // All reachable variables are assigned
            };

            self.decision_level += 1;
            self.decisions += 1;

            // Phase saving / default branch: false
            self.assign(picked_var, false, None);

            loop {
                let conflict = self.unit_propagate();
                let conflict_idx = match conflict {
                    Some(c) => c,
                    None => break, // No conflict, proceed to next decision
                };

                self.conflicts += 1;
                if self.decision_level == 0 {
                    return None; // Conflict at root implies UNSAT
                }

                // 1UIP Conflict Analysis
                let (learned_clause, backtrack_level, asserting_lit) =
                    self.analyze_conflict(conflict_idx);

                // Update variable activity scores (VSIDS)
                for &lit in &learned_clause {
                    let v = lit_to_var(lit);
                    if v <= self.num_vars {
                        self.vsids_scores[v] += self.vsids_inc;
                    }
                }
                self.decay_scores();

                // Non-chronological backjump
                self.backtrack(backtrack_level);
                self.decision_level = backtrack_level;

                // Register learned clause
                let learned_idx = self.formula.len();
                self.formula.push(learned_clause);
                self.add_watch(learned_idx);

                self.assign(
                    lit_to_var(asserting_lit),
                    lit_is_pos(asserting_lit),
                    Some(learned_idx),
                );
            }
        }

        Some(
            (1..=self.num_vars)
                .map(|v| (v, self.assignment.get(&v).map(|a| a.value).unwrap_or(false)))
                .collect(),
        )
    }

    fn pick_branching_variable(&self) -> Option<Var> {
        let mut best_var = None;
        let mut max_s = -1.0;

        for v in 1..=self.num_vars {
            if !self.assignment.contains_key(&v) && self.vsids_scores[v] > max_s {
                max_s = self.vsids_scores[v];
                best_var = Some(v);
            }
        }
        best_var
    }

    fn unit_propagate(&mut self) -> Option<usize> {
        while self.prop_head < self.trail.len() {
            let p = self.trail[self.prop_head];
            self.prop_head += 1;

            let p_val = self.assignment[&p].value;
            let falsified_lit: Lit = if p_val { -(p as Lit) } else { p as Lit };

            let watch_list = match self.watches.remove(&falsified_lit) {
                Some(wl) => wl,
                None => continue,
            };

            let mut retained_watches = Vec::with_capacity(watch_list.len());

            for (i, &c_idx) in watch_list.iter().enumerate() {
                // Normalize watched literals so clause[1] is the falsified literal
                if self.formula[c_idx][0] == falsified_lit {
                    self.formula[c_idx].swap(0, 1);
                }

                // If first watched literal is satisfied, clause remains satisfied
                let w0 = self.formula[c_idx][0];
                let w0_var = lit_to_var(w0);
                if let Some(info) = self.assignment.get(&w0_var) {
                    if lit_is_pos(w0) == info.value {
                        retained_watches.push(c_idx);
                        continue;
                    }
                }

                // Search for an alternative non-falsified literal to watch
                let mut found = false;
                for j in 2..self.formula[c_idx].len() {
                    let lj = self.formula[c_idx][j];
                    let lj_var = lit_to_var(lj);
                    let can_watch = match self.assignment.get(&lj_var) {
                        None => true,
                        Some(info) => lit_is_pos(lj) == info.value,
                    };

                    if can_watch {
                        self.formula[c_idx].swap(1, j);
                        let new_watch = self.formula[c_idx][1];
                        self.watches.entry(new_watch).or_default().push(c_idx);
                        found = true;
                        break;
                    }
                }

                if found {
                    continue;
                }

                // No alternative literal found; clause is unit or conflicting
                retained_watches.push(c_idx);

                match self.assignment.get(&w0_var) {
                    None => {
                        self.assign(w0_var, lit_is_pos(w0), Some(c_idx));
                    }
                    Some(info) if lit_is_pos(w0) != info.value => {
                        // Conflict detected: restore uninspected watches and return clause
                        retained_watches.extend_from_slice(&watch_list[i + 1..]);
                        self.watches.insert(falsified_lit, retained_watches);
                        return Some(c_idx);
                    }
                    _ => {}
                }
            }

            self.watches.insert(falsified_lit, retained_watches);
        }

        None
    }

    fn analyze_conflict(&self, conflict_idx: usize) -> (Clause, usize, Lit) {
        let mut learned: HashSet<Lit> = self.formula[conflict_idx].iter().copied().collect();
        let mut trail_idx = self.trail.len();

        loop {
            let level_lits: Vec<Lit> = learned
                .iter()
                .copied()
                .filter(|&l| {
                    self.assignment
                        .get(&lit_to_var(l))
                        .map(|info| info.level == self.decision_level)
                        .unwrap_or(false)
                })
                .collect();

            if level_lits.len() <= 1 {
                break;
            }

            // Find latest assigned variable involved in the conflict
            let mut resolved_var = None;
            while trail_idx > 0 {
                trail_idx -= 1;
                let v = self.trail[trail_idx];
                if level_lits.iter().any(|&l| lit_to_var(l) == v) {
                    resolved_var = Some(v);
                    break;
                }
            }

            let var = match resolved_var {
                Some(v) => v,
                None => break,
            };

            let reason_idx = match self.assignment[&var].reason {
                Some(idx) => idx,
                None => break,
            };

            let val = self.assignment[&var].value;
            let lit_to_remove: Lit = if val { -(var as Lit) } else { var as Lit };
            learned.remove(&lit_to_remove);

            for &l in &self.formula[reason_idx] {
                if lit_to_var(l) != var {
                    learned.insert(l);
                }
            }
        }

        let asserting_lit = *learned
            .iter()
            .find(|&&l| self.assignment[&lit_to_var(l)].level == self.decision_level)
            .expect("1UIP invariant failed: asserting literal not found");

        let backtrack_level = learned
            .iter()
            .filter(|&&l| lit_to_var(l) != lit_to_var(asserting_lit))
            .map(|&l| self.assignment[&lit_to_var(l)].level)
            .max()
            .unwrap_or(0);

        (learned.into_iter().collect(), backtrack_level, asserting_lit)
    }

    fn backtrack(&mut self, level: usize) {
        while let Some(&var) = self.trail.last() {
            if self.assignment[&var].level <= level {
                break;
            }
            self.trail.pop();
            self.assignment.remove(&var);
        }
        self.prop_head = self.trail.len();
    }

    fn assign(&mut self, var: Var, value: bool, reason: Option<usize>) {
        self.assignment.insert(
            var,
            VarAssignment {
                value,
                level: self.decision_level,
                reason,
            },
        );
        self.trail.push(var);
    }

    fn decay_scores(&mut self) {
        for score in self.vsids_scores.iter_mut().skip(1) {
            *score *= self.vsids_decay;
        }
    }

    fn propagate_initial_units(&mut self) -> Option<usize> {
        for i in 0..self.formula.len() {
            if self.formula[i].len() == 1 {
                let lit = self.formula[i][0];
                let var = lit_to_var(lit);
                let val = lit_is_pos(lit);

                if let Some(info) = self.assignment.get(&var) {
                    if info.value != val {
                        return Some(i);
                    }
                } else {
                    self.assign(var, val, Some(i));
                }
            }
        }
        self.unit_propagate()
    }
}

/// DSL Layer: Bridges human intent and logic bits.
/// Provides standard gate implementations for structural constraints.
#[derive(Default)]
pub struct StructuralDSL {
    pub formula: Vec<Clause>,
    pub var_count: usize,
    pub var_map: HashMap<String, Var>,
}

impl StructuralDSL {
    pub fn new() -> Self {
        Self::default()
    }

    /// Maps a conceptual identifier (e.g., ("cell", 1, 1, 9)) to a unique Boolean ID.
    pub fn get_var<T: Into<String>>(&mut self, tag: T) -> Var {
        let key = tag.into();
        if let Some(&id) = self.var_map.get(&key) {
            id
        } else {
            self.var_count += 1;
            self.var_map.insert(key, self.var_count);
            self.var_count
        }
    }

    /// Helper for variadic/slice-style composite keys.
    pub fn get_var_composite(&mut self, parts: &[&str]) -> Var {
        self.get_var(parts.join("_"))
    }

    /// Constraint: At least one of these variables must be TRUE.
    pub fn add_at_least_one(&mut self, variables: &[Var]) {
        self.formula
            .push(variables.iter().map(|&v| v as Lit).collect());
    }

    /// Constraint: No more than one of these variables can be TRUE.
    pub fn add_at_most_one(&mut self, variables: &[Var]) {
        for i in 0..variables.len() {
            for j in (i + 1)..variables.len() {
                self.formula
                    .push(vec![-(variables[i] as Lit), -(variables[j] as Lit)]);
            }
        }
    }

    /// The "uniqueness" gate: exactly one variable must be TRUE.
    pub fn add_exactly_one(&mut self, variables: &[Var]) {
        self.add_at_least_one(variables);
        self.add_at_most_one(variables);
    }
}

/// Semantic Layer: Performs high-level topological analysis.
/// Used to short-circuit search via mathematical proofs (cliques).
pub struct SemanticAnalyzer {
    pub nodes: Vec<usize>,
    pub adj: HashMap<usize, HashSet<usize>>,
}

impl SemanticAnalyzer {
    pub fn new(nodes: Vec<usize>, edges: &[(usize, usize)]) -> Self {
        let mut adj: HashMap<usize, HashSet<usize>> = HashMap::new();
        for &(u, v) in edges {
            adj.entry(u).or_default().insert(v);
            adj.entry(v).or_default().insert(u);
        }
        Self { nodes, adj }
    }

    /// Greedy heuristic for finding a large complete subgraph.
    pub fn find_max_clique_estimate(&self) -> Vec<usize> {
        let mut clique: Vec<usize> = Vec::new();
        let mut sorted_nodes = self.nodes.clone();

        sorted_nodes.sort_by(|&a, &b| {
            let deg_a = self.adj.get(&a).map(|s| s.len()).unwrap_or(0);
            let deg_b = self.adj.get(&b).map(|s| s.len()).unwrap_or(0);
            deg_b.cmp(&deg_a)
        });

        for n in sorted_nodes {
            let connects_to_all = clique.iter().all(|&m| {
                self.adj
                    .get(&m)
                    .map(|neighbors| neighbors.contains(&n))
                    .unwrap_or(false)
            });

            if connects_to_all {
                clique.push(n);
            }
        }

        clique
    }
}
