#include <iostream>
#include <vector>
#include <string>
#include <chrono>
#include <iomanip>

#include "ReasoningKit.hpp"

void run_php_experiment(int pigeons, int holes) {
    std::cout << "--- [PHP] Starting Experiment: " 
              << pigeons << " pigeons, " << holes << " holes ---" << std::endl;

    StructuralDSL dsl;

    // Helper lambda for relational mapping: "p_i_h_j"
    auto p_var = [&](int i, int j) {
        return dsl.get_var("p_" + std::to_string(i) + "_h_" + std::to_string(j));
    };

    // 1. Relational Mapping: Ensure every pigeon has exactly one hole
    for (int i = 0; i < pigeons; ++i) {
        std::vector<Var> p_vars;
        p_vars.reserve(holes);
        for (int j = 0; j < holes; ++j) {
            p_vars.push_back(p_var(i, j));
        }
        dsl.add_exactly_one(p_vars);
    }

    // 2. Symmetry Identification & Hole Conflict: No two pigeons share a hole
    for (int j = 0; j < holes; ++j) {
        std::vector<Var> hole_occupants;
        hole_occupants.reserve(pigeons);
        for (int i = 0; i < pigeons; ++i) {
            hole_occupants.push_back(p_var(i, j));
        }
        dsl.add_at_most_one(hole_occupants);
    }

    // 3. Symmetry Breaking (Structural Reduction):
    // Fix Pigeon 0 into Hole 0 to break the initial N! orbital permutation symmetry
    Var p0_h0 = p_var(0, 0);
    dsl.formula.push_back({p0_h0});

    std::cout << "  Logic database size: " << dsl.formula.size() 
              << " clauses | Variables: " << dsl.var_count << std::endl;

    // 4. Search Execution & Benchmarking
    ReasoningCore solver(dsl.formula, dsl.var_count);

    auto start = std::chrono::high_resolution_clock::now();
    auto result = solver.solve();
    auto end = std::chrono::high_resolution_clock::now();

    std::chrono::duration<double> duration = end - start;

    std::cout << "  Status: " << (result.has_value() ? "Satisfied" : "Unsatisfiable") << std::endl;
    std::cout << std::fixed << std::setprecision(6);
    std::cout << "  Search Duration: " << duration.count() << " seconds" << std::endl;
    std::cout << "  Decisions: " << solver.decisions 
              << " | Conflicts: " << solver.conflicts << std::endl;
}

int main() {
    // Classic PHP benchmark: 10 pigeons into 9 holes (UNSAT)
    run_php_experiment(10, 9);
    return 0;
}
