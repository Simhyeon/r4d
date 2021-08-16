use crate::error::RadError;
use crate::consts::*;

pub struct Lexor {
    previous_char : Option<char>,
    pub cursor: Cursor,
    pub escape_next : bool,
    pub surrounding : Surrounding,
    pub paren_count : usize,
}

impl Lexor {
    pub fn new() -> Self {
        Lexor {
            previous_char : None,
            cursor: Cursor::None,
            escape_next : false,
            surrounding : Surrounding::None, 
            paren_count : 0,
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
                        self.paren_count = 1;
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
                    self.paren_count = 1;
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
                        self.paren_count = self.paren_count - 1; 
                        if self.paren_count == 0 {
                            self.cursor = Cursor::None;
                            result = LexResult::EndFrag;
                        } else {
                            result = LexResult::AddToFrag(Cursor::Arg);
                        }
                    } else if ch == '(' {
                        self.paren_count = self.paren_count + 1; 
                        result = LexResult::AddToFrag(Cursor::Arg);
                        // TODO
                        // Remove this
                        //self.cursor = Cursor::None;
                        //result = LexResult::ExitFrag;
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

#[derive(Debug)]
pub enum LexResult {
    Ignore,
    AddToRemainder,
    AddToFrag(Cursor),
    EndFrag,
    ExitFrag,
}

pub enum Surrounding {
    None,
    Paren,
    Squared,
    Quote,
    Dquote,
}

#[derive(Clone, Copy, Debug)]
pub enum Cursor {
    None,
    Name,
    NameToArg,
    Arg,
}
