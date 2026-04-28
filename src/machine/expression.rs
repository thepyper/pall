use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};

use super::types::Value;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    BitAnd,
    BitOr,
    BitXor,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Reference {
    pub target: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Expression {
    Value(Value),
    Reference(Reference),
    Parenthesis(Box<Expression>),
    Binary(Box<Expression>, BinaryOperator, Box<Expression>),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum AssignmentOperator {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Statement {
    pub target: String,
    pub operator: AssignmentOperator,
    pub expression: Expression,
}

#[derive(Parser)]
#[grammar = "machine/grammar.pest"]
struct GrammarParser;

struct ParseError;

fn parse_expression(s: &str) -> Result<Expression, ParseError> {
    match GrammarParser::parse(Rule::expression, s) {
        Ok(p) => {
            match p.as_rule() {
                Rule::binary {

                    },
            }
        }
        Err(e) => {}
    }
}

#[test]
fn test_parse_expression() {
    eprintln!(
        "1. {:?}",
        GrammarParser::parse(Rule::expression, r###"1"###).unwrap()
    );
    eprintln!(
        "2. {:?}",
        GrammarParser::parse(Rule::expression, r###"(1)"###).unwrap()
    );
    eprintln!(
        "3. {:?}",
        GrammarParser::parse(Rule::expression, r###"1 + 2"###).unwrap()
    );
    eprintln!(
        "4. {:?}",
        GrammarParser::parse(Rule::expression, r###"(1 + 3)"###).unwrap()
    );
    eprintln!(
        "5. {:?}",
        GrammarParser::parse(Rule::expression, r###"1 + (4)"###).unwrap()
    );
    eprintln!(
        "6. {:?}",
        GrammarParser::parse(Rule::expression, r###"1 + (5 + 6)"###).unwrap()
    );
    eprintln!(
        "7. {:?}",
        GrammarParser::parse(Rule::expression, r###"(1 + 7) + (8 - 9)"###).unwrap()
    );
    eprintln!(
        "8. {:?}",
        GrammarParser::parse(Rule::expression, r###"(1 + (2 + (3 + 4))) + 8"###).unwrap()
    );
    eprintln!(
        "9. {:?}",
        GrammarParser::parse(Rule::expression, r###""hahaha""###).unwrap()
    );
    eprintln!(
        "10. {:?}",
        GrammarParser::parse(Rule::expression, r###""1 + 2""###).unwrap()
    );
    eprintln!(
        "11. {:?}",
        GrammarParser::parse(Rule::expression, r###""(3)""###).unwrap()
    );
    eprintln!(
        "12. {:?}",
        GrammarParser::parse(Rule::expression, r###"'aaaa'"###).unwrap()
    );
    eprintln!(
        "13. {:?}",
        GrammarParser::parse(Rule::expression, r###"'1 + 2'"###).unwrap()
    );
    eprintln!(
        "14. {:?}",
        GrammarParser::parse(Rule::expression, r###"'(3)'"###).unwrap()
    );
    eprintln!(
        "15. {:?}",
        GrammarParser::parse(Rule::expression, r###""'''''""###).unwrap()
    );
    eprintln!(
        "16. {:?}",
        GrammarParser::parse(Rule::expression, r###"'"""""'"###).unwrap()
    );
    eprintln!(
        "17. {:?}",
        GrammarParser::parse(Rule::expression, r###"id1"###).unwrap()
    );
    eprintln!(
        "18. {:?}",
        GrammarParser::parse(Rule::expression, r###"_id2"###).unwrap()
    );
    eprintln!(
        "19. {:?}",
        GrammarParser::parse(Rule::expression, r###"id_3"###).unwrap()
    );
    eprintln!(
        "20. {:?}",
        GrammarParser::parse(Rule::expression, r###"a + b + c + d + e"###).unwrap()
    );
}

#[test]
fn test_parse_statement() {
    eprintln!(
        "1. {:?}",
        GrammarParser::parse(Rule::statement, r###"x = 1"###).unwrap()
    );
    eprintln!(
        "2. {:?}",
        GrammarParser::parse(Rule::statement, r###"y &= 1 + 5"###).unwrap()
    );
    eprintln!(
        "3. {:?}",
        GrammarParser::parse(Rule::statement, r###"z /= 9 + (5 * y)"###).unwrap()
    );
    eprintln!(
        "4. {:?}",
        GrammarParser::parse(Rule::statement, r###"v *= z + y + z"###).unwrap()
    );
    eprintln!(
        "5. {:?}",
        GrammarParser::parse(Rule::statement, r###"u += (1 & 2)"###).unwrap()
    );
}
