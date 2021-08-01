pub mod dtd;
pub mod properties;

use nom::{
    //
    branch::*,
    bytes::complete::*,
    character::complete::*,
    combinator::*,
    error::*,
    multi::*,
    sequence::*,
    *,
};

fn single_line_whitespace<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("whitespace", take_while(|c| " \t".contains(c)))(i)
}

fn rest_of_line<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    map(
        tuple((
            alt((take_until("\n"), rest)), // 0
            opt(char('\n')),               // 1
        )),
        |tuple| tuple.0,
    )(i)
}

fn whitespace<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("whitespace", take_while(|c| " \t\r\n".contains(c)))(i)
}

fn message_key<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &str, E> {
    take_while(|c: char| c.is_ascii_alphanumeric() || '.' == c || '-' == c)(i)
}

fn localization_note<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    map(
        // Capture localization notes.
        tuple((
            tag("LOCALIZATION NOTE"),                   // 0
            opt(whitespace),                            // 1
            delimited(tag("("), message_key, tag(")")), // 2
            opt(whitespace),                            // 3
            opt(char(':')),                             // 4
        )),
        |tuple| tuple.2,
    )(i)
}

// TODO - De-duplicate.
macro_rules! parse {
    ($fn:ident, $text:expr) => {{
        let data: &str = $text;
        match $fn::<nom::error::VerboseError<&str>>(data) {
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                panic!(
                    "\n\nParsing Error:\n\n{}",
                    nom::error::convert_error(data, e)
                );
            }
            Err(nom::Err::Incomplete(needed)) => {
                panic!("Incomplete, needed: {:?}", needed);
            }
            Ok(value) => value,
        }
    }};
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node<'a> {
    Message(Message<'a>),
    Comment(Comment<'a>),
}

impl<'a> From<Message<'a>> for Node<'a> {
    fn from(other: Message<'a>) -> Self {
        Node::Message(other)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Message<'a> {
    pub key: &'a str,
    pub value: String,
}

impl<'a> From<Comment<'a>> for Node<'a> {
    fn from(other: Comment<'a>) -> Self {
        Node::Comment(other)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Comment<'a> {
    pub key: Option<&'a str>,
    pub value: String,
}
