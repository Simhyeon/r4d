use crate::error::RadError;
use crate::consts::*;

pub struct Lexor {
    previous_char : Option<char>,
    pub cursor: Cursor,
    pub escape_next : bool,
    pub literal: bool,
    pub dquote: bool,
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
            literal : false,
            dquote: false,
            paren_count : 0,
        }
    }
    pub fn reset(&mut self) {
        self.previous_char = None;
        self.cursor= Cursor::None;
        self.escape_next = false;
        self.escape_nl = false;
        self.literal = false;
        self.dquote= false;
        self.paren_count = 0;
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
                if self.literal {
                    result = self.branch_arg_literal(ch);
                } else {
                    result = self.branch_arg(ch);
                }
            } // end arg match
        }

        // Set previous character
        self.previous_char.replace(ch);
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
        } 
        // This applies to cases where new lines comes after invocation
        // e.g. $define(..) \n
        // in this case last \n is ignored and deleted
        // escape_nl is only set after define
        else if self.escape_nl && (ch as i32 == 13 || ch as i32 == 10) {
            result = LexResult::Ignore;
        } 
        // Characters other than newline means other characters has been introduced
        // after definition thus, escape_nl is now false
        else {
            self.escape_nl = false;
            result = LexResult::AddToRemainder;
        }
        result
    }

    fn branch_name(&mut self, ch: char) -> LexResult {
        let mut result: LexResult;

        // Start arg if parenthesis was given,
        // whitespaces are ignored and don't trigger exit
        if ch == ' ' {
            self.cursor = Cursor::NameToArg;
            result = LexResult::Ignore;
        } 
        // Left parenthesis trigger macro invocation
        else if ch == '(' {
            // Literal rule
            // $macro\(
            if self.previous_char.unwrap_or('0') == ESCAPE_CHAR {
                self.literal = true;
                self.cursor = Cursor::Arg;
            } 
            // Else
            else {
                self.cursor = Cursor::Arg;
                self.paren_count = 1;
            }
            result = LexResult::StartFrag;
            // Empty name
            if self.previous_char.unwrap_or('0') == '$' {
                result = LexResult::EmptyName;
            }
        } 
        // Put any character in name
        // It is ok not to validate macro name
        // because invalid name cannot be registered anyway
        else {
            result = LexResult::AddToFrag(Cursor::Name);
        }
        result
    }

    /// Space between name and args
    /// e.g.
    /// $define ()
    ///        |-> This is the name to args characters
    fn branch_name_to_arg(&mut self, ch: char) -> LexResult {
        let result: LexResult;

        // White space or tab character is ignored
        if ch == ' ' || ch == '\t' {
            result = LexResult::Ignore;
        } 
        // Parenthesis start arguments
        else if ch == '(' {
            self.cursor = Cursor::Arg;
            self.paren_count = 1;
            result = LexResult::StartFrag;
        } 
        // Other characters are invalid
        else {
            self.cursor = Cursor::None;
            result = LexResult::ExitFrag;
        }
        result
    }

    // Double quote rule is somewhat suspicious?
    fn branch_arg(&mut self, ch: char) -> LexResult {
        let mut result: LexResult = LexResult::AddToFrag(Cursor::Arg);
        // Inside dquotes
        if self.dquote {
            if ch == '"' {
                if self.previous_char.unwrap_or('0') != ESCAPE_CHAR {
                    self.dquote = false;
                }
            }
        } 
        // Not in dquotes
        else {
            // Right paren decreases paren_count
            if ch == ')' {
                self.paren_count = self.paren_count - 1; 
                if self.paren_count == 0 {
                    self.cursor = Cursor::None;
                    result = LexResult::EndFrag;
                }
            } 
            // Left paren increases paren_count
            else if ch == '(' {
                self.paren_count = self.paren_count + 1; 
            }
            // Double quotes triggers dquote
            else if ch == '"' {
                if self.previous_char.unwrap_or('0') != ESCAPE_CHAR {
                    self.dquote = true;
                }
            }
            // Other characters are added normally
        }
        result
    }

    fn branch_arg_literal(&mut self, ch: char) -> LexResult {
        let mut result: LexResult = LexResult::AddToFrag(Cursor::Arg);
        // END with '\)'
        if ch == ')' && self.previous_char.unwrap_or('0') == ESCAPE_CHAR {
            self.literal = false;
            result = LexResult::EndFrag;
            self.cursor = Cursor::None; 
        } 

        result
    }
} 

#[derive(Debug)]
pub enum LexResult {
    Ignore,
    AddToRemainder,
    StartFrag,
    EmptyName,
    AddToFrag(Cursor),
    EndFrag,
    ExitFrag,
}

#[derive(Clone, Copy, Debug)]
pub enum Cursor {
    None,
    Name,
    NameToArg,
    Arg,
}
