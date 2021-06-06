extern crate nom;

use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take_while},
    character::complete::{alpha1, line_ending, multispace0, space0},
    combinator::{map, map_res},
    multi::many_till,
    sequence::terminated,
    IResult,
};

use super::super::vm::instruction::{Instruction, Target};

fn is_digit_or_sign(c: char) -> bool {
    c.is_digit(10) || c == '+' || c == '-'
}

fn is_alpha_or_hash(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '#'
}

fn to_i32(i: &str) -> Result<i32, std::num::ParseIntError> {
    i.parse::<i32>()
}

fn parse_literal(i: &str) -> IResult<&str, i32> {
    map_res(take_while(is_digit_or_sign), to_i32)(i)
}

fn parse_register(i: &str) -> IResult<&str, String> {
    map(take_while(is_alpha_or_hash), |s: &str| {
        s.to_ascii_lowercase()
    })(i)
}

fn parse_instruction(i: &str) -> IResult<&str, String> {
    terminated(
        map(
            alt((
                tag_no_case("void m"),
                tag_no_case("void f"),
                tag_no_case("test mrd"),
                tag_no_case("test eof"),
                alpha1,
            )),
            |s: &str| s.to_ascii_lowercase(),
        ),
        space0,
    )(i)
}

fn parse_target(i: &str) -> IResult<&str, Target> {
    if let Ok(parsed) = parse_literal(i) {
        return Ok((parsed.0, Target::Literal(parsed.1)));
    }

    match parse_register(i) {
        Ok(parsed) => Ok((parsed.0, Target::Register(parsed.1))),
        Err(error) => Err(error),
    }
}

fn parse_line(i: &str) -> IResult<&str, Instruction> {
    let p = parse_instruction(i)?;
    let mut rest = p.0;

    // most instructions just take targets, we deal with those now
    let (r, ts) = many_till(terminated(parse_target, space0), line_ending)(rest)?;
    rest = r;
    let mut targets = ts.0;

    let i = match p.1.as_str() {
        "link" => {
            assert!(targets.len() > 0);
            Instruction::Link(targets.remove(0))
        }
        _ => Instruction::NotFound(),
    };

    rest = multispace0(rest)?.0;
    Ok((rest, i))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_literal() {
        assert_eq!(parse_literal("302 bacon"), Ok((" bacon", 302)));
        assert_eq!(parse_literal("-32 fwef"), Ok((" fwef", -32)));
        // overflow errors
        assert!(parse_literal("10000000000 ohno").is_err());
        assert_eq!(parse_literal("2897bacon"), Ok(("bacon", 2897)));
        // nothing to parse
        assert!(parse_literal("something 123 oh no").is_err());
        // sign shenanigans are an error
        assert!(parse_literal("+-123").is_err());
        assert!(parse_literal("++123").is_err());
        assert!(parse_literal("--123").is_err());
    }

    #[test]
    fn test_register() {
        assert_eq!(
            parse_register("#NRV reg"),
            Ok((" reg", String::from("#nrv")))
        );
        assert_eq!(parse_register("X 123"), Ok((" 123", String::from("x"))));
    }

    #[test]
    fn test_target() {
        assert_eq!(parse_target("329 ok"), Ok((" ok", Target::Literal(329))));
        assert_eq!(parse_target("-1 ok"), Ok((" ok", Target::Literal(-1))));
        assert_eq!(parse_target("+9999 ok"), Ok((" ok", Target::Literal(9999))));

        assert_eq!(
            parse_target("#NRV ok"),
            Ok((" ok", Target::Register(String::from("#nrv")))),
        );
        assert_eq!(
            parse_target("x ok"),
            Ok((" ok", Target::Register(String::from("x")))),
        );
    }

    #[test]
    fn test_instruction() {
        assert_eq!(parse_instruction("link 4"), Ok(("4", String::from("link"))));
        assert_eq!(
            parse_instruction("TEST MRD 4"),
            Ok(("4", String::from("test mrd")))
        );
        assert_eq!(
            parse_instruction("void M 4"),
            Ok(("4", String::from("void m")))
        );
        assert_eq!(
            parse_instruction("teST eof 4"),
            Ok(("4", String::from("test eof")))
        );
        assert_eq!(
            parse_instruction("void f 4"),
            Ok(("4", String::from("void f")))
        );
    }

    #[test]
    fn test_link() {
        assert_eq!(
            parse_line("LINK -1\nok"),
            Ok(("ok", Instruction::Link(Target::Literal(-1))))
        );
        assert_eq!(
            parse_line("LINK #NRV\nok"),
            Ok((
                "ok",
                Instruction::Link(Target::Register(String::from("#nrv")))
            ))
        );
        assert_eq!(
            parse_line("LINK 999   \nok"),
            Ok(("ok", Instruction::Link(Target::Literal(999))))
        );
    }
}
