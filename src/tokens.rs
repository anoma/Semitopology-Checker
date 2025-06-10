//! Lexical tokens for semitopology formulas using Logos
//!
//! This module defines all tokens recognized by the lexer, including:
//! - Logical operators with proper precedence
//! - Quantifiers for points and opens  
//! - Built-in predicates and macros
//! - Variables distinguished by case (lowercase=points, uppercase=opens)

use logos::Logos;

/// Tokens for the semitopology formula language
/// 
/// Uses Logos for fast DFA-based lexing with priority resolution
/// for keywords vs identifiers.
#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    // Logical operators
    #[token("&&")]
    And,
    
    #[token("||")]
    Or,
    
    #[token("=>")]
    Implies,
    
    #[token("!")]
    Not,
    
    #[token("!=")]
    NotEqual,
    
    #[token("=")]
    Equal,
    
    #[token("<=>")]
    Iff,
    
    // Quantifiers
    #[token("AP")]
    AP,
    
    #[token("EP")]
    EP,
    
    #[token("AO")]
    AO,
    
    #[token("EO")]
    EO,
    
    // Primitives
    #[token("in")]
    In,
    
    #[token("inter")]
    Inter,
    
    #[token("nonempty")]
    Nonempty,
    
    #[token("K", priority = 2)]
    K,
    
    #[token("IC")]
    IC,
    
    // Built-in macro keywords
    #[token("transitive")]
    Transitive,
    
    #[token("topen")]
    Topen,
    
    #[token("regular")]
    Regular,
    
    #[token("irregular")]
    Irregular,
    
    #[token("weakly_regular")]
    WeaklyRegular,
    
    #[token("quasiregular")]
    Quasiregular,
    
    #[token("indirectly_regular")]
    IndirectlyRegular,
    
    #[token("hypertransitive")]
    Hypertransitive,
    
    #[token("unconflicted")]
    Unconflicted,
    
    #[token("conflicted")]
    Conflicted,
    
    #[token("conflicted_space")]
    ConflictedSpace,
    
    #[token("unconflicted_space")]
    UnconflictedSpace,
    
    #[token("regular_space")]
    RegularSpace,
    
    #[token("irregular_space")]
    IrregularSpace,
    
    #[token("weakly_regular_space")]
    WeaklyRegularSpace,
    
    #[token("quasiregular_space")]
    QuasiregularSpace,
    
    #[token("indirectly_regular_space")]
    IndirectlyRegularSpace,
    
    #[token("hypertransitive_space")]
    HypertransitiveSpace,
    
    // Variables: case determines semantic type
    // Point variables: lowercase start (x, p, point1)
    #[regex(r"[a-z][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    PointVar(String),
    
    // Open variables: uppercase start (X, T, Open1)  
    // Lower priority than keywords to resolve conflicts
    #[regex(r"[A-Z][a-zA-Z0-9_]*", priority = 1, callback = |lex| lex.slice().to_owned())]
    OpenVar(String),
    
    // Punctuation
    #[token(".")]
    Dot,
    
    #[token("(")]
    LeftParen,
    
    #[token(")")]
    RightParen,
    
    // Whitespace is skipped during lexing
    #[regex(r"[ \t\r\n]+", logos::skip)]
    Error,
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    token_stream: logos::SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            token_stream: Token::lexer(input).spanned(),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, &'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream.next().map(|(token, span)| {
            match token {
                Ok(token) => Ok((span.start, token, span.end)),
                Err(()) => Err("Lexer error"),
            }
        })
    }
}