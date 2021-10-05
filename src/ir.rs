//! Intermediate representation

use std::fmt::{Debug, Display};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until1, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit0, digit1},
    combinator::{map, opt, recognize, value},
    error::{ParseError, VerboseError},
    multi::{many0, many1},
    sequence::{delimited, pair, separated_pair, tuple},
    AsChar, InputTakeAtPosition, Parser,
};
use thiserror::Error;

use crate::{
    attribute::{attribute_parser, Attribute},
    ws::ws,
};

pub type ParseResult<I, O> = nom::IResult<I, O, nom::error::VerboseError<I>>;

#[derive(Debug)]
pub enum GraphKind {
    Directed,
    Undirected,
}

#[derive(Debug)]
pub struct Graph {
    pub kind: GraphKind,
    pub strict: bool,
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Attribute(AttributeStatement),
    Node(NodeStatement),
    Edge(EdgeStatement),
    Definition(DefinitionStatement),
}

#[derive(Debug)]
pub enum AttributeKind {
    Graph,
    Node,
    Edge,
}

#[derive(Debug)]
pub struct AttributeStatement {
    kind: AttributeKind,
    attributes: Vec<Attribute>,
}

pub type Ident = String;

#[derive(Debug)]
pub struct NodeStatement {
    name: Ident,
    attributes: Vec<Attribute>,
}

#[derive(Debug)]
pub struct EdgeStatement {
    list: Vec<Ident>,
    attributes: Vec<Attribute>,
}

#[derive(Debug)]
pub enum EdgeTarget {
    Node(Ident),
}

#[derive(Debug)]
pub struct DefinitionStatement {
    lhs: Ident,
    rhs: Ident,
}

/// Parser that mathces characters in the range of octal values `[\200-\377]`.
fn highbit<I, E>(input: I) -> nom::IResult<I, I, E>
where
    E: ParseError<I>,
    I: InputTakeAtPosition,
    <I as InputTakeAtPosition>::Item: AsChar,
{
    take_while1(|c: <I as InputTakeAtPosition>::Item| {
        let c = c.as_char() as i32;
        c >= 0o200 && c <= 0o377
    })(input)
}

/// Any string of alphabetic ([a-zA-Z\200-\377]) characters, underscores ('_') or digits([0-9]),
/// not beginning with a digit
fn string_ident_parser(input: &str) -> ParseResult<&str, Ident> {
    let start_char = alt((alpha1, highbit, tag("_")));
    let continue_char = alt((alphanumeric1, highbit, tag("_")));
    recognize(pair(start_char, many0(continue_char)))(input).map(|(i, o)| (i, o.to_owned()))
}

/// a numeral [-]?(.[0-9]⁺ | [0-9]⁺(.[0-9]*)? )
fn num_ident_parser(input: &str) -> ParseResult<&str, Ident> {
    recognize(tuple((opt(tag("-")), digit1, tag("."), digit0)))(input)
        .map(|(i, o)| (i, o.to_owned()))
}

fn quote_string_fragment_parser(input: &str) -> ParseResult<&str, &str> {
    let escaped_quote = value(r#"""#, tag(r#"\""#));
    let string_fragment = alt((take_until1(r#"\""#), take_until1(r#"""#)));
    alt((escaped_quote, string_fragment))(input)
}

/// any double-quoted string ("...") possibly containing escaped quotes (\")
fn quote_string_ident_parser(input: &str) -> ParseResult<&str, Ident> {
    delimited(
        char('"'),
        // |i| {
        //     let mut input = i;
        //     let mut result = String::new();
        //     loop {
        //         match quote_string_fragment_parser(input) {
        //             Ok((rest, parsed)) => {
        //                 println!("rest: ##{}## parsed: ##{}##", rest, parsed);
        //                 input = rest;
        //                 result.push_str(parsed);
        //                 println!("result: ##{}##", result);
        //             }
        //             Err(nom::Err::Error(_)) => {
        //                 println!("soft failure ##{}##", result);
        //                 return Ok((input, result));
        //             }
        //             Err(e) => {
        //                 return Err(e);
        //             }
        //         }
        //     }
        // },
        map(
            many0(
                quote_string_fragment_parser,
                // String::new,
                // |mut string, fragment| {
                //     string.push_str(fragment);
                //     string
                // },
            ),
            |v| v.join(""),
        ),
        char('"'),
    )(input)
}

/// An ID is one of the following:
/// * Any string of alphabetic ([a-zA-Z\200-\377]) characters, underscores ('_') or digits([0-9]),
///   not beginning with a digit;
/// * a numeral [-]?(.[0-9]⁺ | [0-9]⁺(.[0-9]*)? );
/// * any double-quoted string ("...") possibly containing escaped quotes (\")¹.
fn ident_parser(input: &str) -> ParseResult<&str, Ident> {
    alt((
        string_ident_parser,
        num_ident_parser,
        quote_string_ident_parser,
    ))(input)
}

fn a_list_parser(input: &str) -> ParseResult<&str, Vec<Attribute>> {
    many1(attribute_parser)(input)
}

fn attr_list_parser(input: &str) -> ParseResult<&str, Vec<Attribute>> {
    let (rest, mut lists) = many1(ws(delimited(char('['), ws(a_list_parser), char(']'))))(input)?;
    return Ok((rest, lists.drain(..).flatten().collect::<Vec<_>>()));
}

fn edge_statement_parser(input: &str) -> ParseResult<&str, EdgeStatement> {
    let (rest, (id, mut rhs_list, attributes)) = tuple((
        ws(ident_parser),
        many1(pair(ws(alt((tag("--"), tag("->")))), ws(ident_parser))),
        opt(attr_list_parser),
    ))(input)?;
    Ok((
        rest,
        EdgeStatement {
            list: vec![id]
                .drain(..)
                .chain(rhs_list.drain(..).map(|(_, id)| id))
                .collect::<Vec<_>>(),
            attributes: attributes.unwrap_or(vec![]),
        },
    ))
}

fn node_statement_parser(input: &str) -> ParseResult<&str, NodeStatement> {
    let (rest, (id, attributes)) = tuple((ws(ident_parser), opt(attr_list_parser)))(input)?;
    Ok((
        rest,
        NodeStatement {
            name: id,
            attributes: attributes.unwrap_or(vec![]),
        },
    ))
}

fn attribute_statement_parser(input: &str) -> ParseResult<&str, AttributeStatement> {
    let (rest, (kind, attributes)) = pair(
        ws(alt((
            tag("graph").map(|_| AttributeKind::Graph),
            tag("node").map(|_| AttributeKind::Node),
            tag("edge").map(|_| AttributeKind::Edge),
        ))),
        attr_list_parser,
    )(input)?;
    Ok((rest, AttributeStatement { kind, attributes }))
}

fn definition_statement_parser(input: &str) -> ParseResult<&str, DefinitionStatement> {
    let (rest, (lhs, rhs)) = separated_pair(ws(ident_parser), char('='), ws(ident_parser))(input)?;
    Ok((rest, DefinitionStatement { lhs, rhs }))
}

fn statement_parser(input: &str) -> ParseResult<&str, Statement> {
    ws(alt((
        edge_statement_parser.map(|s| Statement::Edge(s)),
        node_statement_parser.map(|s| Statement::Node(s)),
        definition_statement_parser.map(|s| Statement::Definition(s)),
        attribute_statement_parser.map(|s| Statement::Attribute(s)),
    )))(input)
}

fn statements_parser(input: &str) -> ParseResult<&str, Vec<Statement>> {
    many0(statement_parser)(input)
}

pub fn graph_parser(input: &str) -> ParseResult<&str, Graph> {
    let (rest, (strict, graph_kind, statements)) = tuple((
        ws(opt(tag("strict"))),
        ws(alt((
            tag("graph").map(|_| GraphKind::Undirected),
            tag("digraph").map(|_| GraphKind::Directed),
        ))),
        delimited(ws(char('{')), statements_parser, ws(char('}'))),
    ))(input)?;
    Ok((
        rest,
        Graph {
            kind: graph_kind,
            strict: strict.is_some(),
            statements,
        },
    ))
}

#[derive(Debug, Error)]
pub enum GraphParseError<I: Debug + Display> {
    #[error("unexpected eof")]
    UnexpectedEof,
    #[error("unexpected additional input: {0}")]
    UnexpectedInput(I),
    #[error("parse error: {0}")]
    ParseError(VerboseError<I>),
}

pub fn parse_graph<'a>(input: &'a str) -> Result<Graph, GraphParseError<&'a str>> {
    match graph_parser(input) {
        Ok((rest, graph)) => {
            if !rest.trim().is_empty() {
                Err(GraphParseError::UnexpectedInput(rest))
            } else {
                Ok(graph)
            }
        }
        Err(nom::Err::Incomplete(_)) => Err(GraphParseError::UnexpectedEof),
        Err(nom::Err::Error(e) | nom::Err::Failure(e)) => Err(GraphParseError::ParseError(e)),
    }
}

#[cfg(test)]
mod tests {
    use nom::error::ErrorKind;
    use std::fmt::{Debug, Display};

    use super::*;

    fn test_parse_result<I: Debug, O: Debug>(
        s: I,
        result: ParseResult<I, O>,
        f: impl Fn(I) -> ParseResult<I, O>,
    ) where
        ParseResult<I, O>: PartialEq<ParseResult<I, O>>,
    {
        assert_eq!(f(s), result);
    }

    fn test_parse_valid<I: Debug + Default, O: Debug>(
        s: I,
        o: O,
        f: impl Fn(I) -> ParseResult<I, O>,
    ) where
        ParseResult<I, O>: PartialEq<ParseResult<I, O>>,
    {
        test_parse_result(s, Ok((I::default(), o)), f);
    }

    fn test_parse_invalid<I: Display + Debug + Default + Copy, O: Debug>(
        s: I,
        error_kind: nom::error::ErrorKind,
        f: impl Fn(I) -> ParseResult<I, O>,
    ) {
        match f(s) {
            Ok(_) => panic!("expected to fail parsing of '{}', but succeeded", s),
            Err(nom::Err::Error(nom::error::VerboseError { errors })) => {
                assert!(errors.iter().any(|e| {
                    if let nom::error::VerboseErrorKind::Nom(kind) = e.1 {
                        kind == error_kind
                    } else {
                        false
                    }
                }))
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    fn valid_string_idents() -> Vec<&'static str> {
        vec![
            "Howdy_there",
            "this_is_valid",
            "_so_is_this",
            "foo",
            "an_actual_typical_value",
        ]
    }

    fn invalid_string_idents() -> Vec<(&'static str, ErrorKind)> {
        vec![("5cantstartwithnumber", nom::error::ErrorKind::Tag)]
    }

    fn expected_rest_string_idents() -> Vec<(&'static str, ParseResult<&'static str, Ident>)> {
        vec![("no-kebab-case", Ok(("-kebab-case", "no".to_owned())))]
    }

    fn valid_quoted_string_idents() -> Vec<(&'static str, &'static str)> {
        vec![
            (r#""foo""#, "foo"),
            (r#""foo\"bar""#, r#"foo"bar"#),
            (r#""this is a longer string""#, "this is a longer string"),
            (
                r#""lots\"of\"extra\"escaped\"quotes""#,
                r#"lots"of"extra"escaped"quotes"#,
            ),
            (r#""""#, ""),
        ]
    }

    // fn invalid_quote_string_idents() -> Vec<(&'static str, &'static str)> {
    //     vec![
    //         r#""missing ending quote"#
    //     ]
    // }

    #[test]
    fn string_ident() {
        for s in valid_string_idents() {
            test_parse_valid(s, s.to_owned(), string_ident_parser);
        }
        for (s, kind) in invalid_string_idents() {
            test_parse_invalid(s, kind, string_ident_parser);
        }
        for (s, result) in expected_rest_string_idents() {
            test_parse_result(s, result, string_ident_parser);
        }
    }

    #[test]
    fn quoted_ident() {
        for (i, o) in valid_quoted_string_idents() {
            test_parse_valid(i, o.into(), quote_string_ident_parser);
        }
        println!(
            "testing: {:?}",
            quote_string_ident_parser(r#""missing ending quote"#)
        );
    }

    #[test]
    fn any_ident() {
        for s in valid_string_idents() {
            test_parse_valid(s, s.to_owned(), ident_parser);
        }

        for (s, o) in valid_quoted_string_idents() {
            test_parse_valid(s, o.to_owned(), ident_parser);
        }
        for (s, kind) in invalid_string_idents() {
            test_parse_invalid(s, kind, ident_parser);
        }
        for (s, result) in expected_rest_string_idents() {
            test_parse_result(s, result, ident_parser);
        }
    }
}
