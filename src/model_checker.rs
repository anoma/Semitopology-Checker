//! Model checker for semitopology propositions.

use crate::canon::Family;
use std::collections::HashMap;

/// Variable identifiers for points and opens
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Variable {
    Point(String),
    Open(String),
}

/// Terms in the proposition language
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    PointVar(String),
    OpenVar(String),
}

/// Atomic propositions
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    /// Point x is in open X
    PointInOpen(String, String),
    /// Open X intersects open Y
    OpenIntersection(String, String),
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
}

impl ModelChecker {
    pub fn new(n: usize, family: Family) -> Self {
        Self { n, family }
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
    
    /// Evaluate an atomic proposition under an assignment
    fn eval_atom(&self, atom: &Atom, assignment: &Assignment) -> bool {
        match atom {
            Atom::PointInOpen(point_var, open_var) => {
                if let (Some(&point), Some(&open)) = (
                    assignment.points.get(point_var),
                    assignment.opens.get(open_var)
                ) {
                    self.point_in_open(point, open)
                } else {
                    false // Undefined variables are false
                }
            }
            Atom::OpenIntersection(open_var1, open_var2) => {
                if let (Some(&open1), Some(&open2)) = (
                    assignment.opens.get(open_var1),
                    assignment.opens.get(open_var2)
                ) {
                    self.opens_intersect(open1, open2)
                } else {
                    false // Undefined variables are false
                }
            }
        }
    }
    
    /// Evaluate a formula under an assignment, returning witnesses for existential quantifiers
    pub fn eval_formula(&self, formula: &Formula, assignment: &Assignment) -> ModelCheckResult {
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
                for &open in &self.family {
                    let new_assignment = assignment.clone_with_open(var.clone(), open);
                    let result = self.eval_formula(f, &new_assignment);
                    if !result.satisfied {
                        return ModelCheckResult::false_result();
                    }
                }
                ModelCheckResult::true_result()
            }
            Formula::ExistsOpens(var, f) => {
                for &open in &self.family {
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
    pub fn check(&self, formula: &Formula) -> ModelCheckResult {
        let assignment = Assignment::new();
        self.eval_formula(formula, &assignment)
    }
}