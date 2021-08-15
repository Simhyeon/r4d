use std::io::{self, StdinLock, Lines};
use std::io::prelude::*;
use crate::error::RadError;
use crate::models::MacroMap;

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
        match self.cursor {
            Cursor::None => {
                if ch == MACRO_START_CHAR 
                    && self.previous_char.unwrap_or('0') != ESCAPE_CHAR {
                        //noprintln!("--MACRO START--");
                        self.cursor = Cursor::Name;
                        result = LexResult::Ignore;
                } else {
                    result = LexResult::AddToRemainder;
                }
            },
            Cursor::Name => {
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
                    //noprintln!("START ARGS");
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
                // if ending parenthesis without surrounding double quotes are 
                // ending of args
                if let Surrounding::Dquote = self.surrounding {
                    result = LexResult::AddToFrag(Cursor::Arg);
                } else {
                    if ch == ')' {
                        self.cursor = Cursor::None;
                        result = LexResult::EndFrag;
                    } else if ch == '(' {
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
    fn set_previous(&mut self, ch: char) {
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

pub struct Processor<'a> {
    macros: MacroMap<'a>,
}
// 1. Get string
// 2. Parse until macro invocation detected
// 3. Return remainder and macro fragments
// 4. Continue parsing with fragments

impl<'a> Processor<'a> {
    pub fn new() -> Self {
        Self {
            macros : MacroMap::new(),
        }
    }
    pub fn from_stdin(&mut self) -> Result<(), RadError> {
        let stdin = io::stdin();
        let mut line_iter = stdin.lock().lines();
        let mut lexor = Lexor::new();
        let mut invoke = MacroFragment::new();
        loop {
            let result = self.parse_line(&mut line_iter, &mut lexor ,&mut invoke)?;
            match result {
                // This means either macro is not found at all
                // or previous macro fragment failed with invalid syntax
                ParseResult::Printable(remainder) => {
                    println!("{}", remainder);
                    // Reset fragment
                    if &invoke.whole_string != "" {
                        invoke = MacroFragment::new();
                    }
                }
                ParseResult::FoundMacro(remainder) => {
                    println!("{}", remainder);
                }
                ParseResult::NoPrint => (), // Do nothing
                // End of input, end loop
                ParseResult::EOI => break,
            }
        } // Loop end

        Ok(())
    }
    
    // TODO
    fn parse_line(&mut self, lines :&mut Lines<StdinLock>, lexor : &mut Lexor ,frag : &mut MacroFragment) -> Result<ParseResult, RadError> {
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
                    LexResult::Ignore => frag.whole_string.push(ch),
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
                        frag.whole_string.push(ch);
                        if frag.name == "define" {
                            // Failed to register macro
                            if let Some((name,args,body)) = Self::parse_define(&frag.args) {
                                self.macros.register(&name, &args, &body)?;
                            } else {
                                remainder.push_str(&frag.whole_string);
                            }
                            // Clear fragment regardless of success
                            frag.clear();
                        } else {
                            // Invoke
                            if let Some(content) = self.macros.evaluate(&frag.name, &frag.args)? {
                                //noprintln!("Evaluated : {}", content);
                                remainder.push_str(&content);
                                frag.clear();
                            }
                        }
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
            //noprintln!("--END OF INPUT--");
            Ok(ParseResult::EOI)
        }
    } // parse_line end

    // Static function
    // NOTE This method expects valid form of macro invocation
    pub fn parse_define(text: &str) -> Option<(String, String, String)> {
        let mut arg_cursor = "name";
        let mut name = String::new();
        let mut args = String::new();
        let mut body = String::new();

        for ch in text.chars() {
            if ch == ',' {
                match arg_cursor {
                    "name" => {arg_cursor = "args"; continue;},
                    "args" => {arg_cursor = "body"; continue;},
                    "body" => return Some((name, args, body)),
                    _ => unreachable!()
                }
            }
            if ch == ' ' || ch == '\n' || ch == '\r' || ch == '\t' {
                // This means pattern like this
                // $define( name ) -> name is registered
                // $define( na me ) -> only "na" is registered
                if arg_cursor == "name" && name.len() != 0 {
                    arg_cursor = "args";
                    continue;
                } 
                // This emans pattern like this
                // $define(name, arg1 ,)
                //                   |
                //                  -This part makes argument empty
                //                  and starts argument evaluation as new state
                else if arg_cursor == "args" && args.len() != 0 {
                    args.clear();
                    continue;
                }
            } else {
                // Body can be any text
                if arg_cursor != "body" {
                    if args.len() == 0 { // Start of string
                        // Not alphabetic and not underscore
                        // $define( 1name ) -> Not valid
                        if !ch.is_alphabetic() && ch != '_' {
                            return None;
                        }
                    } else { // middle of string
                        // Not alphanumeric and not underscore
                        // $define( na*me ) -> Not valid
                        // $define( na_me ) -> Valid
                        if !ch.is_alphanumeric() && ch != '_' {
                            return None;
                        }
                    }
                }
            }

            // Add ch to string, which is used for other evaluation
            match arg_cursor {
                "name" => name.push(ch),
                "args" => args.push(ch),
                "body" => body.push(ch),
                _ => unreachable!()
            }
        }

        Some((name, args, body))
    }
}

#[derive(Clone, Copy)]
pub enum Cursor {
    None,
    Name,
    NameToArg,
    Arg,
}
