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
    context("whitespace", take_while(|c| " \t\r\n".contains(c)))(i)
}

fn whitespace<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("whitespace", take_while(|c| " \t\r\n".contains(c)))(i)
}

fn entity_key<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
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
            tag("LOCALIZATION NOTE"),                  // 0
            opt(whitespace),                           // 1
            delimited(tag("("), entity_key, tag(")")), // 2
            opt(whitespace),                           // 3
            opt(char(':')),                            // 4
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
