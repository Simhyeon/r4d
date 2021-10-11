//! # Lexor module
//!
//! This is not about lexing(compiler) but a character validation.
//!
//! There might be conceptual resemblence however I had never learnt compiler before.
//!
//! Lexor carries lexor cursor(state) and validates if given character is valid and whether the
//! character should be saved as a fragment of macro.

use crate::error::RadError;
use crate::consts::*;
use crate::utils::Utils;

/// Struct that validated a given character
pub struct Lexor {
    previous_char : Option<char>,
    cursor: Cursor,
    lit_count: usize,
    paren_count : usize,
    escape_nl : bool,
}

impl Lexor {
    pub fn new() -> Self {
        Lexor {
            previous_char : None,
            cursor: Cursor::None,
            escape_nl : false,
            lit_count : 0,
            paren_count : 0,
        }
    }
    /// Reset lexor state
    pub fn reset(&mut self) {
        self.previous_char = None;
        self.cursor= Cursor::None;
        self.escape_nl = false;
        self.paren_count = 0;
    }

    /// Escape comming new line
    pub fn escape_next_newline(&mut self) {
        self.escape_nl = true;
    }

    pub fn reset_escape(&mut self) {
        self.escape_nl = false;
    }

    /// Validate the character
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
            Cursor::Arg => {
                result = self.branch_arg(ch);
            } // end arg match
        }

        let replace = ch;

        self.previous_char.replace(replace);
        Ok(result)
    }

    // ----------
    // <BRANCH>
    // Branch methods start

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
            result = LexResult::Discard;
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

        // Blank characters are invalid
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
        else if ch == '$' {
            result = LexResult::RestartName;
        }
        else {
            result = LexResult::AddToFrag(Cursor::Name);
        }
        result
    }

    fn branch_arg(&mut self, ch: char) -> LexResult {
        let mut result: LexResult = LexResult::AddToFrag(Cursor::Arg);
        // Right paren decreases paren_count
        if ch == ')'{
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
        // Other characters are added normally
        result
    }

    // End of branch methods
    // </BRANCH>
    // ----------

    /// Check if given character set starts a literal state
    fn start_literal(&mut self, ch: char) -> bool {
        // if given value is literal character and preceding character is escape
        if ch == LIT_CHAR && self.previous_char.unwrap_or('0') == ESCAPE_CHAR {
            self.lit_count = self.lit_count + 1; 
            true
        } else {
            false
        }
    }

    /// Check if given character set end a literal state
    fn end_literal(&mut self, ch: char) -> bool {
        // if given value is literal character and preceding character is escape
        if ch == ESCAPE_CHAR && self.previous_char.unwrap_or('0') == LIT_CHAR {
            if self.lit_count > 0 {
                self.lit_count = self.lit_count - 1; 
            } // else it is simply a *\ without starting \*
            true
        } else {
            false
        }
    }
} 

#[derive(Debug)]
pub enum LexResult {
    Discard,
    Ignore,
    AddToRemainder,
    StartFrag,
    EmptyName,
    RestartName,
    AddToFrag(Cursor),
    EndFrag,
    ExitFrag,
    Literal(Cursor),
}

/// Cursor that carries state information of lexor
#[derive(Clone, Copy, Debug)]
pub enum Cursor {
    None,
    Name,
    Arg,
}
