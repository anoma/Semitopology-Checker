//! Parser for proposition formulas.

use crate::model_checker::{Formula, Atom};
use std::str::Chars;
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum Token {
    // Logical operators
    And,        // &&
    Or,         // ||
    Implies,    // =>
    Not,        // !
    
    // Quantifiers
    ForAll,     // AO, AP
    Exists,     // EO, EP
    
    // Primitives
    In,         // in
    Inter,      // inter
    Nonempty,   // nonempty
    Community,  // K
    
    // Built-in notations
    Transitive, // transitive
    Topen,      // topen
    Regular,    // regular
    Irregular,  // irregular
    WeaklyRegular, // weakly_regular
    Quasiregular, // quasiregular
    IndirectlyRegular, // indirectly_regular
    Hypertransitive, // hypertransitive
    Unconflicted, // unconflicted
    Conflicted, // conflicted
    ConflictedSpace, // conflicted_space
    UnconflictedSpace, // unconflicted_space
    RegularSpace, // regular_space
    IrregularSpace, // irregular_space
    WeaklyRegularSpace, // weakly_regular_space
    QuasiregularSpace, // quasiregular_space
    IndirectlyRegularSpace, // indirectly_regular_space
    HypertransitiveSpace, // hypertransitive_space
    
    // Identifiers and punctuation
    Identifier(String),
    Dot,        // .
    LeftParen,  // (
    RightParen, // )
    
    EOF,
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    current_char: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars().peekable();
        let current_char = chars.next();
        Self {
            input: chars,
            current_char,
        }
    }
    
    fn advance(&mut self) {
        self.current_char = self.input.next();
    }
    
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn read_identifier(&mut self) -> String {
        let mut identifier = String::new();
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        identifier
    }
    
    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();
        
        match self.current_char {
            None => Ok(Token::EOF),
            Some('(') => {
                self.advance();
                Ok(Token::LeftParen)
            }
            Some(')') => {
                self.advance();
                Ok(Token::RightParen)
            }
            Some('.') => {
                self.advance();
                Ok(Token::Dot)
            }
            Some('!') => {
                self.advance();
                Ok(Token::Not)
            }
            Some('&') => {
                self.advance();
                if self.current_char == Some('&') {
                    self.advance();
                    Ok(Token::And)
                } else {
                    Err("Expected '&&' for logical AND".to_string())
                }
            }
            Some('|') => {
                self.advance();
                if self.current_char == Some('|') {
                    self.advance();
                    Ok(Token::Or)
                } else {
                    Err("Expected '||' for logical OR".to_string())
                }
            }
            Some('=') => {
                self.advance();
                if self.current_char == Some('>') {
                    self.advance();
                    Ok(Token::Implies)
                } else {
                    Err("Expected '=>' for implication".to_string())
                }
            }
            Some(ch) if ch.is_alphabetic() => {
                let identifier = self.read_identifier();
                match identifier.as_str() {
                    "AO" | "AP" => Ok(Token::ForAll),
                    "EO" | "EP" => Ok(Token::Exists),
                    "in" => Ok(Token::In),
                    "inter" => Ok(Token::Inter),
                    "nonempty" => Ok(Token::Nonempty),
                    "K" => Ok(Token::Community),
                    "transitive" => Ok(Token::Transitive),
                    "topen" => Ok(Token::Topen),
                    "regular" => Ok(Token::Regular),
                    "irregular" => Ok(Token::Irregular),
                    "weakly_regular" => Ok(Token::WeaklyRegular),
                    "quasiregular" => Ok(Token::Quasiregular),
                    "indirectly_regular" => Ok(Token::IndirectlyRegular),
                    "hypertransitive" => Ok(Token::Hypertransitive),
                    "unconflicted" => Ok(Token::Unconflicted),
                    "conflicted" => Ok(Token::Conflicted),
                    "conflicted_space" => Ok(Token::ConflictedSpace),
                    "unconflicted_space" => Ok(Token::UnconflictedSpace),
                    "regular_space" => Ok(Token::RegularSpace),
                    "irregular_space" => Ok(Token::IrregularSpace),
                    "weakly_regular_space" => Ok(Token::WeaklyRegularSpace),
                    "quasiregular_space" => Ok(Token::QuasiregularSpace),
                    "indirectly_regular_space" => Ok(Token::IndirectlyRegularSpace),
                    "hypertransitive_space" => Ok(Token::HypertransitiveSpace),
                    _ => Ok(Token::Identifier(identifier)),
                }
            }
            Some(ch) => Err(format!("Unexpected character: {}", ch)),
        }
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token()?;
        Ok(Self {
            lexer,
            current_token,
        })
    }
    
    fn advance(&mut self) -> Result<(), String> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }
    
    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.advance()
        } else {
            Err(format!("Expected {:?}, found {:?}", expected, self.current_token))
        }
    }
    
    pub fn parse(&mut self) -> Result<Formula, String> {
        self.parse_formula()
    }
    
    fn parse_formula(&mut self) -> Result<Formula, String> {
        self.parse_implication()
    }
    
    fn parse_implication(&mut self) -> Result<Formula, String> {
        let mut left = self.parse_or()?;
        
        while matches!(self.current_token, Token::Implies) {
            self.advance()?;
            let right = self.parse_or()?;
            left = Formula::Implies(Box::new(left), Box::new(right));
        }
        
        Ok(left)
    }
    
    fn parse_or(&mut self) -> Result<Formula, String> {
        let mut left = self.parse_and()?;
        
        while matches!(self.current_token, Token::Or) {
            self.advance()?;
            let right = self.parse_and()?;
            left = Formula::Or(Box::new(left), Box::new(right));
        }
        
        Ok(left)
    }
    
    fn parse_and(&mut self) -> Result<Formula, String> {
        let mut left = self.parse_quantifier()?;
        
        while matches!(self.current_token, Token::And) {
            self.advance()?;
            let right = self.parse_quantifier()?;
            left = Formula::And(Box::new(left), Box::new(right));
        }
        
        Ok(left)
    }
    
    fn parse_quantifier(&mut self) -> Result<Formula, String> {
        match &self.current_token {
            Token::ForAll => {
                self.advance()?;
                let var = self.parse_identifier()?;
                self.expect(Token::Dot)?;
                let formula = self.parse_formula()?;
                
                // Determine if it's a point or open quantifier based on the variable name
                // Convention: point variables are lowercase, open variables are uppercase
                if var.chars().next().unwrap().is_lowercase() {
                    Ok(Formula::ForAllPoints(var, Box::new(formula)))
                } else {
                    Ok(Formula::ForAllOpens(var, Box::new(formula)))
                }
            }
            Token::Exists => {
                self.advance()?;
                let var = self.parse_identifier()?;
                self.expect(Token::Dot)?;
                let formula = self.parse_formula()?;
                
                // Determine if it's a point or open quantifier based on the variable name
                if var.chars().next().unwrap().is_lowercase() {
                    Ok(Formula::ExistsPoints(var, Box::new(formula)))
                } else {
                    Ok(Formula::ExistsOpens(var, Box::new(formula)))
                }
            }
            _ => self.parse_unary(),
        }
    }
    
    fn parse_unary(&mut self) -> Result<Formula, String> {
        match &self.current_token {
            Token::Not => {
                self.advance()?;
                let formula = self.parse_unary()?;
                Ok(Formula::Not(Box::new(formula)))
            }
            _ => self.parse_primary(),
        }
    }
    
    fn parse_primary(&mut self) -> Result<Formula, String> {
        match &self.current_token {
            Token::LeftParen => {
                self.advance()?;
                let formula = self.parse_formula()?;
                self.expect(Token::RightParen)?;
                Ok(formula)
            }
            Token::Identifier(_) | Token::Nonempty | Token::Community | 
            Token::Transitive | Token::Topen | Token::Regular | Token::Irregular |
            Token::WeaklyRegular | Token::Quasiregular | Token::IndirectlyRegular |
            Token::Hypertransitive | Token::Unconflicted | Token::Conflicted |
            Token::ConflictedSpace | Token::UnconflictedSpace | Token::RegularSpace |
            Token::IrregularSpace | Token::WeaklyRegularSpace | Token::QuasiregularSpace |
            Token::IndirectlyRegularSpace | Token::HypertransitiveSpace => {
                self.parse_atomic()
            }
            _ => Err(format!("Unexpected token: {:?}", self.current_token)),
        }
    }
    
    fn parse_atomic(&mut self) -> Result<Formula, String> {
        match &self.current_token {
            Token::Nonempty => {
                self.advance()?;
                let open_id = self.parse_identifier()?;
                Ok(Formula::Atom(Atom::OpenNonempty(open_id)))
            }
            Token::Community => {
                self.advance()?;
                let point_id = self.parse_identifier()?;
                Ok(Formula::Atom(Atom::Community(point_id)))
            }
            Token::Identifier(_) => {
                let first_id = self.parse_identifier()?;
                
                match &self.current_token {
                    Token::In => {
                        self.advance()?;
                        if matches!(self.current_token, Token::Community) {
                            self.advance()?; // consume K
                            let community_point = self.parse_identifier()?;
                            Ok(Formula::Atom(Atom::PointInCommunity(first_id, community_point)))
                        } else {
                            let second_id = self.parse_identifier()?;
                            Ok(Formula::Atom(Atom::PointInOpen(first_id, second_id)))
                        }
                    }
                    Token::Inter => {
                        self.advance()?;
                        let second_id = self.parse_identifier()?;
                        
                        // Check for triple inter (O inter P inter Q)
                        if matches!(self.current_token, Token::Inter) {
                            self.advance()?;
                            let third_id = self.parse_identifier()?;
                            
                            // Check if all are points (p inter q inter r = p inter q && q inter r)
                            if first_id.chars().next().unwrap().is_lowercase() && 
                               second_id.chars().next().unwrap().is_lowercase() &&
                               third_id.chars().next().unwrap().is_lowercase() {
                                let inter1 = self.expand_point_inter(first_id, second_id.clone());
                                let inter2 = self.expand_point_inter(second_id, third_id);
                                Ok(Formula::And(Box::new(inter1), Box::new(inter2)))
                            } else {
                                // Transform O inter P inter Q = O inter P && P inter Q
                                let inter1 = Formula::Atom(Atom::OpenIntersection(first_id.clone(), second_id.clone()));
                                let inter2 = Formula::Atom(Atom::OpenIntersection(second_id, third_id));
                                Ok(Formula::And(Box::new(inter1), Box::new(inter2)))
                            }
                        } else {
                            // Check if this is point inter point
                            if first_id.chars().next().unwrap().is_lowercase() && 
                               second_id.chars().next().unwrap().is_lowercase() {
                                // Transform p inter q = AO O. AO P. (p in O && q in P) => O inter P
                                Ok(self.expand_point_inter(first_id, second_id))
                            } else {
                                Ok(Formula::Atom(Atom::OpenIntersection(first_id, second_id)))
                            }
                        }
                    }
                    _ => Err(format!("Expected 'in' or 'inter' after identifier, found {:?}", self.current_token)),
                }
            }
            Token::Transitive => {
                self.advance()?;
                let var = self.parse_identifier()?;
                Ok(self.expand_transitive(var))
            }
            Token::Topen => {
                self.advance()?;
                let var = self.parse_identifier()?;
                Ok(self.expand_topen(var))
            }
            Token::Regular => {
                self.advance()?;
                let var = self.parse_identifier()?;
                Ok(self.expand_regular(var))
            }
            Token::Irregular => {
                self.advance()?;
                let var = self.parse_identifier()?;
                Ok(self.expand_irregular(var))
            }
            Token::WeaklyRegular => {
                self.advance()?;
                let var = self.parse_identifier()?;
                Ok(self.expand_weakly_regular(var))
            }
            Token::Quasiregular => {
                self.advance()?;
                let var = self.parse_identifier()?;
                Ok(self.expand_quasiregular(var))
            }
            Token::IndirectlyRegular => {
                self.advance()?;
                let var = self.parse_identifier()?;
                Ok(self.expand_indirectly_regular(var))
            }
            Token::Hypertransitive => {
                self.advance()?;
                let var = self.parse_identifier()?;
                Ok(self.expand_hypertransitive(var))
            }
            Token::Unconflicted => {
                self.advance()?;
                let var = self.parse_identifier()?;
                Ok(self.expand_unconflicted(var))
            }
            Token::Conflicted => {
                self.advance()?;
                let var = self.parse_identifier()?;
                Ok(self.expand_conflicted(var))
            }
            Token::ConflictedSpace => {
                self.advance()?;
                Ok(self.expand_conflicted_space())
            }
            Token::UnconflictedSpace => {
                self.advance()?;
                Ok(self.expand_unconflicted_space())
            }
            Token::RegularSpace => {
                self.advance()?;
                Ok(self.expand_regular_space())
            }
            Token::IrregularSpace => {
                self.advance()?;
                Ok(self.expand_irregular_space())
            }
            Token::WeaklyRegularSpace => {
                self.advance()?;
                Ok(self.expand_weakly_regular_space())
            }
            Token::QuasiregularSpace => {
                self.advance()?;
                Ok(self.expand_quasiregular_space())
            }
            Token::IndirectlyRegularSpace => {
                self.advance()?;
                Ok(self.expand_indirectly_regular_space())
            }
            Token::HypertransitiveSpace => {
                self.advance()?;
                Ok(self.expand_hypertransitive_space())
            }
            _ => Err(format!("Expected identifier, 'nonempty', or 'K', found {:?}", self.current_token)),
        }
    }
    
    fn parse_identifier(&mut self) -> Result<String, String> {
        match &self.current_token {
            Token::Identifier(id) => {
                let result = id.clone();
                self.advance()?;
                Ok(result)
            }
            _ => Err(format!("Expected identifier, found {:?}", self.current_token)),
        }
    }

    // Generate fresh variable names
    fn fresh_var(&self, base: &str, counter: usize) -> String {
        format!("{}_{}", base, counter)
    }

    // Built-in transformation functions
    fn expand_point_inter(&self, p: String, q: String) -> Formula {
        // p inter q = AO O. AO P. (p in O && q in P) => O inter P
        let o_var = self.fresh_var("O", 0);
        let p_var = self.fresh_var("P", 0);
        
        let p_in_o = Formula::Atom(Atom::PointInOpen(p, o_var.clone()));
        let q_in_p = Formula::Atom(Atom::PointInOpen(q, p_var.clone()));
        let o_inter_p = Formula::Atom(Atom::OpenIntersection(o_var.clone(), p_var.clone()));
        
        let premise = Formula::And(Box::new(p_in_o), Box::new(q_in_p));
        let implication = Formula::Implies(Box::new(premise), Box::new(o_inter_p));
        let inner_forall = Formula::ForAllOpens(p_var, Box::new(implication));
        Formula::ForAllOpens(o_var, Box::new(inner_forall))
    }

    fn expand_transitive(&self, t: String) -> Formula {
        // transitive T = AO O. AO P. (O inter T && T inter P) => O inter P
        let o_var = self.fresh_var("O", 0);
        let p_var = self.fresh_var("P", 0);
        
        let o_inter_t = Formula::Atom(Atom::OpenIntersection(o_var.clone(), t.clone()));
        let t_inter_p = Formula::Atom(Atom::OpenIntersection(t, p_var.clone()));
        let o_inter_p = Formula::Atom(Atom::OpenIntersection(o_var.clone(), p_var.clone()));
        
        let premise = Formula::And(Box::new(o_inter_t), Box::new(t_inter_p));
        let implication = Formula::Implies(Box::new(premise), Box::new(o_inter_p));
        let inner_forall = Formula::ForAllOpens(p_var, Box::new(implication));
        Formula::ForAllOpens(o_var, Box::new(inner_forall))
    }

    fn expand_topen(&self, t: String) -> Formula {
        // topen T = nonempty T && transitive T
        let nonempty_t = Formula::Atom(Atom::OpenNonempty(t.clone()));
        let transitive_t = self.expand_transitive(t);
        Formula::And(Box::new(nonempty_t), Box::new(transitive_t))
    }

    fn expand_regular(&self, p: String) -> Formula {
        // regular p = topen (K p) = nonempty (K p) && transitive (K p)
        // K p is already handled by the Community atom, and transitive can be expanded properly
        let nonempty_k_p = Formula::Atom(Atom::Community(p.clone()));
        
        // transitive (K p) = AO O. AO P. (O inter (K p) && (K p) inter P) => O inter P
        // We need to expand this using the existing point-in-community infrastructure
        let o_var = self.fresh_var("O", 0);
        let p_var = self.fresh_var("P", 0);
        
        // O inter (K p) means: EP x. x in O && x in (K p)
        let x_var1 = self.fresh_var("x", 1);
        let x_in_o = Formula::Atom(Atom::PointInOpen(x_var1.clone(), o_var.clone()));
        let x_in_k_p = Formula::Atom(Atom::PointInCommunity(x_var1.clone(), p.clone()));
        let o_inter_k_p = Formula::ExistsPoints(x_var1, Box::new(Formula::And(Box::new(x_in_o), Box::new(x_in_k_p))));
        
        // (K p) inter P means: EP y. y in (K p) && y in P
        let y_var = self.fresh_var("y", 1);
        let y_in_k_p = Formula::Atom(Atom::PointInCommunity(y_var.clone(), p));
        let y_in_p = Formula::Atom(Atom::PointInOpen(y_var.clone(), p_var.clone()));
        let k_p_inter_p = Formula::ExistsPoints(y_var, Box::new(Formula::And(Box::new(y_in_k_p), Box::new(y_in_p))));
        
        let premise = Formula::And(Box::new(o_inter_k_p), Box::new(k_p_inter_p));
        let o_inter_p = Formula::Atom(Atom::OpenIntersection(o_var.clone(), p_var.clone()));
        let implication = Formula::Implies(Box::new(premise), Box::new(o_inter_p));
        
        let forall_p = Formula::ForAllOpens(p_var, Box::new(implication));
        let transitive_k_p = Formula::ForAllOpens(o_var, Box::new(forall_p));
        
        Formula::And(Box::new(nonempty_k_p), Box::new(transitive_k_p))
    }

    fn expand_irregular(&self, p: String) -> Formula {
        // irregular p = ! (regular p)
        let regular_p = self.expand_regular(p);
        Formula::Not(Box::new(regular_p))
    }

    fn expand_weakly_regular(&self, p: String) -> Formula {
        // weakly_regular p = p in (K p)
        Formula::Atom(Atom::PointInCommunity(p.clone(), p))
    }

    fn expand_quasiregular(&self, p: String) -> Formula {
        // quasiregular p = K p
        Formula::Atom(Atom::Community(p))
    }

    fn expand_indirectly_regular(&self, p: String) -> Formula {
        // indirectly_regular p = EP q. p inter q && regular q
        let q_var = self.fresh_var("q", 0);
        let p_inter_q = self.expand_point_inter(p, q_var.clone());
        let regular_q = self.expand_regular(q_var.clone());
        let premise = Formula::And(Box::new(p_inter_q), Box::new(regular_q));
        Formula::ExistsPoints(q_var, Box::new(premise))
    }

    fn expand_hypertransitive(&self, p: String) -> Formula {
        // hypertransitive p = AO O. AO Q. (AO P. p in P => O inter P inter Q) => O inter Q
        let o_var = self.fresh_var("O", 0);
        let q_var = self.fresh_var("Q", 0);
        let p_var = self.fresh_var("P", 0);
        
        let p_in_p = Formula::Atom(Atom::PointInOpen(p, p_var.clone()));
        let o_inter_p = Formula::Atom(Atom::OpenIntersection(o_var.clone(), p_var.clone()));
        let p_inter_q = Formula::Atom(Atom::OpenIntersection(p_var.clone(), q_var.clone()));
        let o_inter_p_inter_q = Formula::And(Box::new(o_inter_p), Box::new(p_inter_q));
        
        let inner_impl = Formula::Implies(Box::new(p_in_p), Box::new(o_inter_p_inter_q));
        let forall_p = Formula::ForAllOpens(p_var, Box::new(inner_impl));
        
        let o_inter_q = Formula::Atom(Atom::OpenIntersection(o_var.clone(), q_var.clone()));
        let outer_impl = Formula::Implies(Box::new(forall_p), Box::new(o_inter_q));
        
        let forall_q = Formula::ForAllOpens(q_var, Box::new(outer_impl));
        Formula::ForAllOpens(o_var, Box::new(forall_q))
    }

    fn expand_unconflicted(&self, p: String) -> Formula {
        // unconflicted p = AP x. AP y. x inter p inter y => x inter y
        let x_var = self.fresh_var("x", 0);
        let y_var = self.fresh_var("y", 0);
        
        let x_inter_p = self.expand_point_inter(x_var.clone(), p.clone());
        let p_inter_y = self.expand_point_inter(p, y_var.clone());
        let x_inter_y = self.expand_point_inter(x_var.clone(), y_var.clone());
        
        let premise = Formula::And(Box::new(x_inter_p), Box::new(p_inter_y));
        let implication = Formula::Implies(Box::new(premise), Box::new(x_inter_y));
        let forall_y = Formula::ForAllPoints(y_var, Box::new(implication));
        Formula::ForAllPoints(x_var, Box::new(forall_y))
    }

    fn expand_conflicted(&self, p: String) -> Formula {
        // conflicted p = ! (unconflicted p)
        let unconflicted_p = self.expand_unconflicted(p);
        Formula::Not(Box::new(unconflicted_p))
    }

    fn expand_conflicted_space(&self) -> Formula {
        // conflicted_space = AP p. conflicted p
        let p_var = self.fresh_var("p", 0);
        let conflicted_p = self.expand_conflicted(p_var.clone());
        Formula::ForAllPoints(p_var, Box::new(conflicted_p))
    }

    fn expand_unconflicted_space(&self) -> Formula {
        // unconflicted_space = AP p. unconflicted p
        let p_var = self.fresh_var("p", 0);
        let unconflicted_p = self.expand_unconflicted(p_var.clone());
        Formula::ForAllPoints(p_var, Box::new(unconflicted_p))
    }

    fn expand_regular_space(&self) -> Formula {
        // regular_space = AP p. regular p
        let p_var = self.fresh_var("p", 0);
        let regular_p = self.expand_regular(p_var.clone());
        Formula::ForAllPoints(p_var, Box::new(regular_p))
    }

    fn expand_irregular_space(&self) -> Formula {
        // irregular_space = AP p. irregular p
        let p_var = self.fresh_var("p", 0);
        let irregular_p = self.expand_irregular(p_var.clone());
        Formula::ForAllPoints(p_var, Box::new(irregular_p))
    }

    fn expand_weakly_regular_space(&self) -> Formula {
        // weakly_regular_space = AP p. weakly_regular p
        let p_var = self.fresh_var("p", 0);
        let weakly_regular_p = self.expand_weakly_regular(p_var.clone());
        Formula::ForAllPoints(p_var, Box::new(weakly_regular_p))
    }

    fn expand_quasiregular_space(&self) -> Formula {
        // quasiregular_space = AP p. quasiregular p
        let p_var = self.fresh_var("p", 0);
        let quasiregular_p = self.expand_quasiregular(p_var.clone());
        Formula::ForAllPoints(p_var, Box::new(quasiregular_p))
    }

    fn expand_indirectly_regular_space(&self) -> Formula {
        // indirectly_regular_space = AP p. indirectly_regular p
        let p_var = self.fresh_var("p", 0);
        let indirectly_regular_p = self.expand_indirectly_regular(p_var.clone());
        Formula::ForAllPoints(p_var, Box::new(indirectly_regular_p))
    }

    fn expand_hypertransitive_space(&self) -> Formula {
        // hypertransitive_space = AP p. hypertransitive p
        let p_var = self.fresh_var("p", 0);
        let hypertransitive_p = self.expand_hypertransitive(p_var.clone());
        Formula::ForAllPoints(p_var, Box::new(hypertransitive_p))
    }
}

/// Parse a formula string into a Formula AST
pub fn parse_formula(input: &str) -> Result<Formula, String> {
    let mut parser = Parser::new(input)?;
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_atomic() {
        let formula = parse_formula("x in X").unwrap();
        assert_eq!(formula, Formula::Atom(Atom::PointInOpen("x".to_string(), "X".to_string())));
    }
    
    #[test]
    fn test_parse_intersection() {
        let formula = parse_formula("X inter Y").unwrap();
        assert_eq!(formula, Formula::Atom(Atom::OpenIntersection("X".to_string(), "Y".to_string())));
    }
    
    #[test]
    fn test_parse_quantifier() {
        let formula = parse_formula("AO X. x in X").unwrap();
        match formula {
            Formula::ForAllOpens(var, inner) => {
                assert_eq!(var, "X");
                assert_eq!(*inner, Formula::Atom(Atom::PointInOpen("x".to_string(), "X".to_string())));
            }
            _ => panic!("Expected ForAllOpens"),
        }
    }
    
    #[test]
    fn test_parse_nonempty() {
        let formula = parse_formula("nonempty X").unwrap();
        assert_eq!(formula, Formula::Atom(Atom::OpenNonempty("X".to_string())));
    }
    
    #[test]
    fn test_parse_complex() {
        let formula = parse_formula("AO X. EO Y. AP x. (x in X) || (X inter Y) => !(x in Y)").unwrap();
        // This should parse without error - detailed structure testing would be quite complex
        println!("{:?}", formula);
    }
    
    #[test]
    fn test_parse_precedence() {
        // Test that !(nonempty X) || (X inter Y) parses correctly
        let formula = parse_formula("!(nonempty X) || X inter Y").unwrap();
        match formula {
            Formula::Or(left, right) => {
                // Left should be !(nonempty X)
                match *left {
                    Formula::Not(inner) => {
                        match *inner {
                            Formula::Atom(Atom::OpenNonempty(var)) => assert_eq!(var, "X"),
                            _ => panic!("Expected OpenNonempty"),
                        }
                    }
                    _ => panic!("Expected Not"),
                }
                // Right should be X inter Y
                match *right {
                    Formula::Atom(Atom::OpenIntersection(var1, var2)) => {
                        assert_eq!(var1, "X");
                        assert_eq!(var2, "Y");
                    }
                    _ => panic!("Expected OpenIntersection"),
                }
            }
            _ => panic!("Expected Or, got {:?}", formula),
        }
    }

    #[test]
    fn test_parse_built_in_notations() {
        // Test simple built-in predicates
        let formula = parse_formula("transitive T").unwrap();
        // Should parse without error and expand to quantified formula
        println!("transitive T: {:?}", formula);
        
        let formula = parse_formula("topen X").unwrap();
        println!("topen X: {:?}", formula);
        
        let formula = parse_formula("regular p").unwrap();
        println!("regular p: {:?}", formula);
        
        let formula = parse_formula("unconflicted p").unwrap();
        println!("unconflicted p: {:?}", formula);
        
        let formula = parse_formula("conflicted_space").unwrap();
        println!("conflicted_space: {:?}", formula);
    }

    #[test]
    fn test_parse_point_inter() {
        // Test point inter point
        let formula = parse_formula("p inter q").unwrap();
        println!("p inter q: {:?}", formula);
        
        // Test triple point inter
        let formula = parse_formula("p inter q inter r").unwrap();
        println!("p inter q inter r: {:?}", formula);
    }

    #[test]
    fn test_parse_triple_open_inter() {
        // Test triple open inter
        let formula = parse_formula("O inter P inter Q").unwrap();
        match formula {
            Formula::And(left, right) => {
                match (*left, *right) {
                    (Formula::Atom(Atom::OpenIntersection(o1, p1)), 
                     Formula::Atom(Atom::OpenIntersection(p2, q))) => {
                        assert_eq!(o1, "O");
                        assert_eq!(p1, "P");
                        assert_eq!(p2, "P");
                        assert_eq!(q, "Q");
                    }
                    _ => panic!("Expected two OpenIntersection atoms"),
                }
            }
            _ => panic!("Expected And formula"),
        }
    }

    #[test]
    fn test_parse_space_predicates() {
        // Test space predicates
        let formula = parse_formula("regular_space").unwrap();
        println!("regular_space: {:?}", formula);
        
        let formula = parse_formula("hypertransitive_space").unwrap();
        println!("hypertransitive_space: {:?}", formula);
    }

    #[test]
    fn test_parse_community_construction() {
        // Test basic K p construction
        let formula = parse_formula("K p").unwrap();
        assert_eq!(formula, Formula::Atom(Atom::Community("p".to_string())));
        
        // Test x in K p construction
        let formula = parse_formula("x in K p").unwrap();
        assert_eq!(formula, Formula::Atom(Atom::PointInCommunity("x".to_string(), "p".to_string())));
        
        // Test in quantified context
        let formula = parse_formula("AP p. K p").unwrap();
        match formula {
            Formula::ForAllPoints(var, inner) => {
                assert_eq!(var, "p");
                assert_eq!(*inner, Formula::Atom(Atom::Community("p".to_string())));
            }
            _ => panic!("Expected ForAllPoints"),
        }
        
        // Test complex formula with community
        let formula = parse_formula("AP p. EP q. q in K p").unwrap();
        match formula {
            Formula::ForAllPoints(p_var, p_inner) => {
                assert_eq!(p_var, "p");
                match *p_inner {
                    Formula::ExistsPoints(q_var, q_inner) => {
                        assert_eq!(q_var, "q");
                        assert_eq!(*q_inner, Formula::Atom(Atom::PointInCommunity("q".to_string(), "p".to_string())));
                    }
                    _ => panic!("Expected ExistsPoints"),
                }
            }
            _ => panic!("Expected ForAllPoints"),
        }
    }
}