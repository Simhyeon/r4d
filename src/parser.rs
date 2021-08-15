use std::io::{self, StdinLock, Lines};
use std::io::prelude::*;
use regex::Regex;
use crate::error::RadError;
use crate::models::Invocation;

const MACRO_START_CHAR: char = '$';
const ESCAPE_CHAR: char ='\\';

pub struct MacroFragment {
    pub whole_string: String,
    pub name: String,
    pub args: String,
}

impl MacroFragment {
    pub fn new() -> Self {
        MacroFragment {  
            whole_string : String::new(),
            name : String::new(),
            args : String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.whole_string.clear();
        self.name.clear();
        self.args.clear();
    }
}

pub enum Surrounding {
    None,
    Paren,
    Squared,
    Quote,
    Dquote,
}

pub struct Lexor {
    previous_char : Option<char>,
    pub cursor: Cursor,
    pub escape_next : bool,
    pub surrounding : Surrounding,
}

impl Lexor {
    pub fn new() -> Self {
        Lexor {
            previous_char : None,
            cursor: Cursor::None,
            escape_next : false,
            surrounding : Surrounding::None, 
        }
    }

    pub fn lex(&mut self, ch: char) -> Result<LexResult, RadError> {
        let mut result: LexResult = LexResult::Ignore;
        println!("Target character : <<  {}  >>", ch);
        match self.cursor {
            Cursor::None => {
                if ch == MACRO_START_CHAR 
                    && self.previous_char.unwrap_or('0') != ESCAPE_CHAR {
                        println!("--MACRO START--");
                        self.cursor = Cursor::Name;
                        result = LexResult::AddToFrag(Cursor::Name);
                } else {
                    result = LexResult::AddToRemainder;
                }
            },
            Cursor::Name => {
                println!("Type : [[  Name  ]] ");
                // Check whehter name should end or not
                let mut end_name = false;
                // if macro name's first character, then it should be alphabetic
                if self.previous_char.unwrap_or(MACRO_START_CHAR) == MACRO_START_CHAR {
                    if ch.is_alphabetic() || ch == '_' {
                        result = LexResult::AddToFrag(Cursor::Name);
                    } else {
                        end_name = true;
                    }
                } else { // not first character
                    // Can be alphanumeric
                    if ch.is_alphanumeric() || ch == '_' {
                        result = LexResult::AddToFrag(Cursor::Name);
                    } else {
                        end_name = true;
                    }
                }

                // Unallowed character
                // Start arg if parenthesis was given,
                // whitespaces are ignored and don't trigger exit
                if end_name {
                    if ch == ' ' {
                        self.cursor = Cursor::NameToArg;
                        result = LexResult::Ignore;
                    } else if ch == '(' {
                        self.cursor = Cursor::Arg;
                        result = LexResult::Ignore;
                    }
                }
            }
            Cursor::NameToArg => {
                if ch == ' ' {
                    result = LexResult::Ignore;
                }
                // TODO 
                // LexResult should imply that another syntax is required
                else if ch == '(' {
                    println!("START ARGS");
                    self.cursor = Cursor::Arg;
                    result = LexResult::AddToFrag(Cursor::Arg);
                }
                else {
                    self.cursor = Cursor::None;
                    result = LexResult::ExitFrag;
                }
            }
            // TODO
            // Arg needs extra logic to detect double quoted parenthesis
            Cursor::Arg => {
                println!("Type : [[  ARGS  ]]");
                // if ending parenthesis without surrounding double quotes are 
                // ending of args
                if let Surrounding::Dquote = self.surrounding {
                    result = LexResult::AddToFrag(Cursor::Arg);
                } else {
                    if ch == ')' {
                        println!("ENDING...");
                        self.cursor = Cursor::None;
                        result = LexResult::EndFrag;
                    } else if ch == '(' {
                        println!("EXITING...");
                        self.cursor = Cursor::None;
                        result = LexResult::ExitFrag;
                    } else {
                        result = LexResult::AddToFrag(Cursor::Arg);
                    }
                }
            } // end arg match
        }

        self.set_previous(ch);
        Ok(result)
    }

    // TODO
    // Previous this was not a part of lexor struct
    // Make this method to called after lex method so that cursor change or 
    // other setter logic is invoked
    pub fn set_previous(&mut self, ch: char) {
        match ch {
            '(' => self.surrounding = Surrounding::Paren,
            ')' => if let Surrounding::Paren = self.surrounding { self.surrounding = Surrounding::None;}
            '[' => self.surrounding = Surrounding::Squared,
            ']' => if let Surrounding::Squared = self.surrounding { self.surrounding = Surrounding::None;}
            '\'' => {
                if let Surrounding::Quote = self.surrounding { 
                    self.surrounding = Surrounding::None;
                } else {
                    self.surrounding = Surrounding::Quote;
                }
            }
            '"' => {
                if let Surrounding::Dquote = self.surrounding { 
                    self.surrounding = Surrounding::None;
                } else {
                    self.surrounding = Surrounding::Dquote;
                }
            }
            _ => (),
        }
        // Set previous
        self.previous_char.replace(ch);
    }

    pub fn get_previous(&self) -> Option<char> {
        self.previous_char
    }
} 

pub enum ParseResult {
    FoundMacro(String),
    Printable(String),
    NoPrint,
    EOI,
}

pub enum LexResult {
    Ignore,
    AddToRemainder,
    AddToFrag(Cursor),
    EndFrag,
    ExitFrag,
}

pub struct Parser;
// 1. Get string
// 2. Parse until macro invocation detected
// 3. Return remainder and macro fragments
// 4. Continue parsing with fragments

impl Parser {
    pub fn from_stdin() -> Result<(), RadError> {
        let stdin = io::stdin();
        let mut line_iter = stdin.lock().lines();
        let mut lexor = Lexor::new();
        let mut invoke = MacroFragment::new();
        loop {
            let result = Self::parse_line(&mut line_iter, &mut lexor ,&mut invoke)?;
            match result {
                // This means either macro is not found at all
                // or previous macro fragment failed with invalid syntax
                ParseResult::Printable(remainder) => {
                    println!("Remainder from parse_line ::: {}", remainder);
                    // Reset fragment
                    if &invoke.whole_string != "" {
                        invoke = MacroFragment::new();
                    }
                }
                ParseResult::FoundMacro(remainder) => {
                    print!("Found macro {}", remainder);
                }
                ParseResult::NoPrint => (), // Do nothing
                // End of input, end loop
                ParseResult::EOI => break,
            }
        } // Loop end

        Ok(())
    }
    
    // TODO
    fn parse_line(lines :&mut Lines<StdinLock>, lexor : &mut Lexor ,frag : &mut MacroFragment) -> Result<ParseResult, RadError> {
        if let Some(line) = lines.next() {
            // Rip off a result into a string
            let line = line?;
            // Local values
            let mut remainder = String::new();
            for ch in line.chars() {
                let lex_result = lexor.lex(ch)?;
                // TODO
                // Either add character to remainder or fragments
                match lex_result {
                    LexResult::Ignore => continue,
                    LexResult::AddToRemainder => {
                        remainder.push(ch);
                    }
                    LexResult::AddToFrag(cursor) => {
                        match cursor{
                            Cursor::Name => frag.name.push(ch),
                            Cursor::Arg => frag.args.push(ch),
                            _ => unreachable!(),
                        } 
                        frag.whole_string.push(ch);
                    }
                    // TODO
                    // End of fragment
                    // 1. Evaluate macro 
                    // 1.5 -> If define, parse rule applied again.
                    // 2. And append to remainder
                    // 3. Reset fragment
                    LexResult::EndFrag => {
                        if 
                    }
                    // Remove fragment and set to remainder
                    LexResult::ExitFrag => {
                        remainder.push_str(&frag.whole_string);
                        frag.clear();
                    }
                }
            }
            // Non macro string is included
            if remainder.len() != 0 {
                // Fragment is not empty
                if frag.whole_string.len() != 0 {
                    Ok(ParseResult::FoundMacro(remainder))
                } 
                // Print everything
                else {
                    Ok(ParseResult::Printable(remainder))
                }
            } 
            // Nothing to print
            else {
                Ok(ParseResult::NoPrint)
            }
        } else {
            println!("--END OF INPUT--");
            Ok(ParseResult::EOI)
        }
    } // parse_line end
}

#[derive(Clone, Copy)]
pub enum Cursor {
    None,
    Name,
    NameToArg,
    Arg,
}
