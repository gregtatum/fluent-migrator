#![allow(unused_variables, dead_code)]

use super::*;
use crate::*;
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

// TODO - De-duplicate.
#[cfg(test)]
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

impl<'a> From<Entity<'a>> for Node<'a> {
    fn from(other: Entity<'a>) -> Self {
        Node::Entity(other)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Entity<'a> {
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

#[derive(Debug, PartialEq, Clone)]
pub enum Node<'a> {
    Entity(Entity<'a>),
    Comment(Comment<'a>),
}

fn comment<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Comment, E> {
    fold_many0(
        map(
            tuple((
                opt(whitespace),               // 0
                tag("#"),                      // 1
                opt(single_line_whitespace),   // 2
                opt(localization_note),        // 3
                opt(single_line_whitespace),   // 4
                alt((take_until("\n"), rest)), // 5
            )),
            |tuple| (tuple.3, tuple.5.trim()),
        ),
        Comment {
            key: None,
            value: String::new(),
        },
        |mut comment, tuple| {
            if !comment.value.is_empty() {
                comment.value.push_str(" ");
            }
            if let Some(key) = tuple.0 {
                comment.key = Some(key);
            }
            comment.value.push_str(tuple.1);
            comment
        },
    )(i)
}

#[test]
fn test_comment_tag2() {
    #[derive(Debug, PartialEq)]
    struct Test<'a> {
        input: &'a str,
        comment: Comment<'a>,
    }

    let assert = |expected: Test| {
        let actual = Test {
            input: expected.input,
            comment: parse!(comment, expected.input).1,
        };
        assert_eq!(expected, actual);
    };

    assert(Test {
        input: "# multi\n# line\n",
        comment: Comment {
            key: None,
            value: "multi line".into(),
        },
    });
    assert(Test {
        input: "# Comment",
        comment: Comment {
            key: None,
            value: "Comment".into(),
        },
    });
    assert(Test {
        input: "#   Whitespace  ",
        comment: Comment {
            key: None,
            value: "Whitespace".into(),
        },
    });
    assert(Test {
        input: "# LOCALIZATION NOTE (key.value) Comment",
        comment: Comment {
            key: Some("key.value"),
            value: "Comment".into(),
        },
    });
    assert(Test {
        input: "# LOCALIZATION NOTE (key.value) Multi-line\n# comment.",
        comment: Comment {
            key: Some("key.value"),
            value: "Multi-line comment.".into(),
        },
    });
}

pub fn properties<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<Node>, E> {
    context(
        "properties",
        fold_many0(
            tuple((
                opt(whitespace),                                // 0
                map(comment, |comment| Node::Comment(comment)), // 1
            )),
            Vec::new(),
            |mut entities: Vec<_>, tuple| {
                entities.push(tuple.1);
                entities
            },
        ),
    )(i)
}
