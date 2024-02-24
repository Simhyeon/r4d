//! # arg_parser
//!
//! Module about argument parsing

use crate::{
    argument::{ArgType, Argument, MacroInput, ParsedArguments},
    common::MacroAttribute,
    consts::{ESCAPE_CHAR_U8, LIT_CHAR_U8},
    RadError, RadResult,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;

pub static MACRO_START_MATCH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^\$\S*\($"#).expect("Failed to create name regex"));

/// State indicates whether argument should be parsed greedily or not
#[derive(Debug)]
pub enum SplitVariant {
    /// Split argument with given amount
    Amount(usize),
    Greedy,
    Always,
}

/// Argument parser
pub struct ArgParser {
    previous: Option<u8>,
    lit_count: usize,
    paren_stack: Vec<ParenStack>,
    no_previous: bool,
    strip_literal: bool,
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
    pub(crate) fn strip<'a>(&mut self, args: &'a str) -> Cow<'a, str> {
        let attribute = MacroAttribute::default();
        let mut stripped = self.args_to_vec(args, &attribute, b',', SplitVariant::Greedy);
        if let Some(val) = stripped.get_mut(0) {
            std::mem::take(val)
        } else {
            String::new().into()
        }
    }

    /// Check if given length is qualified for given raw arguments
    ///
    /// If length is qualified it returns vector of arguments
    /// if not, "None" is returned instead.
    pub(crate) fn args_with_len<'a>(
        &mut self,
        args: &'a str,
        attribute: &MacroAttribute,
        length: usize,
    ) -> Option<Vec<Cow<'a, str>>> {
        self.reset();
        if length == 0 && !args.is_empty() {
            return None;
        }
        let split_var = if length > 1 {
            SplitVariant::Amount(length - 1)
        } else {
            SplitVariant::Greedy
        };

        let args: Vec<_> = self.args_to_vec(args, attribute, b',', split_var);

        if args.len() < length {
            return None;
        }
        Some(args)
    }

    /// Split raw arguments into a vector
    pub(crate) fn args_to_vec<'b>(
        &mut self,
        arg_values: &'b str,
        attribute: &MacroAttribute,
        delimiter: u8,
        mut split_var: SplitVariant,
    ) -> Vec<Cow<'b, str>> {
        let mut values: Vec<Cow<'b, str>> = vec![];
        self.reset();
        let mut cursor = ArgCursor::Reference(0, 0);
        let trim = attribute.trim_input;

        // This is totally ok to iterate as char_indices rather than chars
        // because "only ASCII char is matched" so there is zero possibilty that
        // char_indices will make any unexpected side effects.
        let arg_iter = arg_values.as_bytes().iter().enumerate();

        // Return empty vector without going through logics
        if arg_values.is_empty() {
            return vec![];
        }

        for (idx, &ch) in arg_iter {
            // Check parenthesis
            self.check_parenthesis(&mut cursor, ch, &arg_values);

            if ch == delimiter {
                if let Some(v) =
                    self.branch_delimiter(&mut cursor, ch, idx, &mut split_var, arg_values, trim)
                {
                    values.push(v);
                }
            } else if ch == ESCAPE_CHAR_U8 {
                self.branch_escape_char(&mut cursor, ch, &arg_values);
            } else {
                // This pushes value in the end, so use continue not push the value
                if ch == LIT_CHAR_U8 {
                    // '*'
                    self.branch_literal_char(&mut cursor, ch, &arg_values);
                } else {
                    // Non literal character are just pushed
                    cursor.push(&[ch]);
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
        let sc = if cursor.is_string() { "" } else { arg_values };
        values.push(cursor.take_value(arg_values.len(), sc, trim));
        values
    }

    /// Check parenthesis for sensible splitting
    fn check_parenthesis(&mut self, cursor: &mut ArgCursor, ch: u8, src: &&str) {
        if self.previous.unwrap_or(b'0') == ESCAPE_CHAR_U8 && (ch == b'(' || ch == b')') {
            cursor.convert_to_modified(src);
            cursor.pop();
            self.previous.replace(b'0');
            return;
        }

        // Early return
        if self.lit_count != 0 {
            return;
        }

        if ch == b'(' {
            let stack = if MACRO_START_MATCH.find(cursor.peek_value(src)).is_some() {
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

    /// Branch on delimiter found
    fn branch_delimiter<'a>(
        &'_ mut self,
        cursor: &mut ArgCursor,
        ch: u8,
        index: usize,
        variant: &mut SplitVariant,
        src: &'a str,
        trim: bool,
    ) -> Option<Cow<'a, str>> {
        let mut ret = None;
        // Either literal or escaped
        if self.lit_count > 0 {
            cursor.push(&[ch]);
        } else if self.previous.unwrap_or(b'0') == ESCAPE_CHAR_U8 {
            cursor.convert_to_modified(src);
            cursor.pop();
            cursor.push(&[ch]);
        } else if self.on_invoke() {
            // If quote is inside parenthesis, simply push it into a value
            cursor.push(&[ch]);
        } else {
            // not literal
            match variant {
                SplitVariant::Amount(count) => {
                    // move to next value
                    let sc = if cursor.is_string() { "" } else { src };
                    let v = cursor.take_value(index + 1, sc, trim);
                    ret.replace(v);
                    let count = *count - 1;
                    if count > 0 {
                        *variant = SplitVariant::Amount(count);
                    } else {
                        *variant = SplitVariant::Greedy;
                    }
                    self.no_previous = true;
                }
                // Push everything to current item, index, value or you name it
                SplitVariant::Greedy => {
                    cursor.push(&[ch]);
                }
                SplitVariant::Always => {
                    // move to next value
                    let sc = if cursor.is_string() { "" } else { src };
                    let v = cursor.take_value(index + 1, sc, trim);
                    ret.replace(v);
                }
            } // Match end
        } // else end
        ret
    }

    // \\ -> \\
    // \( -> (
    // \) -> )
    // \, -> ,

    /// Branch on escape character found
    fn branch_escape_char(&mut self, cursor: &mut ArgCursor, ch: u8, src: &&str) {
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
                    cursor.convert_to_modified(src);

                    // Remove \
                    cursor.pop();
                } else {
                    cursor.push(&[ch]);
                }
            } else {
                // Inside other literal rules
                cursor.push(&[ch]);
            }
        } else {
            // Simply put escape character as part of arguments
            cursor.push(&[ch]);
        }
    } // end function

    /// Branch on literal character found
    ///
    /// Literal character in this term means '*'
    fn branch_literal_char(&mut self, cursor: &mut ArgCursor, ch: u8, src: &&str) {
        if self.previous.unwrap_or(b'0') == ESCAPE_CHAR_U8 {
            self.lit_count += 1;
            // If lit character was given inside literal
            // e.g. \* '\*' *\ -> the one inside quotes
            if self.lit_count > 1 {
                cursor.push(&[ch]);
            } else {
                // First lit character in given args
                // Simply ignore character and don't set previous
                self.no_previous = true;

                // If no strip
                // Push all literal specifier characters
                if !self.strip_literal {
                    cursor.push(&[ch]);
                } else {
                    cursor.convert_to_modified(src);
                    // If strip then strip all related charaters
                    cursor.pop();
                }
            }
        } else {
            cursor.push(&[ch]);
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
}
// --- ENd of arg parser

/// Argument parser
pub struct NewArgParser {
    // Input related
    delimiter: u8,
    trim: bool,
    split_variant: SplitVariant,
    no_type: bool,

    // Stateful
    previous: Option<u8>,
    lit_count: usize,
    paren_stack: Vec<ParenStack>,
    no_previous: bool,
    strip_literal: bool,
    cursor: ArgCursor,
}

impl NewArgParser {
    /// Create a new instance
    pub(crate) fn new() -> Self {
        Self {
            previous: None,
            lit_count: 0,
            no_type: false,
            paren_stack: vec![],
            no_previous: false,
            strip_literal: true,
            cursor: ArgCursor::Reference(0, 0),
            delimiter: b',',
            trim: false,
            split_variant: SplitVariant::Greedy,
        }
    }

    /// Reset variables
    #[deprecated(note = "Possibly wrong")]
    fn reset(&mut self) {
        self.previous = None;
        self.lit_count = 0;
        self.no_previous = false;
        self.cursor = ArgCursor::Reference(0, 0);
        self.paren_stack.clear();
    }

    /// Don't strip literals
    pub(crate) fn no_strip(mut self) -> Self {
        self.strip_literal = false;
        self
    }

    pub(crate) fn set_split(mut self, split_var: SplitVariant) -> Self {
        self.split_variant = split_var;
        self
    }

    pub(crate) fn with_delimiter(mut self, delim: u8) -> Self {
        self.delimiter = delim;
        self
    }

    /// Don't strip literals
    pub(crate) fn set_strip(&mut self, strip_literal: bool) {
        self.strip_literal = strip_literal;
    }

    /// Simply strip literal chunk
    pub(crate) fn strip<'a>(&mut self, args: &'a str) -> RadResult<ParsedArguments<'a>> {
        self.args_with_len(MacroInput::new(args))
    }

    /// Check if given length is qualified for given raw arguments
    ///
    /// If length is qualified it returns vector of arguments
    /// if not, "None" is returned instead.
    pub(crate) fn args_with_len<'a>(
        &mut self,
        input: MacroInput<'a>,
    ) -> RadResult<ParsedArguments<'a>> {
        let length = input.type_len();
        if length == 0 && !input.args.is_empty() {
            return Err(RadError::InvalidArgument("Empty argument - 1".to_string()));
        }
        self.split_variant = if length > 1 {
            SplitVariant::Amount(length - 1)
        } else {
            SplitVariant::Greedy
        };

        let args = self.args_to_vec(input)?;

        if args.len() < length {
            return Err(RadError::InvalidArgument(
                "Insufficient arguments".to_string(),
            ));
        }
        Ok(ParsedArguments::with_args(args))
    }

    /// Split raw arguments into a vector
    pub(crate) fn args_to_vec<'a>(
        &'_ mut self,
        input: MacroInput<'a>,
    ) -> RadResult<Vec<Argument<'a>>> {
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

        // Return empty vector without going through logics
        if input.args.is_empty() {
            return Err(RadError::InvalidArgument("Empty argument - 2".to_string()));
        }

        for (idx, &ch) in arg_iter {
            // Check parenthesis
            self.check_parenthesis(ch, &input.args);

            if ch == self.delimiter {
                // TODO TT
                // This is not a neat solution it works...
                let mut skip = false;
                let ret =
                    self.branch_delimiter(ch, idx, &mut skip, (&mut start, &mut end), input.args);

                if !skip {
                    let value: Cow<'_, str> = if let Some(v) = ret {
                        v.into()
                    } else {
                        input.args[start..end].into()
                    };
                    let arg = Self::validate_arg(at_iter.next().unwrap().arg_type, value)?;
                    values.push(arg);
                }
            } else if ch == ESCAPE_CHAR_U8 {
                self.branch_escape_char(ch, &input.args);
            } else {
                // This pushes value in the end, so use continue not push the value
                if ch == LIT_CHAR_U8 {
                    // '*'
                    self.branch_literal_char(ch, &input.args);
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

        let type_checked_arg = Self::validate_arg(at_iter.next().unwrap().arg_type, value)?;

        values.push(type_checked_arg);
        Ok(values)
    }

    /// Branch on delimiter found
    fn branch_delimiter(
        &mut self,
        ch: u8,
        index: usize,
        skip_split: &mut bool,
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
                    self.no_previous = true;
                }
                // Push everything to current item, index, value or you name it
                SplitVariant::Greedy => {
                    self.cursor.push(&[ch]);
                    *skip_split = true;
                }
                SplitVariant::Always => {
                    ret = self
                        .cursor
                        .get_cursor_range_or_get_string(index + 1, self.trim, range);
                }
            } // Match end
        } // else end
        ret
    }

    /// Check parenthesis for sensible splitting
    fn check_parenthesis(&mut self, ch: u8, src: &&str) {
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
                .find(self.cursor.peek_value(src))
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
    fn branch_escape_char(&mut self, ch: u8, src: &&str) {
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
    fn branch_literal_char(&mut self, ch: u8, src: &&str) {
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
    // -- <TEST>
    fn validate_arg(arg_type: ArgType, source: Cow<'_, str>) -> RadResult<Argument> {
        use crate::Argable;
        source.to_arg(arg_type)
    }
    // -- </TEST>
}

#[derive(Debug)]
enum ArgCursor {
    Reference(usize, usize),
    Modified(Vec<u8>),
}

impl ArgCursor {
    pub fn is_string(&self) -> bool {
        matches!(self, Self::Modified(_))
    }

    #[allow(dead_code)]
    pub fn debug(&self, src: &str) {
        match self {
            Self::Reference(a, b) => {
                eprintln!(">>> -{}-", &src[*a..*b]);
            }
            Self::Modified(vec) => {
                eprintln!(">>> -{}-", std::str::from_utf8(vec).unwrap());
            }
        }
    }

    /// Peek value without taking
    pub fn peek_value<'a>(&'a self, src: &'a str) -> &str {
        match self {
            Self::Reference(s, e) => &src[*s..*e],
            Self::Modified(v) => std::str::from_utf8(&v[..]).unwrap(),
        }
    }

    pub fn get_cursor_range_or_get_string(
        &mut self,
        index: usize,
        trim: bool,
        (start, end): (&mut usize, &mut usize),
    ) -> Option<String> {
        let ret = match self {
            Self::Reference(c, n) => {
                *start = *c;
                *end = *n;
                return None;
            }

            // TODO
            // Check this so that any error can be captured
            // THis is mostsly ok to unwrap because input source is
            Self::Modified(s) => {
                let stred = std::str::from_utf8(&s[..]).unwrap();
                if trim {
                    stred.trim().to_string().into()
                } else {
                    stred.to_string().into()
                }
            }
        };
        *self = Self::Reference(index, index);
        ret
    }

    /// Use "is_string" before taking value and supply empty if the inner vaule is string
    ///
    /// because src is supplied as is while the argument is completely ignored when the inner value
    /// is a string.
    pub fn take_value<'a>(&'_ mut self, index: usize, src: &'a str, trim: bool) -> Cow<'a, str> {
        let ret = match self {
            Self::Reference(c, n) => {
                let val = &src[*c..*n];
                if trim {
                    val.trim().into()
                } else {
                    val.into()
                }
            }

            // TODO
            // Check this so that any error can be captured
            // THis is mostsly ok to unwrap because input source is
            Self::Modified(s) => {
                let stred = std::str::from_utf8(&s[..]).unwrap();
                if trim {
                    stred.trim().to_string().into()
                } else {
                    stred.to_string().into()
                }
            }
        };
        *self = Self::Reference(index, index);
        ret
    }

    pub fn convert_to_modified(&mut self, src: &str) {
        if let Self::Reference(c, n) = self {
            *self = Self::Modified(src[*c..*n].into())
        }
    }

    pub fn push(&mut self, ch: &[u8]) {
        match self {
            Self::Reference(_, n) => *n += 1,
            Self::Modified(st) => st.extend_from_slice(ch),
        }
    }
    pub fn pop(&mut self) {
        if let Self::Modified(st) = self {
            st.pop();
        }
    }
}

pub struct ParenStack {
    macro_invocation: bool,
}
