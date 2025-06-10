//! Model checker for semitopology propositions.

use crate::canon::Family;
use std::collections::HashMap;


/// Open expressions that can be evaluated to concrete open sets
#[derive(Debug, Clone, PartialEq)]
pub enum OpenExpr {
    /// Simple open variable
    Var(String),
    /// Community of a point (K p)
    Community(String),
    /// Interior complement of an open expression (IC O)
    InteriorComplement(Box<OpenExpr>),
}

/// Atomic propositions
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    /// Point x is in open expression
    PointInOpen(String, OpenExpr),
    /// Two open expressions intersect
    OpenIntersection(OpenExpr, OpenExpr),
    /// Open expression is nonempty
    OpenNonempty(OpenExpr),
}

/// Proposition formulas
#[derive(Debug, Clone, PartialEq)]
pub enum Formula {
    /// Atomic proposition
    Atom(Atom),
    /// Logical negation
    Not(Box<Formula>),
    /// Logical conjunction
    And(Box<Formula>, Box<Formula>),
    /// Logical disjunction
    Or(Box<Formula>, Box<Formula>),
    /// Logical implication
    Implies(Box<Formula>, Box<Formula>),
    /// Universal quantification over points
    ForAllPoints(String, Box<Formula>),
    /// Existential quantification over points
    ExistsPoints(String, Box<Formula>),
    /// Universal quantification over opens
    ForAllOpens(String, Box<Formula>),
    /// Existential quantification over opens
    ExistsOpens(String, Box<Formula>),
}

/// Assignment of variables to concrete values
#[derive(Debug, Clone)]
pub struct Assignment {
    /// Maps point variable names to point indices (1-based)
    pub points: HashMap<String, usize>,
    /// Maps open variable names to subset bitmasks
    pub opens: HashMap<String, u32>,
}

impl Assignment {
    pub fn new() -> Self {
        Self {
            points: HashMap::new(),
            opens: HashMap::new(),
        }
    }
    
    pub fn assign_point(&mut self, var: String, point: usize) {
        self.points.insert(var, point);
    }
    
    pub fn assign_open(&mut self, var: String, open: u32) {
        self.opens.insert(var, open);
    }
    
    pub fn clone_with_point(&self, var: String, point: usize) -> Self {
        let mut new_assignment = self.clone();
        new_assignment.assign_point(var, point);
        new_assignment
    }
    
    pub fn clone_with_open(&self, var: String, open: u32) -> Self {
        let mut new_assignment = self.clone();
        new_assignment.assign_open(var, open);
        new_assignment
    }
}

/// Witness for existential quantification
#[derive(Debug, Clone)]
pub enum Witness {
    Point(usize),
    Open(u32),
}

/// Result of model checking with witnesses
#[derive(Debug, Clone)]
pub struct ModelCheckResult {
    pub satisfied: bool,
    pub witnesses: HashMap<String, Witness>,
}

impl ModelCheckResult {
    pub fn true_result() -> Self {
        Self {
            satisfied: true,
            witnesses: HashMap::new(),
        }
    }
    
    pub fn false_result() -> Self {
        Self {
            satisfied: false,
            witnesses: HashMap::new(),
        }
    }
    
    pub fn with_witness(mut self, var: String, witness: Witness) -> Self {
        self.witnesses.insert(var, witness);
        self
    }
}

/// Model checker for propositions against semitopologies
pub struct ModelChecker {
    n: usize,
    family: Family,
    antipode_cache: Option<HashMap<u32, u32>>,
}

impl ModelChecker {
    pub fn new(n: usize, family: Family) -> Self {
        Self { 
            n, 
            family,
            antipode_cache: None,
        }
    }
    
    /// Build the antipode table: anti[O] = ⋃{P ∈ τ | P ∩ O = ∅}
    fn build_antipodes(&self) -> HashMap<u32, u32> {
        let mut anti: HashMap<u32, u32> = HashMap::new();
        for &o in &self.family {
            anti.insert(o, 0);                   // allocate entry
        }
        for &o in &self.family {
            for &q in &self.family {
                if o & q == 0 {                  // disjoint
                    *anti.get_mut(&o).unwrap() |= q;
                }
            }
        }
        anti
    }
    
    /// Calculate interior complement of open O: largest open disjoint from O
    fn interior_complement(&self, o: u32) -> u32 {
        let mut complement = 0u32;
        for &q in &self.family {
            if o & q == 0 {  // q is disjoint from o
                complement |= q;
            }
        }
        complement
    }
    
    /// Calculate community of point p using cached antipode table
    fn community_with_cache(
        &self,
        p: usize,
        anti: &HashMap<u32, u32>,
    ) -> u32 {
        if p == 0 || p > self.n || self.family.is_empty() {
            return 0;
        }

        let universe: u32 = if self.n == 32 { u32::MAX } else { (1u32 << self.n) - 1 };
        let p_bit: u32     = 1u32 << (p - 1);

        // 1) gather everything separable from p via the pre-computed table
        let mut separable: u32 = 0;
        for &o in &self.family {
            if o & p_bit != 0 {
                separable |= anti[&o];           // O ∋ p   ⇒   throw away anti(O)
            }
        }

        // 2) inseparable class
        let class: u32 = universe & !separable;

        // 3) interior
        let mut community: u32 = 0;
        for &o in &self.family {
            if o & !class == 0 {
                community |= o;
            }
        }
        community
    }
    
    /// Ensure antipode cache is built and return reference to it
    fn get_antipode_cache(&mut self) -> &HashMap<u32, u32> {
        if self.antipode_cache.is_none() {
            self.antipode_cache = Some(self.build_antipodes());
        }
        self.antipode_cache.as_ref().unwrap()
    }
    
    /// Check if a point is in an open (subset)
    fn point_in_open(&self, point: usize, open: u32) -> bool {
        if point == 0 || point > self.n {
            false
        } else {
            let bit_pos = point - 1;
            (open >> bit_pos) & 1 == 1
        }
    }
    
    /// Check if two opens (subsets) intersect
    fn opens_intersect(&self, open1: u32, open2: u32) -> bool {
        (open1 & open2) != 0
    }
    
    /// Check if an open (subset) is nonempty
    fn open_is_nonempty(&self, open: u32) -> bool {
        open != 0
    }
    
    /// Evaluate an open expression to a concrete open set
    fn eval_open_expr(&mut self, open_expr: &OpenExpr, assignment: &Assignment) -> Option<u32> {
        match open_expr {
            OpenExpr::Var(var) => {
                assignment.opens.get(var).copied()
            }
            OpenExpr::Community(point_var) => {
                if let Some(&point) = assignment.points.get(point_var) {
                    let anti = self.get_antipode_cache().clone();
                    Some(self.community_with_cache(point, &anti))
                } else {
                    None
                }
            }
            OpenExpr::InteriorComplement(inner_expr) => {
                if let Some(inner_open) = self.eval_open_expr(inner_expr, assignment) {
                    Some(self.interior_complement(inner_open))
                } else {
                    None
                }
            }
        }
    }

    /// Evaluate an atomic proposition under an assignment
    fn eval_atom(&mut self, atom: &Atom, assignment: &Assignment) -> bool {
        match atom {
            Atom::PointInOpen(point_var, open_expr) => {
                if let Some(&point) = assignment.points.get(point_var) {
                    if let Some(open) = self.eval_open_expr(open_expr, assignment) {
                        self.point_in_open(point, open)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Atom::OpenIntersection(open_expr1, open_expr2) => {
                if let (Some(open1), Some(open2)) = (
                    self.eval_open_expr(open_expr1, assignment),
                    self.eval_open_expr(open_expr2, assignment)
                ) {
                    self.opens_intersect(open1, open2)
                } else {
                    false
                }
            }
            Atom::OpenNonempty(open_expr) => {
                if let Some(open) = self.eval_open_expr(open_expr, assignment) {
                    self.open_is_nonempty(open)
                } else {
                    false
                }
            }
        }
    }
    
    /// Evaluate a formula under an assignment, returning witnesses for existential quantifiers
    pub fn eval_formula(&mut self, formula: &Formula, assignment: &Assignment) -> ModelCheckResult {
        match formula {
            Formula::Atom(atom) => {
                if self.eval_atom(atom, assignment) {
                    ModelCheckResult::true_result()
                } else {
                    ModelCheckResult::false_result()
                }
            }
            Formula::Not(f) => {
                let result = self.eval_formula(f, assignment);
                ModelCheckResult {
                    satisfied: !result.satisfied,
                    witnesses: result.witnesses,
                }
            }
            Formula::And(f1, f2) => {
                let result1 = self.eval_formula(f1, assignment);
                if !result1.satisfied {
                    return result1;
                }
                let result2 = self.eval_formula(f2, assignment);
                if !result2.satisfied {
                    return result2;
                }
                // Combine witnesses from both subformulas
                let mut combined_witnesses = result1.witnesses;
                combined_witnesses.extend(result2.witnesses);
                ModelCheckResult {
                    satisfied: true,
                    witnesses: combined_witnesses,
                }
            }
            Formula::Or(f1, f2) => {
                let result1 = self.eval_formula(f1, assignment);
                if result1.satisfied {
                    return result1;
                }
                let result2 = self.eval_formula(f2, assignment);
                if result2.satisfied {
                    return result2;
                }
                ModelCheckResult::false_result()
            }
            Formula::Implies(f1, f2) => {
                let result1 = self.eval_formula(f1, assignment);
                if !result1.satisfied {
                    return ModelCheckResult::true_result();
                }
                self.eval_formula(f2, assignment)
            }
            Formula::ForAllPoints(var, f) => {
                for point in 1..=self.n {
                    let new_assignment = assignment.clone_with_point(var.clone(), point);
                    let result = self.eval_formula(f, &new_assignment);
                    if !result.satisfied {
                        return ModelCheckResult::false_result();
                    }
                }
                ModelCheckResult::true_result()
            }
            Formula::ExistsPoints(var, f) => {
                for point in 1..=self.n {
                    let new_assignment = assignment.clone_with_point(var.clone(), point);
                    let result = self.eval_formula(f, &new_assignment);
                    if result.satisfied {
                        return result.with_witness(var.clone(), Witness::Point(point));
                    }
                }
                ModelCheckResult::false_result()
            }
            Formula::ForAllOpens(var, f) => {
                let family_vec: Vec<u32> = self.family.iter().cloned().collect();
                for open in family_vec {
                    let new_assignment = assignment.clone_with_open(var.clone(), open);
                    let result = self.eval_formula(f, &new_assignment);
                    if !result.satisfied {
                        return ModelCheckResult::false_result();
                    }
                }
                ModelCheckResult::true_result()
            }
            Formula::ExistsOpens(var, f) => {
                let family_vec: Vec<u32> = self.family.iter().cloned().collect();
                for open in family_vec {
                    let new_assignment = assignment.clone_with_open(var.clone(), open);
                    let result = self.eval_formula(f, &new_assignment);
                    if result.satisfied {
                        return result.with_witness(var.clone(), Witness::Open(open));
                    }
                }
                ModelCheckResult::false_result()
            }
        }
    }
    
    /// Check if a formula is satisfied by the semitopology
    pub fn check(&mut self, formula: &Formula) -> ModelCheckResult {
        let assignment = Assignment::new();
        self.eval_formula(formula, &assignment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn test_build_antipodes_basic() {
        // Example from documentation comment
        let mut family = BTreeSet::new();
        family.insert(0b00); // {}
        family.insert(0b01); // {1}
        family.insert(0b10); // {2}
        family.insert(0b11); // {1,2}
        
        let checker = ModelChecker::new(2, family.clone());
        let anti = checker.build_antipodes();
        
        // anti[{}] = union of sets disjoint from {} = all points (0b11 for n=2)
        assert_eq!(anti[&0b00], 0b11);
        // anti[{1}] = union of sets disjoint from {1} = {} ∪ {2}
        assert_eq!(anti[&0b01], 0b10);
        // anti[{2}] = union of sets disjoint from {2} = {} ∪ {1}  
        assert_eq!(anti[&0b10], 0b01);
        // anti[{1,2}] = union of sets disjoint from {1,2} = {}
        assert_eq!(anti[&0b11], 0b00);
    }

    #[test]
    fn test_community_simple_case() {
        // Test community calculation on τ = {∅, {1}, {2}, {1,2}}
        let mut family = BTreeSet::new();
        family.insert(0b00); // {}
        family.insert(0b01); // {1}
        family.insert(0b10); // {2}  
        family.insert(0b11); // {1,2}
        
        let checker = ModelChecker::new(2, family.clone());
        let anti = checker.build_antipodes();
        
        // For K_1:
        // Opens containing 1: {1}, {1,2}
        // Separable from 1: anti[{1}] ∪ anti[{1,2}] = {2} ∪ {} = {2}
        // Class of 1: {1,2} \ {2} = {1}
        // Interior of {1}: {} ∪ {1} = {1}
        let k1 = checker.community_with_cache(1, &anti);
        assert_eq!(k1, 0b01); // {1}
        
        // For K_2: 
        // Opens containing 2: {2}, {1,2}
        // Separable from 2: anti[{2}] ∪ anti[{1,2}] = {1} ∪ {} = {1}
        // Class of 2: {1,2} \ {1} = {2}
        // Interior of {2}: {} ∪ {2} = {2}
        let k2 = checker.community_with_cache(2, &anti);
        assert_eq!(k2, 0b10); // {2}
    }

    #[test]
    fn test_community_sierpinski_case() {
        // Test the famous τ = {∅, {1,2}, {1,3}, {1,2,3}} semitopology
        let mut family = BTreeSet::new();
        family.insert(0b000); // {}
        family.insert(0b011); // {1,2}
        family.insert(0b101); // {1,3}
        family.insert(0b111); // {1,2,3}
        
        let checker = ModelChecker::new(3, family.clone());
        let anti = checker.build_antipodes();
        
        // For K_1:
        // Opens containing 1: {1,2}, {1,3}, {1,2,3}
        // Separable from 1: anti[{1,2}] ∪ anti[{1,3}] ∪ anti[{1,2,3}] = {} ∪ {} ∪ {} = {}
        // Class of 1: {1,2,3} \ {} = {1,2,3}
        // Interior of {1,2,3}: entire family union = {1,2,3}
        let k1 = checker.community_with_cache(1, &anti);
        assert_eq!(k1, 0b111); // {1,2,3}
        
        // For K_2:
        // Opens containing 2: {1,2}, {1,2,3}
        // Separable from 2: anti[{1,2}] ∪ anti[{1,2,3}] = {} ∪ {} = {}
        // Class of 2: {1,2,3} \ {} = {1,2,3}
        // Interior of {1,2,3}: entire family union = {1,2,3}
        let k2 = checker.community_with_cache(2, &anti);
        assert_eq!(k2, 0b111); // {1,2,3}
        
        // For K_3:
        // Opens containing 3: {1,3}, {1,2,3}
        // Separable from 3: anti[{1,3}] ∪ anti[{1,2,3}] = {} ∪ {} = {}
        // Class of 3: {1,2,3} \ {} = {1,2,3}
        // Interior of {1,2,3}: entire family union = {1,2,3}
        let k3 = checker.community_with_cache(3, &anti);
        assert_eq!(k3, 0b111); // {1,2,3}
    }

    #[test]
    fn test_community_disconnected_case() {
        // Test on τ = {∅, {1}, {2}, {3}, {1,2}} where point 3 is isolated
        let mut family = BTreeSet::new();
        family.insert(0b000); // {}
        family.insert(0b001); // {1}
        family.insert(0b010); // {2}
        family.insert(0b100); // {3}
        family.insert(0b011); // {1,2}
        
        let checker = ModelChecker::new(3, family.clone());
        let anti = checker.build_antipodes();
        
        // First verify antipodes are correct
        assert_eq!(anti[&0b000], 0b111); // anti[{}] = everything
        assert_eq!(anti[&0b001], 0b110); // anti[{1}] = {2,3}
        assert_eq!(anti[&0b010], 0b101); // anti[{2}] = {1,3}
        assert_eq!(anti[&0b100], 0b011); // anti[{3}] = {1,2}
        assert_eq!(anti[&0b011], 0b100); // anti[{1,2}] = {3}
        
        // For K_1:
        // Opens containing 1: {1}, {1,2}
        // Separable: anti[{1}] ∪ anti[{1,2}] = {2,3} ∪ {3} = {2,3}
        // Class of 1: {1,2,3} \ {2,3} = {1}
        // Interior: {} ∪ {1} = {1}
        let k1 = checker.community_with_cache(1, &anti);
        assert_eq!(k1, 0b001); // {1}
        
        // For K_2:
        // Opens containing 2: {2}, {1,2}
        // Separable: anti[{2}] ∪ anti[{1,2}] = {1,3} ∪ {3} = {1,3}
        // Class of 2: {1,2,3} \ {1,3} = {2}
        // Interior: {} ∪ {2} = {2}
        let k2 = checker.community_with_cache(2, &anti);
        assert_eq!(k2, 0b010); // {2}
        
        // For K_3:
        // Opens containing 3: {3}
        // Separable: anti[{3}] = {1,2}
        // Class of 3: {1,2,3} \ {1,2} = {3}
        // Interior: {} ∪ {3} = {3}
        let k3 = checker.community_with_cache(3, &anti);
        assert_eq!(k3, 0b100); // {3}
    }

    #[test]
    fn test_community_degenerate_cases() {
        // Test empty family
        let family = BTreeSet::new();
        let checker = ModelChecker::new(2, family);
        let anti = checker.build_antipodes();
        assert_eq!(checker.community_with_cache(1, &anti), 0);
        
        // Test single point family
        let mut family = BTreeSet::new();
        family.insert(0b0); // {}
        let checker = ModelChecker::new(1, family);
        let anti = checker.build_antipodes();
        assert_eq!(checker.community_with_cache(1, &anti), 0); // K_1 = {}
        
        // Test indiscrete topology 
        let mut family = BTreeSet::new();
        family.insert(0b00); // {}
        family.insert(0b11); // {1,2}
        let checker = ModelChecker::new(2, family);
        let anti = checker.build_antipodes();
        
        // Both points have community = entire space
        assert_eq!(checker.community_with_cache(1, &anti), 0b11);
        assert_eq!(checker.community_with_cache(2, &anti), 0b11);
    }

    #[test]
    fn test_community_edge_cases() {
        let mut family = BTreeSet::new();
        family.insert(0b001); // {1}
        let checker = ModelChecker::new(2, family);
        let anti = checker.build_antipodes();
        
        // Invalid point indices
        assert_eq!(checker.community_with_cache(0, &anti), 0);
        assert_eq!(checker.community_with_cache(3, &anti), 0);
        
        // Empty family edge case
        // Note: community_with_cache must not index anti when the family is empty
        let empty_family = BTreeSet::new();
        let empty_checker = ModelChecker::new(2, empty_family);
        let empty_anti = empty_checker.build_antipodes();
        assert_eq!(empty_checker.community_with_cache(1, &empty_anti), 0);
    }

    #[test]
    fn test_community_properties() {
        // Test basic community properties on τ = {∅, {1}, {1,2}}
        let mut family = BTreeSet::new();
        family.insert(0b00); // {}
        family.insert(0b01); // {1}
        family.insert(0b11); // {1,2}
        
        let checker = ModelChecker::new(2, family.clone());
        let anti = checker.build_antipodes();
        
        let k1 = checker.community_with_cache(1, &anti);
        let k2 = checker.community_with_cache(2, &anti);
        
        // K_1 should contain point 1
        assert_ne!(k1 & 0b01, 0); // 1 ∈ K_1
        
        // Every community should be an open set (member of family)
        // This relies on the union-closed hypothesis of semitopologies
        assert!(family.contains(&k1) || k1 == 0);
        assert!(family.contains(&k2) || k2 == 0);
        
        // In this topology τ = {∅, {1}, {1,2}}:
        // K_1: class = {1,2}, interior = {1,2} 
        // K_2: class = {1,2}, interior = {1,2}
        assert_eq!(k1, 0b11);
        assert_eq!(k2, 0b11);
    }

    #[test]
    fn test_community_without_empty_set() {
        // Test a family that does not contain the empty set
        let mut family = BTreeSet::new();
        family.insert(0b01); // {1}
        family.insert(0b11); // {1,2}
        
        let checker = ModelChecker::new(2, family.clone());
        let anti = checker.build_antipodes();
        
        // Verify antipodes for family without empty set
        assert_eq!(anti[&0b01], 0b00); // anti[{1}] = {} (no other disjoint sets)
        assert_eq!(anti[&0b11], 0b00); // anti[{1,2}] = {} (no disjoint sets)
        
        // Both points should have community {1,2} since that's the whole universe
        // that can be reached without going through separating sets
        let k1 = checker.community_with_cache(1, &anti);
        let k2 = checker.community_with_cache(2, &anti);
        
        assert_eq!(k1, 0b11); // K_1 = {1,2}
        assert_eq!(k2, 0b11); // K_2 = {1,2}
    }

    #[test]
    fn test_community_max_size() {
        // Test with n = 32 (highest encodable in u32) to exercise universe = u32::MAX path
        let mut family = BTreeSet::new();
        family.insert(0); // {}
        family.insert(1); // {1}
        family.insert(u32::MAX); // {1,2,...,32}
        
        let checker = ModelChecker::new(32, family.clone());
        let anti = checker.build_antipodes();
        
        // Verify the universe = u32::MAX path is exercised
        let k1 = checker.community_with_cache(1, &anti);
        
        // In this case, point 1 is in {1} and {1,2,...,32}
        // anti[{1}] includes everything disjoint from {1} = {}
        // anti[{1,2,...,32}] = {} (nothing is disjoint from the full set)
        // So separable = {}, class = all points, community = entire family union
        assert_eq!(k1, u32::MAX); // Should be the full set
    }

    #[test] 
    fn test_community_reference_comparison() {
        // Compare against a reference implementation for a small random case
        fn reference_community(p: usize, n: usize, family: &Family) -> u32 {
            if p == 0 || p > n || family.is_empty() {
                return 0;
            }
            
            let universe = if n == 32 { u32::MAX } else { (1u32 << n) - 1 };
            let p_bit = 1u32 << (p - 1);
            
            // Find all sets separable from p (slow O(|τ|²) method)
            let mut separable = 0u32;
            for &o in family {
                if o & p_bit != 0 { // o contains p
                    for &q in family {
                        if o & q == 0 { // q is disjoint from o
                            separable |= q;
                        }
                    }
                }
            }
            
            let class = universe & !separable;
            
            // Find interior of class
            let mut community = 0u32;
            for &o in family {
                if o & !class == 0 { // o ⊆ class
                    community |= o;
                }
            }
            community
        }
        
        // Test case: τ = {∅, {1}, {2}, {1,2}, {3}, {1,3}}
        let mut family = BTreeSet::new();
        family.insert(0b000); // {}
        family.insert(0b001); // {1}
        family.insert(0b010); // {2}
        family.insert(0b011); // {1,2}
        family.insert(0b100); // {3}
        family.insert(0b101); // {1,3}
        
        let checker = ModelChecker::new(3, family.clone());
        let anti = checker.build_antipodes();
        
        // Compare fast and reference implementations
        for p in 1..=3 {
            let fast_result = checker.community_with_cache(p, &anti);
            let ref_result = reference_community(p, 3, &family);
            assert_eq!(fast_result, ref_result, "Mismatch for point {}", p);
        }
    }
}