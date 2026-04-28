use pest::error::Error;
use std::fmt;

mod grammar {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "machine/grammar.pest"]
    pub struct GrammarParser;
}

pub use grammar::GrammarParser;
pub use grammar::Rule;

mod expression;
mod statement;
mod link;

pub use expression::parse_expression;
pub use statement::parse_statement;
pub use link::parse_link;

// ── ParseError ───────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), line: 0, column: 0 }
    }

    pub fn from_pest(_input: &str, err: Error<Rule>) -> Self {
        Self {
            message: err.variant.to_string(),
            line: 1,
            column: 0,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at line {}, column {}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for ParseError {}
