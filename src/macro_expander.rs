//! Macro expansion for semitopology formulas
//!
//! This module converts high-level macro constructs into core logical formulas.
//! All 17 built-in macros are faithfully implemented according to their definitions,
//! with automatic fresh variable generation to prevent variable capture.
//!
//! # Expansion Process
//! 1. **Structural recursion**: Visit every node in the AST
//! 2. **Macro identification**: Replace MacroProp nodes with their definitions
//! 3. **Variable renaming**: Generate fresh variables to avoid capture
//! 4. **Type conversion**: Convert from parser AST to model checker AST

use crate::ast::*;
use crate::model_checker::{Formula, Atom, OpenExpr as ModelOpenExpr};

/// Macro expander with fresh variable generation
///
/// Maintains a counter to ensure all generated variables are unique,
/// preventing accidental variable capture during macro expansion.
pub struct MacroExpander {
    fresh_var_counter: usize,
}

impl MacroExpander {
    /// Create a new macro expander
    pub fn new() -> Self {
        Self {
            fresh_var_counter: 0,
        }
    }

    /// Generate a fresh variable name with the given base
    /// 
    /// Each call produces a unique variable like "O_0", "O_1", "p_0", etc.
    /// This prevents variable capture when expanding nested macros.
    fn fresh_var(&mut self, base: &str) -> String {
        let result = format!("{}_{}", base, self.fresh_var_counter);
        self.fresh_var_counter += 1;
        result
    }

    pub fn expand(&mut self, prop: Prop) -> Result<Formula, String> {
        match prop {
            Prop::Logic(logic_prop) => self.expand_logic_prop(logic_prop),
            Prop::Macro(macro_prop) => self.expand_macro_prop(macro_prop),
        }
    }

    fn expand_logic_prop(&mut self, logic_prop: LogicProp) -> Result<Formula, String> {
        match logic_prop {
            LogicProp::Quant(quant) => self.expand_quant_prop(quant),
            LogicProp::Binary(binary) => self.expand_binary_prop(binary),
            LogicProp::Unary(unary) => self.expand_unary_prop(unary),
            LogicProp::Atomic(atomic) => self.expand_atomic_prop(atomic),
        }
    }

    fn expand_quant_prop(&mut self, quant: QuantProp) -> Result<Formula, String> {
        match quant {
            QuantProp::AP(var, prop) => {
                let inner = self.expand(*prop)?;
                Ok(Formula::ForAllPoints(var, Box::new(inner)))
            }
            QuantProp::EP(var, prop) => {
                let inner = self.expand(*prop)?;
                Ok(Formula::ExistsPoints(var, Box::new(inner)))
            }
            QuantProp::AO(var, prop) => {
                let inner = self.expand(*prop)?;
                Ok(Formula::ForAllOpens(var, Box::new(inner)))
            }
            QuantProp::EO(var, prop) => {
                let inner = self.expand(*prop)?;
                Ok(Formula::ExistsOpens(var, Box::new(inner)))
            }
        }
    }

    fn expand_binary_prop(&mut self, binary: BinaryProp) -> Result<Formula, String> {
        match binary {
            BinaryProp::And(left, right) => {
                let left_expanded = self.expand(*left)?;
                let right_expanded = self.expand(*right)?;
                Ok(Formula::And(Box::new(left_expanded), Box::new(right_expanded)))
            }
            BinaryProp::Or(left, right) => {
                let left_expanded = self.expand(*left)?;
                let right_expanded = self.expand(*right)?;
                Ok(Formula::Or(Box::new(left_expanded), Box::new(right_expanded)))
            }
            BinaryProp::Implies(left, right) => {
                let left_expanded = self.expand(*left)?;
                let right_expanded = self.expand(*right)?;
                Ok(Formula::Implies(Box::new(left_expanded), Box::new(right_expanded)))
            }
            BinaryProp::Iff(left, right) => {
                let left_expanded = self.expand(*left)?;
                let right_expanded = self.expand(*right)?;
                Ok(Formula::Iff(Box::new(left_expanded), Box::new(right_expanded)))
            }
        }
    }

    fn expand_unary_prop(&mut self, unary: UnaryProp) -> Result<Formula, String> {
        match unary {
            UnaryProp::Not(prop) => {
                let inner = self.expand(*prop)?;
                Ok(Formula::Not(Box::new(inner)))
            }
        }
    }

    fn expand_atomic_prop(&mut self, atomic: AtomicProp) -> Result<Formula, String> {
        match atomic {
            AtomicProp::PointInOpen(point_expr, open_expr) => {
                let point_id = self.extract_point_id(point_expr)?;
                let model_open = self.convert_open_expr_to_model(open_expr)?;
                Ok(Formula::Atom(Atom::PointInOpen(point_id, model_open)))
            }
            AtomicProp::OpenInter(open1, open2) => {
                let model_open1 = self.convert_open_expr_to_model(open1)?;
                let model_open2 = self.convert_open_expr_to_model(open2)?;
                Ok(Formula::Atom(Atom::OpenIntersection(model_open1, model_open2)))
            }
            AtomicProp::Nonempty(open_expr) => {
                let model_open = self.convert_open_expr_to_model(open_expr)?;
                Ok(Formula::Atom(Atom::OpenNonempty(model_open)))
            }
            AtomicProp::PointNotEqual(point_expr1, point_expr2) => {
                let point_id1 = self.extract_point_id(point_expr1)?;
                let point_id2 = self.extract_point_id(point_expr2)?;
                Ok(Formula::Atom(Atom::PointNotEqual(point_id1, point_id2)))
            }
            AtomicProp::OpenNotEqual(open_expr1, open_expr2) => {
                let model_open1 = self.convert_open_expr_to_model(open_expr1)?;
                let model_open2 = self.convert_open_expr_to_model(open_expr2)?;
                Ok(Formula::Atom(Atom::OpenNotEqual(model_open1, model_open2)))
            }
            AtomicProp::PointEqual(point_expr1, point_expr2) => {
                let point_id1 = self.extract_point_id(point_expr1)?;
                let point_id2 = self.extract_point_id(point_expr2)?;
                Ok(Formula::Atom(Atom::PointEqual(point_id1, point_id2)))
            }
            AtomicProp::OpenEqual(open_expr1, open_expr2) => {
                let model_open1 = self.convert_open_expr_to_model(open_expr1)?;
                let model_open2 = self.convert_open_expr_to_model(open_expr2)?;
                Ok(Formula::Atom(Atom::OpenEqual(model_open1, model_open2)))
            }
        }
    }

    fn extract_point_id(&self, point_expr: PointExpr) -> Result<String, String> {
        match point_expr {
            PointExpr::PointVar(var) => Ok(var),
        }
    }

    fn convert_open_expr_to_model(&self, open_expr: OpenExpr) -> Result<ModelOpenExpr, String> {
        match open_expr {
            OpenExpr::OpenVar(var) => {
                Ok(ModelOpenExpr::Var(var))
            }
            OpenExpr::K(point_expr) => {
                let point_var = self.extract_point_id(point_expr)?;
                Ok(ModelOpenExpr::Community(point_var))
            }
            OpenExpr::IC(inner_expr) => {
                let inner_model = self.convert_open_expr_to_model(*inner_expr)?;
                Ok(ModelOpenExpr::InteriorComplement(Box::new(inner_model)))
            }
        }
    }

    fn var_to_model_open(&self, var: String) -> ModelOpenExpr {
        ModelOpenExpr::Var(var)
    }

    fn expand_macro_prop(&mut self, macro_prop: MacroProp) -> Result<Formula, String> {
        match macro_prop {
            MacroProp::TripleOpenInter(o, p, q) => {
                // O inter P inter Q = (O inter P) && (P inter Q)
                let o_model = self.convert_open_expr_to_model(o)?;
                let p_model = self.convert_open_expr_to_model(p)?;
                let q_model = self.convert_open_expr_to_model(q)?;
                
                let inter1 = Formula::Atom(Atom::OpenIntersection(o_model, p_model.clone()));
                let inter2 = Formula::Atom(Atom::OpenIntersection(p_model, q_model));
                Ok(Formula::And(Box::new(inter1), Box::new(inter2)))
            }
            
            MacroProp::PointInter(p, q) => {
                // p inter q = AO O. AO P. (p in O && q in P) => (O inter P)
                let p_var = self.extract_point_id(p)?;
                let q_var = self.extract_point_id(q)?;
                let o_var = self.fresh_var("O");
                let big_p_var = self.fresh_var("P");
                
                let p_in_o = Formula::Atom(Atom::PointInOpen(p_var, self.var_to_model_open(o_var.clone())));
                let q_in_p = Formula::Atom(Atom::PointInOpen(q_var, self.var_to_model_open(big_p_var.clone())));
                let o_inter_p = Formula::Atom(Atom::OpenIntersection(
                    self.var_to_model_open(o_var.clone()), 
                    self.var_to_model_open(big_p_var.clone())
                ));
                
                let premise = Formula::And(Box::new(p_in_o), Box::new(q_in_p));
                let implication = Formula::Implies(Box::new(premise), Box::new(o_inter_p));
                let inner_forall = Formula::ForAllOpens(big_p_var, Box::new(implication));
                Ok(Formula::ForAllOpens(o_var, Box::new(inner_forall)))
            }
            
            MacroProp::TriplePointInter(p, q, r) => {
                // p inter q inter r = (p inter q) && (q inter r)
                let p_inter_q = self.expand_macro_prop(MacroProp::PointInter(p, q.clone()))?;
                let q_inter_r = self.expand_macro_prop(MacroProp::PointInter(q, r))?;
                Ok(Formula::And(Box::new(p_inter_q), Box::new(q_inter_r)))
            }
            
            MacroProp::Transitive(t_expr) => {
                // transitive T = AO O. AO P. (O inter T && T inter P) => (O inter P)
                let t_model = self.convert_open_expr_to_model(t_expr)?;
                let o_var = self.fresh_var("O");
                let p_var = self.fresh_var("P");
                
                let o_inter_t = Formula::Atom(Atom::OpenIntersection(self.var_to_model_open(o_var.clone()), t_model.clone()));
                let t_inter_p = Formula::Atom(Atom::OpenIntersection(t_model, self.var_to_model_open(p_var.clone())));
                let o_inter_p = Formula::Atom(Atom::OpenIntersection(
                    self.var_to_model_open(o_var.clone()), 
                    self.var_to_model_open(p_var.clone())
                ));
                
                let premise = Formula::And(Box::new(o_inter_t), Box::new(t_inter_p));
                let implication = Formula::Implies(Box::new(premise), Box::new(o_inter_p));
                let inner_forall = Formula::ForAllOpens(p_var, Box::new(implication));
                Ok(Formula::ForAllOpens(o_var, Box::new(inner_forall)))
            }
            
            MacroProp::Topen(t_expr) => {
                // topen T = nonempty T && transitive T
                let t_model = self.convert_open_expr_to_model(t_expr.clone())?;
                let nonempty_t = Formula::Atom(Atom::OpenNonempty(t_model));
                let transitive_t = self.expand_macro_prop(MacroProp::Transitive(t_expr))?;
                Ok(Formula::And(Box::new(nonempty_t), Box::new(transitive_t)))
            }
            
            MacroProp::Regular(p_expr) => {
                // regular p = topen (K p)
                let p_var = self.extract_point_id(p_expr)?;
                let k_p_expr = OpenExpr::K(PointExpr::PointVar(p_var));
                self.expand_macro_prop(MacroProp::Topen(k_p_expr))
            }
            
            MacroProp::Irregular(p_expr) => {
                // irregular p = !(regular p)
                let regular_p = self.expand_macro_prop(MacroProp::Regular(p_expr))?;
                Ok(Formula::Not(Box::new(regular_p)))
            }
            
            MacroProp::WeaklyRegular(p_expr) => {
                // weakly_regular p = p in (K p)
                let p_var = self.extract_point_id(p_expr)?;
                let k_p = ModelOpenExpr::Community(p_var.clone());
                Ok(Formula::Atom(Atom::PointInOpen(p_var, k_p)))
            }
            
            MacroProp::Quasiregular(p_expr) => {
                // quasiregular p = nonempty (K p)
                let k_p_expr = OpenExpr::K(p_expr);
                self.expand_atomic_prop(AtomicProp::Nonempty(k_p_expr))
            }
            
            MacroProp::IndirectlyRegular(p_expr) => {
                // indirectly_regular p = EP q. (p inter q) && regular q
                let p_var = self.extract_point_id(p_expr)?;
                let q_var = self.fresh_var("q");
                
                let p_inter_q = self.expand_macro_prop(MacroProp::PointInter(
                    PointExpr::PointVar(p_var),
                    PointExpr::PointVar(q_var.clone())
                ))?;
                let regular_q = self.expand_macro_prop(MacroProp::Regular(PointExpr::PointVar(q_var.clone())))?;
                
                let premise = Formula::And(Box::new(p_inter_q), Box::new(regular_q));
                Ok(Formula::ExistsPoints(q_var, Box::new(premise)))
            }
            
            MacroProp::Hypertransitive(p_expr) => {
                // hypertransitive p = AO O. AO Q. (AO P. p in P => (O inter P inter Q)) => (O inter Q)
                let p_var = self.extract_point_id(p_expr)?;
                let o_var = self.fresh_var("O");
                let q_var = self.fresh_var("Q");
                let big_p_var = self.fresh_var("P");
                
                let p_in_p = Formula::Atom(Atom::PointInOpen(p_var, self.var_to_model_open(big_p_var.clone())));
                let o_inter_p = Formula::Atom(Atom::OpenIntersection(
                    self.var_to_model_open(o_var.clone()), 
                    self.var_to_model_open(big_p_var.clone())
                ));
                let p_inter_q = Formula::Atom(Atom::OpenIntersection(
                    self.var_to_model_open(big_p_var.clone()), 
                    self.var_to_model_open(q_var.clone())
                ));
                let o_inter_p_inter_q = Formula::And(Box::new(o_inter_p), Box::new(p_inter_q));
                
                let inner_impl = Formula::Implies(Box::new(p_in_p), Box::new(o_inter_p_inter_q));
                let forall_p = Formula::ForAllOpens(big_p_var, Box::new(inner_impl));
                
                let o_inter_q = Formula::Atom(Atom::OpenIntersection(
                    self.var_to_model_open(o_var.clone()), 
                    self.var_to_model_open(q_var.clone())
                ));
                let outer_impl = Formula::Implies(Box::new(forall_p), Box::new(o_inter_q));
                
                let forall_q = Formula::ForAllOpens(q_var, Box::new(outer_impl));
                Ok(Formula::ForAllOpens(o_var, Box::new(forall_q)))
            }
            
            MacroProp::Unconflicted(p_expr) => {
                // unconflicted p = AP x. AP y. (x inter p inter y) => (x inter y)
                let p_var = self.extract_point_id(p_expr)?;
                let x_var = self.fresh_var("x");
                let y_var = self.fresh_var("y");
                
                let x_inter_p = self.expand_macro_prop(MacroProp::PointInter(
                    PointExpr::PointVar(x_var.clone()),
                    PointExpr::PointVar(p_var.clone())
                ))?;
                let p_inter_y = self.expand_macro_prop(MacroProp::PointInter(
                    PointExpr::PointVar(p_var.clone()),
                    PointExpr::PointVar(y_var.clone())
                ))?;
                let x_inter_y = self.expand_macro_prop(MacroProp::PointInter(
                    PointExpr::PointVar(x_var.clone()),
                    PointExpr::PointVar(y_var.clone())
                ))?;
                
                let premise = Formula::And(Box::new(x_inter_p), Box::new(p_inter_y));
                let implication = Formula::Implies(Box::new(premise), Box::new(x_inter_y));
                let forall_y = Formula::ForAllPoints(y_var, Box::new(implication));
                Ok(Formula::ForAllPoints(x_var, Box::new(forall_y)))
            }
            
            MacroProp::Conflicted(p_expr) => {
                // conflicted p = !(unconflicted p)
                let unconflicted_p = self.expand_macro_prop(MacroProp::Unconflicted(p_expr))?;
                Ok(Formula::Not(Box::new(unconflicted_p)))
            }
            
            // Space predicates
            MacroProp::ConflictedSpace => {
                // conflicted_space = AP p. conflicted p
                let p_var = self.fresh_var("p");
                let conflicted_p = self.expand_macro_prop(MacroProp::Conflicted(PointExpr::PointVar(p_var.clone())))?;
                Ok(Formula::ForAllPoints(p_var, Box::new(conflicted_p)))
            }
            
            MacroProp::UnconflictedSpace => {
                // unconflicted_space = AP p. unconflicted p
                let p_var = self.fresh_var("p");
                let unconflicted_p = self.expand_macro_prop(MacroProp::Unconflicted(PointExpr::PointVar(p_var.clone())))?;
                Ok(Formula::ForAllPoints(p_var, Box::new(unconflicted_p)))
            }
            
            MacroProp::RegularSpace => {
                // regular_space = AP p. regular p
                let p_var = self.fresh_var("p");
                let regular_p = self.expand_macro_prop(MacroProp::Regular(PointExpr::PointVar(p_var.clone())))?;
                Ok(Formula::ForAllPoints(p_var, Box::new(regular_p)))
            }
            
            MacroProp::IrregularSpace => {
                // irregular_space = AP p. irregular p
                let p_var = self.fresh_var("p");
                let irregular_p = self.expand_macro_prop(MacroProp::Irregular(PointExpr::PointVar(p_var.clone())))?;
                Ok(Formula::ForAllPoints(p_var, Box::new(irregular_p)))
            }
            
            MacroProp::WeaklyRegularSpace => {
                // weakly_regular_space = AP p. weakly_regular p
                let p_var = self.fresh_var("p");
                let weakly_regular_p = self.expand_macro_prop(MacroProp::WeaklyRegular(PointExpr::PointVar(p_var.clone())))?;
                Ok(Formula::ForAllPoints(p_var, Box::new(weakly_regular_p)))
            }
            
            MacroProp::QuasiregularSpace => {
                // quasiregular_space = AP p. quasiregular p
                let p_var = self.fresh_var("p");
                let quasiregular_p = self.expand_macro_prop(MacroProp::Quasiregular(PointExpr::PointVar(p_var.clone())))?;
                Ok(Formula::ForAllPoints(p_var, Box::new(quasiregular_p)))
            }
            
            MacroProp::IndirectlyRegularSpace => {
                // indirectly_regular_space = AP p. indirectly_regular p
                let p_var = self.fresh_var("p");
                let indirectly_regular_p = self.expand_macro_prop(MacroProp::IndirectlyRegular(PointExpr::PointVar(p_var.clone())))?;
                Ok(Formula::ForAllPoints(p_var, Box::new(indirectly_regular_p)))
            }
            
            MacroProp::HypertransitiveSpace => {
                // hypertransitive_space = AP p. hypertransitive p
                let p_var = self.fresh_var("p");
                let hypertransitive_p = self.expand_macro_prop(MacroProp::Hypertransitive(PointExpr::PointVar(p_var.clone())))?;
                Ok(Formula::ForAllPoints(p_var, Box::new(hypertransitive_p)))
            }
        }
    }
}