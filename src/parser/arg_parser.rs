//! # arg_parser
//!
//! Module about argument parsing

use crate::{
    argument::{ArgCursor, Argument, MacroInput, ParsedArguments, ParsedCursors},
    common::ETMap,
    consts::{ESCAPE_CHAR_U8, LIT_CHAR_U8},
    Parameter, RadError, RadResult,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{borrow::Cow, slice::Iter};

pub static MACRO_START_MATCH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^\$\S*\($"#).expect("Failed to create name regex"));

/// State indicates whether argument should be parsed greedily or not
#[derive(Debug, PartialEq)]
pub enum SplitVariant {
    /// Split argument with given amount
    Amount(usize),
    Greedy,
    Always,
}

/// Argument parser
pub struct ArgParser {
    // Input related
    delimiter: u8,
    trim: bool,
    split_variant: SplitVariant,

    // Stateful
    previous: Option<u8>,
    lit_count: usize,
    paren_stack: Vec<ParenStack>,
    no_previous: bool,
    strip_literal: bool,
    cursor: ArgCursor,

    // For ParsedCursor generation
    invoke_level: usize,
    macro_name: String,
}

impl ArgParser {
    /// Create a new instance
    pub(crate) fn new() -> Self {
        Self {
            previous: None,
            lit_count: 0,
            paren_stack: vec![],
            no_previous: false,
            strip_literal: true,
            cursor: ArgCursor::Reference(0, 0),
            delimiter: b',',
            trim: false,
            split_variant: SplitVariant::Greedy,
            invoke_level: 0,
            macro_name: String::default(),
        }
    }

    /// Don't strip literals
    pub(crate) fn no_strip(mut self) -> Self {
        self.strip_literal = false;
        self
    }

    pub(crate) fn level(mut self, level: usize) -> Self {
        self.invoke_level = level;
        self
    }

    pub(crate) fn split(mut self, split: SplitVariant) -> Self {
        self.split_variant = split;
        self
    }

    pub(crate) fn macro_name(mut self, name: &str) -> Self {
        self.macro_name = name.to_string();
        self
    }

    /// Don't strip literals
    pub(crate) fn set_strip(&mut self, strip_literal: bool) {
        self.strip_literal = strip_literal;
    }

    /// Simply strip literal chunk
    pub(crate) fn strip(self, name: &str, args: &str) -> RadResult<String> {
        self.args_with_len(MacroInput::new(name, args))?
            .get_text(0)
            .map(|s| s.to_string())
    }

    /// Check if given length is qualified for given raw arguments
    ///
    /// If length is qualified it returns vector of arguments
    pub(crate) fn cursors_with_len(mut self, input: MacroInput) -> RadResult<ParsedCursors> {
        let len_src = input.type_len();
        let length = len_src + if input.optional.is_some() { 1 } else { 0 };
        if input.args.trim().is_empty() {
            if len_src > 0 {
                return Err(RadError::InvalidArgument(format!(
                    "Macro [{}] received empty arguments",
                    input.name
                )));
            }
            return Ok(ParsedCursors::new(input.name));
        }

        self.set_split_variant(length);

        let curs = self.get_cursor_list(&input)?;

        if curs.len() < len_src {
            return Err(RadError::InvalidArgument(format!(
                "Macro [{}] requires [{}] arguments but given [{}] arguments",
                input.name,
                len_src,
                curs.len()
            )));
        }
        Ok(ParsedCursors::new(input.args)
            .with_cursors(curs)
            .with_params(input.params.clone())
            .level(self.invoke_level)
            .macro_name(self.macro_name))
    }

    /// Check if given length is qualified for given raw arguments
    pub(crate) fn args_with_len(mut self, input: MacroInput) -> RadResult<ParsedArguments> {
        let len_src = input.type_len();
        let length = len_src + if input.optional.is_some() { 1 } else { 0 };
        if input.args.trim().is_empty() {
            if len_src > 0 {
                return Err(RadError::InvalidArgument(format!(
                    "Macro [{}] received empty arguments",
                    input.name
                )));
            }
            return Ok(ParsedArguments::with_args(vec![]));
        }

        self.set_split_variant(length);

        let name = input.name;
        let args = self.get_arg_list(input)?;

        if args.len() < len_src {
            return Err(RadError::InvalidArgument(format!(
                "Macro [{}] requires [{}] arguments but given [{}] arguments",
                name,
                len_src,
                args.len()
            )));
        }
        Ok(ParsedArguments::with_args(args))
    }

    /// Check if given length is qualified for given raw arguments
    pub(crate) fn texts_with_len(mut self, input: MacroInput) -> RadResult<Vec<Cow<'_, str>>> {
        let len_src = input.type_len();
        let length = len_src + if input.optional.is_some() { 1 } else { 0 };
        if input.args.trim().is_empty() {
            if len_src > 0 {
                return Err(RadError::InvalidArgument(format!(
                    "Macro [{}] received empty arguments",
                    input.name
                )));
            }
            return Ok(Vec::new());
        }

        self.set_split_variant(length);

        let name = input.name;
        let args = self.get_text_list(input)?;

        if args.len() < len_src {
            return Err(RadError::InvalidArgument(format!(
                "Macro [{}] requires [{}] arguments but given [{}] arguments",
                name,
                len_src,
                args.len()
            )));
        }
        Ok(args)
    }

    /// Split raw arguments into cursors
    ///
    /// THis is used for parsing deterred macro arguments
    pub(crate) fn get_cursor_list(&mut self, input: &MacroInput) -> RadResult<Vec<ArgCursor>> {
        let mut values: Vec<ArgCursor> = vec![];
        self.cursor = ArgCursor::Reference(0, 0);
        let (mut start, mut end) = (0, 0);

        // This is totally ok to iterate as char_indices rather than chars
        // because "only ASCII char is matched" so there is zero possibilty that
        // char_indices will make any unexpected side effects.
        let arg_iter = input.args.as_bytes().iter().enumerate();

        // Return empty vector without going through logics
        if input.args.is_empty() {
            return Err(RadError::InvalidArgument(format!(
                "Macro [{}] received empty arguments",
                input.name
            )));
        }

        for (idx, &ch) in arg_iter {
            // Check parenthesis
            self.check_parenthesis(ch, input.args);

            if ch == self.delimiter {
                // TODO TT
                // This is not a neat solution it works...
                let mut split = false;
                self.branch_delimiter(ch, idx, &mut split, (&mut start, &mut end), input.args);

                if split {
                    values.push(ArgCursor::Reference(start, end));
                }
            } else if ch == ESCAPE_CHAR_U8 {
                self.branch_escape_char(ch, input.args);
            } else {
                // This pushes value in the end, so use continue not push the value
                if ch == LIT_CHAR_U8 {
                    // '*'
                    self.branch_literal_char(ch, input.args);
                } else {
                    // Non literal character are just pushed
                    self.cursor.push(&[ch]);
                }
            }

            if self.no_previous {
                self.previous.replace(b'0');
                self.no_previous = false;
            } else {
                self.previous.replace(ch);
            }
        } // while end
          // Add last arg
        let value = self.cursor.take(input.args.len());
        values.push(value);
        Ok(values)
    }

    /// Split raw arguments into a vector
    ///
    /// This is used for parsing aruments ofr runtime macro
    fn get_text_list<'a>(&mut self, input: MacroInput<'a>) -> RadResult<Vec<Cow<'a, str>>> {
        let mut values: Vec<Cow<'a, str>> = vec![];
        self.cursor = ArgCursor::Reference(0, 0);
        let (mut start, mut end) = (0, 0);
        let trim = input.attr.trim_input;

        // This is totally ok to iterate as char_indices rather than chars
        // because "only ASCII char is matched" so there is zero possibilty that
        // char_indices will make any unexpected side effects.
        let arg_iter = input.args.as_bytes().iter().enumerate();

        // TODO TT
        // If arg_type is empty, every type is treated as text
        let mut at_iter = input.params.iter();

        #[inline]
        fn get_next_type<'b>(
            iter: &mut Iter<'b, Parameter>,
            optional: Option<&'b Parameter>,
        ) -> RadResult<&'b Parameter> {
            if let Some(v) = iter.next() {
                Ok(v)
            } else if let Some(p) = optional {
                Ok(p)
            } else {
                Err(RadError::InvalidExecution(
                    "Argument doesn't match argument type".to_string(),
                ))
            }
        }

        // Return empty vector without going through logics
        if input.args.is_empty() {
            return Err(RadError::InvalidArgument(format!(
                "Macro [{}] received empty arguments",
                input.name
            )));
        }

        for (idx, &ch) in arg_iter {
            // Check parenthesis
            self.check_parenthesis(ch, input.args);

            if ch == self.delimiter {
                // TODO TT
                // This is not a neat solution it works...
                let mut split = false;
                let ret =
                    self.branch_delimiter(ch, idx, &mut split, (&mut start, &mut end), input.args);

                if split {
                    let value: Cow<'_, str> = if let Some(v) = ret {
                        v.into()
                    } else {
                        input.args[start..end].into()
                    };

                    // Only validate and drop arg
                    Self::validate_arg_no_return(
                        get_next_type(&mut at_iter, input.optional.as_ref())?,
                        &value,
                    )?;

                    values.push(value);
                }
            } else if ch == ESCAPE_CHAR_U8 {
                self.branch_escape_char(ch, input.args);
            } else {
                // This pushes value in the end, so use continue not push the value
                if ch == LIT_CHAR_U8 {
                    // '*'
                    self.branch_literal_char(ch, input.args);
                } else {
                    // Non literal character are just pushed
                    self.cursor.push(&[ch]);
                }
            }

            if self.no_previous {
                self.previous.replace(b'0');
                self.no_previous = false;
            } else {
                self.previous.replace(ch);
            }
        } // while end
          // Add last arg
        let value: Cow<'_, str> = if let Some(v) = self.cursor.get_cursor_range_or_get_string(
            input.args.len(),
            trim,
            (&mut start, &mut end),
        ) {
            v.into()
        } else {
            input.args[start..end].into()
        };

        Self::validate_arg_no_return(
            get_next_type(&mut at_iter, input.optional.as_ref())?,
            &value,
        )?;

        values.push(value);
        Ok(values)
    }

    /// Split raw arguments into a vector
    ///
    /// THis is used for parsing function macros
    fn get_arg_list<'a>(&mut self, input: MacroInput<'a>) -> RadResult<Vec<Argument<'a>>> {
        let mut values: Vec<Argument> = vec![];
        self.cursor = ArgCursor::Reference(0, 0);
        let (mut start, mut end) = (0, 0);
        let trim = input.attr.trim_input;

        // This is totally ok to iterate as char_indices rather than chars
        // because "only ASCII char is matched" so there is zero possibilty that
        // char_indices will make any unexpected side effects.
        let arg_iter = input.args.as_bytes().iter().enumerate();

        // TODO TT
        // If arg_type is empty, every type is treated as text
        let mut at_iter = input.params.iter();

        #[inline]
        fn get_next_type<'b>(
            iter: &mut Iter<'b, Parameter>,
            optional: Option<&'b Parameter>,
        ) -> RadResult<&'b Parameter> {
            if let Some(v) = iter.next() {
                Ok(v)
            } else if let Some(p) = optional {
                Ok(p)
            } else {
                Err(RadError::InvalidExecution(
                    "Argument doesn't match argument type".to_string(),
                ))
            }
        }

        // Return empty vector without going through logics
        if input.args.is_empty() {
            return Err(RadError::InvalidArgument(format!(
                "Macro [{}] received empty arguments",
                input.name
            )));
        }

        for (idx, &ch) in arg_iter {
            // Check parenthesis
            self.check_parenthesis(ch, input.args);

            if ch == self.delimiter {
                // TODO TT
                // This is not a neat solution it works...
                let mut split = false;
                let ret =
                    self.branch_delimiter(ch, idx, &mut split, (&mut start, &mut end), input.args);

                if split {
                    let value: Cow<'_, str> = if let Some(v) = ret {
                        v.into()
                    } else {
                        input.args[start..end].into()
                    };
                    let arg = Self::validate_arg(
                        get_next_type(&mut at_iter, input.optional.as_ref())?,
                        value,
                        input.enum_table,
                    )?;
                    values.push(arg);
                }
            } else if ch == ESCAPE_CHAR_U8 {
                self.branch_escape_char(ch, input.args);
            } else {
                // This pushes value in the end, so use continue not push the value
                if ch == LIT_CHAR_U8 {
                    // '*'
                    self.branch_literal_char(ch, input.args);
                } else {
                    // Non literal character are just pushed
                    self.cursor.push(&[ch]);
                }
            }

            if self.no_previous {
                self.previous.replace(b'0');
                self.no_previous = false;
            } else {
                self.previous.replace(ch);
            }
        } // while end
          // Add last arg
        let value: Cow<'_, str> = if let Some(v) = self.cursor.get_cursor_range_or_get_string(
            input.args.len(),
            trim,
            (&mut start, &mut end),
        ) {
            // Return String
            v.into()
        } else {
            let mut src = &input.args[start..end];
            if trim {
                src = src.trim();
            }
            src.into()
        };

        let type_checked_arg = Self::validate_arg(
            get_next_type(&mut at_iter, input.optional.as_ref())?,
            value,
            input.enum_table,
        )?;

        values.push(type_checked_arg);
        Ok(values)
    }

    fn set_split_variant(&mut self, length: usize) {
        if self.split_variant == SplitVariant::Always {
            return;
        }
        self.split_variant = if length > 1 {
            SplitVariant::Amount(length - 1)
        } else {
            SplitVariant::Greedy
        };
    }

    /// Branch on delimiter found
    fn branch_delimiter(
        &mut self,
        ch: u8,
        index: usize,
        do_split: &mut bool,
        range: (&mut usize, &mut usize),
        src: &str,
    ) -> Option<String> {
        let mut ret = None;
        // Either literal or escaped
        if self.lit_count > 0 {
            self.cursor.push(&[ch]);
        } else if self.previous.unwrap_or(b'0') == ESCAPE_CHAR_U8 {
            self.cursor.convert_to_modified(src);
            self.cursor.pop();
            self.cursor.push(&[ch]);
        } else if self.on_invoke() {
            // If quote is inside parenthesis, simply push it into a value
            self.cursor.push(&[ch]);
        } else {
            // not literal
            match self.split_variant {
                SplitVariant::Amount(count) => {
                    // move to next value
                    ret = self
                        .cursor
                        .get_cursor_range_or_get_string(index + 1, self.trim, range);
                    let count = count - 1;
                    if count > 0 {
                        self.split_variant = SplitVariant::Amount(count);
                    } else {
                        self.split_variant = SplitVariant::Greedy;
                    }
                    *do_split = true;
                    self.no_previous = true;
                }
                // Push everything to current item, index, value or you name it
                SplitVariant::Greedy => {
                    self.cursor.push(&[ch]);
                }
                SplitVariant::Always => {
                    *do_split = true;
                    ret = self
                        .cursor
                        .get_cursor_range_or_get_string(index + 1, self.trim, range);
                }
            } // Match end
        } // else end
        ret
    }

    /// Check parenthesis for sensible splitting
    fn check_parenthesis(&mut self, ch: u8, src: &str) {
        if self.previous.unwrap_or(b'0') == ESCAPE_CHAR_U8 && (ch == b'(' || ch == b')') {
            self.cursor.convert_to_modified(src);
            self.cursor.pop();
            self.previous.replace(b'0');
            return;
        }

        // Early return
        if self.lit_count != 0 {
            return;
        }

        if ch == b'(' {
            let stack = if MACRO_START_MATCH
                .find(self.cursor.peek_value(src).trim())
                .is_some()
            {
                ParenStack {
                    macro_invocation: true,
                }
            } else {
                ParenStack {
                    macro_invocation: false,
                }
            };
            self.paren_stack.push(stack);
        } else if ch == b')' && !self.paren_stack.is_empty() {
            self.paren_stack.pop();
        }
    }

    // ----------
    // <BRANCH>
    // Start of branch methods

    // \\ -> \\
    // \( -> (
    // \) -> )
    // \, -> ,

    /// Branch on escape character found
    fn branch_escape_char(&mut self, ch: u8, src: &str) {
        // Next is escape char and not inside lit_count
        // *\
        if self.previous.unwrap_or(b' ') == LIT_CHAR_U8 {
            self.lit_count = self.lit_count.saturating_sub(1);
            self.no_previous = true; // This prevetns *\* from expanding into end and both start of
                                     // literal character
                                     // Ideally *\* should be represented as end of literal chunk
                                     // and with a surplus character of asterisk.
            if self.lit_count == 0 {
                // Lit end was outter most one
                if self.strip_literal {
                    self.cursor.convert_to_modified(src);

                    // Remove \
                    self.cursor.pop();
                } else {
                    self.cursor.push(&[ch]);
                }
            } else {
                // Inside other literal rules
                self.cursor.push(&[ch]);
            }
        } else {
            // Simply put escape character as part of arguments
            self.cursor.push(&[ch]);
        }
    } // end function

    /// Branch on literal character found
    ///
    /// Literal character in this term means '*'
    fn branch_literal_char(&mut self, ch: u8, src: &str) {
        if self.previous.unwrap_or(b'0') == ESCAPE_CHAR_U8 {
            self.lit_count += 1;
            // If lit character was given inside literal
            // e.g. \* '\*' *\ -> the one inside quotes
            if self.lit_count > 1 {
                self.cursor.push(&[ch]);
            } else {
                // First lit character in given args
                // Simply ignore character and don't set previous
                self.no_previous = true;

                // If no strip
                // Push all literal specifier characters
                if !self.strip_literal {
                    self.cursor.push(&[ch]);
                } else {
                    self.cursor.convert_to_modified(src);
                    // If strip then strip all related charaters
                    self.cursor.pop();
                }
            }
        } else {
            self.cursor.push(&[ch]);
        }
    } // end function

    // End of branch methods
    // </BRANCH>
    // ----------

    /// Check if macro invocation chunk is on process
    ///
    /// This is used for delimiter escaping
    fn on_invoke(&self) -> bool {
        if self.paren_stack.is_empty() {
            return false;
        }

        self.paren_stack.last().unwrap().macro_invocation
    }

    fn validate_arg_no_return(param: &Parameter, source: &str) -> RadResult<()> {
        use crate::ArgableStr;
        source.is_argable(param)?;
        Ok(())
    }

    fn validate_arg<'a>(
        param: &Parameter,
        source: Cow<'a, str>,
        etable: Option<&ETMap>,
    ) -> RadResult<Argument<'a>> {
        use crate::ArgableCow;
        let et = if let Some(etos) = etable {
            etos.tables.get(&param.name)
        } else {
            None
        };
        source.to_arg(
            param, // TODO TT
            et,
        )
    }
}

#[derive(Debug)]
pub struct ParenStack {
    macro_invocation: bool,
}
