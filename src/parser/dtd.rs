use super::*;
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

// fn blank<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
//     i: &'a str,
// ) -> IResult<&'a str, &'a str, E> {}

// <!-- LOCALIZATION NOTE (securityOverride.warningContent) Lorem ipsum -->
// 0000
fn comment_tag<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Option<Node>, E> {
    map(
        tuple((
            tag("<!--"),            // 0
            opt(whitespace),        // 1
            opt(localization_note), // 2
            opt(whitespace),        // 3
            take_until("-->"),      // 4
            opt(whitespace),        // 5
            tag("-->"),             // 6
        )),
        |tuple| {
            Some(
                Comment {
                    key: tuple.2,
                    value: tuple.4.trim_end().into(),
                }
                .into(),
            )
        },
    )(i)
}

fn quoted_string<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, String, E> {
    map(
        tuple((
            char('\"'),
            opt(escaped(none_of("\\\""), '\\', one_of("\\\""))),
            char('\"'),
        )),
        |tuple: (char, Option<&str>, char)| {
            if let Some(string) = tuple.1 {
                string.replace("\\\"", "\"").replace("\\\\", "\\")
            } else {
                String::from("")
            }
        },
    )(i)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node<'a> {
    Entity(Entity<'a>),
    Comment(Comment<'a>),
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
    pub value: &'a str,
}

/// <!ENTITY ldb.visualDebugging.label "Visual Debugging">
///          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
fn entity_attributes<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Option<Node>, E> {
    map(
        tuple((entity_key, whitespace, quoted_string, tag(">"))),
        |tuple| {
            Some(
                Entity {
                    key: tuple.0,
                    value: tuple.2.to_string(),
                }
                .into(),
            )
        },
    )(i)
}

fn ascii_alphanumeric<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        "ascii_alphanumeric",
        take_while(|c: char| c.is_ascii_alphanumeric()),
    )(i)
}

// Discard the following information:
//
// <!ENTITY % brandDTD SYSTEM "chrome://branding/locale/brand.dtd" >
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//          12333333334555555677777777777777777777777777777777777789A
// %brandDTD;
// ^^^^^^^^^^
// BCCCCCCCCD
fn entity_percent_attribute<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Option<Node>, E> {
    value(
        None,
        context(
            "entity_percent_attribute",
            tuple((
                char('%'),          // 1
                whitespace,         // 2
                ascii_alphanumeric, // 3 brandDTD
                whitespace,         // 4,
                ascii_alphanumeric, // 5 SYSTEM
                whitespace,         // 6
                quoted_string,      // 7 "chrome://branding/locale/brand.dtd"
                opt(whitespace),    // 8
                char('>'),          // 9
                opt(whitespace),    // A
                char('%'),          // B
                ascii_alphanumeric, // C brandDTD
                char(';'),          // D
            )),
        ),
    )(i)
}

fn entity_tag<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Option<Node>, E> {
    map(
        context(
            "entity",
            tuple((
                tag("<!ENTITY"),
                whitespace,
                alt((entity_attributes, entity_percent_attribute)),
            )),
        ),
        |tuple| tuple.2,
    )(i)
}

pub fn dtd<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<Node>, E> {
    context(
        "dtd",
        fold_many0(
            tuple((opt(whitespace), alt((comment_tag, entity_tag)))),
            Vec::new(),
            |mut entities: Vec<_>, tuple| {
                if let Some(entity) = tuple.1 {
                    // Kind of a bad clone.
                    entities.push(entity);
                }
                entities
            },
        ),
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;

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

    #[test]
    fn test_comments() {
        let assert = |comment: Comment| {
            let result = parse!(comment_tag, comment.input);
            let parsed_comment = match result.1 {
                Some(Node::Comment(comment)) => comment,
                _ => panic!(""),
            };
            assert_eq!(
                comment,
                Comment {
                    input: comment.input,
                    comment: parsed_comment.value,
                    key: parsed_comment.key,
                    after: result.0,
                }
            );
        };
        let assert_err = |input| assert!(comment_tag::<()>(input).is_err());

        #[derive(Debug, PartialEq)]
        struct Comment<'s> {
            input: &'s str,
            comment: &'s str,
            key: Option<&'s str>,
            after: &'s str,
        }

        assert(Comment {
            input: "<!-- This is a comment -->",
            comment: "This is a comment",
            key: None,
            after: "",
        });
        assert(Comment {
            input: "<!-- This is a comment --> after",
            comment: "This is a comment",
            key: None,
            after: " after",
        });
        assert(Comment {
            input: "<!-- --> -->",
            comment: "",
            key: None,
            after: " -->",
        });
        assert(Comment {
            input: "<!-- --> after",
            comment: "",
            key: None,
            after: " after",
        });
        assert(Comment {
            input: "<!---->",
            comment: "",
            key: None,
            after: "",
        });
        assert(Comment {
            input: "<!----> after ",
            comment: "",
            key: None,
            after: " after ",
        });
        assert(Comment {
            input: "<!----> <!---->",
            comment: "",
            key: None,
            after: " <!---->",
        });
        assert(Comment {
            input: "<!-- LOCALIZATION NOTE (key.value) Comment -->",
            comment: "Comment",
            key: Some("key.value"),
            after: "",
        });

        assert_err("");
        assert_err("<!--");
        assert_err("-->");
        assert_err("<!-->");
    }

    #[test]
    #[rustfmt::skip]
    fn test_quoted_strings() {
        let assert = |text, value| assert_eq!(value, parse!(quoted_string, text).1);
        let assert_err = |text| quoted_string::<()>(text).unwrap_err();

        assert_err("\"no trailing");
        assert_err("no leading\"");
        assert("\"test\"", "test");
        assert("\"\"", "");
        assert("\"test \\\" escaped\"", "test \" escaped");
        assert("\"test \\\\ escaped\"", "test \\ escaped");
        assert("\"test \\\" escaped \\\" twice\"", "test \" escaped \" twice");
        assert_err("\"text with a newline\n");
    }

    #[test]
    fn test_entities() {
        assert_eq!(
            parse!(
                entity_tag,
                "<!ENTITY ldb.MainWindow.title \"Layout Debugger\">"
            )
            .1
            .unwrap(),
            Entity {
                key: "ldb.MainWindow.title".into(),
                value: "Layout Debugger".into(),
            }
            .into()
        );
        assert_eq!(
            parse!(
                entity_tag,
                "<!ENTITY performanceUI.toolbar.js-calltree \"Call Tree\">"
            )
            .1
            .unwrap(),
            Entity {
                key: "performanceUI.toolbar.js-calltree".into(),
                value: "Call Tree".into(),
            }
            .into()
        );
    }

    #[test]
    fn test_entity_percent() {
        let text =
            "<!ENTITY % brandDTD SYSTEM \"chrome://branding/locale/brand.dtd\" >\n%brandDTD;";
        assert_eq!(parse!(entity_tag, text).1, None);
    }

    #[test]
    fn test_dtd() {
        let (_, entities) = parse!(
            dtd,
            "
                <!-- preamble -->
                <!ENTITY ldb.MainWindow.title \"Layout Debugger\">

                <!ENTITY ldb.BackButton.label \"Back\">
                <!ENTITY ldb.ForwardButton.label \"Forward\">
                <!ENTITY ldb.ReloadButton.label \"Reload\">
                <!ENTITY ldb.StopButton.label \"Stop\">
                <!ENTITY ldb.StopButton.label2 \"Stop Again\">
            "
        );
        assert_eq!(
            entities,
            [
                Node::Comment(Comment {
                    value: "preamble",
                    key: None
                }),
                Node::Entity(Entity {
                    key: "ldb.MainWindow.title".into(),
                    value: "Layout Debugger".into(),
                }),
                Node::Entity(Entity {
                    key: "ldb.BackButton.label".into(),
                    value: "Back".into(),
                }),
                Node::Entity(Entity {
                    key: "ldb.ForwardButton.label".into(),
                    value: "Forward".into(),
                }),
                Node::Entity(Entity {
                    key: "ldb.ReloadButton.label".into(),
                    value: "Reload".into(),
                }),
                Node::Entity(Entity {
                    key: "ldb.StopButton.label".into(),
                    value: "Stop".into(),
                }),
                Node::Entity(Entity {
                    key: "ldb.StopButton.label2".into(),
                    value: "Stop Again".into(),
                }),
            ]
        );
    }
}
