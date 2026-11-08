use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

// =====================================================================
// PART 1: DOMAIN PRIMITIVES (CLONAL SIGNALING & TEMPORAL PHENOTYPES)
// =====================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CypRole {
    None,
    Substrate,
    Inhibitor,
    Inducer,
}

#[derive(Clone, Debug)]
pub struct CypProfile {
    pub cyp3a4: CypRole,
    pub cyp2d6: CypRole,
}

#[derive(Clone, Debug)]
pub struct OrganToxicity {
    pub hepatic: usize,
    pub cardiac: usize,
    pub renal: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegulatoryStatus {
    FdaOnLabel,
    FdaOffLabel,
    ClinicalTrial,
}

#[derive(Clone, Debug)]
pub struct Drug {
    pub id: String,
    pub name: String,
    pub targets: HashSet<String>,
    pub tox: OrganToxicity,
    pub cyp: CypProfile,
    pub regulatory: RegulatoryStatus,
    pub trial_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SomaticMutation {
    pub gene: String,
    pub variant: String,
    pub vaf: f64,
}

#[derive(Clone, Debug)]
pub struct Subclone {
    pub id: usize,
    pub name: String,
    pub vaf: f64,
    pub drivers: HashSet<String>,
}

#[derive(Clone, Debug)]
pub struct ClonalTumorProfile {
    pub name: String,
    pub nodes: HashSet<String>,
    pub edges: Vec<(String, String)>,
    pub subclones: Vec<Subclone>,
    pub targets: HashSet<String>,
}

#[derive(Clone, Debug)]
pub struct SolutionRegimen {
    pub rank: usize,
    pub drugs: Vec<Drug>,
    pub tot_hepatic: usize,
    pub tot_cardiac: usize,
    pub tot_renal: usize,
    pub total_toxicity_score: usize,
    pub inhibited_checkpoints: HashSet<String>,
    pub covered_subclones: usize,
    pub requires_clinical_trial: bool,
    pub has_off_label: bool,
}

#[derive(Clone, Debug)]
pub struct CollateralSensitivityRule {
    pub primary_drug_id: String,
    pub sensitized_drug_id: String,
    pub target_subclone_driver: String,
    pub mechanism: String,
}

#[derive(Clone, Debug)]
pub struct TemporalScheduleResult {
    pub num_cycles: usize,
    pub cycle_drugs: Vec<Vec<Drug>>,
    pub cycle_toxicities: Vec<(usize, usize, usize)>,
    pub subclone_trajectories: HashMap<String, Vec<bool>>,
    pub collateral_switches_triggered: Vec<String>,
    pub elapsed_micros: u128,
}

// Complexity & Spectral Phase-Transition Metric Bundle
#[derive(Clone, Debug, Default)]
pub struct SpectralComplexityMetric {
    pub domain_label: String,
    pub num_vars: usize,
    pub num_clauses: usize,
    pub alpha: f64,              // Clauses / Variables
    pub alpha_critical: f64,     // Canonical 3-SAT threshold ~ 4.267
    pub raw_search_space_log2: f64, // n
    pub resolved_steps: usize,
    pub reduction_s: f64,        // s = n - log2(steps)
    pub compression_ratio: f64,  // s / n * 100%
    pub regime: String,
}

pub struct CanonicalOncologyKnowledgeBase {
    pub all_edges: Vec<(&'static str, &'static str)>,
    pub known_genes: HashSet<&'static str>,
}

impl CanonicalOncologyKnowledgeBase {
    pub fn new() -> Self {
        let all_edges = vec![
            ("EGFR", "KRAS"), ("EGFR", "PI3K"),
            ("MET", "PI3K"), ("MET", "KRAS"),
            ("KRAS", "BRAF"), ("KRAS", "PI3K"),
            ("BRAF", "MEK"), ("CRAF", "MEK"),
            ("MEK", "ERK"),
            ("ERK", "PROLIFERATION"), ("ERK", "CDK4"), ("ERK", "MYC"),
            ("PI3K", "AKT"),
            ("AKT", "MTOR"), ("AKT", "PROLIFERATION"),
            ("MTOR", "PROLIFERATION"),
            ("CDK4", "RB1"), ("RB1", "PROLIFERATION"),
            ("MYC", "PROLIFERATION"),
        ];

        let mut known_genes = HashSet::new();
        for (u, v) in &all_edges {
            known_genes.insert(*u);
            known_genes.insert(*v);
        }

        Self { all_edges, known_genes }
    }

    pub fn parse_vaf_report(&self, report_text: &str, vaf_threshold: f64) -> Result<ClonalTumorProfile, String> {
        let mut parsed_mutations: Vec<SomaticMutation> = Vec::new();
        let clean_text = report_text.replace([';', '\r'], "\n");

        for line in clean_text.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            for part in parts {
                let segment = part.trim();
                if segment.is_empty() {
                    continue;
                }

                let tokens: Vec<&str> = segment.split_whitespace().collect();
                if tokens.is_empty() {
                    continue;
                }

                let mut gene = tokens[0].to_uppercase();
                if gene == "PIK3CA" {
                    gene = "PI3K".to_string();
                } else if gene == "ERBB1" {
                    gene = "EGFR".to_string();
                }

                if !self.known_genes.contains(gene.as_str()) || gene == "PROLIFERATION" {
                    continue;
                }

                let mut vaf = 0.30;
                let upper_seg = segment.to_uppercase();
                if let Some(pos) = upper_seg.find("VAF") {
                    let after_vaf = &upper_seg[pos + 3..];
                    let digits: String = after_vaf
                        .chars()
                        .filter(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    if let Ok(num) = digits.parse::<f64>() {
                        vaf = if num > 1.0 { num / 100.0 } else { num };
                    }
                }

                let variant_str = if tokens.len() > 1 { tokens[1].to_string() } else { "mut".to_string() };

                parsed_mutations.push(SomaticMutation {
                    gene,
                    variant: variant_str,
                    vaf,
                });
            }
        }

        if parsed_mutations.is_empty() {
            return Err("No recognized somatic mutations found. Try: EGFR L858R (VAF 45%), MET amp (VAF 18%)".into());
        }

        let filtered_muts: Vec<SomaticMutation> = parsed_mutations
            .into_iter()
            .filter(|m| m.vaf >= vaf_threshold)
            .collect();

        if filtered_muts.is_empty() {
            return Err(format!("All identified variants fall below the VAF cutoff ({:.1}%).", vaf_threshold * 100.0));
        }

        let mut sorted_muts = filtered_muts;
        sorted_muts.sort_by(|a, b| b.vaf.partial_cmp(&a.vaf).unwrap());

        let truncal = &sorted_muts[0];
        let mut subclones = Vec::new();

        subclones.push(Subclone {
            id: 0,
            name: format!("Truncal Clone (VAF {:.1}%)", truncal.vaf * 100.0),
            vaf: truncal.vaf,
            drivers: HashSet::from([truncal.gene.clone()]),
        });

        for (idx, sub_mut) in sorted_muts.iter().enumerate().skip(1) {
            let drivers = HashSet::from([truncal.gene.clone(), sub_mut.gene.clone()]);
            subclones.push(Subclone {
                id: idx,
                name: format!("Subclone {} [{} {}] (VAF {:.1}%)", idx, sub_mut.gene, sub_mut.variant, sub_mut.vaf * 100.0),
                vaf: sub_mut.vaf,
                drivers,
            });
        }

        let target_endpoint = "PROLIFERATION".to_string();
        let mut active_nodes = HashSet::new();
        let mut active_edges = Vec::new();

        for sc in &subclones {
            let mut queue: Vec<String> = sc.drivers.iter().cloned().collect();
            for d in &sc.drivers {
                active_nodes.insert(d.clone());
            }

            while let Some(current) = queue.pop() {
                for &(u, v) in &self.all_edges {
                    if u == current {
                        if active_nodes.insert(v.to_string()) {
                            queue.push(v.to_string());
                        }
                        if !active_edges.contains(&(u.to_string(), v.to_string())) {
                            active_edges.push((u.to_string(), v.to_string()));
                        }
                    }
                }
            }
        }

        Ok(ClonalTumorProfile {
            name: format!("Clonal Tumor: {} distinct sub-populations", subclones.len()),
            nodes: active_nodes,
            edges: active_edges,
            subclones,
            targets: HashSet::from([target_endpoint]),
        })
    }
}

// =====================================================================
// PART 2: MICRO DOMAIN PRIMITIVES (DISTANCE-GEOMETRY & POCKET ANCHORS)
// =====================================================================

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FeatureType {
    HDonor,
    HAcceptor,
    AromaticRing,
    Hydrophobic,
}

#[derive(Clone, Debug)]
pub struct PocketAnchor {
    pub id: usize,
    pub target_gene: String,
    pub label: String,
    pub feature: FeatureType,
    pub coord: [f64; 3],
    pub tolerance: f64,
}

impl PocketAnchor {
    pub fn distance_to(&self, other: &PocketAnchor) -> f64 {
        let dx = self.coord[0] - other.coord[0];
        let dy = self.coord[1] - other.coord[1];
        let dz = self.coord[2] - other.coord[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FragmentKind {
    CarbonSp3,
    NitrogenDonor,
    OxygenAcceptor,
    BenzeneRing,
    PyridineRing,
}

impl FragmentKind {
    pub fn features(&self) -> Vec<FeatureType> {
        match self {
            FragmentKind::CarbonSp3 => vec![FeatureType::Hydrophobic],
            FragmentKind::NitrogenDonor => vec![FeatureType::HDonor],
            FragmentKind::OxygenAcceptor => vec![FeatureType::HAcceptor],
            FragmentKind::BenzeneRing => vec![FeatureType::AromaticRing, FeatureType::Hydrophobic],
            FragmentKind::PyridineRing => vec![FeatureType::AromaticRing, FeatureType::HAcceptor],
        }
    }

    pub fn effective_radius(&self) -> f64 {
        match self {
            FragmentKind::BenzeneRing | FragmentKind::PyridineRing => 1.40,
            _ => 0.77,
        }
    }

    pub fn element_symbol(&self) -> &'static str {
        match self {
            FragmentKind::CarbonSp3 | FragmentKind::BenzeneRing => "C",
            FragmentKind::NitrogenDonor | FragmentKind::PyridineRing => "N",
            FragmentKind::OxygenAcceptor => "O",
        }
    }
}

const BOND_LEN_MIN: f64 = 1.20;
const BOND_LEN_MAX: f64 = 1.54;

pub struct PocketCleftRepository {
    pub gene_pockets: HashMap<String, Vec<PocketAnchor>>,
}

impl PocketCleftRepository {
    pub fn new() -> Self {
        let mut gene_pockets = HashMap::new();

        gene_pockets.insert(
            "EGFR".to_string(),
            vec![
                PocketAnchor { id: 0, target_gene: "EGFR".into(), label: "Met793 Hinge C=O".into(), feature: FeatureType::HAcceptor, coord: [0.0, 0.0, 0.0], tolerance: 0.6 },
                PocketAnchor { id: 1, target_gene: "EGFR".into(), label: "Thr790 Gatekeeper NH".into(), feature: FeatureType::HDonor, coord: [3.8, 1.2, 0.5], tolerance: 0.6 },
                PocketAnchor { id: 2, target_gene: "EGFR".into(), label: "Hydrophobic Sub-Pocket".into(), feature: FeatureType::AromaticRing, coord: [6.2, 3.5, 1.8], tolerance: 0.8 },
            ],
        );

        gene_pockets.insert(
            "MET".to_string(),
            vec![
                PocketAnchor { id: 3, target_gene: "MET".into(), label: "Pro1158 Hinge Acceptor".into(), feature: FeatureType::HAcceptor, coord: [1.2, 0.4, 0.2], tolerance: 0.6 },
                PocketAnchor { id: 4, target_gene: "MET".into(), label: "Tyr1230 Activation Loop".into(), feature: FeatureType::HDonor, coord: [4.4, 1.8, -0.6], tolerance: 0.6 },
                PocketAnchor { id: 5, target_gene: "MET".into(), label: "Deep Hydrophobic Channel".into(), feature: FeatureType::AromaticRing, coord: [7.1, 4.0, 1.4], tolerance: 0.8 },
            ],
        );

        gene_pockets.insert(
            "PI3K".to_string(),
            vec![
                PocketAnchor { id: 6, target_gene: "PI3K".into(), label: "Val851 Hinge Acceptor".into(), feature: FeatureType::HAcceptor, coord: [0.3, 0.2, 0.1], tolerance: 0.6 },
                PocketAnchor { id: 7, target_gene: "PI3K".into(), label: "Lys802 Catalytic Amine".into(), feature: FeatureType::HDonor, coord: [3.5, 1.5, 0.8], tolerance: 0.6 },
                PocketAnchor { id: 8, target_gene: "PI3K".into(), label: "Lipophilic Affinity Region".into(), feature: FeatureType::Hydrophobic, coord: [5.8, 3.2, 1.6], tolerance: 0.7 },
            ],
        );

        Self { gene_pockets }
    }

    pub fn build_dual_inhibitor_pocket(&self, targets: &[String]) -> Vec<PocketAnchor> {
        let mut combined = Vec::new();
        let mut id_counter = 0;

        for (t_idx, target) in targets.iter().enumerate() {
            if let Some(anchors) = self.gene_pockets.get(target) {
                let offset_x = (t_idx as f64) * 4.5;
                for a in anchors {
                    let mut cloned = a.clone();
                    cloned.id = id_counter;
                    cloned.coord[0] += offset_x;
                    combined.push(cloned);
                    id_counter += 1;
                }
            }
        }
        combined
    }
}

// =====================================================================
// PART 3: SAT & CDCL(T) LOGIC SOLVER CORE WITH SPECTRAL INSTRUMENTATION
// =====================================================================

type Lit = i32;

#[derive(Clone)]
pub struct SATFormula {
    pub num_vars: usize,
    pub clauses: Vec<Vec<Lit>>,
    pub var_names: HashMap<String, usize>,
    pub rev_var_names: HashMap<usize, String>,
}

enum PropResult {
    Ok,
    Conflict,
}

impl SATFormula {
    pub fn new() -> Self {
        Self {
            num_vars: 0,
            clauses: Vec::new(),
            var_names: HashMap::new(),
            rev_var_names: HashMap::new(),
        }
    }

    pub fn alloc_var(&mut self, name: &str) -> usize {
        if let Some(&v) = self.var_names.get(name) {
            v
        } else {
            self.num_vars += 1;
            let v = self.num_vars;
            self.var_names.insert(name.to_string(), v);
            self.rev_var_names.insert(v, name.to_string());
            v
        }
    }

    pub fn add_clause(&mut self, clause: Vec<Lit>) {
        if !clause.is_empty() {
            self.clauses.push(clause);
        }
    }

    fn find_unit(&self, assignment: &HashMap<usize, bool>) -> Option<Result<Lit, ()>> {
        for clause in &self.clauses {
            let mut satisfied = false;
            let mut unassigned = Vec::new();

            for &lit in clause {
                let var = lit.unsigned_abs() as usize;
                match assignment.get(&var) {
                    Some(&val) => {
                        if (lit > 0 && val) || (lit < 0 && !val) {
                            satisfied = true;
                            break;
                        }
                    }
                    None => unassigned.push(lit),
                }
            }

            if !satisfied {
                if unassigned.is_empty() {
                    return Some(Err(()));
                }
                if unassigned.len() == 1 {
                    return Some(Ok(unassigned[0]));
                }
            }
        }
        None
    }

    fn unit_propagate(&self, assignment: &mut HashMap<usize, bool>) -> PropResult {
        loop {
            match self.find_unit(assignment) {
                Some(Ok(unit_lit)) => {
                    let var = unit_lit.unsigned_abs() as usize;
                    let val = unit_lit > 0;
                    if let Some(&existing) = assignment.get(&var) {
                        if existing != val {
                            return PropResult::Conflict;
                        }
                    } else {
                        assignment.insert(var, val);
                    }
                }
                Some(Err(())) => return PropResult::Conflict,
                None => return PropResult::Ok,
            }
        }
    }

    pub fn solve(&self) -> (Option<HashMap<usize, bool>>, usize) {
        fn dpll(
            formula: &SATFormula,
            mut assignment: HashMap<usize, bool>,
            steps: &mut usize,
        ) -> Option<HashMap<usize, bool>> {
            *steps += 1;
            if let PropResult::Conflict = formula.unit_propagate(&mut assignment) {
                return None;
            }

            let unassigned: Vec<usize> = (1..=formula.num_vars)
                .filter(|v| !assignment.contains_key(v))
                .collect();

            if unassigned.is_empty() {
                return Some(assignment);
            }

            let branch_var = unassigned[0];
            let mut try_true = assignment.clone();
            try_true.insert(branch_var, true);
            if let Some(sol) = dpll(formula, try_true, steps) {
                return Some(sol);
            }

            let mut try_false = assignment;
            try_false.insert(branch_var, false);
            dpll(formula, try_false, steps)
        }

        let mut steps = 0;
        let res = dpll(self, HashMap::new(), &mut steps);
        (res, steps)
    }

    pub fn compute_spectral_metric(&self, domain_label: &str, steps: usize) -> SpectralComplexityMetric {
        let n = self.num_vars.max(1);
        let m = self.clauses.len();
        let alpha = (m as f64) / (n as f64);
        let alpha_c = 4.267; // Canonical 3-SAT critical phase boundary

        let raw_log2 = n as f64;
        let eff_log2 = ((steps.max(1)) as f64).log2();
        let s = (raw_log2 - eff_log2).max(0.0);
        let compression = (s / raw_log2) * 100.0;

        let regime = if alpha < 3.2 {
            "Under-constrained (Permissive Solution Space)".to_string()
        } else if alpha <= 4.8 {
            "Critical Phase Boundary (Hardest / Complex Branching)".to_string()
        } else {
            "Over-constrained (Invariant Collapse / Deterministic)".to_string()
        };

        SpectralComplexityMetric {
            domain_label: domain_label.to_string(),
            num_vars: n,
            num_clauses: m,
            alpha,
            alpha_critical: alpha_c,
            raw_search_space_log2: raw_log2,
            resolved_steps: steps,
            reduction_s: s,
            compression_ratio: compression,
            regime,
        }
    }
}

pub struct DistanceGeometryTheory {
    pub pocket_anchors: Vec<PocketAnchor>,
    pub num_scaffold_nodes: usize,
}

impl DistanceGeometryTheory {
    pub fn new(pocket_anchors: Vec<PocketAnchor>, num_scaffold_nodes: usize) -> Self {
        Self { pocket_anchors, num_scaffold_nodes }
    }

    pub fn compute_graph_distances(&self, adj_matrix: &[Vec<bool>]) -> Vec<Vec<usize>> {
        let n = self.num_scaffold_nodes;
        let mut dist = vec![vec![usize::MAX / 2; n]; n];

        for i in 0..n {
            dist[i][i] = 0;
            for j in 0..n {
                if adj_matrix[i][j] {
                    dist[i][j] = 1;
                }
            }
        }

        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    if dist[i][k] + dist[k][j] < dist[i][j] {
                        dist[i][j] = dist[i][k] + dist[k][j];
                    }
                }
            }
        }
        dist
    }

    pub fn verify_geometry(
        &self,
        matchings: &HashMap<usize, usize>,
        adj_matrix: &[Vec<bool>],
        fragment_kinds: &HashMap<usize, FragmentKind>,
    ) -> Result<(), (usize, usize, usize, usize)> {
        if matchings.len() < 2 {
            return Ok(());
        }

        let graph_dist = self.compute_graph_distances(adj_matrix);
        let pairs: Vec<(&usize, &usize)> = matchings.iter().collect();

        for i in 0..pairs.len() {
            for j in (i + 1)..pairs.len() {
                let (&u, &anchor_a_id) = pairs[i];
                let (&v, &anchor_b_id) = pairs[j];

                let hops = graph_dist[u][v];
                if hops >= usize::MAX / 2 {
                    return Err((u, anchor_a_id, v, anchor_b_id));
                }

                let target_dist = self.pocket_anchors[anchor_a_id].distance_to(&self.pocket_anchors[anchor_b_id]);
                let tol = self.pocket_anchors[anchor_a_id].tolerance + self.pocket_anchors[anchor_b_id].tolerance;

                let r_u = fragment_kinds.get(&u).map(|k| k.effective_radius()).unwrap_or(0.77);
                let r_v = fragment_kinds.get(&v).map(|k| k.effective_radius()).unwrap_or(0.77);

                let max_span = (hops as f64) * BOND_LEN_MAX + (r_u + r_v);
                let min_span = if hops == 1 { BOND_LEN_MIN } else { 2.40 };

                if target_dist - tol > max_span || (hops > 1 && target_dist + tol < min_span) {
                    return Err((u, anchor_a_id, v, anchor_b_id));
                }
            }
        }

        Ok(())
    }
}

pub struct MolecularInversionPipeline {
    pub pocket_anchors: Vec<PocketAnchor>,
    pub num_nodes: usize,
    pub candidate_fragments: Vec<FragmentKind>,
}

impl MolecularInversionPipeline {
    pub fn new(pocket_anchors: Vec<PocketAnchor>, num_nodes: usize) -> Self {
        Self {
            pocket_anchors,
            num_nodes,
            candidate_fragments: vec![
                FragmentKind::CarbonSp3,
                FragmentKind::NitrogenDonor,
                FragmentKind::OxygenAcceptor,
                FragmentKind::BenzeneRing,
                FragmentKind::PyridineRing,
            ],
        }
    }

    pub fn synthesize(
        &self,
    ) -> (
        Option<(Vec<(usize, FragmentKind)>, Vec<(usize, usize)>, Vec<(usize, usize)>)>,
        SpectralComplexityMetric,
    ) {
        let mut sat = SATFormula::new();
        let n = self.num_nodes;
        let m = self.pocket_anchors.len();

        let mut u_vars = HashMap::new();
        for i in 0..n {
            for &frag in &self.candidate_fragments {
                let v = sat.alloc_var(&format!("u_{}_{:?}", i, frag));
                u_vars.insert((i, frag), v);
            }
        }

        let mut e_vars = HashMap::new();
        for i in 0..n {
            for j in (i + 1)..n {
                let v = sat.alloc_var(&format!("e_{}_{}", i, j));
                e_vars.insert((i, j), v);
            }
        }

        let mut match_vars = HashMap::new();
        for i in 0..n {
            for a in 0..m {
                let v = sat.alloc_var(&format!("match_{}_{}", i, a));
                match_vars.insert((i, a), v);
            }
        }

        for i in 0..n {
            let mut type_clause = Vec::new();
            for &frag in &self.candidate_fragments {
                type_clause.push(u_vars[&(i, frag)] as Lit);
            }
            sat.add_clause(type_clause);

            for a in 0..self.candidate_fragments.len() {
                for b in (a + 1)..self.candidate_fragments.len() {
                    let fa = self.candidate_fragments[a];
                    let fb = self.candidate_fragments[b];
                    sat.add_clause(vec![-(u_vars[&(i, fa)] as Lit), -(u_vars[&(i, fb)] as Lit)]);
                }
            }

            let mut neighbors = Vec::new();
            for j in 0..n {
                if i != j {
                    let key = if i < j { (i, j) } else { (j, i) };
                    neighbors.push(e_vars[&key]);
                }
            }

            let u_ox = u_vars[&(i, FragmentKind::OxygenAcceptor)] as Lit;
            for x in 0..neighbors.len().min(6) {
                for y in (x + 1)..neighbors.len().min(6) {
                    for z in (y + 1)..neighbors.len().min(6) {
                        sat.add_clause(vec![
                            -u_ox,
                            -(neighbors[x] as Lit),
                            -(neighbors[y] as Lit),
                            -(neighbors[z] as Lit),
                        ]);
                    }
                }
            }
        }

        for a in 0..m {
            let anchor = &self.pocket_anchors[a];
            let compatible: Vec<FragmentKind> = self.candidate_fragments
                .iter()
                .filter(|f| f.features().contains(&anchor.feature))
                .cloned()
                .collect();

            for i in 0..n {
                if let Some(&m_ia) = match_vars.get(&(i, a)) {
                    let mut fwd = vec![-(m_ia as Lit)];
                    for &f in &compatible {
                        if let Some(&u_if) = u_vars.get(&(i, f)) {
                            fwd.push(u_if as Lit);
                        }
                    }
                    sat.add_clause(fwd);
                }
            }

            for i in 0..n {
                for j in (i + 1)..n {
                    if let (Some(&m_ia), Some(&m_ja)) = (match_vars.get(&(i, a)), match_vars.get(&(j, a))) {
                        sat.add_clause(vec![-(m_ia as Lit), -(m_ja as Lit)]);
                    }
                }
            }
        }

        let core_anchors = m.min(n);
        for a in 0..core_anchors {
            let mut cover = Vec::new();
            for i in 0..n {
                if let Some(&m_ia) = match_vars.get(&(i, a)) {
                    cover.push(m_ia as Lit);
                }
            }
            sat.add_clause(cover);
        }

        for a in 0..m {
            for b in (a + 1)..m {
                let dist = self.pocket_anchors[a].distance_to(&self.pocket_anchors[b]);
                let tol = self.pocket_anchors[a].tolerance + self.pocket_anchors[b].tolerance;
                if dist - tol > (BOND_LEN_MAX + 1.40) {
                    for i in 0..n {
                        for j in (i + 1)..n {
                            if let (Some(&m_ia), Some(&m_jb), Some(&e_ij)) = (
                                match_vars.get(&(i, a)),
                                match_vars.get(&(j, b)),
                                e_vars.get(&(i, j)),
                            ) {
                                sat.add_clause(vec![-(m_ia as Lit), -(m_jb as Lit), -(e_ij as Lit)]);
                            }
                            if let (Some(&m_ib), Some(&m_ja), Some(&e_ij)) = (
                                match_vars.get(&(i, b)),
                                match_vars.get(&(j, a)),
                                e_vars.get(&(i, j)),
                            ) {
                                sat.add_clause(vec![-(m_ib as Lit), -(m_ja as Lit), -(e_ij as Lit)]);
                            }
                        }
                    }
                }
            }
        }

        let geom_theory = DistanceGeometryTheory::new(self.pocket_anchors.clone(), n);

        fn dpll_t(
            sat: &mut SATFormula,
            geom_theory: &DistanceGeometryTheory,
            mut assignment: HashMap<usize, bool>,
            match_vars: &HashMap<(usize, usize), usize>,
            e_vars: &HashMap<(usize, usize), usize>,
            u_vars: &HashMap<(usize, FragmentKind), usize>,
            candidate_fragments: &[FragmentKind],
            n: usize,
            m: usize,
            steps: &mut usize,
        ) -> Option<HashMap<usize, bool>> {
            *steps += 1;
            if *steps > 80_000 {
                return None;
            }

            if let PropResult::Conflict = sat.unit_propagate(&mut assignment) {
                return None;
            }

            let unassigned: Vec<usize> = (1..=sat.num_vars)
                .filter(|v| !assignment.contains_key(v))
                .collect();

            if unassigned.is_empty() {
                let mut current_matches = HashMap::new();
                for i in 0..n {
                    for a in 0..m {
                        if let Some(&vid) = match_vars.get(&(i, a)) {
                            if *assignment.get(&vid).unwrap_or(&false) {
                                current_matches.insert(i, a);
                            }
                        }
                    }
                }

                let mut current_frags = HashMap::new();
                for i in 0..n {
                    for &frag in candidate_fragments {
                        if let Some(&vid) = u_vars.get(&(i, frag)) {
                            if *assignment.get(&vid).unwrap_or(&false) {
                                current_frags.insert(i, frag);
                                break;
                            }
                        }
                    }
                }

                let mut adj_matrix = vec![vec![false; n]; n];
                for i in 0..n {
                    for j in (i + 1)..n {
                        if let Some(&vid) = e_vars.get(&(i, j)) {
                            if *assignment.get(&vid).unwrap_or(&false) {
                                adj_matrix[i][j] = true;
                                adj_matrix[j][i] = true;
                            }
                        }
                    }
                }

                if geom_theory.verify_geometry(&current_matches, &adj_matrix, &current_frags).is_ok() {
                    return Some(assignment);
                } else {
                    return None;
                }
            }

            let branch_var = unassigned[0];
            let mut try_true = assignment.clone();
            try_true.insert(branch_var, true);
            if let Some(sol) = dpll_t(sat, geom_theory, try_true, match_vars, e_vars, u_vars, candidate_fragments, n, m, steps) {
                return Some(sol);
            }

            let mut try_false = assignment;
            try_false.insert(branch_var, false);
            dpll_t(sat, geom_theory, try_false, match_vars, e_vars, u_vars, candidate_fragments, n, m, steps)
        }

        let mut steps = 0;
        let opt_sol = dpll_t(
            &mut sat,
            &geom_theory,
            HashMap::new(),
            &match_vars,
            &e_vars,
            &u_vars,
            &self.candidate_fragments,
            n,
            m,
            &mut steps,
        );

        let metric = sat.compute_spectral_metric("Micro De Novo Inversion", steps);

        if let Some(solution) = opt_sol {
            let mut constructed_nodes = Vec::new();
            for i in 0..n {
                for &frag in &self.candidate_fragments {
                    if let Some(&vid) = u_vars.get(&(i, frag)) {
                        if *solution.get(&vid).unwrap_or(&false) {
                            constructed_nodes.push((i, frag));
                            break;
                        }
                    }
                }
            }

            let mut constructed_bonds = Vec::new();
            for i in 0..n {
                for j in (i + 1)..n {
                    if let Some(&vid) = e_vars.get(&(i, j)) {
                        if *solution.get(&vid).unwrap_or(&false) {
                            constructed_bonds.push((i, j));
                        }
                    }
                }
            }

            let mut verified_matches = Vec::new();
            for i in 0..n {
                for a in 0..m {
                    if let Some(&vid) = match_vars.get(&(i, a)) {
                        if *solution.get(&vid).unwrap_or(&false) {
                            verified_matches.push((i, a));
                        }
                    }
                }
            }

            (Some((constructed_nodes, constructed_bonds, verified_matches)), metric)
        } else {
            (None, metric)
        }
    }
}

// =====================================================================
// PART 4: COMPUTATIONAL CHEMISTRY EXPORTERS (SMILES, SDF, PDB)
// =====================================================================

pub struct ChemistryExporter;

impl ChemistryExporter {
    pub fn to_smiles(nodes: &[(usize, FragmentKind)], bonds: &[(usize, usize)]) -> String {
        if nodes.is_empty() {
            return String::new();
        }

        let mut adj = vec![Vec::new(); nodes.len()];
        for &(u, v) in bonds {
            adj[u].push(v);
            adj[v].push(u);
        }

        let mut visited = vec![false; nodes.len()];
        let mut smiles = String::new();

        fn visit(u: usize, adj: &[Vec<usize>], nodes: &[(usize, FragmentKind)], visited: &mut [bool], out: &mut String) {
            visited[u] = true;
            let symbol = match nodes[u].1 {
                FragmentKind::BenzeneRing => "c1ccccc1",
                FragmentKind::PyridineRing => "c1ccncc1",
                FragmentKind::CarbonSp3 => "C",
                FragmentKind::NitrogenDonor => "N",
                FragmentKind::OxygenAcceptor => "O",
            };
            out.push_str(symbol);

            let unvisited_neighbors: Vec<usize> = adj[u].iter().copied().filter(|&v| !visited[v]).collect();
            for (idx, &v) in unvisited_neighbors.iter().enumerate() {
                if idx < unvisited_neighbors.len() - 1 {
                    out.push('(');
                    visit(v, adj, nodes, visited, out);
                    out.push(')');
                } else {
                    visit(v, adj, nodes, visited, out);
                }
            }
        }

        visit(0, &adj, nodes, &mut visited, &mut smiles);
        smiles
    }

    pub fn compute_3d_coordinates(
        nodes: &[(usize, FragmentKind)],
        matches: &[(usize, usize)],
        anchors: &[PocketAnchor],
    ) -> Vec<[f64; 3]> {
        let n = nodes.len();
        let mut coords = vec![[0.0, 0.0, 0.0]; n];
        let match_map: HashMap<usize, usize> = matches.iter().copied().collect();

        for i in 0..n {
            if let Some(&anchor_idx) = match_map.get(&i) {
                if anchor_idx < anchors.len() {
                    coords[i] = anchors[anchor_idx].coord;
                }
            } else {
                let angle = (i as f64) * 0.8;
                coords[i] = [
                    (i as f64) * 1.5,
                    angle.sin() * 2.0,
                    angle.cos() * 1.5,
                ];
            }
        }
        coords
    }

    pub fn export_sdf(
        filepath: &str,
        nodes: &[(usize, FragmentKind)],
        bonds: &[(usize, usize)],
        coords: &[[f64; 3]],
    ) -> std::io::Result<()> {
        let mut content = String::new();
        content.push_str("REASONING_ENGINE_DE_NOVO_LIGAND\n");
        content.push_str("  CDCL(T)_Synthesis_Precision_Oncology\n\n");
        content.push_str(&format!("{:>3}{:>3}  0  0  0  0  0  0  0  0999 V2000\n", nodes.len(), bonds.len()));

        for (i, (_, kind)) in nodes.iter().enumerate() {
            let [x, y, z] = coords[i];
            let sym = kind.element_symbol();
            content.push_str(&format!("{:>10.4}{:>10.4}{:>10.4} {:<3} 0  0  0  0  0  0  0  0  0  0  0  0\n", x, y, z, sym));
        }

        for &(u, v) in bonds {
            content.push_str(&format!("{:>3}{:>3}  1  0  0  0  0\n", u + 1, v + 1));
        }

        content.push_str("M  END\n$$$$\n");

        if let Some(parent) = Path::new(filepath).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(filepath, content)
    }

    pub fn export_pdb(
        filepath: &str,
        nodes: &[(usize, FragmentKind)],
        bonds: &[(usize, usize)],
        coords: &[[f64; 3]],
    ) -> std::io::Result<()> {
        let mut content = String::new();
        content.push_str("HEADER    DE NOVO PRECISION ONCOLOGY LIGAND\n");
        content.push_str("COMPND    REASONING ENGINE BIFUNCTIONAL INHIBITOR\n");

        for (i, (_, kind)) in nodes.iter().enumerate() {
            let [x, y, z] = coords[i];
            let sym = kind.element_symbol();
            let atom_name = format!("{}{}", sym, i + 1);
            content.push_str(&format!(
                "HETATM{:>5} {:<4} LIG A   1    {:>8.3}{:>8.3}{:>8.3}{:>6.2}{:>6.2}          {:<2}\n",
                i + 1, atom_name, x, y, z, 1.00, 20.00, sym
            ));
        }

        for &(u, v) in bonds {
            content.push_str(&format!("CONECT{:>5}{:>5}\n", u + 1, v + 1));
        }

        content.push_str("END\n");

        if let Some(parent) = Path::new(filepath).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(filepath, content)
    }
}

// =====================================================================
// PART 5: TEMPORAL EVOLUTIONARY SCHEDULER (COLLATERAL SENSITIVITY)
// =====================================================================

pub struct EvolutionaryTemporalScheduler {
    pub subclone_drivers: Vec<String>,
    pub candidate_drugs: Vec<Drug>,
    pub num_cycles: usize,
    pub max_hep_per_cycle: usize,
    pub max_card_per_cycle: usize,
    pub max_ren_per_cycle: usize,
    pub max_cumulative_tox: usize,
    pub enforce_cyp: bool,
    pub collateral_rules: Vec<CollateralSensitivityRule>,
}

impl EvolutionaryTemporalScheduler {
    pub fn new(
        subclone_drivers: Vec<String>,
        candidate_drugs: Vec<Drug>,
        num_cycles: usize,
        max_hep_per_cycle: usize,
        max_card_per_cycle: usize,
        max_ren_per_cycle: usize,
        max_cumulative_tox: usize,
        enforce_cyp: bool,
    ) -> Self {
        let collateral_rules = vec![
            CollateralSensitivityRule {
                primary_drug_id: "OSI".into(),
                sensitized_drug_id: "CAP".into(),
                target_subclone_driver: "MET".into(),
                mechanism: "EGFR blockade drives MET bypass up-regulation, rendering clone hyper-susceptible to Capmatinib".into(),
            },
            CollateralSensitivityRule {
                primary_drug_id: "CAP".into(),
                sensitized_drug_id: "ALP".into(),
                target_subclone_driver: "PI3K".into(),
                mechanism: "MET inhibition steers metabolic survival into PI3K dependence, priming response to Alpelisib".into(),
            },
            CollateralSensitivityRule {
                primary_drug_id: "TRA".into(),
                sensitized_drug_id: "OSI".into(),
                target_subclone_driver: "EGFR".into(),
                mechanism: "MEK inhibition induces receptor tyrosine kinase feedback loops, re-sensitizing cells to EGFR blockade".into(),
            },
            CollateralSensitivityRule {
                primary_drug_id: "SOT".into(),
                sensitized_drug_id: "OSI".into(),
                target_subclone_driver: "EGFR".into(),
                mechanism: "KRAS-G12C inhibition triggers rapid RTK/EGFR rebound, rendering clone hypersensitive to Osimertinib".into(),
            },
            CollateralSensitivityRule {
                primary_drug_id: "ENC".into(),
                sensitized_drug_id: "PAL".into(),
                target_subclone_driver: "CDK4".into(),
                mechanism: "BRAF inhibition causes cell cycle escape via cyclin D/CDK4/6 axis, priming response to Palbociclib".into(),
            },
        ];

        Self {
            subclone_drivers,
            candidate_drugs,
            num_cycles,
            max_hep_per_cycle,
            max_card_per_cycle,
            max_ren_per_cycle,
            max_cumulative_tox,
            enforce_cyp,
            collateral_rules,
        }
    }

    pub fn schedule(&self) -> (Option<TemporalScheduleResult>, SpectralComplexityMetric) {
        let start = std::time::Instant::now();
        let mut sat = SATFormula::new();
        let t_max = self.num_cycles;
        let d_list = &self.candidate_drugs;

        let mut x_vars = HashMap::new();
        for d in d_list {
            for t in 1..=t_max {
                let v = sat.alloc_var(&format!("x_{}_{}", d.id, t));
                x_vars.insert((d.id.clone(), t), v);
            }
        }

        let mut v_vars = HashMap::new();
        for k in &self.subclone_drivers {
            for t in 1..=t_max {
                let v = sat.alloc_var(&format!("v_{}_{}", k, t));
                v_vars.insert((k.clone(), t), v);
            }
        }

        // 1. Clonal suppression vs expansion invariant per cycle
        for k in &self.subclone_drivers {
            for t in 1..=t_max {
                if let Some(&vid_v) = v_vars.get(&(k.clone(), t)) {
                    let v_kt = vid_v as Lit;
                    let hitting_drugs: Vec<&Drug> = d_list.iter().filter(|d| d.targets.contains(k)).collect();

                    for d in &hitting_drugs {
                        if let Some(&vid_x) = x_vars.get(&(d.id.clone(), t)) {
                            sat.add_clause(vec![-(vid_x as Lit), -v_kt]);
                        }
                    }

                    let mut unhit_clause = vec![v_kt];
                    for d in hitting_drugs {
                        if let Some(&vid_x) = x_vars.get(&(d.id.clone(), t)) {
                            unhit_clause.push(vid_x as Lit);
                        }
                    }
                    sat.add_clause(unhit_clause);
                }
            }
        }

        // 2. Horizon Coverage: Every subclone driver must be hit in at least one cycle
        for k in &self.subclone_drivers {
            let mut hit_across_horizon = Vec::new();
            for t in 1..=t_max {
                if let Some(&vid_v) = v_vars.get(&(k.clone(), t)) {
                    hit_across_horizon.push(-(vid_v as Lit));
                }
            }
            sat.add_clause(hit_across_horizon);
        }

        // 3. Per-Cycle Organ Budget Compilation directly into SAT clauses
        for t in 1..=t_max {
            for d in d_list {
                if let Some(&vid) = x_vars.get(&(d.id.clone(), t)) {
                    if d.tox.hepatic > self.max_hep_per_cycle
                        || d.tox.cardiac > self.max_card_per_cycle
                        || d.tox.renal > self.max_ren_per_cycle
                    {
                        sat.add_clause(vec![-(vid as Lit)]);
                    }
                }
            }

            for i in 0..d_list.len() {
                for j in (i + 1)..d_list.len() {
                    let da = &d_list[i];
                    let db = &d_list[j];
                    if let (Some(&xa), Some(&xb)) = (
                        x_vars.get(&(da.id.clone(), t)),
                        x_vars.get(&(db.id.clone(), t)),
                    ) {
                        if da.tox.hepatic + db.tox.hepatic > self.max_hep_per_cycle
                            || da.tox.cardiac + db.tox.cardiac > self.max_card_per_cycle
                            || da.tox.renal + db.tox.renal > self.max_ren_per_cycle
                        {
                            sat.add_clause(vec![-(xa as Lit), -(xb as Lit)]);
                        }
                    }
                }
            }
        }

        // 4. Per-cycle Cocktail Size (1 to 2 drugs max per cycle)
        for t in 1..=t_max {
            let mut at_least_one = Vec::new();
            for d in d_list {
                if let Some(&vid) = x_vars.get(&(d.id.clone(), t)) {
                    at_least_one.push(vid as Lit);
                }
            }
            sat.add_clause(at_least_one);

            for i in 0..d_list.len() {
                for j in (i + 1)..d_list.len() {
                    for l in (j + 1)..d_list.len() {
                        if let (Some(&xi), Some(&xj), Some(&xl)) = (
                            x_vars.get(&(d_list[i].id.clone(), t)),
                            x_vars.get(&(d_list[j].id.clone(), t)),
                            x_vars.get(&(d_list[l].id.clone(), t)),
                        ) {
                            sat.add_clause(vec![-(xi as Lit), -(xj as Lit), -(xl as Lit)]);
                        }
                    }
                }
            }
        }

        // 5. CYP Clash Exclusions per cycle
        if self.enforce_cyp {
            for t in 1..=t_max {
                for i in 0..d_list.len() {
                    let da = &d_list[i];
                    for j in (i + 1)..d_list.len() {
                        let db = &d_list[j];
                        let mut clash = false;
                        if (da.cyp.cyp3a4 == CypRole::Inhibitor && db.cyp.cyp3a4 == CypRole::Substrate)
                            || (db.cyp.cyp3a4 == CypRole::Inhibitor && da.cyp.cyp3a4 == CypRole::Substrate)
                            || (da.cyp.cyp3a4 == CypRole::Inducer && db.cyp.cyp3a4 == CypRole::Substrate)
                            || (db.cyp.cyp3a4 == CypRole::Inducer && da.cyp.cyp3a4 == CypRole::Substrate)
                        {
                            clash = true;
                        }
                        if clash {
                            if let (Some(&xa), Some(&xb)) = (
                                x_vars.get(&(da.id.clone(), t)),
                                x_vars.get(&(db.id.clone(), t)),
                            ) {
                                sat.add_clause(vec![-(xa as Lit), -(xb as Lit)]);
                            }
                        }
                    }
                }
            }
        }

        // 6. Collateral Sensitivity Steering
        for rule in &self.collateral_rules {
            for t in 1..t_max {
                if let (Some(&xp), Some(&xs), Some(&v_target)) = (
                    x_vars.get(&(rule.primary_drug_id.clone(), t)),
                    x_vars.get(&(rule.sensitized_drug_id.clone(), t + 1)),
                    v_vars.get(&(rule.target_subclone_driver.clone(), t)),
                ) {
                    sat.add_clause(vec![-(xp as Lit), -(v_target as Lit), xs as Lit]);
                }
            }
        }

        // 7. Solving loop with Cumulative Toxicity Feedback
        let mut total_solver_steps = 0;
        let mut final_result = None;

        for _attempt in 0..100 {
            let (sol_opt, steps) = sat.solve();
            total_solver_steps += steps;

            match sol_opt {
                Some(sol) => {
                    let mut cycle_drugs = vec![Vec::new(); t_max];
                    let mut cycle_tox = vec![(0, 0, 0); t_max];
                    let mut cum_tox = 0;
                    let mut blocking_clause = Vec::new();

                    for t in 1..=t_max {
                        let mut tot_h = 0;
                        let mut tot_c = 0;
                        let mut tot_r = 0;

                        for d in d_list {
                            if let Some(&vid) = x_vars.get(&(d.id.clone(), t)) {
                                if *sol.get(&vid).unwrap_or(&false) {
                                    tot_h += d.tox.hepatic;
                                    tot_c += d.tox.cardiac;
                                    tot_r += d.tox.renal;
                                    cycle_drugs[t - 1].push(d.clone());
                                    blocking_clause.push(-(vid as Lit));
                                }
                            }
                        }

                        cum_tox += tot_h + tot_c + tot_r;
                        cycle_tox[t - 1] = (tot_h, tot_c, tot_r);
                    }

                    if cum_tox > self.max_cumulative_tox {
                        sat.add_clause(blocking_clause);
                        continue;
                    }

                    let mut subclone_trajectories = HashMap::new();
                    for k in &self.subclone_drivers {
                        let mut trajectory = Vec::new();
                        for t in 1..=t_max {
                            if let Some(&vid) = v_vars.get(&(k.clone(), t)) {
                                let active = *sol.get(&vid).unwrap_or(&false);
                                trajectory.push(active);
                            } else {
                                trajectory.push(false);
                            }
                        }
                        subclone_trajectories.insert(k.clone(), trajectory);
                    }

                    let mut triggered_switches = Vec::new();
                    for rule in &self.collateral_rules {
                        for t in 1..t_max {
                            let p_used = cycle_drugs[t - 1].iter().any(|d| d.id == rule.primary_drug_id);
                            let s_used = cycle_drugs[t].iter().any(|d| d.id == rule.sensitized_drug_id);
                            if p_used && s_used {
                                triggered_switches.push(format!(
                                    "Cycle {} ➔ {}: {} followed by {} ({})",
                                    t, t + 1, rule.primary_drug_id, rule.sensitized_drug_id, rule.mechanism
                                ));
                            }
                        }
                    }

                    let elapsed = start.elapsed().as_micros();
                    final_result = Some(TemporalScheduleResult {
                        num_cycles: t_max,
                        cycle_drugs,
                        cycle_toxicities: cycle_tox,
                        subclone_trajectories,
                        collateral_switches_triggered: triggered_switches,
                        elapsed_micros: elapsed,
                    });
                    break;
                }
                None => break,
            }
        }

        let metric = sat.compute_spectral_metric("Temporal Evolutionary Scheduler", total_solver_steps);
        (final_result, metric)
    }
}

// =====================================================================
// PART 6: UNIFIED NATIVE DESKTOP UI (EFRAME / EGUI)
// =====================================================================

#[derive(PartialEq)]
enum PipelineTab {
    MacroRegimen,
    MicroDeNovoInversion,
    EvolutionaryScheduling,
    SpectralComplexityGauges,
}

struct UnifiedOncologyApp {
    current_tab: PipelineTab,

    // Macro State
    kb: CanonicalOncologyKnowledgeBase,
    ngs_input_text: String,
    vaf_threshold: f64,
    detected_subclones: Vec<Subclone>,
    drug_library: Vec<Drug>,
    selected_drugs: Vec<bool>,
    k_min: usize,
    k_max: usize,
    max_hepatic: usize,
    max_cardiac: usize,
    max_renal: usize,
    enforce_cyp: bool,
    top_k_options: usize,
    macro_solutions: Vec<SolutionRegimen>,
    macro_unsat_reason: Option<String>,
    unresolved_block_targets: Vec<String>,
    macro_elapsed_micros: u128,
    macro_metric: Option<SpectralComplexityMetric>,

    // Micro State
    cleft_repo: PocketCleftRepository,
    active_anchors: Vec<PocketAnchor>,
    num_micro_nodes: usize,
    micro_solution: Option<Result<(Vec<(usize, FragmentKind)>, Vec<(usize, usize)>, Vec<(usize, usize)>, u128), String>>,
    generated_smiles: String,
    export_status_msg: Option<String>,
    micro_metric: Option<SpectralComplexityMetric>,

    // Evolutionary Scheduling State
    num_cycles: usize,
    max_cum_toxicity: usize,
    schedule_result: Option<Result<TemporalScheduleResult, String>>,
    sched_metric: Option<SpectralComplexityMetric>,
}

impl Default for UnifiedOncologyApp {
    fn default() -> Self {
        let kb = CanonicalOncologyKnowledgeBase::new();
        let cleft_repo = PocketCleftRepository::new();

        let drugs = vec![
            Drug {
                id: "OSI".into(),
                name: "Osimertinib".into(),
                targets: HashSet::from(["EGFR".into()]),
                tox: OrganToxicity { hepatic: 1, cardiac: 2, renal: 1 },
                cyp: CypProfile { cyp3a4: CypRole::Substrate, cyp2d6: CypRole::None },
                regulatory: RegulatoryStatus::FdaOnLabel,
                trial_id: None,
            },
            Drug {
                id: "CAP".into(),
                name: "Capmatinib".into(),
                targets: HashSet::from(["MET".into()]),
                tox: OrganToxicity { hepatic: 2, cardiac: 1, renal: 2 },
                cyp: CypProfile { cyp3a4: CypRole::Substrate, cyp2d6: CypRole::None },
                regulatory: RegulatoryStatus::FdaOnLabel,
                trial_id: None,
            },
            Drug {
                id: "SOT".into(),
                name: "Sotorasib".into(),
                targets: HashSet::from(["KRAS".into()]),
                tox: OrganToxicity { hepatic: 2, cardiac: 1, renal: 1 },
                cyp: CypProfile { cyp3a4: CypRole::Inducer, cyp2d6: CypRole::None },
                regulatory: RegulatoryStatus::FdaOnLabel,
                trial_id: None,
            },
            Drug {
                id: "ENC".into(),
                name: "Encorafenib".into(),
                targets: HashSet::from(["BRAF".into()]),
                tox: OrganToxicity { hepatic: 1, cardiac: 2, renal: 1 },
                cyp: CypProfile { cyp3a4: CypRole::Substrate, cyp2d6: CypRole::None },
                regulatory: RegulatoryStatus::FdaOnLabel,
                trial_id: None,
            },
            Drug {
                id: "TRA".into(),
                name: "Trametinib".into(),
                targets: HashSet::from(["MEK".into()]),
                tox: OrganToxicity { hepatic: 1, cardiac: 3, renal: 1 },
                cyp: CypProfile { cyp3a4: CypRole::None, cyp2d6: CypRole::None },
                regulatory: RegulatoryStatus::FdaOnLabel,
                trial_id: None,
            },
            Drug {
                id: "PAL".into(),
                name: "Palbociclib".into(),
                targets: HashSet::from(["CDK4".into()]),
                tox: OrganToxicity { hepatic: 1, cardiac: 2, renal: 1 },
                cyp: CypProfile { cyp3a4: CypRole::Substrate, cyp2d6: CypRole::None },
                regulatory: RegulatoryStatus::FdaOffLabel,
                trial_id: None,
            },
            Drug {
                id: "ALP".into(),
                name: "Alpelisib".into(),
                targets: HashSet::from(["PI3K".into()]),
                tox: OrganToxicity { hepatic: 3, cardiac: 1, renal: 1 },
                cyp: CypProfile { cyp3a4: CypRole::Inducer, cyp2d6: CypRole::None },
                regulatory: RegulatoryStatus::FdaOffLabel,
                trial_id: None,
            },
            Drug {
                id: "EVE".into(),
                name: "Everolimus".into(),
                targets: HashSet::from(["MTOR".into()]),
                tox: OrganToxicity { hepatic: 2, cardiac: 1, renal: 3 },
                cyp: CypProfile { cyp3a4: CypRole::Substrate, cyp2d6: CypRole::None },
                regulatory: RegulatoryStatus::FdaOffLabel,
                trial_id: None,
            },
            Drug {
                id: "ULI".into(),
                name: "Ulixertinib (ERK1/2 TKI)".into(),
                targets: HashSet::from(["ERK".into()]),
                tox: OrganToxicity { hepatic: 2, cardiac: 2, renal: 1 },
                cyp: CypProfile { cyp3a4: CypRole::None, cyp2d6: CypRole::None },
                regulatory: RegulatoryStatus::ClinicalTrial,
                trial_id: Some("NCT04566393".into()),
            },
        ];

        let n_drugs = drugs.len();
        let initial_anchors = cleft_repo.build_dual_inhibitor_pocket(&["EGFR".into(), "MET".into()]);

        Self {
            current_tab: PipelineTab::MacroRegimen,
            kb,
            ngs_input_text: "EGFR L858R (VAF 52%), MET amp (VAF 20%), PIK3CA E545K (VAF 15%)".into(),
            vaf_threshold: 0.05,
            detected_subclones: Vec::new(),
            drug_library: drugs,
            selected_drugs: vec![true; n_drugs],
            k_min: 2,
            k_max: 3,
            max_hepatic: 5,
            max_cardiac: 5,
            max_renal: 4,
            enforce_cyp: true,
            top_k_options: 3,
            macro_solutions: Vec::new(),
            macro_unsat_reason: None,
            unresolved_block_targets: Vec::new(),
            macro_elapsed_micros: 0,
            macro_metric: None,

            cleft_repo,
            active_anchors: initial_anchors,
            num_micro_nodes: 7,
            micro_solution: None,
            generated_smiles: String::new(),
            export_status_msg: None,
            micro_metric: None,

            num_cycles: 3,
            max_cum_toxicity: 24,
            schedule_result: None,
            sched_metric: None,
        }
    }
}

impl eframe::App for UnifiedOncologyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Reasoning Engine: Unified Precision Oncology Pipeline");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.selectable_value(&mut self.current_tab, PipelineTab::SpectralComplexityGauges, "4. Spectral Phase Gauge");
                    ui.selectable_value(&mut self.current_tab, PipelineTab::EvolutionaryScheduling, "3. Evolutionary Scheduling");
                    ui.selectable_value(&mut self.current_tab, PipelineTab::MicroDeNovoInversion, "2. Micro De Novo Inversion");
                    ui.selectable_value(&mut self.current_tab, PipelineTab::MacroRegimen, "1. Macro Clonal Regimen");
                });
            });
            ui.label("End-to-End Structural Logic: Clonal Invariants, Pocket Inversion & Spectral Phase-Transition Gauges");
            ui.separator();

            match self.current_tab {
                // =========================================================
                // TAB 1: MACRO CLONAL REGIMEN SOLVER
                // =========================================================
                PipelineTab::MacroRegimen => {
                    egui::Grid::new("macro_grid").num_columns(2).spacing([30.0, 10.0]).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("1. Patient NGS Somatic Profile (with VAF %)").strong());
                                ui.text_edit_singleline(&mut self.ngs_input_text);
                                ui.horizontal(|ui| {
                                    ui.label("VAF Cutoff:");
                                    let mut vaf_pct = (self.vaf_threshold * 100.0).round() as usize;
                                    if ui.add(egui::Slider::new(&mut vaf_pct, 1..=30).suffix("%")).changed() {
                                        self.vaf_threshold = (vaf_pct as f64) / 100.0;
                                    }
                                });
                            });

                            ui.add_space(6.0);
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("2. Patient Organ Reserves (Tolerance Budgets)").strong());
                                ui.horizontal(|ui| {
                                    ui.label("Hepatic:"); ui.add(egui::Slider::new(&mut self.max_hepatic, 1..=6));
                                    ui.label("Cardiac:"); ui.add(egui::Slider::new(&mut self.max_cardiac, 1..=6));
                                    ui.label("Renal:"); ui.add(egui::Slider::new(&mut self.max_renal, 1..=6));
                                });
                            });

                            ui.add_space(6.0);
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("3. Clinical Invariants & Regimen Rank").strong());
                                ui.checkbox(&mut self.enforce_cyp, "Enforce CYP3A4 / CYP2D6 Drug-Drug Exclusions");
                                ui.horizontal(|ui| {
                                    ui.label("Regimens to Enumerate:");
                                    ui.add(egui::Slider::new(&mut self.top_k_options, 1..=5));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("k_min:"); ui.add(egui::Slider::new(&mut self.k_min, 1..=3));
                                    ui.label("k_max:"); ui.add(egui::Slider::new(&mut self.k_max, 2..=4));
                                });
                            });

                            ui.add_space(6.0);
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("4. Formulary Library & Regulatory Badges").strong());
                                for (i, d) in self.drug_library.iter().enumerate() {
                                    let badge = match d.regulatory {
                                        RegulatoryStatus::FdaOnLabel => "🟢 On-Label",
                                        RegulatoryStatus::FdaOffLabel => "🟡 Off-Label",
                                        RegulatoryStatus::ClinicalTrial => "🟣 Trial",
                                    };
                                    let label = format!("{} [{}] [H:{}, C:{}, R:{}] ({})", 
                                        d.name, badge, d.tox.hepatic, d.tox.cardiac, d.tox.renal, 
                                        d.targets.iter().cloned().collect::<Vec<_>>().join(","));
                                    ui.checkbox(&mut self.selected_drugs[i], label);
                                }
                            });

                            ui.add_space(10.0);
                            if ui.button(egui::RichText::new("Synthesize Clinical Regimens").heading()).clicked() {
                                self.macro_solutions.clear();
                                self.macro_unsat_reason = None;

                                let parsed_profile = match self.kb.parse_vaf_report(&self.ngs_input_text, self.vaf_threshold) {
                                    Ok(p) => p,
                                    Err(e) => {
                                        self.macro_unsat_reason = Some(e);
                                        return;
                                    }
                                };

                                self.detected_subclones = parsed_profile.subclones.clone();
                                let mut active_targets = HashSet::new();
                                for sc in &self.detected_subclones {
                                    active_targets.extend(sc.drivers.clone());
                                }
                                self.unresolved_block_targets = active_targets.into_iter().collect();

                                let start = std::time::Instant::now();
                                let mut sat = SATFormula::new();
                                let mut d_vars = HashMap::new();
                                for (i, d) in self.drug_library.iter().enumerate() {
                                    if self.selected_drugs[i] {
                                        let v = sat.alloc_var(&d.id);
                                        d_vars.insert(d.id.clone(), v);
                                    }
                                }

                                for sc in &self.detected_subclones {
                                    let mut covered = Vec::new();
                                    for d in &self.drug_library {
                                        if !d.targets.is_disjoint(&sc.drivers) && d_vars.contains_key(&d.id) {
                                            covered.push(d_vars[&d.id] as Lit);
                                        }
                                    }
                                    sat.add_clause(covered);
                                }

                                if self.enforce_cyp {
                                    for i in 0..self.drug_library.len() {
                                        let da = &self.drug_library[i];
                                        for j in (i + 1)..self.drug_library.len() {
                                            let db = &self.drug_library[j];
                                            if d_vars.contains_key(&da.id) && d_vars.contains_key(&db.id) {
                                                let mut clash = false;
                                                if (da.cyp.cyp3a4 == CypRole::Inhibitor && db.cyp.cyp3a4 == CypRole::Substrate)
                                                    || (db.cyp.cyp3a4 == CypRole::Inhibitor && da.cyp.cyp3a4 == CypRole::Substrate)
                                                    || (da.cyp.cyp3a4 == CypRole::Inducer && db.cyp.cyp3a4 == CypRole::Substrate)
                                                    || (db.cyp.cyp3a4 == CypRole::Inducer && da.cyp.cyp3a4 == CypRole::Substrate)
                                                {
                                                    clash = true;
                                                }
                                                if clash {
                                                    sat.add_clause(vec![-(d_vars[&da.id] as Lit), -(d_vars[&db.id] as Lit)]);
                                                }
                                            }
                                        }
                                    }
                                }

                                let mut gathered = Vec::new();
                                let mut total_macro_steps = 0;
                                for _ in 0..self.top_k_options {
                                    let (sol_opt, steps) = sat.solve();
                                    total_macro_steps += steps;
                                    match sol_opt {
                                        Some(sol) => {
                                            let mut picked = Vec::new();
                                            let mut blocking_clause = Vec::new();
                                            let mut tot_h = 0; let mut tot_c = 0; let mut tot_r = 0;
                                            let mut blocked_checkpoints = HashSet::new();
                                            let mut has_trial = false;
                                            let mut has_off = false;

                                            for d in &self.drug_library {
                                                if let Some(&vid) = d_vars.get(&d.id) {
                                                    if *sol.get(&vid).unwrap_or(&false) {
                                                        tot_h += d.tox.hepatic;
                                                        tot_c += d.tox.cardiac;
                                                        tot_r += d.tox.renal;
                                                        blocked_checkpoints.extend(d.targets.clone());
                                                        if d.regulatory == RegulatoryStatus::ClinicalTrial {
                                                            has_trial = true;
                                                        }
                                                        if d.regulatory == RegulatoryStatus::FdaOffLabel {
                                                            has_off = true;
                                                        }
                                                        picked.push(d.clone());
                                                        blocking_clause.push(-(vid as Lit));
                                                    }
                                                }
                                            }

                                            if tot_h <= self.max_hepatic && tot_c <= self.max_cardiac && tot_r <= self.max_renal && picked.len() >= self.k_min && picked.len() <= self.k_max {
                                                let score = tot_h + tot_c + tot_r;
                                                gathered.push(SolutionRegimen {
                                                    rank: 0,
                                                    drugs: picked,
                                                    tot_hepatic: tot_h,
                                                    tot_cardiac: tot_c,
                                                    tot_renal: tot_r,
                                                    total_toxicity_score: score,
                                                    inhibited_checkpoints: blocked_checkpoints,
                                                    covered_subclones: self.detected_subclones.len(),
                                                    requires_clinical_trial: has_trial,
                                                    has_off_label: has_off,
                                                });
                                            }

                                            sat.add_clause(blocking_clause);
                                        }
                                        None => break,
                                    }
                                }

                                self.macro_elapsed_micros = start.elapsed().as_micros();
                                self.macro_metric = Some(sat.compute_spectral_metric("Macro Clonal Regimen", total_macro_steps));

                                if gathered.is_empty() {
                                    self.macro_unsat_reason = Some(
                                        "UNSAT: All combinations exceed patient organ limits or create CYP3A4 pharmacokinetic clashes.".into()
                                    );
                                } else {
                                    gathered.sort_by(|a, b| a.total_toxicity_score.cmp(&b.total_toxicity_score).then_with(|| a.drugs.len().cmp(&b.drugs.len())));
                                    for (idx, r) in gathered.iter_mut().enumerate() {
                                        r.rank = idx + 1;
                                    }
                                    self.macro_solutions = gathered;
                                }
                            }
                        });

                        ui.vertical(|ui| {
                            if !self.detected_subclones.is_empty() {
                                ui.group(|ui| {
                                    ui.label(egui::RichText::new("Targeted Clonal Architecture").strong());
                                    for sc in &self.detected_subclones {
                                        ui.label(format!("• {}: Mutations = {:?}", sc.name, sc.drivers));
                                    }
                                });
                                ui.add_space(6.0);
                            }

                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Synthesized Regimens & Clinical Trial Matches").strong());

                                if let Some(ref reason) = self.macro_unsat_reason {
                                    ui.colored_label(egui::Color32::from_rgb(255, 80, 80), reason);
                                    ui.add_space(8.0);
                                    ui.colored_label(
                                        egui::Color32::from_rgb(255, 200, 50),
                                        "⚡ Multi-Drug Resistance Detected: Continuous combination therapy is non-viable.",
                                    );
                                    ui.label("Two computational options are available:");
                                    ui.horizontal(|ui| {
                                        if ui.button(egui::RichText::new("1. De Novo Pocket Inversion ➔").strong()).clicked() {
                                            self.active_anchors = self.cleft_repo.build_dual_inhibitor_pocket(&self.unresolved_block_targets);
                                            self.current_tab = PipelineTab::MicroDeNovoInversion;
                                            self.micro_solution = None;
                                        }
                                        if ui.button(egui::RichText::new("2. Evolutionary Scheduling (Idea 3) ➔").strong()).clicked() {
                                            self.current_tab = PipelineTab::EvolutionaryScheduling;
                                            self.schedule_result = None;
                                        }
                                    });
                                } else if self.macro_solutions.is_empty() {
                                    ui.label("Click 'Synthesize Clinical Regimens' to evaluate therapeutic combinations.");
                                } else {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(0, 200, 100),
                                        format!("Verified {} clinical solution(s) in {} µs", self.macro_solutions.len(), self.macro_elapsed_micros),
                                    );

                                    egui::ScrollArea::vertical().max_height(460.0).show(ui, |ui| {
                                        for sol in &self.macro_solutions {
                                            ui.add_space(4.0);
                                            ui.group(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.colored_label(
                                                        egui::Color32::from_rgb(255, 200, 80),
                                                        format!("Regimen #{}", sol.rank),
                                                    );
                                                    ui.label(format!("| Cocktail Size: {}", sol.drugs.len()));
                                                    ui.label(format!("| Total Organ Burden: {}", sol.total_toxicity_score));
                                                });

                                                ui.horizontal(|ui| {
                                                    if sol.requires_clinical_trial {
                                                        ui.colored_label(egui::Color32::from_rgb(180, 120, 255), "🗹 REQUIRES CLINICAL TRIAL ENROLLMENT");
                                                    } else if sol.has_off_label {
                                                        ui.colored_label(egui::Color32::from_rgb(255, 200, 50), "🗹 CONTAINS FDA OFF-LABEL COMPONENT");
                                                    } else {
                                                        ui.colored_label(egui::Color32::from_rgb(50, 220, 120), "🗹 100% FDA ON-LABEL STANDARD OF CARE");
                                                    }
                                                });

                                                ui.label(format!(
                                                    "Organ Reserves: Hepatic: {}/{}, Cardiac: {}/{}, Renal: {}/{}",
                                                    sol.tot_hepatic, self.max_hepatic,
                                                    sol.tot_cardiac, self.max_cardiac,
                                                    sol.tot_renal, self.max_renal
                                                ));

                                                ui.label(egui::RichText::new("Components:").underline());
                                                for d in &sol.drugs {
                                                    ui.horizontal(|ui| {
                                                        match d.regulatory {
                                                            RegulatoryStatus::FdaOnLabel => {
                                                                ui.colored_label(egui::Color32::from_rgb(50, 220, 120), "[FDA On-Label]");
                                                            }
                                                            RegulatoryStatus::FdaOffLabel => {
                                                                ui.colored_label(egui::Color32::from_rgb(255, 200, 50), "[Off-Label]");
                                                            }
                                                            RegulatoryStatus::ClinicalTrial => {
                                                                let trial_str = d.trial_id.as_deref().unwrap_or("Investigational");
                                                                ui.colored_label(egui::Color32::from_rgb(180, 120, 255), format!("[Trial {}]", trial_str));
                                                            }
                                                        }
                                                        ui.label(format!("{} (Targets: {:?})", d.name, d.targets));
                                                    });
                                                }

                                                ui.colored_label(
                                                    egui::Color32::from_rgb(80, 180, 255),
                                                    format!("Severed Checkpoints: {:?}", sol.inhibited_checkpoints),
                                                );
                                            });
                                        }
                                    });
                                }
                            });
                        });
                    });
                }

                // =========================================================
                // TAB 2: MICRO DE NOVO MOLECULAR INVERSION
                // =========================================================
                PipelineTab::MicroDeNovoInversion => {
                    egui::Grid::new("micro_grid").num_columns(2).spacing([30.0, 10.0]).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("1. Active Crystallographic Pocket Anchors").strong());
                                ui.label("Merged Target Cleft Coordinates (in Å):");
                                for a in &self.active_anchors {
                                    ui.label(format!("• [{}] {}: {:?} at [{:.1}, {:.1}, {:.1}]", a.target_gene, a.label, a.feature, a.coord[0], a.coord[1], a.coord[2]));
                                }
                            });

                            ui.add_space(6.0);
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("2. Scaffold Synthesis Budget").strong());
                                ui.horizontal(|ui| {
                                    ui.label("Candidate Template Nodes:");
                                    ui.add(egui::Slider::new(&mut self.num_micro_nodes, 4..=9));
                                });
                                ui.label("Chemical Fragments: [C(sp3), N(Donor), O(Acceptor), Benzene (Ph), Pyridine (Pyr)]");
                            });

                            ui.add_space(10.0);
                            if ui.button(egui::RichText::new("Invert Dual-Pocket & Synthesize").heading()).clicked() {
                                let pipeline = MolecularInversionPipeline::new(self.active_anchors.clone(), self.num_micro_nodes);
                                let start = std::time::Instant::now();
                                let (sol, metric) = pipeline.synthesize();
                                let elapsed = start.elapsed().as_micros();
                                self.micro_metric = Some(metric);

                                match sol {
                                    Some((nodes, bonds, matches)) => {
                                        self.generated_smiles = ChemistryExporter::to_smiles(&nodes, &bonds);
                                        self.export_status_msg = None;
                                        self.micro_solution = Some(Ok((nodes, bonds, matches, elapsed)));
                                    }
                                    None => {
                                        self.generated_smiles.clear();
                                        self.micro_solution = Some(Err("UNSAT: Template nodes cannot bridge merged binding pockets without steric clash.".into()));
                                    }
                                }
                            }
                        });

                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Synthesized De Novo Dual-Inhibitor").strong());

                                match &self.micro_solution {
                                    Some(Ok((nodes, bonds, matches, micros))) => {
                                        ui.colored_label(egui::Color32::from_rgb(0, 220, 120), format!("SAT: Scaffold Inverted in {} µs", micros));
                                        ui.label(format!("Composition: {} fragments, {} covalent linkers", nodes.len(), bonds.len()));

                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("Canonical SMILES:").strong());
                                            ui.text_edit_singleline(&mut self.generated_smiles);
                                        });

                                        ui.separator();
                                        ui.label(egui::RichText::new("Target Pocket Engagements:").underline());
                                        for (node_id, anchor_id) in matches {
                                            let a = &self.active_anchors[*anchor_id];
                                            ui.label(format!("• Node #{} ({:?}) ──> Pins [{}] {} ({:?})", 
                                                node_id, nodes[*node_id].1, a.target_gene, a.label, a.feature));
                                        }

                                        ui.separator();
                                        ui.label(egui::RichText::new("Dual-Inhibitor Topology:").strong());

                                        let (response, painter) = ui.allocate_painter(egui::vec2(440.0, 240.0), egui::Sense::hover());
                                        let rect = response.rect;
                                        painter.rect_filled(rect, 4.0_f32, egui::Color32::from_rgb(20, 20, 24));

                                        let n = nodes.len();
                                        let center = rect.center();
                                        let radius = 95.0_f32;

                                        let mut positions = Vec::new();
                                        for i in 0..n {
                                            let angle = (i as f32) * (2.0_f32 * std::f32::consts::PI / (n as f32)) - std::f32::consts::FRAC_PI_2;
                                            let pos = center + egui::vec2(radius * angle.cos(), radius * angle.sin());
                                            positions.push(pos);
                                        }

                                        for &(u, v) in bonds {
                                            painter.line_segment([positions[u], positions[v]], egui::Stroke::new(3.0_f32, egui::Color32::from_rgb(180, 180, 190)));
                                        }

                                        for (id, frag) in nodes {
                                            let pos = positions[*id];
                                            let is_pinned = matches.iter().any(|(nid, _)| *nid == *id);
                                            let stroke = if is_pinned {
                                                egui::Stroke::new(2.5_f32, egui::Color32::from_rgb(0, 240, 140))
                                            } else {
                                                egui::Stroke::new(1.0_f32, egui::Color32::WHITE)
                                            };

                                            match frag {
                                                FragmentKind::BenzeneRing => {
                                                    let hex_r = 18.0_f32;
                                                    let mut hex_points = Vec::new();
                                                    for h in 0..6 {
                                                        let h_ang = (h as f32) * (std::f32::consts::PI / 3.0_f32);
                                                        hex_points.push(pos + egui::vec2(hex_r * h_ang.cos(), hex_r * h_ang.sin()));
                                                    }
                                                    painter.add(egui::Shape::convex_polygon(hex_points, egui::Color32::from_rgb(80, 50, 110), stroke));
                                                    painter.circle_stroke(pos, 8.0_f32, egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(200, 170, 255)));
                                                    painter.text(pos, egui::Align2::CENTER_CENTER, "Ph", egui::FontId::proportional(11.0), egui::Color32::WHITE);
                                                }
                                                FragmentKind::PyridineRing => {
                                                    let hex_r = 18.0_f32;
                                                    let mut hex_points = Vec::new();
                                                    for h in 0..6 {
                                                        let h_ang = (h as f32) * (std::f32::consts::PI / 3.0_f32);
                                                        hex_points.push(pos + egui::vec2(hex_r * h_ang.cos(), hex_r * h_ang.sin()));
                                                    }
                                                    painter.add(egui::Shape::convex_polygon(hex_points, egui::Color32::from_rgb(30, 80, 140), stroke));
                                                    painter.circle_stroke(pos, 8.0_f32, egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(120, 200, 255)));
                                                    painter.text(pos, egui::Align2::CENTER_CENTER, "Pyr", egui::FontId::proportional(10.0), egui::Color32::WHITE);
                                                }
                                                FragmentKind::CarbonSp3 => {
                                                    painter.circle(pos, 13.0_f32, egui::Color32::from_rgb(100, 100, 110), stroke);
                                                    painter.text(pos, egui::Align2::CENTER_CENTER, "C", egui::FontId::proportional(12.0), egui::Color32::WHITE);
                                                }
                                                FragmentKind::NitrogenDonor => {
                                                    painter.circle(pos, 13.0_f32, egui::Color32::from_rgb(60, 140, 255), stroke);
                                                    painter.text(pos, egui::Align2::CENTER_CENTER, "N", egui::FontId::proportional(12.0), egui::Color32::WHITE);
                                                }
                                                FragmentKind::OxygenAcceptor => {
                                                    painter.circle(pos, 13.0_f32, egui::Color32::from_rgb(255, 70, 70), stroke);
                                                    painter.text(pos, egui::Align2::CENTER_CENTER, "O", egui::FontId::proportional(12.0), egui::Color32::WHITE);
                                                }
                                            }
                                        }

                                        ui.add_space(8.0);
                                        ui.separator();
                                        ui.label(egui::RichText::new("Computational Chemistry Exporters").strong());

                                        ui.horizontal(|ui| {
                                            if ui.button(egui::RichText::new("📁 Export to .SDF (V2000)").strong()).clicked() {
                                                let coords = ChemistryExporter::compute_3d_coordinates(nodes, matches, &self.active_anchors);
                                                let filepath = "exports/synthesized_dual_inhibitor.sdf";
                                                match ChemistryExporter::export_sdf(filepath, nodes, bonds, &coords) {
                                                    Ok(_) => self.export_status_msg = Some(format!("✓ Saved: {}", filepath)),
                                                    Err(e) => self.export_status_msg = Some(format!("Export failed: {}", e)),
                                                }
                                            }

                                            if ui.button(egui::RichText::new("📁 Export to .PDB (3D)").strong()).clicked() {
                                                let coords = ChemistryExporter::compute_3d_coordinates(nodes, matches, &self.active_anchors);
                                                let filepath = "exports/synthesized_dual_inhibitor.pdb";
                                                match ChemistryExporter::export_pdb(filepath, nodes, bonds, &coords) {
                                                    Ok(_) => self.export_status_msg = Some(format!("✓ Saved: {}", filepath)),
                                                    Err(e) => self.export_status_msg = Some(format!("Export failed: {}", e)),
                                                }
                                            }
                                        });

                                        if let Some(ref msg) = self.export_status_msg {
                                            ui.colored_label(egui::Color32::from_rgb(80, 220, 255), msg);
                                        }
                                    }
                                    Some(Err(err)) => {
                                        ui.colored_label(egui::Color32::from_rgb(255, 80, 80), err);
                                    }
                                    None => {
                                        ui.label("Click 'Invert Dual-Pocket & Synthesize' to build the bifunctional ligand.");
                                    }
                                }
                            });
                        });
                    });
                }

                // =========================================================
                // TAB 3: EVOLUTIONARY TEMPORAL SCHEDULING (IDEA 3)
                // =========================================================
                PipelineTab::EvolutionaryScheduling => {
                    egui::Grid::new("sched_grid").num_columns(2).spacing([30.0, 10.0]).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("1. Active Subclone Drivers to Steer").strong());
                                if self.unresolved_block_targets.is_empty() {
                                    ui.label("No active drivers imported from NGS panel. Run Macro Regimen first or use default:");
                                    ui.colored_label(egui::Color32::from_rgb(80, 180, 255), "Default Targets: [\"EGFR\", \"MET\", \"PI3K\"]");
                                } else {
                                    ui.colored_label(egui::Color32::from_rgb(80, 180, 255), format!("Imported Targets: {:?}", self.unresolved_block_targets));
                                }
                            });

                            ui.add_space(6.0);
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("2. Temporal Scheduling Parameters").strong());
                                ui.horizontal(|ui| {
                                    ui.label("Treatment Cycles (T):");
                                    ui.add(egui::Slider::new(&mut self.num_cycles, 2..=5));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Cumulative Tox Budget:");
                                    ui.add(egui::Slider::new(&mut self.max_cum_toxicity, 12..=36));
                                });
                                ui.checkbox(&mut self.enforce_cyp, "Enforce Per-Cycle CYP3A4 Exclusions");
                            });

                            ui.add_space(6.0);
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("3. Documented Collateral Sensitivity Rules").strong());
                                ui.label("• Cycle t (Osimertinib) ➔ Steers MET up-regulation ➔ Primes Cycle t+1 (Capmatinib)");
                                ui.label("• Cycle t (Capmatinib) ➔ Steers PI3K dependence ➔ Primes Cycle t+1 (Alpelisib)");
                                ui.label("• Cycle t (Trametinib) ➔ RTK feedback rebound ➔ Re-sensitizes Cycle t+1 (Osimertinib)");
                                ui.label("• Cycle t (Sotorasib) ➔ RTK feedback rebound ➔ Primes Cycle t+1 (Osimertinib)");
                                ui.label("• Cycle t (Encorafenib) ➔ CDK4 escape ➔ Primes Cycle t+1 (Palbociclib)");
                            });

                            ui.add_space(10.0);
                            if ui.button(egui::RichText::new("Compute Evolutionary Schedule").heading()).clicked() {
                                let active_drivers = if self.unresolved_block_targets.is_empty() {
                                    vec!["EGFR".into(), "MET".into(), "PI3K".into()]
                                } else {
                                    self.unresolved_block_targets.clone()
                                };

                                let active_drugs: Vec<Drug> = self.drug_library.iter()
                                    .enumerate()
                                    .filter(|(i, _)| self.selected_drugs[*i])
                                    .map(|(_, d)| d.clone())
                                    .collect();

                                let scheduler = EvolutionaryTemporalScheduler::new(
                                    active_drivers,
                                    active_drugs,
                                    self.num_cycles,
                                    self.max_hepatic,
                                    self.max_cardiac,
                                    self.max_renal,
                                    self.max_cum_toxicity,
                                    self.enforce_cyp,
                                );

                                let (res, metric) = scheduler.schedule();
                                self.sched_metric = Some(metric);

                                match res {
                                    Some(sched) => self.schedule_result = Some(Ok(sched)),
                                    None => self.schedule_result = Some(Err("UNSAT: No alternating schedule satisfies evolutionary suppression and cumulative organ limits.".into())),
                                }
                            }
                        });

                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Synthesized Multi-Cycle Evolutionary Schedule").strong());

                                match &self.schedule_result {
                                    Some(Ok(res)) => {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(0, 220, 120),
                                            format!("SAT: Dynamic Schedule Optimized in {} µs", res.elapsed_micros),
                                        );
                                        ui.label(format!("Horizon: {} treatment cycles with organ rest windows", res.num_cycles));

                                        ui.separator();
                                        ui.label(egui::RichText::new("Pulsed Cycle Plan:").underline());

                                        for (c_idx, drugs) in res.cycle_drugs.iter().enumerate() {
                                            let (h, c, r) = res.cycle_toxicities[c_idx];
                                            ui.group(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.colored_label(
                                                        egui::Color32::from_rgb(255, 200, 80),
                                                        format!("Treatment Cycle #{}", c_idx + 1),
                                                    );
                                                    ui.label(format!("(Organ Load: H:{}, C:{}, R:{})", h, c, r));
                                                });

                                                ui.horizontal(|ui| {
                                                    ui.label("Administered Compounds:");
                                                    for d in drugs {
                                                        ui.colored_label(egui::Color32::from_rgb(80, 220, 255), format!("[{}]", d.name));
                                                    }
                                                });
                                            });
                                        }

                                        ui.separator();
                                        ui.label(egui::RichText::new("Clonal Viability Trajectories:").strong());
                                        for (driver, traj) in &res.subclone_trajectories {
                                            ui.horizontal(|ui| {
                                                ui.label(format!("• Subclone [{}] State:", driver));
                                                for (t_idx, &is_active) in traj.iter().enumerate() {
                                                    if is_active {
                                                        ui.colored_label(egui::Color32::from_rgb(255, 80, 80), format!("C{}:Active", t_idx + 1));
                                                    } else {
                                                        ui.colored_label(egui::Color32::from_rgb(50, 220, 120), format!("C{}:Suppressed", t_idx + 1));
                                                    }
                                                }
                                            });
                                        }

                                        if !res.collateral_switches_triggered.is_empty() {
                                            ui.separator();
                                            ui.label(egui::RichText::new("Evolutionary Traps Triggered:").strong());
                                            for switch in &res.collateral_switches_triggered {
                                                ui.colored_label(egui::Color32::from_rgb(180, 140, 255), format!("⚡ {}", switch));
                                            }
                                        }
                                    }
                                    Some(Err(err)) => {
                                        ui.colored_label(egui::Color32::from_rgb(255, 80, 80), err);
                                    }
                                    None => {
                                        ui.label("Click 'Compute Evolutionary Schedule' to synthesize an alternating regimen.");
                                    }
                                }
                            });
                        });
                    });
                }

                // =========================================================
                // TAB 4: STRUCTURAL COMPLEXITY & SPECTRAL PHASE GAUGES (IDEA 4)
                // =========================================================
                PipelineTab::SpectralComplexityGauges => {
                    ui.heading("Structural Complexity Theory & Spectral Phase-Transition Density Gauges");
                    ui.label("Formal mapping of computational hardness, entropy reduction exponent s, and solution collapse 2^(n-s).");
                    ui.separator();

                    let metrics: Vec<(&str, Option<&SpectralComplexityMetric>)> = vec![
                        ("1. Macro Clonal Hitting Set", self.macro_metric.as_ref()),
                        ("2. Micro De Novo Pocket Inversion", self.micro_metric.as_ref()),
                        ("3. Temporal Evolutionary Game", self.sched_metric.as_ref()),
                    ];

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (title, opt_metric) in metrics {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new(title).strong().heading());

                                match opt_metric {
                                    Some(m) => {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("Formula Size: n = {} variables, m = {} clauses", m.num_vars, m.num_clauses));
                                            ui.colored_label(egui::Color32::from_rgb(80, 220, 255), format!("|  α = m/n = {:.3}", m.alpha));
                                            ui.label(format!("|  α_c (Critical Boundary) = {:.3}", m.alpha_critical));
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label(format!("Search Space: 2^{:.1} states  ➔  Resolved in {} steps", m.raw_search_space_log2, m.resolved_steps));
                                            ui.colored_label(egui::Color32::from_rgb(0, 240, 140), format!("|  Reduction Exponent: s = {:.2}", m.reduction_s));
                                            ui.label(format!("|  Search Tree Compression: {:.1}%", m.compression_ratio));
                                        });

                                        ui.label(egui::RichText::new(format!("Operational Regime: {}", m.regime)).italics());

                                        // Render Spectral Phase Density Bar (Painter)
                                        let (response, painter) = ui.allocate_painter(egui::vec2(600.0, 36.0), egui::Sense::hover());
                                        let rect = response.rect;

                                        // Background spectrum gradient representation
                                        let segments = 50;
                                        let dx = rect.width() / (segments as f32);
                                        for k in 0..segments {
                                            let frac = (k as f32) / (segments as f32);
                                            let seg_rect = egui::Rect::from_min_size(
                                                egui::pos2(rect.min.x + (k as f32) * dx, rect.min.y + 6.0),
                                                egui::vec2(dx, 18.0),
                                            );
                                            // Color ramp: Green (under-constrained) -> Yellow/Red (critical) -> Purple (over-constrained)
                                            let color = if frac < 0.4 {
                                                egui::Color32::from_rgb(40, 140, 80)
                                            } else if frac < 0.65 {
                                                egui::Color32::from_rgb(220, 160, 40)
                                            } else {
                                                egui::Color32::from_rgb(140, 50, 180)
                                            };
                                            painter.rect_filled(seg_rect, 1.0_f32, color);
                                        }

                                        // Critical boundary line at alpha ~ 4.267 (approx 55% of the gauge)
                                        let crit_x = rect.min.x + rect.width() * 0.55;
                                        painter.line_segment(
                                            [egui::pos2(crit_x, rect.min.y), egui::pos2(crit_x, rect.max.y)],
                                            egui::Stroke::new(2.0_f32, egui::Color32::WHITE),
                                        );

                                        // Current alpha needle
                                        let needle_frac = ((m.alpha / 8.0) as f32).clamp(0.02, 0.98);
                                        let needle_x = rect.min.x + rect.width() * needle_frac;
                                        painter.circle_filled(egui::pos2(needle_x, rect.center().y), 6.5_f32, egui::Color32::from_rgb(0, 255, 255));
                                        painter.line_segment(
                                            [egui::pos2(needle_x, rect.min.y - 2.0), egui::pos2(needle_x, rect.max.y + 2.0)],
                                            egui::Stroke::new(2.5_f32, egui::Color32::from_rgb(0, 255, 255)),
                                        );
                                    }
                                    None => {
                                        ui.label(egui::RichText::new("Awaiting execution. Run this solver module to record its spectral density and reduction metrics.").italics());
                                    }
                                }
                            });
                            ui.add_space(8.0);
                        }
                    });
                }
            }
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 860.0])
            .with_title("Reasoning Engine - Unified Precision Oncology Pipeline"),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Unified Precision Oncology Pipeline",
        native_options,
        Box::new(|_cc| Box::new(UnifiedOncologyApp::default())),
    );
}
