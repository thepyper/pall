use pest::iterators::Pair;
use pest::Parser;

use super::{GrammarParser, Rule, ParseError};
use crate::machine::link::Link;

/// Parse a plain string into a Link AST.
pub fn parse_link(input: &str) -> Result<Link, ParseError> {
    let mut pairs = GrammarParser::parse(Rule::link, input)
        .map_err(|e| ParseError::from_pest(input, e))?;

    let pair = pairs.next().ok_or_else(|| ParseError::new("empty link"))?;
    link_from_pair(pair)
}

fn link_from_pair(pair: Pair<Rule>) -> Result<Link, ParseError> {
    let children: Vec<Pair<Rule>> = pair.into_inner().collect();

    // Collect all identifiers (skip the "." literal which may appear silently)
    let identifiers: Vec<String> = children
        .iter()
        .filter(|p| p.as_rule() == Rule::identifier)
        .map(|p| p.as_str().to_string())
        .collect();

    if identifiers.len() != 2 {
        return Err(ParseError::new(format!(
            "link: expected 2 identifiers, found {}",
            identifiers.len()
        )));
    }

    Ok(Link {
        id: identifiers[0].clone(),
        output: identifiers[1].clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expect_link(input: &str) -> Link {
        parse_link(input).unwrap_or_else(|e| panic!("parse failed for '{}': {}", input, e))
    }

    #[test]
    fn test_simple_link() {
        let link = expect_link("myid.myoutput");
        assert_eq!(link.id, "myid");
        assert_eq!(link.output, "myoutput");
    }

    #[test]
    fn test_link_with_underscore() {
        let link = expect_link("id_1.output_2");
        assert_eq!(link.id, "id_1");
        assert_eq!(link.output, "output_2");
    }

    #[test]
    fn test_link_error() {
        let result = parse_link("invalid_no_dot");
        assert!(result.is_err());
    }
}
