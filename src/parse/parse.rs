extern crate nom;

use super::super::vm::instruction::Instruction;
use super::parts::parse_line;
use super::preprocess::preprocess_text;

use nom::multi::many1;

pub fn parse_text(i: &str) -> Result<Vec<Instruction>, String> {
    let text = preprocess_text(i);

    let parsed = many1(parse_line)(&text);
    match parsed {
        Ok(p) => Ok(p.1),
        Err(e) => return Err(e.to_string()),
    }
}
