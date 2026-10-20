use eframe::egui;
use reasoningkit::{lit_is_pos, lit_to_var, Lit, ReasoningCore, StructuralDSL};
use std::collections::HashMap;
use std::time::Instant;

#[derive(PartialEq, Clone, Copy)]
enum ProblemArchetype {
    PigeonholePrinciple,
    TravelingSalesperson,
}

#[derive(PartialEq)]
enum ActiveTab {
    PlainTextRules,
    DimacsCnf,
    VariableTable,
}

struct TrailEntry {
    step: usize,
    level: usize,
    var: usize,
    entity: String,
    value: bool,
    inference: &'static str,
}

pub struct ReasoningEngineApp {
    problem_type: ProblemArchetype,
    active_tab: ActiveTab,

    // PHP Inputs
    pigeons: usize,
    holes: usize,
    enable_sym_breaking: bool,

    // TSP Inputs
    n_cities: usize,
    max_edge: usize,

    // Generated Rules State
    rule_descriptions: Vec<String>,
    cnf_output: String,
    var_table: Vec<(usize, String)>,
    formula: Vec<Vec<Lit>>,
    var_count: usize,
    var_map: HashMap<String, usize>,

    // Solver Result State
    status: Option<bool>,
    duration_secs: f64,
    decisions: usize,
    conflicts: usize,
    trail_log: Vec<TrailEntry>,
}

impl Default for ReasoningEngineApp {
    fn default() -> Self {
        let mut app = Self {
            problem_type: ProblemArchetype::PigeonholePrinciple,
            active_tab: ActiveTab::PlainTextRules,
            pigeons: 4,
            holes: 3,
            enable_sym_breaking: true,
            n_cities: 4,
            max_edge: 25,
            rule_descriptions: Vec::new(),
            cnf_output: String::new(),
            var_table: Vec::new(),
            formula: Vec::new(),
            var_count: 0,
            var_map: HashMap::new(),
            status: None,
            duration_secs: 0.0,
            decisions: 0,
            conflicts: 0,
            trail_log: Vec::new(),
        };
        app.rebuild_problem();
        app
    }
}

impl ReasoningEngineApp {
    fn rebuild_problem(&mut self) {
        let mut dsl = StructuralDSL::new();
        self.rule_descriptions.clear();

        match self.problem_type {
            ProblemArchetype::PigeonholePrinciple => {
                for i in 0..self.pigeons {
                    let mut p_vars = Vec::with_capacity(self.holes);
                    for j in 0..self.holes {
                        p_vars.push(dsl.get_var(format!("p_{}_h_{}", i, j)));
                    }
                    dsl.add_exactly_one(&p_vars);
                    self.rule_descriptions.push(format!(
                        "Pigeon {} must occupy exactly one hole: {}",
                        i,
                        p_vars.iter().map(|v| format!("p_{}", v)).collect::<Vec<_>>().join(" ∨ ")
                    ));
                }

                for j in 0..self.holes {
                    let mut hole_occupants = Vec::with_capacity(self.pigeons);
                    for i in 0..self.pigeons {
                        hole_occupants.push(dsl.get_var(format!("p_{}_h_{}", i, j)));
                    }
                    dsl.add_at_most_one(&hole_occupants);
                    self.rule_descriptions.push(format!("Hole {} holds at most one pigeon.", j));
                }

                if self.enable_sym_breaking {
                    let p0_h0 = dsl.get_var("p_0_h_0");
                    dsl.formula.push(vec![p0_h0 as Lit]);
                    self.rule_descriptions.push("Symmetry Breaking Anchor: Force Pigeon 0 into Hole 0.".into());
                }
            }
            ProblemArchetype::TravelingSalesperson => {
                for t in 0..self.n_cities {
                    let mut slot_vars = Vec::with_capacity(self.n_cities);
                    for c in 0..self.n_cities {
                        slot_vars.push(dsl.get_var(format!("c_{}_t_{}", c, t)));
                    }
                    dsl.add_exactly_one(&slot_vars);
                    self.rule_descriptions.push(format!("Time slot {} must contain exactly one city.", t));
                }

                for c in 0..self.n_cities {
                    let mut city_vars = Vec::with_capacity(self.n_cities);
                    for t in 0..self.n_cities {
                        city_vars.push(dsl.get_var(format!("c_{}_t_{}", c, t)));
                    }
                    dsl.add_exactly_one(&city_vars);
                    self.rule_descriptions.push(format!("City {} must be visited at exactly one step.", c));
                }

                let c0_t0 = dsl.get_var("c_0_t_0");
                dsl.formula.push(vec![c0_t0 as Lit]);
                self.rule_descriptions.push("Symmetry Breaking: Anchor City 0 at Time 0.".into());
            }
        }

        // DIMACS CNF representation
        let mut cnf = format!("p cnf {} {}\n", dsl.var_count, dsl.formula.len());
        for clause in &dsl.formula {
            for &lit in clause {
                cnf.push_str(&format!("{} ", lit));
            }
            cnf.push_str("0\n");
        }
        self.cnf_output = cnf;

        // Inverted variable table
        let mut sorted_vars: Vec<(usize, String)> = dsl.var_map.iter().map(|(k, v)| (*v, k.clone())).collect();
        sorted_vars.sort_by_key(|&(id, _)| id);
        self.var_table = sorted_vars;

        self.var_count = dsl.var_count;
        self.var_map = dsl.var_map;
        self.formula = dsl.formula;
    }

    fn run_solver(&mut self) {
        let mut solver = ReasoningCore::new(self.formula.clone(), self.var_count);
        let start = Instant::now();
        let result = solver.solve();
        self.duration_secs = start.elapsed().as_secs_f64();

        self.status = Some(result.is_some());
        self.decisions = solver.decisions;
        self.conflicts = solver.conflicts;

        // Invert variable map: ID -> Name
        let id_to_name: HashMap<usize, String> =
            self.var_map.iter().map(|(k, v)| (*v, k.clone())).collect();

        self.trail_log.clear();
        for (step, &var) in solver.trail.iter().enumerate() {
            let entity_name = id_to_name.get(&var).cloned().unwrap_or_else(|| format!("Var {}", var));
            if let Some(info) = solver.assignment.get(&var) {
                self.trail_log.push(TrailEntry {
                    step: step + 1,
                    level: info.level,
                    var,
                    entity: entity_name,
                    value: info.value,
                    inference: if info.reason.is_some() {
                        "Unit Propagation (2WL)"
                    } else {
                        "Branch Decision"
                    },
                });
            }
        }
    }
}

impl eframe::App for ReasoningEngineApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Left Control Panel (Sidebar equivalent)
        egui::SidePanel::left("control_panel").resizable(true).default_width(320.0).show(ctx, |ui| {
            ui.heading("1. Problem Configuration");
            ui.add_space(8.0);

            let prev_type = self.problem_type;
            ui.label("Archetype:");
            ui.radio_value(&mut self.problem_type, ProblemArchetype::PigeonholePrinciple, "Pigeonhole Principle (PHP)");
            ui.radio_value(&mut self.problem_type, ProblemArchetype::TravelingSalesperson, "Traveling Salesperson (TSP)");

            let mut changed = prev_type != self.problem_type;
            ui.separator();

            match self.problem_type {
                ProblemArchetype::PigeonholePrinciple => {
                    ui.label("Pigeons:");
                    changed |= ui.add(egui::Slider::new(&mut self.pigeons, 2..=20)).changed();

                    ui.label("Holes:");
                    changed |= ui.add(egui::Slider::new(&mut self.holes, 1..=19)).changed();

                    changed |= ui.checkbox(&mut self.enable_sym_breaking, "Enable Orbit Collapsing (Symmetry Breaking)").changed();
                }
                ProblemArchetype::TravelingSalesperson => {
                    ui.label("Number of Cities:");
                    changed |= ui.add(egui::Slider::new(&mut self.n_cities, 3..=8)).changed();

                    ui.label("Max Edge Bottleneck:");
                    changed |= ui.add(egui::Slider::new(&mut self.max_edge, 5..=50)).changed();
                }
            }

            if changed {
                self.rebuild_problem();
            }

            ui.add_space(16.0);
            if ui.button("🚀 Run Deductive Solver").clicked() {
                self.run_solver();
            }
        });

        // Main Central View
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Reasoning Engine: Structural Complexity Reduction");
            ui.separator();

            // Tabs for Inspection & Print Output
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, ActiveTab::PlainTextRules, "Structural Rules");
                ui.selectable_value(&mut self.active_tab, ActiveTab::DimacsCnf, "DIMACS CNF");
                ui.selectable_value(&mut self.active_tab, ActiveTab::VariableTable, "Variable Table");
            });

            match self.active_tab {
                ActiveTab::PlainTextRules => {
                    let mut text = self.rule_descriptions.join("\n");
                    ui.add(egui::TextEdit::multiline(&mut text).desired_rows(8).desired_width(f32::INFINITY));
                }
                ActiveTab::DimacsCnf => {
                    let mut text = self.cnf_output.clone();
                    ui.add(egui::TextEdit::multiline(&mut text).desired_rows(8).desired_width(f32::INFINITY));
                }
                ActiveTab::VariableTable => {
                    egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                        egui::Grid::new("var_grid").striped(true).show(ui, |ui| {
                            ui.strong("ID");
                            ui.strong("Canonical Concept");
                            ui.end_row();
                            for (id, concept) in &self.var_table {
                                ui.label(id.to_string());
                                ui.label(concept);
                                ui.end_row();
                            }
                        });
                    });
                }
            }

            ui.separator();
            ui.heading("Execution Metrics");

            ui.horizontal(|ui| {
                let status_text = match self.status {
                    Some(true) => "SATISFIABLE",
                    Some(false) => "UNSATISFIABLE",
                    None => "Ready",
                };
                ui.label(format!("Status: {}", status_text));
                ui.separator();
                ui.label(format!("Duration: {:.6} s", self.duration_secs));
                ui.separator();
                ui.label(format!("Decisions: {}", self.decisions));
                ui.separator();
                ui.label(format!("Conflicts: {}", self.conflicts));
            });

            ui.separator();
            ui.heading("📍 Deduction Node Path (Trail)");

            egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                egui::Grid::new("trail_grid").striped(true).min_col_width(80.0).show(ui, |ui| {
                    ui.strong("Step");
                    ui.strong("Level");
                    ui.strong("ID");
                    ui.strong("Entity State");
                    ui.strong("Assigned");
                    ui.strong("Inference Type");
                    ui.end_row();

                    for entry in &self.trail_log {
                        ui.label(entry.step.to_string());
                        ui.label(entry.level.to_string());
                        ui.label(entry.var.to_string());
                        ui.label(&entry.entity);
                        ui.label(if entry.value { "TRUE" } else { "FALSE" });
                        ui.label(entry.inference);
                        ui.end_row();
                    }
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Reasoning Engine GUI",
        options,
        Box::new(|_cc| Box::new(ReasoningEngineApp::default())),
    )
}
