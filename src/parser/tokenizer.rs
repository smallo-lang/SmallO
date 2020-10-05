#![allow(dead_code)]

use super::input_stream::InputStream;
use crate::ast::*;
use std::collections::HashSet;
use std::io::Read;


#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    Atom(Atom),
    Keyword(Opcode),
    Path(Path),
    Punc(u8),
}

type Check = fn(&u8) -> bool;

pub struct Tokenizer<R> {
    stream: InputStream<R>,
    keywords: HashSet<&'static str>,
    last: Option<Token>,
    buf: Vec<u8>,
}

impl<R: Read> Tokenizer<R> {
    pub fn new(stream: InputStream<R>) -> Self {
        let opcodes: HashSet<&'static str> = ["put", "add", "sub", "mul", "div",
            "mod", "gth", "lth", "geq", "leq", "eq", "neq", "ini", "ins", "out", "outl",
            "nl", "con", "sti", "not", "and", "or", "jump", "jmpt", "jmpf", "br", "brt",
            "brf", "back", "err", "end"]
            .iter()
            .map(|op| *op)
            .collect();

        Self {
            stream,
            keywords: opcodes,
            last: None,
            buf: vec![],
        }
    }

    /// We don't care about whitespace here. Reading until EOF is reached.
    pub fn next(&mut self) -> Option<Token> {
        self.read_while(u8::is_ascii_whitespace);
        if self.stream.eof() { return None; }
        let ch = self.stream.peek().unwrap();

        // Whitespace is skipped and EOF is still not there. Let's peek and see what this is.
        // It can be an Atom, Keyword, Path, or Punctuation.
        // Atom::Name
        if ch == b'_' || ch.is_ascii_alphabetic() {
            return self.read_name();
        }

        None
    }

    fn consume(&mut self, tok: Option<Token>) -> Option<Token> {
        self.last = tok.clone();
        tok
    }

    fn is_name(byte: &u8) -> bool {
        byte.is_ascii_alphanumeric() || *byte == b'_'
    }

    fn buf_match(&mut self, byte: Option<u8>) -> Option<u8> {
        match byte {
            None => (),
            Some(b) => self.buf.push(b),
        }
        byte
    }

    fn read_if(&mut self, byte: Option<u8>, check: Check) -> Option<u8> {
        let option = if let Some(b) = byte {
            if check(&b) {
                byte
            } else {
                None
            }
        } else {
            None
        };
        self.buf_match(option)
    }

    fn read_while(&mut self, check: Check) -> String {
        self.read_if(self.stream.peek(), check);
        loop {
            let next = self.stream.next();
            if let None = self.read_if(next, check) { break; }
        }
        let buf = String::from_utf8_lossy(&*self.buf).into();
        self.buf.clear();
        buf
    }

    fn read_until(&mut self, check: Check, end: Check) -> Option<String> {
        let read = self.read_while(check);
        if self.stream.eof() || end(&self.stream.peek().unwrap()) {
            Some(read)
        } else {
            None
        }
    }

    fn read_name(&mut self) -> Option<Token> {
        let name = self.read_until(Self::is_name,
                                   u8::is_ascii_whitespace);
        let name = if let Some(tok) = name {
            if self.keywords.contains(&*tok) {
                Some(Token::Keyword(tok))
            } else {
                Some(Token::Atom(Atom::Name(tok)))
            }
        } else {
            None
        };
        self.consume(name)
    }
}

#[cfg(test)]
mod tokenizer_tests {
    use super::*;

    fn tokenizer(input: &[u8]) -> Tokenizer<&[u8]> {
        let is = InputStream::new(input);
        Tokenizer::new(is)
    }

    #[test]
    fn skip_whitespace() {
        let input = b"  jump main".to_vec();
        let mut tokenizer = tokenizer(&*input);
        tokenizer.read_while(u8::is_ascii_whitespace);
        assert_eq!(Some(b'j'), tokenizer.stream.peek(),
                   "skipped too much or too little");
    }

    #[test]
    fn next() {
        let input = b"  jump main".to_vec();
        let mut tokenizer = tokenizer(&*input);
        assert_eq!(Some(Token::Keyword("jump".to_string())), tokenizer.next(),
                   "expected 'jump'");
        assert_eq!(Some(Token::Atom(Atom::Name("main".to_string()))),
                   tokenizer.next(), "expected 'main'");
    }

    #[test]
    fn next_destructive() {
        let input = b"  jump-1".to_vec();
        let mut tokenizer = tokenizer(&*input);
        assert_eq!(None, tokenizer.next(), "expected 'jump'");
    }
}
