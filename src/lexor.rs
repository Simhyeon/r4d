use crate::error::RadError;
use crate::consts::*;

pub struct Lexor {
    previous_char : Option<char>,
    pub cursor: Cursor,
    pub escape_next : bool,
    pub surrounding : Surrounding,
    pub paren_count : usize,
    pub escape_nl : bool,
}

impl Lexor {
    pub fn new() -> Self {
        Lexor {
            previous_char : None,
            cursor: Cursor::None,
            escape_next : false,
            escape_nl : false,
            surrounding : Surrounding::None, 
            paren_count : 0,
        }
    }

    pub fn lex(&mut self, ch: char) -> Result<LexResult, RadError> {
        let result: LexResult;
        match self.cursor {
            Cursor::None => {
                result = self.branch_none(ch);
            },
            Cursor::Name => {
                result = self.branch_name(ch);
            }
            Cursor::NameToArg => {
                result = self.branch_name_to_arg(ch);
            }
            Cursor::Arg => {
                result = self.branch_arg(ch);
            } // end arg match
        }

        self.set_previous(ch);
        Ok(result)
    }

    fn branch_none(&mut self, ch: char) -> LexResult {
        let result: LexResult;
        if ch == MACRO_START_CHAR 
            && self.previous_char.unwrap_or('0') != ESCAPE_CHAR 
        {
            self.cursor = Cursor::Name;
            result = LexResult::Ignore;
            self.escape_nl = false;
        } else if self.escape_nl && (ch as i32 == 13 || ch as i32 == 10) {
            result = LexResult::Ignore;
        } else {
            self.escape_nl = false;
            result = LexResult::AddToRemainder;
        }
        result
    }

    fn branch_name(&mut self, ch: char) -> LexResult {
        let mut result = LexResult::Ignore;
        // Check whehter name should end or not
        let mut end_name = false;
        // if macro name's first character, then it should be alphabetic
        if self.previous_char.unwrap_or(MACRO_START_CHAR) == MACRO_START_CHAR {
            if ch.is_alphabetic() {
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
            // CHECK -> Maybe unncessary
            // Exit when unallowed character is given
            else {
                self.cursor = Cursor::None;
                result = LexResult::ExitFrag;
            }
        }
        result
    }

    fn branch_name_to_arg(&mut self, ch: char) -> LexResult {
        let result: LexResult;

        if ch == ' ' {
            result = LexResult::Ignore;
        } else if ch == '(' {
            self.cursor = Cursor::Arg;
            self.paren_count = 1;
            result = LexResult::AddToFrag(Cursor::Arg);
        } else {
            self.cursor = Cursor::None;
            result = LexResult::ExitFrag;
        }
        result
    }

    fn branch_arg(&mut self, ch: char) -> LexResult {
        let result: LexResult;
        // if given ending parenthesis without surrounding double quotes,
        // it means end of args
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
            } else {
                result = LexResult::AddToFrag(Cursor::Arg);
            }
        }
        result
    }

    fn set_previous(&mut self, ch: char) {
        match ch {
            '"' => {
                if self.previous_char.unwrap_or(' ') != ESCAPE_CHAR {
                    if let Surrounding::Dquote = self.surrounding { 
                        self.surrounding = Surrounding::None;
                    } else {
                        self.surrounding = Surrounding::Dquote;
                    }
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

#[derive(Debug)]
pub enum Surrounding {
    None,
    Dquote,
}

#[derive(Clone, Copy, Debug)]
pub enum Cursor {
    None,
    Name,
    NameToArg,
    Arg,
}
