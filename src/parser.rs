use std::io;
use std::io::prelude::*;

pub struct Parser;

impl Parser {
    pub fn from_stdin() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            println!("{}", line.unwrap());
        }
    }

    // Parse input stream
    // Let's think about logics
    // First find macro
    // But how?
    // Regex is a good starter, however this read file as line by line so it doesn't work like that
    // Another approach is to read file as a whole string which is... fine considering that gddt
    // file's content is purely text file. It is mostly up to megabytes
    // and expand macro
    pub fn parse(s: &str) {
        let mut macro_idx = 0;
        for (idx,ch) in s.char_indices() {
            if ch != '$' { print!("{}", ch); } 
            else { macro_idx = idx; break;}
        }
    }
}
