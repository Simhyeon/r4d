use std::borrow::Cow;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::common::MacroAttribute;
use crate::RadStr;
use crate::{Processor, RadError, RadResult};
use serde::{Deserialize, Serialize};

pub(crate) struct ExInput<'a> {
    pub index: usize,
    pub trim: bool,

    pub macro_name: &'a str,
    pub level: usize,
}

impl<'a> ExInput<'a> {
    pub fn new(macro_name: &'a str) -> Self {
        Self {
            index: 0,
            level: 0,
            macro_name,
            trim: false,
        }
    }

    pub fn index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn level(mut self, level: usize) -> Self {
        self.level = level;
        self
    }

    pub fn trim(mut self) -> Self {
        self.trim = true;
        self
    }
}

#[derive(Debug)]
pub struct MacroInput<'a> {
    pub params: Vec<Parameter>,
    pub optional: Option<Parameter>,
    pub attr: MacroAttribute,
    pub name: &'a str,
    pub args: &'a str,
}

impl<'a> MacroInput<'a> {
    pub fn new(name: &'a str, args: &'a str) -> Self {
        Self {
            params: Vec::new(),
            optional: None,
            attr: MacroAttribute::default(),
            name,
            args,
        }
    }

    pub fn parameter(mut self, params: &[Parameter]) -> Self {
        self.params = params.to_vec();
        self
    }

    pub fn optional(mut self, param: Option<Parameter>) -> Self {
        self.optional = param;
        self
    }

    pub fn attr(mut self, attr: MacroAttribute) -> Self {
        self.attr = attr;
        self
    }

    pub fn type_len(&self) -> usize {
        self.params.len()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub arg_type: ArgType,
}

impl Parameter {
    pub fn new(at: ArgType, name: &str) -> Self {
        Self {
            name: name.to_string(),
            arg_type: at,
        }
    }
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {:#?}", self.name, self.arg_type)
    }
}
pub(crate) trait ArgableStr<'a> {
    fn is_argable(&self, param: &Parameter) -> RadResult<()>;
}
impl<'a> ArgableStr<'a> for str {
    fn is_argable(&self, param: &Parameter) -> RadResult<()> {
        match param.arg_type {
            ArgType::Bool => {
                self.is_arg_true().map_err(|_| {
                    RadError::InvalidArgument(format!(
                        "[Parameter: {}] : Could not convert a given value \"{}\" into a type [Bool]",
                        param.name, self
                    ))
                })?;
            }
            ArgType::Int => {
                self.trim().parse::<isize>().map_err(|_| {
                    RadError::InvalidArgument(format!(
                    "[Parameter: {}] : Could not convert a given value \"{}\" into a type [Int]",
                        param.name, self
                    ))
                })?;
            }
            ArgType::Uint => {
                self.trim().parse::<usize>().map_err(|_| {
                    RadError::InvalidArgument(format!(
                    "[Parameter: {}] : Could not convert a given value \"{}\" into a type [UInt]",
                        param.name, self
                    ))
                })?;
            }
            ArgType::Float => {
                self.trim().parse::<f32>().map_err(|_| {
                    RadError::InvalidArgument(format!(
                    "[Parameter: {}] : Could not convert a given value \"{}\" into a type [Float]",
                        param.name, self
                ))
                })?;
            }
            _ => (),
        };
        Ok(())
    }
}

pub(crate) trait ArgableCow<'a> {
    fn to_arg(self, param: &Parameter) -> RadResult<Argument<'a>>;
    fn to_expanded(&self, p: &mut Processor, input: &ExInput) -> RadResult<String>;
}

impl<'a> ArgableCow<'a> for Cow<'a, str> {
    fn to_arg(self, param: &Parameter) -> RadResult<Argument<'a>> {
        let arg = match param.arg_type {
            ArgType::Bool => Argument::Bool(self.is_arg_true().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "[Parameter: {}] : Could not convert a given value \"{}\" into a type [Bool]",
                    param.name, self
                ))
            })?),
            ArgType::Int => Argument::Int(self.trim().parse::<isize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "[Parameter: {}] : Could not convert a given value \"{}\" into a type [Int]",
                    param.name, self
                ))
            })?),
            ArgType::Uint => Argument::Uint(self.trim().parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "[Parameter: {}] : Could not convert a given value \"{}\" into a type [UInt]",
                    param.name, self
                ))
            })?),
            ArgType::Float => Argument::Float(self.trim().parse::<f32>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "[Parameter: {}] : Could not convert a given value \"{}\" into a type [Float]",
                    param.name, self
                ))
            })?),
            ArgType::Path => Argument::Path(PathBuf::from(self.as_ref())),
            ArgType::CText | ArgType::Text | ArgType::Enum => Argument::Text(self),
        };
        Ok(arg)
    }

    fn to_expanded(&self, p: &mut Processor, input: &ExInput) -> RadResult<String> {
        let arg = if input.trim { self.trim() } else { self };
        p.parse_chunk(input.level, input.macro_name, arg)
    }
}

#[derive(Debug)]
pub enum Argument<'a> {
    Text(Cow<'a, str>),
    Bool(bool),
    Uint(usize),
    Int(isize),
    Path(PathBuf),
    Float(f32),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ArgType {
    Bool,
    CText,
    Enum,
    Float,
    Int,
    Path,
    Text,
    Uint,
}

impl FromStr for ArgType {
    type Err = RadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ret = match s.trim().to_lowercase().as_str() {
            "b" | "bool" => Self::Bool,
            "c" | "ctext" => Self::CText,
            "e" | "enum" => Self::Enum,
            "f" | "float" => Self::Float,
            "i" | "int" => Self::Int,
            "p" | "path" => Self::Path,
            "t" | "text" => Self::Text,
            "u" | "uint" => Self::Uint,
            _ => {
                return Err(RadError::InvalidArgument(format!(
                    "Given type \"{}\" is not a valid argument type",
                    s
                )))
            }
        };
        Ok(ret)
    }
}

#[derive(Debug)]
pub struct ParsedCursors<'a> {
    src: &'a str,
    level: usize,
    macro_name: String,
    params: Vec<Parameter>,
    cursors: Vec<ArgCursor>,
}

impl<'a> ParsedCursors<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            params: Vec::new(),
            cursors: Vec::new(),
            level: 0,
            macro_name: String::new(),
        }
    }

    pub(crate) fn level(mut self, level: usize) -> Self {
        self.level = level;
        self
    }

    pub(crate) fn macro_name(mut self, name: String) -> Self {
        self.macro_name = name;
        self
    }

    pub fn with_params(mut self, params: Vec<Parameter>) -> Self {
        self.params = params;
        self
    }

    pub fn with_cursors(mut self, cursors: Vec<ArgCursor>) -> Self {
        self.cursors = cursors;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.cursors.is_empty()
    }

    // TODO TT
    // Notify the value that user tried to get
    fn get(&self, index: usize) -> RadResult<Cow<'a, str>> {
        let cursor = self
            .cursors
            .get(index)
            .ok_or(RadError::InvalidExecution("Index out of error".to_string()))?;
        match cursor {
            ArgCursor::Reference(star, end) => Ok(self.src[*star..*end].into()),
            ArgCursor::Modified(val) => {
                Ok(std::str::from_utf8(&val[..]).unwrap().to_string().into())
            }
        }
    }

    // TODO TT
    //
    // Currently I'm simply unwrapping, but getting params can always fail.

    // Getter
    pub fn get_bool(&'a self, p: &mut Processor, index: usize) -> RadResult<bool> {
        let input = ExInput::new(&self.macro_name)
            .index(index)
            .level(self.level);
        let expanded: Cow<'a, str> = self.get(input.index)?.to_expanded(p, &input)?.into();
        match expanded.to_arg(&self.params[input.index]) {
            Ok(Argument::Bool(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidArgument("".to_string())),
        }
    }

    pub fn get_path(&'a self, p: &mut Processor, index: usize) -> RadResult<PathBuf> {
        let input = ExInput::new(&self.macro_name)
            .index(index)
            .level(self.level);
        let expanded: Cow<'a, str> = self.get(input.index)?.to_expanded(p, &input)?.into();
        match expanded.to_arg(&self.params[input.index]) {
            Ok(Argument::Path(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidArgument("".to_string())),
        }
    }

    pub fn get_uint(&'a self, p: &mut Processor, index: usize) -> RadResult<usize> {
        let input = ExInput::new(&self.macro_name)
            .index(index)
            .level(self.level);
        let expanded: Cow<'a, str> = self.get(input.index)?.to_expanded(p, &input)?.into();
        match expanded.to_arg(&self.params[input.index]) {
            Ok(Argument::Uint(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidArgument("".to_string())),
        }
    }

    pub fn get_int(&'a self, p: &mut Processor, index: usize) -> RadResult<isize> {
        let input = ExInput::new(&self.macro_name)
            .index(index)
            .level(self.level);
        let expanded: Cow<'a, str> = self.get(input.index)?.to_expanded(p, &input)?.into();
        match expanded.to_arg(&self.params[input.index]) {
            Ok(Argument::Int(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidArgument("".to_string())),
        }
    }

    pub fn get_float(&'a self, p: &mut Processor, index: usize) -> RadResult<f32> {
        let input = ExInput::new(&self.macro_name)
            .index(index)
            .level(self.level);
        let expanded: Cow<'a, str> = self.get(input.index)?.to_expanded(p, &input)?.into();
        match expanded.to_arg(&self.params[input.index]) {
            Ok(Argument::Float(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidArgument("".to_string())),
        }
    }

    pub fn get_text(&'a self, p: &mut Processor, index: usize) -> RadResult<String> {
        let input = ExInput::new(&self.macro_name)
            .index(index)
            .level(self.level);
        self.get(input.index)?.to_expanded(p, &input)
    }

    pub fn get_ctext(&'a self, p: &mut Processor, index: usize) -> RadResult<String> {
        let input = ExInput::new(&self.macro_name)
            .index(index)
            .level(self.level)
            .trim();
        self.get(input.index)?.to_expanded(p, &input)
    }

    pub fn get_custom<T>(
        &'a self,
        p: &mut Processor,
        index: usize,
        f: fn(&str) -> RadResult<T>,
    ) -> RadResult<T> {
        let input = ExInput::new(&self.macro_name)
            .index(index)
            .level(self.level)
            .trim();
        let source = self.get(input.index)?.to_expanded(p, &input)?;

        // Conert to custom type
        f(&source)
    }
}

#[derive(Debug)]
pub struct ParsedArguments<'a> {
    args: Vec<Argument<'a>>,
}

impl<'a> ParsedArguments<'a> {
    pub fn with_args(args: Vec<Argument<'a>>) -> Self {
        Self { args }
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    // ---- GETTERS
    pub fn get(&'a self, index: usize) -> RadResult<&Argument<'a>> {
        match self.args.get(index) {
            Some(val) => Ok(val),
            None => Err(crate::RadError::InvalidArgument("".to_string())),
        }
    }

    pub fn get_bool(&'a self, index: usize) -> RadResult<bool> {
        match self.args.get(index) {
            Some(Argument::Bool(val)) => Ok(*val),
            _ => Err(crate::RadError::InvalidArgument("".to_string())),
        }
    }

    pub fn get_path(&'a self, index: usize) -> RadResult<&'a Path> {
        match self.args.get(index) {
            Some(Argument::Path(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidArgument("".to_string())),
        }
    }

    pub fn get_text(&'a self, index: usize) -> RadResult<&str> {
        match self.args.get(index) {
            Some(Argument::Text(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidArgument("".to_string())),
        }
    }

    pub fn get_ctext(&'a self, index: usize) -> RadResult<&str> {
        match self.args.get(index) {
            Some(Argument::Text(val)) => Ok(val.trim()),
            _ => Err(crate::RadError::InvalidArgument("".to_string())),
        }
    }

    pub fn get_uint(&'a self, index: usize) -> RadResult<usize> {
        match self.args.get(index) {
            Some(Argument::Uint(val)) => Ok(*val),
            _ => Err(crate::RadError::InvalidArgument("".to_string())),
        }
    }

    pub fn get_int(&'a self, index: usize) -> RadResult<isize> {
        match self.args.get(index) {
            Some(Argument::Int(val)) => Ok(*val),
            _ => Err(crate::RadError::InvalidArgument("".to_string())),
        }
    }

    pub fn get_float(&'a self, index: usize) -> RadResult<f32> {
        match self.args.get(index) {
            Some(Argument::Float(val)) => Ok(*val),
            _ => Err(crate::RadError::InvalidArgument("".to_string())),
        }
    }

    pub fn get_enum<T>(&'a self, index: usize, f: fn(&str) -> RadResult<T>) -> RadResult<T> {
        let arg = match self.args.get(index) {
            Some(val) => Ok(val),
            None => Err(crate::RadError::InvalidArgument("".to_string())),
        }?;
        let source = match arg {
            Argument::Text(text) => Ok(text),
            _ => Err(crate::RadError::InvalidArgument("".to_string())),
        }?;

        // Conert to custom type
        f(source)
    }
}

#[derive(Debug)]
pub enum ArgCursor {
    Reference(usize, usize),
    Modified(Vec<u8>),
}

impl Default for ArgCursor {
    fn default() -> Self {
        Self::Reference(0, 0)
    }
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

    pub fn take(&mut self, index: usize) -> Self {
        std::mem::replace(self, Self::Reference(index, index))
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
                None
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
