extern crate nom;

use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take_while},
    character::complete::{line_ending, space0, space1},
    combinator::{map, map_res},
    sequence::tuple,
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

fn parse_register_target(i: &str) -> IResult<&str, Target> {
    match parse_register(i) {
        Ok(parsed) => Ok((parsed.0, Target::Register(parsed.1))),
        Err(error) => Err(error),
    }
}

fn parse_target(i: &str) -> IResult<&str, Target> {
    if let Ok(parsed) = parse_literal(i) {
        return Ok((parsed.0, Target::Literal(parsed.1)));
    }

    parse_register_target(i)
}

fn parse_copy(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((
        tag_no_case("copy"),
        space1,
        parse_target,
        space1,
        parse_target,
    ))(i)?;
    Ok((t.0, Instruction::Copy(t.1 .2, t.1 .4)))
}

fn parse_link(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("link"), space1, parse_target))(i)?;
    Ok((t.0, Instruction::Link(t.1 .2)))
}

fn parse_addi(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((
        tag_no_case("addi"),
        space1,
        parse_target,
        space1,
        parse_target,
        space1,
        parse_register_target,
    ))(i)?;
    Ok((t.0, Instruction::Addi(t.1 .2, t.1 .4, t.1 .6)))
}

fn parse_muli(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((
        tag_no_case("muli"),
        space1,
        parse_target,
        space1,
        parse_target,
        space1,
        parse_register_target,
    ))(i)?;
    Ok((t.0, Instruction::Muli(t.1 .2, t.1 .4, t.1 .6)))
}

pub fn parse_line(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((
        alt((parse_link, parse_copy, parse_addi, parse_muli)),
        space0,
        line_ending,
    ))(i)?;
    Ok((t.0, t.1 .0))
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

    #[test]
    fn test_copy() {
        assert_eq!(
            parse_line("copy 5887 x\nlink 1"),
            Ok((
                "link 1",
                Instruction::Copy(Target::Literal(5887), Target::Register(String::from("x")))
            )),
        )
    }
}
