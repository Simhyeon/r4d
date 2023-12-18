//! # arg_parser
//!
//! Module about argument parsing

use crate::consts::{ESCAPE_CHAR, LIT_CHAR};
use std::{iter::Peekable, str::Chars};

/// Argument parser
pub struct ArgParser {
    values: Vec<String>,
    previous: Option<char>,
    lit_count: usize,
    paren_count: usize,
    no_previous: bool,
    strip_literal: bool,
}

impl ArgParser {
    /// Create a new instance
    pub(crate) fn new() -> Self {
        Self {
            values: vec![],
            previous: None,
            lit_count: 0,
            paren_count: 0,
            no_previous: false,
            strip_literal: true,
        }
    }

    /// Reset variables
    fn reset(&mut self) {
        self.values.clear();
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
        self.args_to_vec(args, ',', GreedyState::Greedy)[0].to_owned()
    }

    /// Check if given length is qualified for given raw arguments
    ///
    /// If length is qualified it returns vector of arguments
    /// if not, "None" is returned instead.
    pub(crate) fn args_with_len(&mut self, args: &str, length: usize) -> Option<Vec<String>> {
        self.reset();
        let greedy_state = if length > 1 {
            GreedyState::Deterred(length - 1)
        } else {
            GreedyState::Greedy
        };

        let args: Vec<_> = self.args_to_vec(args, ',', greedy_state);

        if args.len() < length {
            return None;
        }

        Some(args)
    }

    /// Split raw arguments into a vector
    pub(crate) fn args_to_vec(
        &mut self,
        arg_values: &str,
        delimiter: char,
        mut greedy_state: GreedyState,
    ) -> Vec<String> {
        self.reset();
        let mut value = String::new();
        let mut arg_iter = arg_values.chars().peekable();

        // Return empty vector without going through logics
        if arg_values.is_empty() {
            return vec![];
        }

        while let Some(ch) = arg_iter.next() {
            // Check parenthesis
            self.check_parenthesis(&mut value, ch);

            if ch == delimiter {
                self.branch_delimiter(ch, &mut value, &mut greedy_state);
            } else if ch == ESCAPE_CHAR {
                self.branch_escape_char(ch, &mut value, arg_iter.peek());
            } else {
                // This pushes value in the end, so use continue not push the value
                if ch == LIT_CHAR {
                    // '*'
                    self.branch_literal_char(ch, &mut value, &mut arg_iter);
                } else {
                    // Non literal character are just pushed
                    value.push(ch);
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
        self.values.push(value);

        std::mem::take(&mut self.values)
    }

    /// Check parenthesis for sensible splitting
    fn check_parenthesis(&mut self, value: &mut String, ch: char) {
        if self.previous.unwrap_or('0') == ESCAPE_CHAR && (ch == '(' || ch == ')') {
            value.pop();
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
    fn branch_delimiter(&mut self, ch: char, value: &mut String, greedy_state: &mut GreedyState) {
        // Either literal or escaped
        if self.lit_count > 0 {
            value.push(ch);
        } else if self.previous.unwrap_or('0') == ESCAPE_CHAR {
            value.pop();
            value.push(ch);
        } else if self.paren_count > 0 {
            // If quote is inside parenthesis, simply push it into a value
            value.push(ch);
        } else {
            // not literal
            match greedy_state {
                GreedyState::Deterred(count) => {
                    // move to next value
                    self.values.push(std::mem::take(value));
                    let count = *count - 1;
                    if count > 0 {
                        *greedy_state = GreedyState::Deterred(count);
                    } else {
                        *greedy_state = GreedyState::Greedy;
                    }
                    self.no_previous = true;
                }
                // Push everything to current item, index, value or you name it
                GreedyState::Greedy => {
                    value.push(ch);
                }
                GreedyState::Never => {
                    // move to next value
                    self.values.push(std::mem::take(value));
                }
            } // Match end
        } // else end
    }

    /// Branch on escape character found
    fn branch_escape_char(&mut self, ch: char, value: &mut String, next: Option<&char>) {
        if self.previous.unwrap_or(' ') == ESCAPE_CHAR {
            self.no_previous = true;
        } else if let Some(&LIT_CHAR) = next {
            if !self.strip_literal || self.lit_count > 0 {
                value.push(ch);
            }
            // if next is literal character and previous was not a escape character
            // Do nothing
        } else {
            // If literal print everything without escaping
            // or next is anything simply add
            value.push(ch);
        }
    } // end function

    /// Branch on literal character found
    fn branch_literal_char(
        &mut self,
        ch: char,
        value: &mut String,
        arg_iter: &mut Peekable<Chars>,
    ) {
        if self.previous.unwrap_or('0') == ESCAPE_CHAR {
            self.lit_count += 1;
            // If lit character was given inside literal
            // e.g. \* '\*' *\ -> the one inside quotes
            if self.lit_count > 1 {
                value.push(ch);
            }
            // First lit character in given args
            // Simply ignore character and don't set previous
            else {
                self.no_previous = true;
                if !self.strip_literal {
                    value.push(ch);
                }
            }
        } else if let Some(&ch_next) = arg_iter.peek() {
            // Next is escape char and not inside lit_count
            // *\
            if ch_next == ESCAPE_CHAR && self.lit_count >= 1 {
                self.lit_count -= 1;
                arg_iter.next(); // Conume next escape_char
                                 // Lit end was outter most one
                if self.lit_count == 0 {
                    self.no_previous = true;
                    if !self.strip_literal {
                        value.push(LIT_CHAR);
                        value.push(ESCAPE_CHAR);
                    }
                }
                // Inside other literal rules
                else {
                    value.push(LIT_CHAR);
                    value.push(ESCAPE_CHAR);
                    self.no_previous = true;
                }
            }
            // When *\ Comes first without matching pair
            // This is just a string without any meaning
            else {
                value.push(ch);
            }
        }
        // Meaningless literal charcter are just pushed
        else {
            value.push(ch);
        }
    } // end function

    // End of branch methods
    // </BRANCH>
    // ----------
}

/// State indicates whether argument should be parsed greedily or not
#[derive(Debug)]
pub enum GreedyState {
    /// Split argument with given amount
    Deterred(usize),
    Greedy,
    Never,
}
