//! # arg_parser
//!
//! Module about argument parsing

use std::{iter::Peekable, str::Chars};
use crate::consts::{ESCAPE_CHAR, LIT_CHAR};

pub(crate) struct ArgParser{
    values :Vec<String>,
    previous : Option<char>,
    lit_count : usize,
    no_previous : bool,
}

/// State indicates whether argument should be parsed greedily or not
#[derive(Debug)]
pub(crate) enum GreedyState {
    Deterred(usize),
    Greedy,
    Never,
}

impl ArgParser {
    pub(crate) fn new() -> Self {
        Self {
            values :vec![],
            previous : None,
            lit_count : 0,
            no_previous : false,
        }
    }

    /// Check if given length is qualified for given raw arguments
    ///
    /// If length is qualified it returns vector of arguments
    /// if not, "None" is returned instead.
    pub(crate) fn args_with_len<'a>(&mut self, args: &'a str, length: usize, greedy: bool) -> Option<Vec<String>> {
        let greedy_state = if greedy { 
            if length > 1 {
                GreedyState::Deterred(length - 1)
            } else {
                GreedyState::Greedy
            }
        } else { 
            GreedyState::Never
        };
        let args: Vec<_> = self.args_to_vec(args, ',', greedy_state);

        if args.len() < length {
            return None;
        } 

        Some(args)
    }

    /// Split raw arguments into a vector
    pub(crate) fn args_to_vec(&mut self, arg_values: &str, delimiter: char, mut greedy_state: GreedyState) -> Vec<String> {
        let mut value = String::new();
        let mut arg_iter = arg_values.chars().peekable();

        while let Some(ch) = arg_iter.next() {
            if ch == delimiter {
                self.branch_delimiter(ch, &mut value, &mut greedy_state);
            } else if ch == ESCAPE_CHAR { 
                self.branch_escape_char(ch, &mut value, arg_iter.peek());
            }
            else { // This pushes value in the end, so use continue not push the value
                if ch == LIT_CHAR { // '*'
                    self.branch_literal_char(ch, &mut value, &mut arg_iter);
                } else { // Non literal character are just pushed
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

        std::mem::replace(&mut self.values, vec![])
    }

    // ----------
    // <BRANCH>
    // Start of branch methods

    fn branch_delimiter(&mut self, ch: char,value: &mut String, greedy_state: &mut GreedyState) {
        // Either literal or escaped
        if self.lit_count > 0 || self.previous.unwrap_or('0') == ESCAPE_CHAR 
        { 
            value.push(ch); 
        } else { // not literal
            match greedy_state {
                GreedyState::Deterred(count) => {
                    // move to next value
                    self.values.push(std::mem::replace(value, String::new()));
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
                    // TODO check this line if this is neceesary 
                    // continue; = self.no_previous = true;
                }
                GreedyState::Never => {
                    // move to next value
                    self.values.push(std::mem::replace(value, String::new()));
                }
            } // Match end
        } // else end
    }

    fn branch_escape_char(&mut self, ch: char, value: &mut String, next: Option<&char>) {
        // If literal print everything without escaping
        if self.lit_count > 0 {
            value.push(ch);
        } else if let Some(&LIT_CHAR) = next{
            // if next is literal character
            // Do nothing
        } else {
            // if next is anything simply add
            value.push(ch);
        }
    } // end function

    fn branch_literal_char(&mut self, ch: char, value: &mut String ,arg_iter: &mut Peekable<Chars>) {
        if self.previous.unwrap_or('0') == ESCAPE_CHAR {
            self.lit_count = self.lit_count + 1;
            // If lit character was given inside literal
            // e.g. \* '\*' *\ -> the one inside quotes
            if self.lit_count > 1 {
                value.push(ch);
            } 
            // First lit character in given args
            // Simply ignore character and don't set previous
            else { 
                self.no_previous = true;
            }
        } else if let Some(&ch_next) = arg_iter.peek() {
            // Next is escape char and not inside lit_count
            // *\
            if ch_next == ESCAPE_CHAR && self.lit_count >= 1 {
                self.lit_count = self.lit_count - 1; 
                arg_iter.next(); // Conume next escape_char
                // Lit end was outter most one
                if self.lit_count == 0 {
                    self.no_previous = true;
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
