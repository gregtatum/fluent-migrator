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

impl<'a> From<Message<'a>> for Node<'a> {
    fn from(other: Message<'a>) -> Self {
        Node::Message(other)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Message<'a> {
    pub key: &'a str,
    pub value: &'a str,
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
    Message(Message<'a>),
    Comment(Comment<'a>),
}

fn message_key<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &str, E> {
    take_while(|c: char| c.is_ascii_alphanumeric() || '.' == c || '-' == c)(i)
}

fn message<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Message, E> {
    map(
        tuple((
            opt(single_line_whitespace), // 0
            message_key,                 // 1
            opt(single_line_whitespace), // 2
            char('='),                   // 3
            rest_of_line,                // 4
        )),
        |tuple| Message {
            key: tuple.1,
            value: tuple.4.trim(),
        },
    )(i)
}

#[test]
fn test_message() {
    #[derive(Debug, PartialEq)]
    struct Test<'a> {
        input: &'a str,
        comment: Message<'a>,
    }

    let assert = |expected: Test| {
        let actual = Test {
            input: expected.input,
            comment: parse!(message, expected.input).1,
        };
        assert_eq!(expected, actual);
    };

    assert(Test {
        input: "heapview.field.name=Group",
        comment: Message {
            key: "heapview.field.name",
            value: "Group",
        },
    });
    assert(Test {
        input: "heapview.field.name = Group",
        comment: Message {
            key: "heapview.field.name",
            value: "Group",
        },
    });
    assert(Test {
        input: "key-prop = This is a long message \n# Comment",
        comment: Message {
            key: "key-prop",
            value: "This is a long message",
        },
    });
    assert(Test {
        input: "    key   =   value    ",
        comment: Message {
            key: "key",
            value: "value",
        },
    });
}

fn comment<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Comment, E> {
    fold_many1(
        map(
            tuple((
                opt(single_line_whitespace), // 0
                tag("#"),                    // 1
                opt(single_line_whitespace), // 2
                opt(localization_note),      // 3
                rest_of_line,                // 4
            )),
            |tuple| (tuple.3, tuple.4.trim()),
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
fn test_comment() {
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
        input: "# comment 1\n\n# comment 2\n",
        comment: Comment {
            key: None,
            value: "comment 1".into(),
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
    // alt((
    //     map(message, |message| Node::Message(message)),
    //     map(comment, |comment| Node::Comment(comment)),
    // ))(i)

    context(
        "properties",
        fold_many0(
            tuple((
                opt(whitespace), // 0
                alt((
                    map(comment, |comment| Node::Comment(comment)),
                    map(message, |message| Node::Message(message)),
                )), // 1
            )),
            Vec::new(),
            |mut nodes: Vec<_>, tuple| {
                nodes.push(tuple.1);
                nodes
            },
        ),
    )(i)
}

#[test]
fn test_properties() {
    let text = "# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# LOCALIZATION NOTE (snapshot.io.save): The label for the link that saves a
# snapshot to disk.
snapshot.io.save=Save

# LOCALIZATION NOTE (snapshot.io.delete): The label for the link that deletes
# a snapshot
snapshot.io.delete=Delete

# LOCALIZATION NOTE (snapshot.io.save.window): The title for the window
# displayed when saving a snapshot to disk.
snapshot.io.save.window=Save Snapshot
    ";
    let (rest_text, nodes) = parse!(properties, text);

    assert_eq!(nodes, [
        Comment {
            key: None,
            value: "This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/.".into(),
        }.into(),
        Comment {
            key: Some("snapshot.io.save"),
            value: "The label for the link that saves a snapshot to disk.".into(),
        }.into(),
        Message {
            key: "snapshot.io.save",
            value: "Save".into(),
        }.into(),
        Comment {
            key: Some("snapshot.io.delete"),
            value: "The label for the link that deletes a snapshot".into(),
        }.into(),
        Message {
            key: "snapshot.io.delete",
            value: "Delete",
        }.into(),
        Comment {
            key: Some("snapshot.io.save.window"),
            value: "The title for the window displayed when saving a snapshot to disk.".into(),
        }.into(),
        Message {
            key: "snapshot.io.save.window",
            value: "Save Snapshot",
        }.into()
    ]);
}
