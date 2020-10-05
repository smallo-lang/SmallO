#![allow(dead_code)]

pub type AST = Vec<Statement>;

pub enum Statement {
    Include(Path),
    Label(Name),
    Instruction(Opcode, Operand)
}

pub type Path = String;

#[derive(PartialEq, Debug, Clone)]
pub enum Atom {
    Int(i64),
    Str(String),
    Name(Name),
}

pub type Name = String;

pub type Opcode = String;

pub type Operand = Vec<Atom>;
