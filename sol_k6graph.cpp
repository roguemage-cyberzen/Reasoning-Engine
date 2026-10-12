#include <iostream>
#include <vector>
#include <utility>
#include <string>

#include "ReasoningKit.hpp"

std::string run_k6_coloring(int colors) {
    std::cout << "--- [K6] Adjacency Test: " << colors << " Colors ---" << std::endl;

    // Complete Graph K6: 6 vertices (0..5)
    std::vector<int> nodes = {0, 1, 2, 3, 4, 5};
    std::vector<std::pair<int, int>> edges;

    // Generate all pairwise edges for K6: 6 * 5 / 2 = 15 edges
    for (size_t i = 0; i < nodes.size(); ++i) {
        for (size_t j = i + 1; j < nodes.size(); ++j) {
            edges.emplace_back(nodes[i], nodes[j]);
        }
    }

    // 1. Semantic Analysis (Topological Invariant Short-circuit)
    SemanticAnalyzer analyzer(nodes, edges);
    std::vector<int> clique = analyzer.find_max_clique_estimate();

    std::cout << "  Topological Scan: Max Clique size " << clique.size() << " identified." << std::endl;

    // Chromatic number lower bound: chi(G) >= omega(G)
    if (static_cast<int>(clique.size()) > colors) {
        std::cout << "  Semantic Result: UNSATISFIABLE (Topological Proof)." << std::endl;
        return "UNSAT";
    }

    // 2. Logic Core Fallback (if clique <= colors)
    std::cout << "  Semantic check passed. Transitioning to Search..." << std::endl;

    StructuralDSL dsl;

    // Variable mapping helper: node n assigned color c
    auto color_var = [&](int n, int c) {
        return dsl.get_var("node_" + std::to_string(n) + "_color_" + std::to_string(c));
    };

    // Constraint A: Each node gets at least one color
    for (int n : nodes) {
        std::vector<Var> node_colors;
        node_colors.reserve(colors);
        for (int c = 0; c < colors; ++c) {
            node_colors.push_back(color_var(n, c));
        }
        dsl.add_at_least_one(node_colors);
    }

    // Constraint B: Adjacent nodes cannot share the same color
    for (const auto& [u, v] : edges) {
        for (int c = 0; c < colors; ++c) {
            dsl.formula.push_back({-color_var(u, c), -color_var(v, c)});
        }
    }

    ReasoningCore solver(dsl.formula, dsl.var_count);
    auto result = solver.solve();

    if (result.has_value()) {
        std::cout << "  Core Search Result: SATISFIABLE." << std::endl;
        return "SAT";
    } else {
        std::cout << "  Core Search Result: UNSATISFIABLE." << std::endl;
        return "UNSAT";
    }
}

int main() {
    // 5 colors on K6: clique size 6 > 5 -> triggers immediate UNSAT short-circuit
    run_k6_coloring(5);

    std::cout << std::endl;

    // 6 colors on K6: passes semantic check and proves SAT via ReasoningCore
    run_k6_coloring(6);

    return 0;
}
