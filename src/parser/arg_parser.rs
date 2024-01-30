//! # arg_parser
//!
//! Module about argument parsing

use crate::consts::{ESCAPE_CHAR, LIT_CHAR};
use std::{borrow::Cow, iter::Peekable, str::CharIndices};

/// State indicates whether argument should be parsed greedily or not
#[derive(Debug)]
pub enum SplitVariant {
    /// Split argument with given amount
    Deterred(usize),
    GreedyStrip,
    Always,
}

/// Argument parser
pub struct NewArgParser {
    previous: Option<char>,
    lit_count: usize,
    paren_count: usize,
    no_previous: bool,
    strip_literal: bool,
}

impl NewArgParser {
    /// Create a new instance
    pub(crate) fn new() -> Self {
        Self {
            previous: None,
            lit_count: 0,
            paren_count: 0,
            no_previous: false,
            strip_literal: true,
        }
    }

    /// Reset variables
    fn reset(&mut self) {
        self.previous = None;
        self.lit_count = 0;
        self.no_previous = false;
    }

    /// Don't strip literals
    pub(crate) fn no_strip(mut self) -> Self {
        self.strip_literal = false;
        self
    }

    /// Don't strip literals
    pub(crate) fn set_strip(&mut self, strip_literal: bool) {
        self.strip_literal = strip_literal;
    }

    /// Simply strip literal chunk
    pub(crate) fn strip(&mut self, args: &str) -> String {
        self.args_to_vec(args, ',', SplitVariant::GreedyStrip)
            .first()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    /// Check if given length is qualified for given raw arguments
    ///
    /// If length is qualified it returns vector of arguments
    /// if not, "None" is returned instead.
    pub(crate) fn args_with_len<'a>(
        &mut self,
        args: &'a str,
        length: usize,
    ) -> Option<Vec<Cow<'a, str>>> {
        self.reset();
        if length == 0 && !args.is_empty() {
            return None;
        }
        let split_var = if length > 1 {
            SplitVariant::Deterred(length - 1)
        } else {
            SplitVariant::GreedyStrip
        };

        let args: Vec<_> = self.args_to_vec(args, ',', split_var);

        if args.len() < length {
            return None;
        }
        Some(args)
    }

    /// Split raw arguments into a vector
    pub(crate) fn args_to_vec<'a>(
        &mut self,
        arg_values: &'a str,
        delimiter: char,
        mut split_var: SplitVariant,
    ) -> Vec<Cow<'a, str>> {
        let mut values: Vec<Cow<'a, str>> = vec![];
        self.reset();
        let mut cursor = ArgCursor::Reference(0, 0);

        // This is totally ok to iterate as char_indices rather than chars
        // because "only ASCII char is matched" so there is zero possibilty that
        // char_indices will make any unexpected side effects.
        let mut arg_iter = arg_values.char_indices().peekable();

        // Return empty vector without going through logics
        if arg_values.is_empty() {
            return vec![];
        }

        while let Some((idx, ch)) = arg_iter.next() {
            // Check parenthesis
            self.check_parenthesis(&mut cursor, ch, &arg_values);

            if ch == delimiter {
                if let Some(v) =
                    self.branch_delimiter(&mut cursor, ch, idx, &mut split_var, arg_values)
                {
                    values.push(v);
                }
            } else if ch == ESCAPE_CHAR {
                self.branch_escape_char(&mut cursor, ch, arg_iter.peek());
            } else {
                // This pushes value in the end, so use continue not push the value
                if ch == LIT_CHAR {
                    // '*'
                    self.branch_literal_char(&mut cursor, ch, &mut arg_iter, &arg_values);
                } else {
                    // Non literal character are just pushed
                    cursor.push(ch);
                }
            }

            if self.no_previous {
                self.previous.replace('0');
                self.no_previous = false;
            } else {
                self.previous.replace(ch);
            }
        } // while end
          // Add last arg
        let sc = if cursor.is_string() { "" } else { arg_values };
        values.push(cursor.take_value(arg_values.len(), sc));
        values
    }

    /// Check parenthesis for sensible splitting
    fn check_parenthesis(&mut self, cursor: &mut ArgCursor, ch: char, src: &&str) {
        if self.previous.unwrap_or('0') == ESCAPE_CHAR && (ch == '(' || ch == ')') {
            cursor.convert_to_modified(src);
            cursor.pop();
            self.previous.replace('0');
        } else if ch == '(' {
            self.paren_count += 1;
        } else if ch == ')' && self.paren_count > 0 {
            self.paren_count -= 1;
        }
    }

    // ----------
    // <BRANCH>
    // Start of branch methods

    /// Branch on delimiter found
    fn branch_delimiter<'a>(
        &mut self,
        cursor: &mut ArgCursor,
        ch: char,
        index: usize,
        variant: &mut SplitVariant,
        src: &'a str,
    ) -> Option<Cow<'a, str>> {
        let mut ret = None;
        // Either literal or escaped
        if self.lit_count > 0 {
            cursor.push(ch);
        } else if self.previous.unwrap_or('0') == ESCAPE_CHAR {
            cursor.convert_to_modified(src);
            cursor.pop();
            cursor.push(ch);
        } else if self.paren_count > 0 {
            // If quote is inside parenthesis, simply push it into a value
            cursor.push(ch);
        } else {
            // not literal
            match variant {
                SplitVariant::Deterred(count) => {
                    // move to next value
                    let sc = if cursor.is_string() { "" } else { src };
                    let v = cursor.take_value(index + 1, sc);
                    ret.replace(v);
                    let count = *count - 1;
                    if count > 0 {
                        *variant = SplitVariant::Deterred(count);
                    } else {
                        *variant = SplitVariant::GreedyStrip;
                    }
                    self.no_previous = true;
                }
                // Push everything to current item, index, value or you name it
                SplitVariant::GreedyStrip => {
                    cursor.push(ch);
                }
                SplitVariant::Always => {
                    // move to next value
                    let sc = if cursor.is_string() { "" } else { src };
                    let v = cursor.take_value(index + 1, sc);
                    ret.replace(v);
                }
            } // Match end
        } // else end
        ret
    }

    /// Branch on escape character found
    fn branch_escape_char(
        &mut self,
        cursor: &mut ArgCursor,
        ch: char,
        next: Option<&(usize, char)>,
    ) {
        if self.previous.unwrap_or(' ') == ESCAPE_CHAR {
            self.no_previous = true;
        } else if let Some((_, LIT_CHAR)) = next {
            if !self.strip_literal || self.lit_count > 0 {
                cursor.push(ch);
            }
            // if next is literal character and previous was not a escape character
            // Do nothing
        } else {
            // If literal print everything without escaping
            // or next is anything simply add
            cursor.push(ch);
        }
    } // end function

    /// Branch on literal character found
    fn branch_literal_char(
        &mut self,
        cursor: &mut ArgCursor,
        ch: char,
        arg_iter: &mut Peekable<CharIndices>,
        src: &&str,
    ) {
        if self.previous.unwrap_or('0') == ESCAPE_CHAR {
            self.lit_count += 1;
            // If lit character was given inside literal
            // e.g. \* '\*' *\ -> the one inside quotes
            if self.lit_count > 1 {
                cursor.push(ch);
            }
            // First lit character in given args
            // Simply ignore character and don't set previous
            else {
                self.no_previous = true;
                if !self.strip_literal {
                    cursor.push(ch);
                }
            }
        } else if let Some((_, ch_next)) = arg_iter.peek() {
            // Next is escape char and not inside lit_count
            // *\
            if *ch_next == ESCAPE_CHAR && self.lit_count >= 1 {
                self.lit_count -= 1;
                arg_iter.next(); // Conume next escape_char
                                 // Lit end was outter most one
                if self.lit_count == 0 {
                    self.no_previous = true;
                    if !self.strip_literal {
                        cursor.convert_to_modified(src);
                        cursor.push(LIT_CHAR);
                        cursor.push(ESCAPE_CHAR);
                    }
                }
                // Inside other literal rules
                else {
                    cursor.convert_to_modified(src);
                    cursor.push(LIT_CHAR);
                    cursor.push(ESCAPE_CHAR);
                    self.no_previous = true;
                }
            }
            // When *\ Comes first without matching pair
            // This is just a string without any meaning
            else {
                cursor.push(ch);
            }
        }
        // Meaningless literal charcter are just pushed
        else {
            cursor.push(ch);
        }
    } // end function

    // End of branch methods
    // </BRANCH>
    // ----------
}

enum ArgCursor {
    Reference(usize, usize),
    Modified(String),
}

impl ArgCursor {
    pub fn is_string(&self) -> bool {
        matches!(self, Self::Modified(_))
    }

    /// Use is_string before taking value and supply empty if the inner vaule is string
    ///
    /// because src is supplied as is while the argument is completely ignored when the inner value
    /// is a string.
    pub fn take_value<'a>(&mut self, index: usize, src: &'a str) -> Cow<'a, str> {
        match self {
            Self::Reference(c, n) => {
                let val = src[*c..*n].into();
                *self = Self::Reference(index, index);
                val
            }
            Self::Modified(s) => std::mem::take(s).into(),
        }
    }

    pub fn convert_to_modified(&mut self, src: &str) {
        if let Self::Reference(c, n) = self {
            *self = Self::Modified(src[*c..*n].to_string())
        }
    }

    pub fn push(&mut self, ch: char) {
        match self {
            Self::Reference(_, n) => *n += 1,
            Self::Modified(st) => st.push(ch),
        }
        if let Self::Modified(st) = self {
            st.push(ch);
        }
    }
    pub fn pop(&mut self) {
        if let Self::Modified(st) = self {
            st.pop();
        }
    }
}
