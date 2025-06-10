//! Parser for semitopology formulas using LALRPOP + Logos
//!
//! This module provides the main parsing interface that combines:
//! 1. Logos-based lexical analysis for fast tokenization
//! 2. LALRPOP-generated LR(1) parser for syntax analysis  
//! 3. Macro expansion to convert high-level constructs to core logic
//!
//! The parser handles the complete grammar including quantifiers, logical operators,
//! built-in predicates, and complex macro expansions with proper variable scoping.

use crate::model_checker::Formula;
use crate::tokens::Lexer;
use crate::macro_expander::MacroExpander;
use lalrpop_util::lalrpop_mod;

// Include the LALRPOP-generated parser
lalrpop_mod!(pub grammar);

/// Parse a formula string into the model checker's Formula AST
///
/// This is the main entry point for parsing. It performs three stages:
/// 1. **Lexical analysis**: Tokenize input using Logos DFA lexer
/// 2. **Syntax analysis**: Parse tokens using LALRPOP LR(1) parser  
/// 3. **Macro expansion**: Expand all macro constructs with fresh variable generation
///
/// # Arguments
/// * `input` - The formula string to parse
///
/// # Returns
/// * `Ok(Formula)` - Successfully parsed and expanded formula
/// * `Err(String)` - Parse or expansion error with description
///
/// # Examples
/// ```
/// let formula = parse_formula("EO X. EP x. x in X")?;
/// let complex = parse_formula("AO T. transitive T => regular_space")?;
/// ```
pub fn parse_formula(input: &str) -> Result<Formula, String> {
    // Stage 1: Lexical analysis
    let lexer = Lexer::new(input);
    
    // Stage 2: Syntax analysis  
    let parser = grammar::PropParser::new();
    let ast = parser.parse(lexer)
        .map_err(|e| format!("Parse error: {:?}", e))?;
    
    // Stage 3: Macro expansion
    let mut expander = MacroExpander::new();
    expander.expand(ast)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Core atomic propositions
    #[test]
    fn test_a01_point_in_open() {
        let result = parse_formula("x in X");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_a02_open_inter() {
        let result = parse_formula("X inter Y");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_a03_nonempty() {
        let result = parse_formula("nonempty T");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_a04_point_in_community() {
        let result = parse_formula("y in K x");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_a05_point_in_ic() {
        let result = parse_formula("x in IC Y");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    // Operator precedence and associativity
    #[test]
    fn test_p01_negation_precedence() {
        let result = parse_formula("!nonempty X || nonempty X");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_p02_and_or_precedence() {
        let result = parse_formula("nonempty A && nonempty B || nonempty C");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_p03_implies_associativity() {
        let result = parse_formula("nonempty A => nonempty B => nonempty C");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_p04_complex_precedence() {
        let result = parse_formula("!(AP x. x in X) && nonempty X");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    // Quantifiers with weakest precedence
    #[test]
    fn test_q01_nested_quantifiers() {
        let result = parse_formula("AP x. EP y. x in X && y in Y");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_q02_open_quantifiers() {
        let result = parse_formula("AO X. AO Y. X inter Y => (AP x. x in X)");
        match result {
            Ok(_) => {},
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    // Open intersection macros
    #[test]
    fn test_m01_triple_open_inter() {
        let result = parse_formula("O inter P inter Q");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    // Point intersection macros
    #[test]
    fn test_m02_point_inter() {
        let result = parse_formula("p inter q");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_m03_triple_point_inter() {
        let result = parse_formula("p inter q inter r");
        assert!(result.is_ok(), "Formula should parse successfully");
    }

    #[test]
    fn test_m03b_triple_inter_in_complex_expression() {
        // Test triple intersections (both point and open) within complex logical expressions
        let result = parse_formula("AP p. EP q. EP r. (p inter q inter r) => (AO X. AO Y. AO Z. (X inter Y inter Z) && nonempty X)");
        assert!(result.is_ok(), "Complex formula with triple intersections should parse successfully");
        
        // Test mixed intersection types in conjunction with proper quantification
        let result2 = parse_formula("AP p. EP q. EP r. AO X. AO Y. AO Z. ((p inter q inter r) && (X inter Y inter Z)) => (EP x. x in X)");
        assert!(result2.is_ok(), "Mixed triple intersections in conjunction should parse successfully");
    }
    
    // Built-in predicates (single-argument macros)
    #[test]
    fn test_m04_transitive() {
        let result = parse_formula("transitive T");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_m05_topen() {
        let result = parse_formula("topen U");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_m06_regular() {
        let result = parse_formula("regular p");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_m07_irregular() {
        let result = parse_formula("irregular p");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_m08_weakly_regular() {
        let result = parse_formula("weakly_regular p");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_m09_quasiregular() {
        let result = parse_formula("quasiregular p");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_m10_indirectly_regular() {
        let result = parse_formula("indirectly_regular p");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_m11_hypertransitive() {
        let result = parse_formula("hypertransitive p");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_m12_unconflicted() {
        let result = parse_formula("unconflicted p");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_m13_conflicted() {
        let result = parse_formula("conflicted p");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    // Space-wide predicates (zero-argument macros)
    #[test]
    fn test_m14_regular_space() {
        let result = parse_formula("regular_space");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_m15_conflicted_space() {
        let result = parse_formula("conflicted_space");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_m16_hypertransitive_space() {
        let result = parse_formula("hypertransitive_space");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    // Complex formulas mixing primitives and macros
    #[test] 
    fn test_c01_mixed_transitive() {
        // Try a simpler case first
        let result1 = parse_formula("transitive T && nonempty T");
        match result1 {
            Ok(_) => println!("Simple case works"),
            Err(e) => println!("Simple case error: {}", e),
        }
        
        let result = parse_formula("EO T. transitive T && nonempty T");
        match result {
            Ok(_) => {},
            Err(e) => panic!("Parse error: {}", e),
        }
    }
    
    #[test]
    fn test_c02_mixed_regular() {
        let result = parse_formula("regular p && x in IC (K p)");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    #[test]
    fn test_c03_complex_formula() {
        let result = parse_formula("AO X. EO Y. AP x. (x in X) || (X inter Y) => !(x in Y)");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    // Variable capture prevention
    #[test]
    fn test_ar1_variable_capture() {
        let result = parse_formula("EP x. AO X. x in X");
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    // Integration tests
    #[test]
    fn test_big_complex_formula() {
        let result = parse_formula(
            "AO X. EO Y. AP p. EP q. ((p in X) && (q inter p) && regular q) => (hypertransitive p || !nonempty Y)"
        );
        assert!(result.is_ok(), "Formula should parse successfully");
    }
    
    // Error handling tests (expected failures)
    #[test]
    fn test_e01_missing_dot() {
        let result = parse_formula("AP x x in X");
        assert!(result.is_err(), "Formula should fail to parse (missing dot)");
    }
    
    #[test]
    fn test_e02_point_open_confusion() {
        let result = parse_formula("AO X. X in x");
        assert!(result.is_err(), "Formula should fail to parse (point/open confusion)");
    }
    
    #[test]
    fn test_e03_incomplete_binary() {
        let result = parse_formula("X inter");
        assert!(result.is_err(), "Formula should fail to parse (incomplete binary operator)");
    }
    
    #[test]
    fn test_e04_missing_operand() {
        let result = parse_formula("nonempty");
        assert!(result.is_err(), "Formula should fail to parse (missing operand)");
    }
    
    #[test]
    fn test_e05_missing_k_argument() {
        let result = parse_formula("K");
        assert!(result.is_err(), "Formula should fail to parse (missing K argument)");
    }
    
    #[test]
    fn test_e06_missing_macro_argument() {
        let result = parse_formula("regular");
        assert!(result.is_err(), "Formula should fail to parse (macro needs argument)");
    }
    
    #[test]
    fn test_e07_unmatched_paren() {
        let result = parse_formula("!(AO X. X inter Y");
        assert!(result.is_err(), "Formula should fail to parse (unmatched parenthesis)");
    }
    
    #[test]
    fn test_e08_unknown_keyword() {
        let result = parse_formula("unknown_kw p");
        assert!(result.is_err(), "Formula should fail to parse (unknown keyword)");
    }
    
    // Basic functionality tests
    #[test]
    fn test_parse_simple_atomic() {
        let formula = parse_formula("p in X").unwrap();
        println!("{:?}", formula);
    }
    
    #[test]
    fn test_parse_quantifier() {
        let formula = parse_formula("AP p. p in X").unwrap();
        println!("{:?}", formula);
    }
    
    #[test]
    fn test_parse_macro_regular() {
        let formula = parse_formula("regular p").unwrap();
        println!("{:?}", formula);
    }
    
    #[test]
    fn test_parse_space_predicate() {
        let formula = parse_formula("regular_space").unwrap();
        println!("{:?}", formula);
    }
    
    #[test]
    fn test_parse_point_intersection() {
        let formula = parse_formula("p inter q").unwrap();
        println!("{:?}", formula);
    }
    
    #[test]
    fn test_parse_triple_intersection() {
        let formula = parse_formula("O inter P inter Q").unwrap();
        println!("{:?}", formula);
    }
    
    #[test]
    fn test_parse_logical_operations() {
        let formula = parse_formula("p in X && q in Y").unwrap();
        println!("{:?}", formula);
    }
    
    #[test]
    fn test_parse_implications() {
        let formula = parse_formula("p in X => q in Y").unwrap();
        println!("{:?}", formula);
    }
    
    #[test]
    fn test_parse_negation() {
        let formula = parse_formula("!(p in X)").unwrap();
        println!("{:?}", formula);
    }
    
    #[test]
    fn test_parse_community() {
        let formula = parse_formula("nonempty (K p)").unwrap();
        println!("{:?}", formula);
    }
    
    #[test]
    fn test_parse_interior_complement() {
        let formula = parse_formula("p in (IC X)").unwrap();
        println!("{:?}", formula);
    }
}