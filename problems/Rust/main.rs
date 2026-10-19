use reasoningkit::{ReasoningCore, SemanticAnalyzer, StructuralDSL};
use std::time::Instant;

fn main() {
    println!("=== Testing ReasoningKit in Rust ===");

    // 1. DSL Test: Exactly-One Constraint over 3 variables
    let mut dsl = StructuralDSL::new();
    let x1 = dsl.get_var("cell_1");
    let x2 = dsl.get_var("cell_2");
    let x3 = dsl.get_var("cell_3");

    dsl.add_exactly_one(&[x1, x2, x3]);

    println!(
        "DSL Generated: {} clauses, {} variables",
        dsl.formula.len(),
        dsl.var_count
    );

    // 2. CDCL Search Execution
    let mut solver = ReasoningCore::new(dsl.formula, dsl.var_count);
    let start = Instant::now();
    let result = solver.solve();
    let duration = start.elapsed();

    match result {
        Some(model) => {
            println!("Status: SATISFIABLE (solved in {:?})", duration);
            for v in 1..=3 {
                println!("  Var {} = {}", v, model[&v]);
            }
        }
        None => println!("Status: UNSATISFIABLE"),
    }

    // 3. Semantic Layer Test (K4 Graph Clique)
    let nodes = vec![1, 2, 3, 4];
    let edges = vec![(1, 2), (2, 3), (3, 1), (3, 4), (1, 4), (2, 4)];
    let analyzer = SemanticAnalyzer::new(nodes, &edges);
    let clique = analyzer.find_max_clique_estimate();

    println!("Semantic Topological Check: Max Clique Size = {}", clique.len());
    println!("Clique Nodes: {:?}", clique);
}
