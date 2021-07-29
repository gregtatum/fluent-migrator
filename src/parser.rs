use nom::*;
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while},
    character::complete::{alphanumeric1 as alphanumeric, char, one_of},
    combinator::{cut, map, opt, value},
    error::{context, convert_error, ContextError, ErrorKind, ParseError, VerboseError},
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, preceded, separated_pair, terminated},
    Err, IResult,
};

named!(
  whitespace<&str, ()>,
  value!((), take_while!(|c| " \t\r\n".contains(c)))
);

named!(
    comment_tag<&str, ()>,
    value!(
      (), // Discard the comment.
      tuple!(tag!("<!--"), take_until!("-->"), tag!("-->"))
    )
);

named!(
    quoted_string<&str, String>,
    map!(
        tuple!(
            char!('\"'),
            escaped!(
                none_of!("\\\""),
                '\\',
                one_of!("\\\"")
            ),
            char!('\"')
        ),
        |tuple| { tuple.1.replace("\\\"", "\"").replace("\\\\", "\\") }
    )
);

#[derive(Debug, PartialEq)]
struct Entity {
    pub key: String,
    pub value: String,
}

named!(
    entity_tag<&str, Entity>,
    map!(
        tuple!(
            tag!("<!ENTITY"),
            whitespace,
            take_while!(|c: char| c.is_ascii_alphabetic() || '.' == c),
            whitespace,
            quoted_string,
            tag!(">")
        ),
        |tuple| Entity {
            key: tuple.2.to_string(),
            value: tuple.4,
        }
    )
);

named!(
    dtd<&str, Vec<Option<Entity>>>,
    separated_list0!(
        whitespace,
        alt!(
            map!(comment_tag, |_| None) |
            map!(entity_tag, |entity| Some(entity))
        )
    )
);

/// parser combinators are constructed from the bottom up:
/// first we write parsers for the smallest elements (here a space character),
/// then we'll combine them in larger parsers
fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";

    // nom combinators like `take_while` return a function. That function is the
    // parser,to which we can pass the input
    take_while(move |c| chars.contains(c))(i)
}

fn root<'a, E: error::ParseError<&'a str> + error::ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<String>, E> {
    error::context("root", multi::separated_list0(sp, quoted_string))(i)
}

pub fn parse_dtd(input: &str) {
    let result = comment_tag(input).unwrap();
    println!("{:?}", result);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_comments() {
        let assert = |text, after| assert_eq!(Ok((after, ())), comment_tag(text));

        assert("<!-- This is a comment -->",       "");
        assert("<!-- This is a comment --> after", " after");
        assert("<!-- --> -->",                    " -->");
        assert("<!-- --> after",             " after");
        assert("<!---->",                          "");
    }

    #[test]
    #[rustfmt::skip]
    fn test_quoted_strings() {
        let assert = |text, value| assert_eq!(value, quoted_string(text).unwrap().1);
        let assert_err = |text| quoted_string(text).unwrap_err();

        assert_err("\"no trailing");
        assert_err("no leading\"");
        assert("\"test\"", "test");
        assert("\"test \\\" escaped\"", "test \" escaped");
        assert("\"test \\\\ escaped\"", "test \\ escaped");
        assert("\"test \\\" escaped \\\" twice\"", "test \" escaped \" twice");
    }

    #[test]
    fn test_entities() {
        let entity = entity_tag("<!ENTITY ldb.MainWindow.title \"Layout Debugger\">").unwrap();
        assert_eq!(
            entity.1,
            Entity {
                key: "ldb.MainWindow.title".into(),
                value: "Layout Debugger".into(),
            },
        );
    }

    #[test]
    fn test_root() {
        let data = "\"string\"";
        match root::<error::VerboseError<&str>>(data) {
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                println!(
                    "verbose errors - `root::<VerboseError>(data)`:\n{}",
                    error::convert_error(data, e)
                );
            }
            _ => {}
        }
    }

    #[test]
    fn test_dtd() {
        dtd("<!-- file, You can obtain one at http://mozilla.org/MPL/2.0/. -->").unwrap();
    }
}
