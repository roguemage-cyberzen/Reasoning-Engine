import time
from ReasoningKit import StructuralDSL, ReasoningCore

class TSPSolver:
    def __init__(self, cities, distance_matrix):
        self.n = cities
        self.dist = distance_matrix

    def solve_bounded(self, max_edge_weight):
        """Builds the SAT constraints for a tour bounded by a max per-edge weight threshold."""
        dsl = StructuralDSL()

        def step_var(city, t):
            return dsl.get_var(f"c_{city}_t_{t}")

        # 1. Time Uniqueness: Exactly one city visited at each step t
        for t in range(self.n):
            cities_at_t = [step_var(i, t) for i in range(self.n)]
            dsl.add_exactly_one(cities_at_t)

        # 2. Space Uniqueness: Each city is visited at exactly one step t
        for i in range(self.n):
            times_for_i = [step_var(i, t) for t in range(self.n)]
            dsl.add_exactly_one(times_for_i)

        # 3. Symmetry Breaking: Fix City 0 at Step 0 (eliminates N cyclic shifts)
        dsl.formula.append([step_var(0, 0)])

        # 4. Edge Pruning: Forbid consecutive transitions if dist[u][v] > max_edge_weight
        for u in range(self.n):
            for v in range(self.n):
                if u == v:
                    continue
                if self.dist[u][v] > max_edge_weight:
                    for t in range(self.n):
                        next_t = (t + 1) % self.n
                        # Prohibit u at step t followed by v at step t+1: ~(u_t AND v_{t+1})
                        dsl.formula.append([-step_var(u, t), -step_var(v, next_t)])

        # 5. Deduction Execution
        solver = ReasoningCore(dsl.formula, dsl.var_count)
        result = solver.solve()

        if result is None:
            return None

        # Decode the permutation tour from assignments
        tour = [-1] * self.n
        for t in range(self.n):
            for i in range(self.n):
                vid = step_var(i, t)
                if result.get(vid, False):
                    tour[t] = i
                    break
        return tour

    def optimize(self):
        """Finds the optimal minimax edge tour via binary search."""
        print(f"--- [TSP] Initiating Structural CDCL Solver ({self.n} Cities) ---")

        # Collect and sort all distinct edge weights
        weights = sorted({
            self.dist[i][j]
            for i in range(self.n)
            for j in range(self.n)
            if i != j
        })

        low = 0
        high = len(weights) - 1
        best_weight = -1
        best_tour = None

        start_time = time.perf_counter()

        while low <= high:
            mid = low + (high - low) // 2
            threshold = weights[mid]

            print(f"  Testing Edge Bottleneck Threshold <= {threshold}... ", end="", flush=True)
            res = self.solve_bounded(threshold)

            if res is not None:
                print("[FEASIBLE]")
                best_weight = threshold
                best_tour = res
                high = mid - 1  # Try to tighten threshold
            else:
                print("[INFEASIBLE]")
                low = mid + 1   # Relax threshold

        duration = time.perf_counter() - start_time

        print("\n--- Optimization Summary ---")
        if best_tour is None:
            print("No valid Hamiltonian cycle exists.")
            return

        total_cost = 0
        print("Tour: ", end="")
        for t in range(self.n):
            u = best_tour[t]
            v = best_tour[(t + 1) % self.n]
            total_cost += self.dist[u][v]
            print(f"{u} -> ", end="")
        print(best_tour[0])

        print(f"Max Edge in Tour: {best_weight}")
        print(f"Total Tour Length: {total_cost}")
        print(f"Optimization Duration: {duration:.4f} seconds")


if __name__ == "__main__":
    # Distance matrix for a 6-city benchmark problem
    distance_matrix = [
        [0, 10, 15, 20, 25, 30],
        [10,  0, 35, 25, 18, 22],
        [15, 35,  0, 30, 28, 12],
        [20, 25, 30,  0, 14, 16],
        [25, 18, 28, 14,  0, 24],
        [30, 22, 12, 16, 24,  0]
    ]

    tsp = TSPSolver(6, distance_matrix)
    tsp.optimize()
