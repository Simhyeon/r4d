//! # Lexor module
//!
//! This is not about lexing(compiler) but a character validation.
//!
//! There might be conceptual resemblence however I had never learnt compiler before.
//!
//! Lexor carries lexor cursor(state) and validates if given character is valid and whether the
//! character should be saved as a fragment of macro.

use crate::consts::*;
use crate::models::CommentType;
use crate::utils::Utils;

/// Struct that validated a given character
pub struct Lexor {
    previous_char: Option<char>,
    cursor: Cursor,
    literal_count: usize,     // Literal nest level
    parenthesis_count: usize, // Parenthesis nest level
    macro_char: char,
    comment_char: Option<char>,
    consume_previous: bool,
}

impl Lexor {
    pub fn new(macro_char: char, comment_char: char, comment_type: &CommentType) -> Self {
        let comment_char = if let CommentType::Any = comment_type {
            Some(comment_char)
        } else {
            None
        };
        Lexor {
            previous_char: None,
            cursor: Cursor::None,
            literal_count: 0,
            parenthesis_count: 0,
            macro_char,
            comment_char,
            consume_previous: false,
        }
    }
    /// Reset lexor state
    pub fn reset(&mut self) {
        self.previous_char = None;
        self.cursor = Cursor::None;
        self.parenthesis_count = 0;
        self.consume_previous = false;
    }

    /// Validate the character
    pub fn lex(&mut self, ch: char) -> LexResult {
        // Literal related
        if self.start_literal(ch) || self.end_literal(ch) {
            self.previous_char.replace('0');
            return LexResult::Literal(self.cursor);
        } else if self.literal_count > 0 {
            self.previous_char.replace(ch);
            return LexResult::Literal(self.cursor);
        }

        // Exit if comment_type is configured
        // cch == comment char
        if let Some(cch) = self.comment_char {
            if cch == ch {
                self.reset();
                return LexResult::CommentExit;
            }
        }

        // Non literal related logics
        let result = match self.cursor {
            Cursor::None => self.branch_none(ch),
            Cursor::Name => self.branch_name(ch),
            Cursor::Arg => self.branch_arg(ch),
        }; // end arg match

        if self.consume_previous {
            self.previous_char.replace(' ');
            self.consume_previous = false;
        } else {
            let replace = ch;
            self.previous_char.replace(replace);
        }
        result
    }

    // ----------
    // <BRANCH>
    // Branch methods start

    fn branch_none(&mut self, ch: char) -> LexResult {
        let result: LexResult;
        if ch == self.macro_char && self.previous_char.unwrap_or('0') != ESCAPE_CHAR {
            self.cursor = Cursor::Name;
            result = LexResult::Ignore;
        }
        // Characters other than newline means other characters has been introduced
        else {
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
            self.parenthesis_count = 1;
            result = LexResult::StartFrag;
            // Empty name
            if self.previous_char.unwrap_or('0') == self.macro_char {
                result = LexResult::EmptyName;
            }
        } else if ch == self.macro_char {
            result = LexResult::RestartName;
        } else {
            result = LexResult::AddToFrag(Cursor::Name);
        }
        result
    }

    fn branch_arg(&mut self, ch: char) -> LexResult {
        let mut result: LexResult = LexResult::AddToFrag(Cursor::Arg);
        // Right paren decreases paren_count
        if ch == ')' {
            self.parenthesis_count -= 1;
            if self.parenthesis_count == 0 {
                self.cursor = Cursor::None;
                result = LexResult::EndFrag;
            }
        }
        // Left paren increases paren_count
        else if ch == '(' {
            self.parenthesis_count += 1;
        } else if ch == ESCAPE_CHAR && self.previous_char.unwrap_or(' ') == ESCAPE_CHAR {
            // If current ch is \ and previous was also \ consume previous and paste it
            self.consume_previous = true;
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
            self.literal_count += 1;
            true
        } else {
            false
        }
    }

    /// Check if given character set end a literal state
    fn end_literal(&mut self, ch: char) -> bool {
        // if given value is literal character and preceding character is escape
        if ch == ESCAPE_CHAR && self.previous_char.unwrap_or('0') == LIT_CHAR {
            if self.literal_count > 0 {
                self.literal_count -= 1;
            } // else it is simply a *\ without starting \*
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
    RestartName,
    AddToFrag(Cursor),
    EndFrag,
    ExitFrag,
    Literal(Cursor),
    CommentExit,
}

/// Cursor that carries state information of lexor
#[derive(Clone, Copy, Debug)]
pub enum Cursor {
    None,
    Name,
    Arg,
}
