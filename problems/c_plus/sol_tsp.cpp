#include <iostream>
#include <vector>
#include <string>
#include <chrono>
#include <iomanip>
#include <algorithm>
#include <cmath>

#include "ReasoningKit.hpp"

struct TSPSolver {
    int n;
    std::vector<std::vector<int>> dist;

    TSPSolver(int cities, const std::vector<std::vector<int>>& distance_matrix)
        : n(cities), dist(distance_matrix) {}

    // Builds the SAT constraints for a tour bounded by a max per-edge weight threshold
    std::optional<std::vector<int>> solve_bounded(int max_edge_weight) {
        StructuralDSL dsl;

        auto step_var = [&](int city, int t) {
            return dsl.get_var("c_" + std::to_string(city) + "_t_" + std::to_string(t));
        };

        // 1. Time Uniqueness: Exactly one city visited at each step t
        for (int t = 0; t < n; ++t) {
            std::vector<Var> cities_at_t;
            cities_at_t.reserve(n);
            for (int i = 0; i < n; ++i) {
                cities_at_t.push_back(step_var(i, t));
            }
            dsl.add_exactly_one(cities_at_t);
        }

        // 2. Space Uniqueness: Each city is visited at exactly one step t
        for (int i = 0; i < n; ++i) {
            std::vector<Var> times_for_i;
            times_for_i.reserve(n);
            for (int t = 0; t < n; ++t) {
                times_for_i.push_back(step_var(i, t));
            }
            dsl.add_exactly_one(times_for_i);
        }

        // 3. Symmetry Breaking: Fix City 0 at Step 0 (eliminates N cyclic shifts)
        dsl.formula.push_back({step_var(0, 0)});

        // 4. Edge Pruning: Forbid consecutive transitions if dist[u][v] > max_edge_weight
        for (int u = 0; u < n; ++u) {
            for (int v = 0; v < n; ++v) {
                if (u == v) continue;
                if (dist[u][v] > max_edge_weight) {
                    for (int t = 0; t < n; ++t) {
                        int next_t = (t + 1) % n;
                        // Prohibit u at step t followed by v at step t+1: ~(u_t AND v_{t+1})
                        dsl.formula.push_back({-step_var(u, t), -step_var(v, next_t)});
                    }
                }
            }
        }

        // 5. Deduction Execution
        ReasoningCore solver(dsl.formula, dsl.var_count);
        auto result = solver.solve();

        if (!result.has_value()) {
            return std::nullopt;
        }

        // Decode the permutation tour from assignments
        const auto& model = result.value();
        std::vector<int> tour(n, -1);
        for (int t = 0; t < n; ++t) {
            for (int i = 0; i < n; ++i) {
                Var vid = step_var(i, t);
                auto it = model.find(vid);
                if (it != model.end() && it->second) {
                    tour[t] = i;
                    break;
                }
            }
        }
        return tour;
    }

    // Finds the optimal minimax edge tour via binary search
    void optimize() {
        std::cout << "--- [TSP] Initiating Structural CDCL Solver (" << n << " Cities) ---" << std::endl;

        // Collect and sort all distinct edge weights
        std::vector<int> weights;
        for (int i = 0; i < n; ++i) {
            for (int j = 0; j < n; ++j) {
                if (i != j) weights.push_back(dist[i][j]);
            }
        }
        std::sort(weights.begin(), weights.end());
        weights.erase(std::unique(weights.begin(), weights.end()), weights.end());

        int low = 0, high = static_cast<int>(weights.size()) - 1;
        int best_weight = -1;
        std::vector<int> best_tour;

        auto start = std::chrono::high_resolution_clock::now();

        while (low <= high) {
            int mid = low + (high - low) / 2;
            int threshold = weights[mid];

            std::cout << "  Testing Edge Bottleneck Threshold <= " << threshold << "... ";
            auto res = solve_bounded(threshold);

            if (res.has_value()) {
                std::cout << "[FEASIBLE]\n";
                best_weight = threshold;
                best_tour = res.value();
                high = mid - 1; // Try to tighten threshold
            } else {
                std::cout << "[INFEASIBLE]\n";
                low = mid + 1;  // Relax threshold
            }
        }

        auto end = std::chrono::high_resolution_clock::now();
        std::chrono::duration<double> duration = end - start;

        std::cout << "\n--- Optimization Summary ---\n";
        if (best_tour.empty()) {
            std::cout << "No valid Hamiltonian cycle exists.\n";
            return;
        }

        int total_cost = 0;
        std::cout << "Tour: ";
        for (int t = 0; t < n; ++t) {
            int from = best_tour[t];
            int to = best_tour[(t + 1) % n];
            total_cost += dist[from][to];
            std::cout << from << " -> ";
        }
        std::cout << best_tour[0] << "\n";
        std::cout << "Max Edge in Tour: " << best_weight << "\n";
        std::cout << "Total Tour Length: " << total_cost << "\n";
        std::cout << std::fixed << std::setprecision(4);
        std::cout << "Optimization Duration: " << duration.count() << " seconds\n";
    }
};

int main() {
    // Distance matrix for a 6-city benchmark problem
    std::vector<std::vector<int>> distance_matrix = {
        { 0, 10, 15, 20, 25, 30 },
        { 10,  0, 35, 25, 18, 22 },
        { 15, 35,  0, 30, 28, 12 },
        { 20, 25, 30,  0, 14, 16 },
        { 25, 18, 28, 14,  0, 24 },
        { 30, 22, 12, 16, 24,  0 }
    };

    TSPSolver tsp(6, distance_matrix);
    tsp.optimize();

    return 0;
}
