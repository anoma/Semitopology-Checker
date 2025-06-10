//! Abstract Syntax Tree for semitopology formulas
//!
//! This module defines the AST types produced by the parser before macro expansion.
//! The AST separates logical constructs from macro constructs to enable clean
//! expansion while preserving the structure of the original grammar.
//!
//! # Design Principles
//! - **Separation**: LogicProp for core logic, MacroProp for expandable macros
//! - **Composability**: All types can be nested and combined freely
//! - **Expandability**: AST structure supports adding new constructs easily

/// Top-level proposition: either core logic or an expandable macro
#[derive(Debug, Clone, PartialEq)]
pub enum Prop {
    Logic(LogicProp),
    Macro(MacroProp),
}

/// Core logical propositions (quantifiers, operators, atoms)
#[derive(Debug, Clone, PartialEq)]
pub enum LogicProp {
    Quant(QuantProp),
    Binary(BinaryProp),
    Unary(UnaryProp),
    Atomic(AtomicProp),
}

/// Quantified propositions over points and opens
#[derive(Debug, Clone, PartialEq)]
pub enum QuantProp {
    /// Universal quantification over points: ∀p. φ
    AP(String, Box<Prop>),
    /// Existential quantification over points: ∃p. φ  
    EP(String, Box<Prop>),
    /// Universal quantification over opens: ∀X. φ
    AO(String, Box<Prop>),
    /// Existential quantification over opens: ∃X. φ
    EO(String, Box<Prop>),
}

/// Binary logical operators with standard semantics
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryProp {
    /// Logical conjunction: φ ∧ ψ
    And(Box<Prop>, Box<Prop>),
    /// Logical disjunction: φ ∨ ψ
    Or(Box<Prop>, Box<Prop>),
    /// Logical implication: φ → ψ
    Implies(Box<Prop>, Box<Prop>),
    /// Material equivalence: φ ↔ ψ
    Iff(Box<Prop>, Box<Prop>),
}

/// Unary logical operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryProp {
    /// Logical negation: ¬φ
    Not(Box<Prop>),
}

/// Atomic propositions - the basic building blocks
#[derive(Debug, Clone, PartialEq)]
pub enum AtomicProp {
    /// Point membership: p ∈ X
    PointInOpen(PointExpr, OpenExpr),
    /// Open intersection: X ∩ Y ≠ ∅
    OpenInter(OpenExpr, OpenExpr),
    /// Non-emptiness: X ≠ ∅
    Nonempty(OpenExpr),
    /// Point inequality: p ≠ q
    PointNotEqual(PointExpr, PointExpr),
    /// Open inequality: X ≠ Y
    OpenNotEqual(OpenExpr, OpenExpr),
    /// Point equality: p = q
    PointEqual(PointExpr, PointExpr),
    /// Open equality: X = Y
    OpenEqual(OpenExpr, OpenExpr),
}

/// Point expressions - represent individual elements
#[derive(Debug, Clone, PartialEq)]
pub enum PointExpr {
    /// Point variable: p, q, x, etc.
    PointVar(String),
}

/// Open expressions - represent sets in the semitopology  
#[derive(Debug, Clone, PartialEq)]
pub enum OpenExpr {
    /// Open variable: X, Y, T, etc.
    OpenVar(String),
    /// Community of a point: K(p)
    K(PointExpr),
    /// Interior complement: IC(X)
    IC(Box<OpenExpr>),
}

/// Macro propositions - high-level constructs that expand to complex formulas
/// 
/// These represent the 17 built-in definitions from the README that get
/// expanded into core logical formulas with fresh variable generation.
#[derive(Debug, Clone, PartialEq)]
pub enum MacroProp {
    // Intersection macros
    /// Triple open intersection: X ∩ Y ∩ Z
    TripleOpenInter(OpenExpr, OpenExpr, OpenExpr),
    /// Point intersection: p ∩ q  
    PointInter(PointExpr, PointExpr),
    /// Triple point intersection: p ∩ q ∩ r
    TriplePointInter(PointExpr, PointExpr, PointExpr),
    
    // Topological predicates for opens
    /// Transitivity: ∀O,P. (O∩T ∧ T∩P) → O∩P
    Transitive(OpenExpr),
    /// Transitive open: nonempty and transitive
    Topen(OpenExpr),
    
    // Regularity predicates for points
    /// Regular point: K(p) is transitive and nonempty
    Regular(PointExpr),
    /// Irregular point: negation of regular
    Irregular(PointExpr), 
    /// Weakly regular: p ∈ K(p)
    WeaklyRegular(PointExpr),
    /// Quasiregular: K(p) is nonempty
    Quasiregular(PointExpr),
    /// Indirectly regular: ∃q. p∩q ∧ regular(q)
    IndirectlyRegular(PointExpr),
    /// Hypertransitive: complex interaction property
    Hypertransitive(PointExpr),
    /// Unconflicted: ∀x,y. x∩p∩y → x∩y
    Unconflicted(PointExpr),
    /// Conflicted: negation of unconflicted
    Conflicted(PointExpr),
    
    // Space-wide predicates (apply to all points)
    /// Every point is conflicted
    ConflictedSpace,
    /// Every point is unconflicted  
    UnconflictedSpace,
    /// Every point is regular
    RegularSpace,
    /// Every point is irregular
    IrregularSpace,
    /// Every point is weakly regular
    WeaklyRegularSpace,
    /// Every point is quasiregular
    QuasiregularSpace,
    /// Every point is indirectly regular
    IndirectlyRegularSpace,
    /// Every point is hypertransitive
    HypertransitiveSpace,
}