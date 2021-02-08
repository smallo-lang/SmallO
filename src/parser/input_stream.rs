#![allow(dead_code)]

extern crate idioma;
use idioma::*;

use std::io::{prelude::*, Bytes, BufReader};
use std::fmt::Display;


pub struct InputStream<R> {
    reader: Bytes<BufReader<R>>,
    last: Option<u8>,
    line: usize,
    col: usize,
}


impl<R: Read> InputStream<R> {
    pub fn new(reader: R) -> Self {
         Self {
            reader: BufReader::new(reader).bytes(),
            last: None,
            line: 1,
            col: 0,
         }
    }

    /// `BufReader` returns `None` to signify EOF and `Result` to signify some I/O error. However,
    /// we don't care about low level details here. We simply want to return either a byte, or
    /// `None`. This way if we receive `None` where not expected, it's an error and we `screech`.
    pub fn next(&mut self) -> Option<u8> {
        let byte = self.reader.next();
        if byte.is_none() {
            return self.consume(None);
        }

        let byte = byte.unwrap();
        if byte.is_err() {
            return self.consume(None);
        }

        let byte = byte.unwrap();
        if byte == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }

        self.consume(Some(byte))
    }

    pub fn peek(&self) -> Option<u8> {
        self.last
    }

    pub fn eof(&self) -> bool {
        self.peek().is_none()
    }

    pub fn screech<D>(&self, message: D) -> Error where D: Display {
        error(format!("[{}:{}] {}", self.line, self.col, message))
    }

    fn consume(&mut self, byte: Option<u8>) -> Option<u8> {
        self.last = byte;
        byte
    }
}

#[cfg(test)]
mod input_stream_tests {
    use super::*;
    use std::io;

    fn is_space(byte: u8) -> bool { byte == b' ' }
    fn is_letter(byte: u8) -> bool { b'a' <= byte && byte <= b'z' }

    #[test]
    fn basic() -> io::Result<()> {
        let input = b"  jump main".to_vec();
        let mut is = InputStream::new(&*input);
        let mut buf: Vec<u8> = vec![];
        let mut toks: Vec<String> = vec![];

        loop {
            let byte = is.next();
            if is.eof() {
                if buf.len() > 0 {
                    let string: String = String::from_utf8_lossy(&*buf).into();
                    toks.push(string);
                }
                break;
            }
            let byte = byte.unwrap();

            if is_space(byte) {
                if buf.len() > 0 {
                    let string: String = String::from_utf8_lossy(&*buf).into();
                    toks.push(string);
                    buf.clear();
                }
            } else if is_letter(byte) {
                buf.push(byte);
            }
        }

        assert_eq!(is.col, input.len(), "unexpected None before EOF");
        assert_eq!(vec!["jump", "main"], toks, "tokens didn't match");
        Ok(())
    }
}
