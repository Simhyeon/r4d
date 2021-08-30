use crate::error::RadError;
use crate::consts::*;
use crate::utils::Utils;

pub struct Lexor {
    previous_char : Option<char>,
    pub cursor: Cursor,
    pub escape_next : bool,
    pub lit_count: usize,
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
            lit_count : 0,
            paren_count : 0,
        }
    }
    pub fn reset(&mut self) {
        self.previous_char = None;
        self.cursor= Cursor::None;
        self.escape_next = false;
        self.escape_nl = false;
        self.paren_count = 0;
        // CHECK TODO is it necessary?
        // Don't reset literal
    }

    pub fn lex(&mut self, ch: char) -> Result<LexResult, RadError> {
        let result: LexResult;
        if self.start_literal(ch) {
            self.previous_char.replace('0');
            return Ok(LexResult::Literal(self.cursor)); 
        } else if self.end_literal(ch){
            self.previous_char.replace('0');
            return Ok(LexResult::Literal(self.cursor)); 
        } else if self.lit_count >0 {
            self.previous_char.replace(ch);
            return Ok(LexResult::Literal(self.cursor)); 
        }
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

        let replace = ch;

        // Reserved for later usage
        // Don't set previous as it is when double ESCAPE_CHAR is used. 
        //if ch == ESCAPE_CHAR && self.previous_char.unwrap_or('0') == ESCAPE_CHAR {
            //replace = '0';
        //} 
        self.previous_char.replace(replace);
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

        // Blank characters are nvalid
        if Utils::is_blank_char(ch) {
            self.cursor = Cursor::None;
            result = LexResult::ExitFrag;
        } 
        // Left parenthesis trigger macro invocation
        else if ch == '(' {
            self.cursor = Cursor::Arg;
            self.paren_count = 1;
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
        // Right paren decreases paren_count
        if ch == ')' && self.previous_char.unwrap_or('0') != ESCAPE_CHAR {
            self.paren_count = self.paren_count - 1; 
            if self.paren_count == 0 {
                self.cursor = Cursor::None;
                result = LexResult::EndFrag;
            }
        } 
        // Left paren increases paren_count
        else if ch == '(' && self.previous_char.unwrap_or('0') != ESCAPE_CHAR {
            self.paren_count = self.paren_count + 1; 
        }
        // Other characters are added normally
        result
    }

    fn start_literal(&mut self, ch: char) -> bool {
        // if given value is literal character and preceding character is escape
        if ch == LIT_CHAR && self.previous_char.unwrap_or('0') == ESCAPE_CHAR {
            self.lit_count = self.lit_count + 1; 
            true
        } else {
            false
        }
    }

    fn end_literal(&mut self, ch: char) -> bool {
        // if given value is literal character and preceding character is escape
        if ch == ESCAPE_CHAR && self.previous_char.unwrap_or('0') == LIT_CHAR {
            self.lit_count = self.lit_count - 1; 
            true
        } else {
            false
        }
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
    Literal(Cursor),
}

#[derive(Clone, Copy, Debug)]
pub enum Cursor {
    None,
    Name,
    NameToArg,
    Arg,
}
