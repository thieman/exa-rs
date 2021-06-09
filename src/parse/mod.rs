extern crate nom;

mod parts;
mod preprocess;

use super::vm::instruction::Instruction;
use parts::parse_line;
use preprocess::preprocess_text;

use nom::multi::many1;

pub fn parse_text(i: &str) -> Result<Vec<Instruction>, String> {
    let text = preprocess_text(i);

    let parsed = many1(parse_line)(&text);
    match parsed {
        Ok(p) => Ok(p.1),
        Err(e) => return Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::super::vm::instruction::Target;
    use super::*;

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
}
