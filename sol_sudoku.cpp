#include <iostream>
#include <vector>
#include <array>
#include <chrono>
#include <iomanip>
#include <optional>

// Include or link ReasoningKit classes here:
#include "ReasoningKit.hpp" 
// (Ensure ReasoningCore and StructuralDSL are declared/included)

class SudokuSolver {
public:
    using Grid = std::array<std::array<int, 9>, 9>;

    static std::optional<Grid> solve(const Grid& initial_grid) {
        std::cout << "--- [Sudoku] Initiating 9x9 Solver ---" << std::endl;
        StructuralDSL dsl;

        // Helper lambda for consistent 3D bit-cube string keys
        auto cell_var = [&](int r, int c, int v) {
            return dsl.get_var("cell_" + std::to_string(r) + "_" + 
                               std::to_string(c) + "_val_" + std::to_string(v));
        };

        // 1. 3D Bit-Cube Mapping: Each cell (r, c) contains exactly one digit [1..9]
        for (int r = 0; r < 9; ++r) {
            for (int c = 0; c < 9; ++c) {
                std::vector<Var> cell_digits;
                cell_digits.reserve(9);
                for (int v = 1; v <= 9; ++v) {
                    cell_digits.push_back(cell_var(r, c, v));
                }
                dsl.add_exactly_one(cell_digits);
            }
        }

        // 2. Constraint Pipeline (Row, Column, and 3x3 Block Uniqueness)
        for (int v = 1; v <= 9; ++v) {
            // Row uniqueness: Digit v appears at most once per row
            for (int r = 0; r < 9; ++r) {
                std::vector<Var> row_vars;
                row_vars.reserve(9);
                for (int c = 0; c < 9; ++c) {
                    row_vars.push_back(cell_var(r, c, v));
                }
                dsl.add_at_most_one(row_vars);
            }

            // Column uniqueness: Digit v appears at most once per col
            for (int c = 0; c < 9; ++c) {
                std::vector<Var> col_vars;
                col_vars.reserve(9);
                for (int r = 0; r < 9; ++r) {
                    col_vars.push_back(cell_var(r, c, v));
                }
                dsl.add_at_most_one(col_vars);
            }

            // 3x3 Block uniqueness: Digit v appears at most once per block
            for (int br = 0; br < 3; ++br) {
                for (int bc = 0; bc < 3; ++bc) {
                    std::vector<Var> block_vars;
                    block_vars.reserve(9);
                    for (int dr = 0; dr < 3; ++dr) {
                        for (int dc = 0; dc < 3; ++dc) {
                            block_vars.push_back(cell_var(br * 3 + dr, bc * 3 + dc, v));
                        }
                    }
                    dsl.add_at_most_one(block_vars);
                }
            }
        }

        // 3. Unit Clue Injection
        for (int r = 0; r < 9; ++r) {
            for (int c = 0; c < 9; ++c) {
                int clue = initial_grid[r][c];
                if (clue >= 1 && clue <= 9) {
                    dsl.formula.push_back({cell_var(r, c, clue)});
                }
            }
        }

        std::cout << "  Clauses generated: " << dsl.formula.size() 
                  << " | Variables: " << dsl.var_count << std::endl;

        // 4. Deductive Search Execution & Benchmarking
        ReasoningCore solver(dsl.formula, dsl.var_count);

        auto start = std::chrono::high_resolution_clock::now();
        auto result = solver.solve();
        auto end = std::chrono::high_resolution_clock::now();

        std::chrono::duration<double> duration = end - start;
        std::cout << std::fixed << std::setprecision(4);
        std::cout << "  Puzzle solved in " << duration.count() << " seconds." << std::endl;
        std::cout << "  Decisions: " << solver.decisions 
                  << " | Conflicts: " << solver.conflicts << std::endl;

        if (!result.has_value()) {
            return std::nullopt;
        }

        // Decode assignment back into a 9x9 grid
        const auto& model = result.value();
        Grid solved_grid{};
        for (int r = 0; r < 9; ++r) {
            for (int c = 0; c < 9; ++c) {
                for (int v = 1; v <= 9; ++v) {
                    Var var_id = cell_var(r, c, v);
                    auto it = model.find(var_id);
                    if (it != model.end() && it->second) {
                        solved_grid[r][c] = v;
                        break;
                    }
                }
            }
        }

        return solved_grid;
    }

    static void printGrid(const Grid& g) {
        for (int r = 0; r < 9; ++r) {
            if (r > 0 && r % 3 == 0) {
                std::cout << "------+-------+------\n";
            }
            for (int c = 0; c < 9; ++c) {
                if (c > 0 && c % 3 == 0) std::cout << "| ";
                std::cout << g[r][c] << " ";
            }
            std::cout << "\n";
        }
    }
};

int main() {
    // Benchmark Grid (Classic 'Easy' to benchmark propagation rate)
    SudokuSolver::Grid board = {{
        {5, 3, 0,  0, 7, 0,  0, 0, 0},
        {6, 0, 0,  1, 9, 5,  0, 0, 0},
        {0, 9, 8,  0, 0, 0,  0, 6, 0},

        {8, 0, 0,  0, 6, 0,  0, 0, 3},
        {4, 0, 0,  8, 0, 3,  0, 0, 1},
        {7, 0, 0,  0, 2, 0,  0, 0, 6},

        {0, 6, 0,  0, 0, 0,  2, 8, 0},
        {0, 0, 0,  4, 1, 9,  0, 0, 5},
        {0, 0, 0,  0, 8, 0,  0, 7, 9}
    }};

    auto solution = SudokuSolver::solve(board);

    if (solution.has_value()) {
        std::cout << "\n--- Solution Matrix ---\n";
        SudokuSolver::printGrid(solution.value());
    } else {
        std::cout << "\nSTATUS: Puzzle is UNSATISFIABLE.\n";
    }

    return 0;
}
