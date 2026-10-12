#include <iostream>
#include <vector>
#include <string>
#include <unordered_map>
#include <unordered_set>
#include <algorithm>
#include <cmath>
#include <optional>
#include <cstdint>

// ============================================================================
// Data Types and Definitions
// ============================================================================
using Var = int32_t;
using Lit = int32_t;
using Clause = std::vector<Lit>;

struct VarInfo {
    bool val{false};
    int level{-1};
    int reason{-1}; // Index into formula/clauses array, -1 if decision/none
};

// ============================================================================
// ReasoningCore: CDCL Engine with 2WL and VSIDS
// ============================================================================
class ReasoningCore {
public:
    int num_vars;
    std::vector<Clause> formula;

    // Assignment & Implication state
    std::vector<std::optional<VarInfo>> assignment;
    int decision_level{0};
    std::vector<Var> trail;
    size_t prop_head{0};

    // 2-Watched Literals: watch_map[Lit] -> list of clause indices
    std::unordered_map<Lit, std::vector<int>> watches;

    // VSIDS Heuristics
    std::vector<double> vsids_scores;
    double vsids_inc{1.0};
    double vsids_decay{0.95};

    // Search Metrics
    int conflicts{0};
    int decisions{0};

    ReasoningCore(const std::vector<Clause>& input_formula, int vars)
        : num_vars(vars), formula(input_formula) {
        assignment.resize(num_vars + 1, std::nullopt);
        vsids_scores.resize(num_vars + 1, 0.0);

        for (int i = 0; i < static_cast<int>(formula.size()); ++i) {
            add_watch(i);
        }
    }

    void add_watch(int clause_idx) {
        const Clause& c = formula[clause_idx];
        if (c.size() >= 2) {
            watches[c[0]].push_back(clause_idx);
            watches[c[1]].push_back(clause_idx);
        }
    }

    std::optional<std::unordered_map<Var, bool>> solve() {
        if (propagate_initial_units().has_value()) {
            return std::nullopt; // Direct root contradiction
        }

        while (true) {
            std::optional<Var> picked_var = pick_branching_variable();
            if (!picked_var.has_value()) {
                break; // All variables assigned -> SAT
            }

            decision_level++;
            decisions++;

            // Phase saving / default branch: false
            assign(picked_var.value(), false, -1);

            while (true) {
                std::optional<int> conflict_idx = unit_propagate();
                if (!conflict_idx.has_value()) {
                    break;
                }

                conflicts++;
                if (decision_level == 0) {
                    return std::nullopt; // Root conflict -> UNSAT
                }

                int backtrack_level = 0;
                Lit asserting_lit = 0;
                Clause learned_clause = analyze_conflict(conflict_idx.value(), backtrack_level, asserting_lit);

                // Update VSIDS scores
                for (Lit lit : learned_clause) {
                    vsids_scores[std::abs(lit)] += vsids_inc;
                }
                decay_scores();

                // Non-chronological backjump
                backtrack(backtrack_level);
                decision_level = backtrack_level;

                // Register learned clause
                int learned_idx = static_cast<int>(formula.size());
                formula.push_back(learned_clause);
                add_watch(learned_idx);
                assign(std::abs(asserting_lit), asserting_lit > 0, learned_idx);
            }
        }

        std::unordered_map<Var, bool> model;
        for (Var v = 1; v <= num_vars; ++v) {
            if (assignment[v].has_value()) {
                model[v] = assignment[v]->val;
            }
        }
        return model;
    }

private:
    void assign(Var var, bool value, int reason_idx) {
        assignment[var] = VarInfo{value, decision_level, reason_idx};
        trail.push_back(var);
    }

    std::optional<Var> pick_branching_variable() {
        Var best_var = -1;
        double max_s = -1.0;
        for (Var v = 1; v <= num_vars; ++v) {
            if (!assignment[v].has_value() && vsids_scores[v] > max_s) {
                max_s = vsids_scores[v];
                best_var = v;
            }
        }
        if (best_var == -1) return std::nullopt;
        return best_var;
    }

    std::optional<int> unit_propagate() {
        while (prop_head < trail.size()) {
            Var p = trail[prop_head++];
            bool p_val = assignment[p]->val;
            Lit falsified_lit = p_val ? -p : p;

            auto it = watches.find(falsified_lit);
            if (it == watches.end()) continue;

            std::vector<int> watch_list = std::move(it->second);
            it->second.clear();

            for (size_t idx = 0; idx < watch_list.size(); ++idx) {
                int c_idx = watch_list[idx];
                Clause& clause = formula[c_idx];

                if (clause[0] == falsified_lit) {
                    std::swap(clause[0], clause[1]);
                }

                // If first watched literal is satisfied, keep watch
                Var w0_var = std::abs(clause[0]);
                if (assignment[w0_var].has_value() &&
                    (clause[0] > 0) == assignment[w0_var]->val) {
                    watches[falsified_lit].push_back(c_idx);
                    continue;
                }

                // Find a replacement literal to watch
                bool found = false;
                for (size_t j = 2; j < clause.size(); ++j) {
                    Var lj_var = std::abs(clause[j]);
                    if (!assignment[lj_var].has_value() ||
                        (clause[j] > 0) == assignment[lj_var]->val) {
                        std::swap(clause[1], clause[j]);
                        watches[clause[1]].push_back(c_idx);
                        found = true;
                        break;
                    }
                }
                if (found) continue;

                // Clause is unit or conflicting
                watches[falsified_lit].push_back(c_idx);
                if (!assignment[w0_var].has_value()) {
                    assign(w0_var, clause[0] > 0, c_idx);
                } else if ((clause[0] > 0) != assignment[w0_var]->val) {
                    // Conflict detected: restore unvisited watchers
                    for (size_t rem = idx + 1; rem < watch_list.size(); ++rem) {
                        watches[falsified_lit].push_back(watch_list[rem]);
                    }
                    return c_idx;
                }
            }
        }
        return std::nullopt;
    }

    Clause analyze_conflict(int conflict_idx, int& backtrack_level, Lit& asserting_lit) {
        std::unordered_set<Lit> learned(formula[conflict_idx].begin(), formula[conflict_idx].end());

        while (true) {
            std::vector<Lit> level_lits;
            for (Lit l : learned) {
                Var v = std::abs(l);
                if (assignment[v].has_value() && assignment[v]->level == decision_level) {
                    level_lits.push_back(l);
                }
            }
            if (level_lits.size() <= 1) break;

            Var resolved_var = -1;
            for (auto rit = trail.rbegin(); rit != trail.rend(); ++rit) {
                Var v = *rit;
                bool match = false;
                for (Lit l : level_lits) {
                    if (std::abs(l) == v) { match = true; break; }
                }
                if (match) {
                    resolved_var = v;
                    break;
                }
            }

            int reason_idx = assignment[resolved_var]->reason;
            if (reason_idx == -1) break;

            Lit lit_to_remove = assignment[resolved_var]->val ? -resolved_var : resolved_var;
            learned.erase(lit_to_remove);

            for (Lit l : formula[reason_idx]) {
                if (std::abs(l) != resolved_var) {
                    learned.insert(l);
                }
            }
        }

        asserting_lit = 0;
        for (Lit l : learned) {
            if (assignment[std::abs(l)]->level == decision_level) {
                asserting_lit = l;
                break;
            }
        }

        backtrack_level = 0;
        for (Lit l : learned) {
            Var v = std::abs(l);
            if (assignment[v].has_value() && assignment[v]->level < decision_level) {
                backtrack_level = std::max(backtrack_level, assignment[v]->level);
            }
        }

        return Clause(learned.begin(), learned.end());
    }

    void backtrack(int target_level) {
        while (!trail.empty()) {
            Var v = trail.back();
            if (assignment[v]->level <= target_level) break;
            trail.pop_back();
            assignment[v] = std::nullopt;
        }
        prop_head = trail.size();
    }

    void decay_scores() {
        for (size_t v = 1; v <= static_cast<size_t>(num_vars); ++v) {
            vsids_scores[v] *= vsids_decay;
        }
    }

    std::optional<int> propagate_initial_units() {
        for (size_t i = 0; i < formula.size(); ++i) {
            if (formula[i].size() == 1) {
                Lit lit = formula[i][0];
                Var var = std::abs(lit);
                bool val = lit > 0;
                if (assignment[var].has_value() && assignment[var]->val != val) {
                    return static_cast<int>(i);
                }
                assign(var, val, static_cast<int>(i));
            }
        }
        return unit_propagate();
    }
};

// ============================================================================
// StructuralDSL: Constraint Formulation Layer
// ============================================================================
class StructuralDSL {
public:
    std::vector<Clause> formula;
    int var_count{0};
    std::unordered_map<std::string, Var> var_map;

    Var get_var(const std::string& key) {
        if (var_map.find(key) == var_map.end()) {
            var_count++;
            var_map[key] = var_count;
        }
        return var_map[key];
    }

    void add_at_least_one(const std::vector<Var>& variables) {
        Clause c;
        for (Var v : variables) c.push_back(v);
        formula.push_back(std::move(c));
    }

    void add_at_most_one(const std::vector<Var>& variables) {
        for (size_t i = 0; i < variables.size(); ++i) {
            for (size_t j = i + 1; j < variables.size(); ++j) {
                formula.push_back({-variables[i], -variables[j]});
            }
        }
    }

    void add_exactly_one(const std::vector<Var>& variables) {
        add_at_least_one(variables);
        add_at_most_one(variables);
    }
};

// ============================================================================
// SemanticAnalyzer: Graph & Topological Reduction Layer
// ============================================================================
class SemanticAnalyzer {
public:
    std::vector<int> nodes;
    std::unordered_map<int, std::unordered_set<int>> adj;

    SemanticAnalyzer(const std::vector<int>& n, const std::vector<std::pair<int, int>>& edges)
        : nodes(n) {
        for (const auto& [u, v] : edges) {
            adj[u].insert(v);
            adj[v].insert(u);
        }
    }

    std::vector<int> find_max_clique_estimate() {
        std::vector<int> clique;
        std::vector<int> sorted_nodes = nodes;

        std::sort(sorted_nodes.begin(), sorted_nodes.end(), [this](int a, int b) {
            size_t deg_a = adj.count(a) ? adj.at(a).size() : 0;
            size_t deg_b = adj.count(b) ? adj.at(b).size() : 0;
            return deg_a > deg_b;
        });

        for (int n : sorted_nodes) {
            bool connects_to_all = true;
            for (int m : clique) {
                if (adj[m].find(n) == adj[m].end()) {
                    connects_to_all = false;
                    break;
                }
            }
            if (connects_to_all) {
                clique.push_back(n);
            }
        }
        return clique;
    }
};

// ============================================================================
// Verification / Example Execution
// ============================================================================
int main() {
    StructuralDSL dsl;

    // Encode a 3-element pigeonhole-style choice constraint
    Var x1 = dsl.get_var("cell_1");
    Var x2 = dsl.get_var("cell_2");
    Var x3 = dsl.get_var("cell_3");

    dsl.add_exactly_one({x1, x2, x3});

    // Run CDCL Search
    ReasoningCore solver(dsl.formula, dsl.var_count);
    auto result = solver.solve();

    if (result.has_value()) {
        std::cout << "STATUS: SATISFIABLE\n";
        for (const auto& [var, val] : result.value()) {
            std::cout << "Var " << var << " = " << (val ? "TRUE" : "FALSE") << "\n";
        }
    } else {
        std::cout << "STATUS: UNSATISFIABLE\n";
    }

    // Graph Clique Test
    SemanticAnalyzer analyzer({1, 2, 3, 4}, {{1, 2}, {2, 3}, {3, 1}, {3, 4}});
    auto clique = analyzer.find_max_clique_estimate();
    std::cout << "Estimated Max Clique Size: " << clique.size() << " (Nodes: ";
    for (int node : clique) std::cout << node << " ";
    std::cout << ")\n";

    return 0;
}
