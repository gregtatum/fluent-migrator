use nom::*;
use nom::{
    //
    branch::*,
    bytes::complete::*,
    character::complete::*,
    combinator::*,
    error::*,
    multi::*,
    sequence::*,
};

// fn blank<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
//     i: &'a str,
// ) -> IResult<&'a str, &'a str, E> {}

fn whitespace<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("whitespace", take_while(|c| " \t\r\n".contains(c)))(i)
}

fn comment_tag<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, (), E> {
    value((), tuple((tag("<!--"), take_until("-->"), tag("-->"))))(i)
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
pub struct Entity {
    pub key: String,
    pub value: String,
}

/// <!ENTITY ldb.visualDebugging.label "Visual Debugging">
///          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
fn entity_attributes<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Option<Entity>, E> {
    map(
        tuple((
            take_while(|c: char| c.is_ascii_alphanumeric() || '.' == c || '-' == c),
            whitespace,
            quoted_string,
            tag(">"),
        )),
        |tuple| {
            Some(Entity {
                key: tuple.0.to_string(),
                value: tuple.2.to_string(),
            })
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
) -> IResult<&'a str, Option<Entity>, E> {
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
) -> IResult<&'a str, Option<Entity>, E> {
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
) -> IResult<&'a str, Vec<Entity>, E> {
    context(
        "dtd",
        fold_many0(
            tuple((
                opt(whitespace),
                alt((
                    map(comment_tag, |_| None),
                    map(entity_tag, |entity| Some(entity)),
                )),
            )),
            Vec::new(),
            |mut entities: Vec<_>, tuple| {
                if let Some(Some(entity)) = tuple.1 {
                    // Kind of a bad clone.
                    entities.push(entity);
                }
                entities
            },
        ),
    )(i)
}

macro_rules! parse {
    ($fn:tt, $text:expr) => {{
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_comments() {
        let assert = |text, after| assert_eq!((after, ()), parse!(comment_tag, text));
        let assert_err = |text| assert!(comment_tag::<()>(text).is_err());

        assert("<!-- This is a comment -->",       "");
        assert("<!-- This is a comment --> after", " after");
        assert("<!-- --> -->",                     " -->");
        assert("<!-- --> after",                   " after");
        assert("<!---->",                          "");
        assert("<!----> after ",                   " after ");
        assert("<!----> <!---->",                  " <!---->");

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
            .1,
            Some(Entity {
                key: "ldb.MainWindow.title".into(),
                value: "Layout Debugger".into(),
            }),
        );
        assert_eq!(
            parse!(
                entity_tag,
                "<!ENTITY performanceUI.toolbar.js-calltree \"Call Tree\">"
            )
            .1,
            Some(Entity {
                key: "performanceUI.toolbar.js-calltree".into(),
                value: "Call Tree".into(),
            }),
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
                Entity {
                    key: "ldb.MainWindow.title".into(),
                    value: "Layout Debugger".into()
                },
                Entity {
                    key: "ldb.BackButton.label".into(),
                    value: "Back".into()
                },
                Entity {
                    key: "ldb.ForwardButton.label".into(),
                    value: "Forward".into()
                },
                Entity {
                    key: "ldb.ReloadButton.label".into(),
                    value: "Reload".into()
                },
                Entity {
                    key: "ldb.StopButton.label".into(),
                    value: "Stop".into()
                },
                Entity {
                    key: "ldb.StopButton.label2".into(),
                    value: "Stop Again".into()
                }
            ]
        );
    }
}
