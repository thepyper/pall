use pest_derive::Parser;
use pest::Parser;

#[derive(Parser)]
#[grammar = "machine/expression.pest"]
struct ExpressionParser;
