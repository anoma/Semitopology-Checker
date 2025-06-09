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
    
    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
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
                let formula = self.parse_quantifier()?;
                
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
                let formula = self.parse_quantifier()?;
                
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
            Token::Identifier(_) => {
                self.parse_atomic()
            }
            _ => Err(format!("Unexpected token: {:?}", self.current_token)),
        }
    }
    
    fn parse_atomic(&mut self) -> Result<Formula, String> {
        let first_id = self.parse_identifier()?;
        
        match &self.current_token {
            Token::In => {
                self.advance()?;
                let second_id = self.parse_identifier()?;
                Ok(Formula::Atom(Atom::PointInOpen(first_id, second_id)))
            }
            Token::Inter => {
                self.advance()?;
                let second_id = self.parse_identifier()?;
                Ok(Formula::Atom(Atom::OpenIntersection(first_id, second_id)))
            }
            _ => Err(format!("Expected 'in' or 'inter' after identifier, found {:?}", self.current_token)),
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
    fn test_parse_complex() {
        let formula = parse_formula("AO X. EO Y. AP x. (x in X) || (X inter Y) => !(x in Y)").unwrap();
        // This should parse without error - detailed structure testing would be quite complex
        println!("{:?}", formula);
    }
}