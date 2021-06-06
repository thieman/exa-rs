extern crate nom;

use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take_while},
    character::complete::{line_ending, one_of, space0, space1},
    combinator::{map, map_res},
    sequence::tuple,
    IResult,
};

use super::super::vm::instruction::{Comparator, Instruction, Label, Target};

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

fn parse_label(i: &str) -> IResult<&str, Label> {
    map(take_while(|c: char| c.is_alphanumeric()), |s: &str| {
        s.to_ascii_lowercase()
    })(i)
}

fn parse_copy(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((
        tag_no_case("copy"),
        space1,
        parse_target,
        space1,
        parse_register_target,
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

fn parse_subi(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((
        tag_no_case("subi"),
        space1,
        parse_target,
        space1,
        parse_target,
        space1,
        parse_register_target,
    ))(i)?;
    Ok((t.0, Instruction::Subi(t.1 .2, t.1 .4, t.1 .6)))
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

fn parse_divi(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((
        tag_no_case("divi"),
        space1,
        parse_target,
        space1,
        parse_target,
        space1,
        parse_register_target,
    ))(i)?;
    Ok((t.0, Instruction::Divi(t.1 .2, t.1 .4, t.1 .6)))
}

fn parse_modi(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((
        tag_no_case("modi"),
        space1,
        parse_target,
        space1,
        parse_target,
        space1,
        parse_register_target,
    ))(i)?;
    Ok((t.0, Instruction::Modi(t.1 .2, t.1 .4, t.1 .6)))
}

fn parse_swiz(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((
        tag_no_case("swiz"),
        space1,
        parse_target,
        space1,
        parse_target,
        space1,
        parse_register_target,
    ))(i)?;
    Ok((t.0, Instruction::Swiz(t.1 .2, t.1 .4, t.1 .6)))
}

fn parse_mark(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("mark"), space1, parse_label))(i)?;
    Ok((t.0, Instruction::Mark(t.1 .2)))
}

fn parse_jump(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("jump"), space1, parse_label))(i)?;
    Ok((t.0, Instruction::Jump(t.1 .2)))
}

fn parse_tjmp(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("tjmp"), space1, parse_label))(i)?;
    Ok((t.0, Instruction::Tjmp(t.1 .2)))
}

fn parse_fjmp(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("fjmp"), space1, parse_label))(i)?;
    Ok((t.0, Instruction::Fjmp(t.1 .2)))
}

fn parse_comparator(i: &str) -> IResult<&str, Comparator> {
    map(one_of("=><"), |s: char| match s {
        '=' => Comparator::Equal,
        '>' => Comparator::GreaterThan,
        '<' => Comparator::LessThan,
        _ => panic!("unreachable"),
    })(i)
}

fn parse_test(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((
        tag_no_case("test"),
        space1,
        parse_target,
        space1,
        parse_comparator,
        space1,
        parse_target,
    ))(i)?;
    Ok((t.0, Instruction::Test(t.1 .2, t.1 .4, t.1 .6)))
}

fn parse_repl(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("repl"), space1, parse_label))(i)?;
    Ok((t.0, Instruction::Repl(t.1 .2)))
}

fn parse_halt(i: &str) -> IResult<&str, Instruction> {
    let t = tag_no_case("halt")(i)?;
    Ok((t.0, Instruction::Halt))
}

fn parse_kill(i: &str) -> IResult<&str, Instruction> {
    let t = tag_no_case("kill")(i)?;
    Ok((t.0, Instruction::Kill))
}

fn parse_host(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("kill"), space1, parse_register_target))(i)?;
    Ok((t.0, Instruction::Host(t.1 .2)))
}

fn parse_mode(i: &str) -> IResult<&str, Instruction> {
    let t = tag_no_case("mode")(i)?;
    Ok((t.0, Instruction::Mode))
}

fn parse_void_m(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("void"), space1, tag_no_case("m")))(i)?;
    Ok((t.0, Instruction::VoidM))
}

fn parse_test_mrd(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("test"), space1, tag_no_case("mrd")))(i)?;
    Ok((t.0, Instruction::TestMrd))
}

fn parse_make(i: &str) -> IResult<&str, Instruction> {
    let t = tag_no_case("make")(i)?;
    Ok((t.0, Instruction::Make))
}

fn parse_grab(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("grab"), space1, parse_target))(i)?;
    Ok((t.0, Instruction::Grab(t.1 .2)))
}

fn parse_file(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("file"), space1, parse_register_target))(i)?;
    Ok((t.0, Instruction::File(t.1 .2)))
}

fn parse_seek(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("seek"), space1, parse_target))(i)?;
    Ok((t.0, Instruction::Seek(t.1 .2)))
}

fn parse_void_f(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("void"), space1, tag_no_case("f")))(i)?;
    Ok((t.0, Instruction::VoidF))
}

fn parse_drop(i: &str) -> IResult<&str, Instruction> {
    let t = tag_no_case("drop")(i)?;
    Ok((t.0, Instruction::Drop))
}

fn parse_wipe(i: &str) -> IResult<&str, Instruction> {
    let t = tag_no_case("wipe")(i)?;
    Ok((t.0, Instruction::Wipe))
}

fn parse_test_eof(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((tag_no_case("test"), space1, tag_no_case("eof")))(i)?;
    Ok((t.0, Instruction::TestEof))
}

fn parse_noop(i: &str) -> IResult<&str, Instruction> {
    let t = tag_no_case("noop")(i)?;
    Ok((t.0, Instruction::Noop))
}

fn parse_rand(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((
        tag_no_case("rand"),
        space1,
        parse_target,
        space1,
        parse_target,
        space1,
        parse_register_target,
    ))(i)?;
    Ok((t.0, Instruction::Rand(t.1 .2, t.1 .4, t.1 .6)))
}

fn parse_wait(i: &str) -> IResult<&str, Instruction> {
    let t = tag_no_case("wait")(i)?;
    Ok((t.0, Instruction::Wait))
}

pub fn parse_line(i: &str) -> IResult<&str, Instruction> {
    let t = tuple((
        alt((
            // Broken down based on the categories in the wiki, but it's
            // not semantically meaningful
            parse_copy,
            alt((
                parse_addi, parse_subi, parse_muli, parse_divi, parse_modi, parse_swiz,
            )),
            alt((parse_mark, parse_jump, parse_tjmp, parse_fjmp)),
            parse_test,
            alt((parse_repl, parse_halt, parse_kill)),
            alt((parse_link, parse_host)),
            alt((parse_mode, parse_void_m, parse_test_mrd)),
            alt((
                parse_make,
                parse_grab,
                parse_file,
                parse_seek,
                parse_void_f,
                parse_drop,
                parse_wipe,
                parse_test_eof,
            )),
            alt((parse_noop, parse_rand)),
            parse_wait,
        )),
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
    fn test_label() {
        assert_eq!(
            parse_label("jumppoint 3"),
            Ok((" 3", String::from("jumppoint")))
        );

        assert_eq!(
            parse_label("label1withnumber whatever"),
            Ok((" whatever", String::from("label1withnumber")))
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

    #[test]
    fn test_mark() {
        assert_eq!(
            parse_mark("mark up1\n"),
            Ok(("\n", Instruction::Mark(String::from("up1"))))
        )
    }

    #[test]
    fn test_test() {
        assert_eq!(
            parse_test("test x = 3\n"),
            Ok((
                "\n",
                Instruction::Test(
                    Target::Register(String::from("x")),
                    Comparator::Equal,
                    Target::Literal(3)
                )
            )),
        );

        assert_eq!(
            parse_test("test x > t\n"),
            Ok((
                "\n",
                Instruction::Test(
                    Target::Register(String::from("x")),
                    Comparator::GreaterThan,
                    Target::Register(String::from("t")),
                )
            )),
        );

        assert_eq!(
            parse_test("test x < #nrv\n"),
            Ok((
                "\n",
                Instruction::Test(
                    Target::Register(String::from("x")),
                    Comparator::LessThan,
                    Target::Register(String::from("#nrv")),
                )
            )),
        );
    }
}
