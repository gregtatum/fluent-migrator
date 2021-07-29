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
            escaped(none_of("\\\""), '\\', one_of("\\\"")),
            char('\"'),
        )),
        |tuple: (char, &str, char)| tuple.1.replace("\\\"", "\"").replace("\\\\", "\\"),
    )(i)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Entity {
    pub key: String,
    pub value: String,
}

fn entity_tag<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Entity, E> {
    map(
        context(
            "entity",
            tuple((
                tag("<!ENTITY"),
                whitespace,
                take_while(|c: char| c.is_ascii_alphanumeric() || '.' == c),
                whitespace,
                quoted_string,
                tag(">"),
            )),
        ),
        |tuple| Entity {
            key: tuple.2.to_string(),
            value: tuple.4,
        },
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
                if let Some(entity) = tuple.1 {
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
        assert("\"test \\\" escaped\"", "test \" escaped");
        assert("\"test \\\\ escaped\"", "test \\ escaped");
        assert("\"test \\\" escaped \\\" twice\"", "test \" escaped \" twice");
        assert_err("\"text with a newline\n");
    }

    #[test]
    fn test_entities() {
        let entity =
            entity_tag::<()>("<!ENTITY ldb.MainWindow.title \"Layout Debugger\">").unwrap();
        assert_eq!(
            entity.1,
            Entity {
                key: "ldb.MainWindow.title".into(),
                value: "Layout Debugger".into(),
            },
        );
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
