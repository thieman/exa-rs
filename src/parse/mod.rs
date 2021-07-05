extern crate nom;

mod parts;
mod preprocess;

use super::vm::instruction::{Instruction, Target};
use parts::parse_line;
use preprocess::preprocess_text;

use nom::multi::many0;

pub fn parse_text(i: &str) -> Result<Vec<Instruction>, String> {
    let text = preprocess_text(i);

    let parsed = many0(parse_line)(&text);
    let insts = match parsed {
        Ok(p) => p.1,
        Err(e) => return Err(e.to_string()),
    };

    validate_instructions(&insts)?;

    Ok(insts)
}

fn validate_instructions(insts: &Vec<Instruction>) -> Result<(), String> {
    for i in insts.iter() {
        match i {
            Instruction::Copy(a, b) => validate_targets(&[a, b])?,
            Instruction::Addi(a, b, c) => validate_targets(&[a, b, c])?,
            Instruction::Subi(a, b, c) => validate_targets(&[a, b, c])?,
            Instruction::Muli(a, b, c) => validate_targets(&[a, b, c])?,
            Instruction::Divi(a, b, c) => validate_targets(&[a, b, c])?,
            Instruction::Modi(a, b, c) => validate_targets(&[a, b, c])?,
            Instruction::Swiz(a, b, c) => validate_targets(&[a, b, c])?,
            Instruction::Test(a, _comp, b) => validate_targets(&[a, b])?,
            Instruction::Link(a) => validate_targets(&[a])?,
            Instruction::Host(a) => validate_targets(&[a])?,
            Instruction::Grab(a) => validate_targets(&[a])?,
            Instruction::File(a) => validate_targets(&[a])?,
            Instruction::Seek(a) => validate_targets(&[a])?,
            Instruction::Rand(a, b, c) => validate_targets(&[a, b, c])?,
            _ => (),
        }
    }

    Ok(())
}

fn validate_targets(ts: &[&Target]) -> Result<(), String> {
    let mut found_ms = 0;
    for t in ts.iter() {
        match t {
            Target::Literal(value) => {
                if *value < -9999 || *value > 9999 {
                    return Err("literal out of range".into());
                }
            }
            Target::Register(specifier) => {
                if specifier == "m" {
                    found_ms += 1;
                }
            }
        }
    }
    if found_ms > 1 {
        return Err("cannot reference M register more than once in one instruction".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::vm::instruction::Target;
    use super::*;

    #[test]
    fn test_parse_empty() {
        assert_eq!(parse_text(""), Ok(vec![]),);
    }

    #[test]
    fn test_parse_text() {
        let s = "LINK 800
        copy   1    x \t

@rep 2
 addi @{-5,-4} 1 x ; comment
     @end
muli 1 0 #nrv
note we groovin";
        assert_eq!(
            parse_text(s),
            Ok(vec![
                Instruction::Link(Target::Literal(800)),
                Instruction::Copy(Target::Literal(1), Target::Register(String::from("x"))),
                Instruction::Addi(
                    Target::Literal(-5),
                    Target::Literal(1),
                    Target::Register(String::from("x"))
                ),
                Instruction::Addi(
                    Target::Literal(-9),
                    Target::Literal(1),
                    Target::Register(String::from("x"))
                ),
                Instruction::Muli(
                    Target::Literal(1),
                    Target::Literal(0),
                    Target::Register(String::from("#nrv"))
                ),
            ])
        );
    }

    #[test]
    fn test_literal_bounds() {
        let s = "addi -9999 9999 x\n";
        assert_eq!(
            parse_text(s),
            Ok(vec![Instruction::Addi(
                Target::Literal(-9999),
                Target::Literal(9999),
                Target::Register("x".into()),
            )])
        );

        let s = "copy 10000 x\n";
        assert_eq!(parse_text(s), Err("literal out of range".into()),);
    }

    #[test]
    fn test_m_limit() {
        let s = "copy 1 m\n";
        assert_eq!(
            parse_text(s),
            Ok(vec![Instruction::Copy(
                Target::Literal(1),
                Target::Register("m".into()),
            )])
        );

        let s = "copy m m\n";
        assert_eq!(
            parse_text(s),
            Err("cannot reference M register more than once in one instruction".into()),
        );
    }

    #[test]
    fn test_mrd() {
        let s = "test mrd\n noop\n";
        assert_eq!(
            parse_text(s),
            Ok(vec![Instruction::TestMrd, Instruction::Noop])
        );
    }

    #[test]
    fn test_bad_label_parse() {
        let s = "MODE\n ; INIT STATE\n DATA 0 0 0 0 0 0 0 0\n DATA 0 0 0 0 0 0 0 0\n \n ; INIT 2 RANDOMS\n RAND 0 15 X\n \n SEEK X\n COPY 1 F\n \n MARK INITLOOP\n SEEK -9999\n RAND 0 15 X\n SEEK X\n TEST F = 0\n FJMP INITLOOP\n SEEK -1\n COPY 1 F\n \n ; RENDER BOARD STATE\n COPY 0 X\n SEEK -9999\n \n MARK RENDER\n COPY F T\n REPL SPRITE\n ADDI X 1 X\n TEST EOF\n FJMP RENDER\n COPY 0 X \n JUMP WAIT\n \n MARK SPRITE\n LINK 801\n COPY T CO\n TEST T = 0\n TJMP BLANKSPRITE\n ADDI 327 CO GP\n MARK BLANKSPRITE\n COPY CO T\n MODI X 4 CO\n MULI 25 CO T\n ADDI 5 T GX\n \n DIVI X 4 CO\n MULI 25 CO T\n ADDI 5 T GY\n \n MARK FOREVER\n WAIT\n JUMP FOREVER\n \n MARK WAIT\n DROP\n VOID M\n REPL KILLER\n @REP 17\n COPY T T\n @END\n GRAB 400\n JUMP RENDER\n \n MARK KILLER\n LINK 801\n @REP 16\n KILL\n @END\n HALT\n";
        let parsed = parse_text(s).unwrap();
        for i in parsed.iter() {
            match i {
                Instruction::Mark(ref label) => {
                    if label == "killer" {
                        return;
                    }
                }
                _ => (),
            }
        }
        println!("{:?}", parsed);
        assert!(false, "killer mark not found");
    }

    #[test]
    fn test_hardware_reg() {
        let s = "link 801\n copy 60 #sqr0\n mark a\n wait\n jump a\n";
        let parsed = parse_text(s).unwrap();
        assert_eq!(
            parsed,
            vec![
                Instruction::Link(Target::Literal(801)),
                Instruction::Copy(Target::Literal(60), Target::Register("#sqr0".into())),
                Instruction::Mark("a".into()),
                Instruction::Wait,
                Instruction::Jump("a".into()),
            ]
        )
    }
}
