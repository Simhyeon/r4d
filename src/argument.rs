use itertools::Itertools;
use regex::Regex;
use std::borrow::Cow;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::common::{ETMap, ETable, MacroAttribute, PipeInput};
use crate::consts::RET_ETABLE;
use crate::env::PROC_ENV;
use crate::runtime_map::RuntimeMacro;
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
    pub enum_table: Option<&'a ETMap>,
    pub piped_args: Option<Vec<String>>,
    pub attr: MacroAttribute,
    pub name: &'a str,
    pub args: &'a str,
    pub level: usize,
}

impl<'a> MacroInput<'a> {
    pub fn new(name: &'a str, args: &'a str) -> Self {
        Self {
            params: Vec::new(),
            optional: None,
            attr: MacroAttribute::default(),
            piped_args: None,
            enum_table: None,
            name,
            args,
            level: 0,
        }
    }

    pub(crate) fn add_pipe_input(&mut self, pipe_input: PipeInput, piped: Option<Vec<String>>) {
        match pipe_input {
            PipeInput::Vector => self.piped_args = piped,
            PipeInput::Single => self.piped_args = piped.map(|s| vec![s.join(",")]),
            _ => (),
        }
    }

    pub fn level(mut self, level: usize) -> Self {
        self.level = level;
        self
    }

    pub fn enum_table(mut self, table: &'a ETMap) -> Self {
        self.enum_table.replace(table);
        self
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
    pub arg_type: ValueType,
}

impl Parameter {
    pub fn new(at: ValueType, name: &str) -> Self {
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

pub(crate) trait ArgableCow<'a> {
    fn to_arg(self, param: &Parameter, candidates: Option<&ETable>) -> RadResult<Argument<'a>>;
    fn to_expanded(&self, p: &mut Processor, input: &ExInput) -> RadResult<String>;
}

impl<'a> ArgableCow<'a> for Cow<'a, str> {
    /// Intenal method for primary conversion
    fn to_arg(self, param: &Parameter, candidates: Option<&ETable>) -> RadResult<Argument<'a>> {
        let arg = match param.arg_type {
            ValueType::None => unreachable!("This is a logic error"),
            ValueType::Bool => Argument::Bool(self.is_arg_true().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "[Parameter: {}] : Could not convert a given value \"{}\" into a type [Bool]",
                    param.name, self
                ))
            })?),
            ValueType::Int => Argument::Int(self.trim().parse::<isize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "[Parameter: {}] : Could not convert a given value \"{}\" into a type [Int]",
                    param.name, self
                ))
            })?),
            ValueType::Uint => Argument::Uint(self.trim().parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "[Parameter: {}] : Could not convert a given value \"{}\" into a type [UInt]",
                    param.name, self
                ))
            })?),
            ValueType::Float => Argument::Float(self.trim().parse::<f32>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "[Parameter: {}] : Could not convert a given value \"{}\" into a type [Float]",
                    param.name, self
                ))
            })?),
            ValueType::Path => Argument::Path(PathBuf::from(self.as_ref())),
            ValueType::CText => match self {
                Cow::Owned(v) => Argument::Text(Cow::Owned(v.trim().to_string())),
                Cow::Borrowed(v) => Argument::Text(Cow::Borrowed(v.trim())),
            },
            ValueType::Text | ValueType::Regex => Argument::Text(self),
            ValueType::Enum => {
                if candidates.is_none() {
                    return Err(RadError::InvalidExecution(format!(
                    "[Parameter: {}] : Could not convert a given value \"{}\" into a type [Enum] because etable was empty.",
                    param.name, self
                )));
                }

                let tab = candidates.unwrap();
                let comparator = self.trim().to_lowercase();
                let err = tab
                    .candidates
                    .iter()
                    .filter(|&s| s == &comparator)
                    .collect_vec()
                    .is_empty();

                if err {
                    return Err(RadError::InvalidArgument(format!(
                        "[Parameter: {}] : Could not convert a given value \"{}\" into a value among {:?}",
                        param.name, self, tab.candidates
                    )));
                }

                Argument::Text(self)
            }
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
    Enum(String),
    Bool(bool),
    Uint(usize),
    Int(isize),
    Path(PathBuf),
    Float(f32),
    Regex(&'a str),
}

impl<'a> Display for Argument<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = match self {
            Self::Text(v) => v.to_string(),
            Self::Bool(v) => v.to_string(),
            Self::Uint(v) => v.to_string(),
            Self::Int(v) => v.to_string(),
            Self::Path(v) => v.display().to_string(),
            Self::Float(v) => v.to_string(),
            Self::Regex(v) => v.to_string(),
            Self::Enum(v) => v.to_string(),
        };
        write!(f, "{}", ret)
    }
}

#[derive(Debug)]
pub enum Raturn {
    None,
    Text(String),
    Enum(String),
    Bool(bool),
    Uint(usize),
    Int(isize),
    Path(PathBuf),
    Float(f32),
    Regex(String),
}

impl Raturn {
    pub fn from_string(
        rule: &RuntimeMacro,
        value: impl Into<String>,
        candidates: Option<ETable>,
    ) -> RadResult<Self> {
        let value: String = value.into();

        #[inline]
        fn make_err(value: &str, rule: &RuntimeMacro) -> RadError {
            RadError::InvalidConversion(format!(
                "[RET] : Could not convert a given value \"{}\" into a type [{}]",
                value, rule.return_type
            ))
        }

        let ret = match rule.return_type {
            ValueType::None => Self::None,
            ValueType::Bool => value
                .parse::<bool>()
                .map_err(|_| make_err(&value, rule))?
                .into(),
            ValueType::Float => value
                .parse::<f32>()
                .map_err(|_| make_err(&value, rule))?
                .into(),
            ValueType::Uint => value
                .parse::<usize>()
                .map_err(|_| make_err(&value, rule))?
                .into(),
            ValueType::Int => value
                .parse::<isize>()
                .map_err(|_| make_err(&value, rule))?
                .into(),
            ValueType::Path => PathBuf::from(value).into(),
            ValueType::Text | ValueType::Regex => Raturn::Text(value),
            ValueType::CText => Raturn::Text(value.trim().to_string()),
            ValueType::Enum => {
                if candidates.is_none() {
                    return Err(RadError::InvalidConversion(String::new()));
                }

                let tab = candidates.unwrap();
                let comparator = value.trim().to_lowercase();
                let err = tab
                    .candidates
                    .iter()
                    .filter(|&s| s == &comparator)
                    .collect_vec()
                    .is_empty();

                if err {
                    return Err(RadError::InvalidArgument(format!(
                        "[RET] : Could not convert a given value \"{}\" into a value among {:?}",
                        value, tab.candidates
                    )));
                }
                Raturn::Text(value.trim().to_string())
            }
        };

        Ok(ret)
    }

    pub fn convert_empty_to_none(self) -> Self {
        if PROC_ENV.no_consume {
            return self;
        }
        if let Self::Text(text) = &self {
            if text.is_empty() {
                return Self::None;
            }
        }
        self
    }

    pub fn negate(self) -> RadResult<Self> {
        let negated_arg = match self {
            Self::Bool(v) => (!v).into(),
            Self::Uint(v) => (-(v as isize)).into(),
            Self::Int(v) => (-v).into(),
            Self::Float(v) => (-v).into(),
            _ => {
                return Err(RadError::InvalidArgument(format!(
                    "Failed to negate a value \"{}\" because it is not logically available",
                    self
                )))
            }
        };

        Ok(negated_arg)
    }

    /// THis is a legacy method used among processor pipeline
    ///
    /// use to_string method if you want simple string form
    pub fn printable(self) -> Option<String> {
        match self {
            Self::None => None,
            _ => Some(self.to_string()),
        }
    }

    pub fn get_type(&self) -> ValueType {
        match self {
            Self::None => ValueType::None,
            Self::Text(_) => ValueType::Text,
            Self::Bool(_) => ValueType::Bool,
            Self::Uint(_) => ValueType::Uint,
            Self::Int(_) => ValueType::Int,
            Self::Path(_) => ValueType::Path,
            Self::Float(_) => ValueType::Float,
            Self::Regex(_) => ValueType::Regex,
            Self::Enum(_) => ValueType::Enum,
        }
    }
}

impl From<PathBuf> for Raturn {
    fn from(value: PathBuf) -> Self {
        Raturn::Path(value)
    }
}

impl From<bool> for Raturn {
    fn from(value: bool) -> Self {
        Raturn::Bool(value)
    }
}

impl From<f32> for Raturn {
    fn from(value: f32) -> Self {
        Raturn::Float(value)
    }
}

impl From<usize> for Raturn {
    fn from(value: usize) -> Self {
        Raturn::Uint(value)
    }
}

impl From<isize> for Raturn {
    fn from(value: isize) -> Self {
        Raturn::Int(value)
    }
}

impl From<String> for Raturn {
    fn from(value: String) -> Self {
        Raturn::Text(value)
    }
}

impl From<Option<String>> for Raturn {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(vv) => Raturn::Text(vv),
            None => Raturn::None,
        }
    }
}

impl From<Option<&str>> for Raturn {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some(vv) => Raturn::Text(vv.to_string()),
            None => Raturn::None,
        }
    }
}

impl From<&str> for Raturn {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl Display for Raturn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = match self {
            Self::None => String::new(),
            Self::Text(v) => v.to_string(),
            Self::Bool(v) => v.to_string(),
            Self::Uint(v) => v.to_string(),
            Self::Int(v) => v.to_string(),
            Self::Path(v) => v.display().to_string(),
            Self::Float(v) => v.to_string(),
            Self::Regex(v) => v.to_string(),
            Self::Enum(v) => v.to_string(),
        };
        write!(f, "{}", ret)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValueType {
    None,
    Bool,
    Float,
    Uint,
    Int,
    Path,
    Text,
    CText,
    Enum,
    Regex,
}

impl Default for ValueType {
    fn default() -> Self {
        Self::Text
    }
}

impl ValueType {
    pub fn is_valid_return_type(&self, ret: &Raturn, etable: Option<&ETable>) -> RadResult<()> {
        let ret_type = ret.get_type();

        // If argument's type is not same with given type => Error
        if *self != ret_type {
            return Err(RadError::InvalidExecution(format!(
                "Return type out of sync. Expected : \"{}\" type but got \"{}\" type",
                self, ret_type
            )));
        }

        if let Raturn::Enum(inn) = ret {
            // If enum doesn't match variatns => Error
            if let Err(RadError::InvalidArgument(stros)) =
                Cow::Borrowed(inn.as_str()).to_arg(&Parameter::new(*self, RET_ETABLE), etable)
            {
                return Err(RadError::InvalidExecution(
                    stros
                        + &format!(
                            "
= Expected a return type [{}] but validation failed",
                            self
                        ),
                ));
            }
        }

        Ok(())
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Bool => "Bool",
            Self::CText => "CText",
            Self::Enum => "Enum",
            Self::Float => "Float",
            Self::Int => "Int",
            Self::Path => "Path",
            Self::Text => "Text",
            Self::Uint => "Uint",
            Self::Regex => "Regex",
            Self::None => "NONE",
        };
        write!(f, "{}", text)
    }
}

impl FromStr for ValueType {
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
#[allow(dead_code)] // TODO TT
pub(crate) struct ParsedCursors<'a> {
    src: &'a str,
    level: usize,
    macro_name: String,
    trim_input: bool,
    params: Vec<Parameter>,
    cursors: Vec<ArgCursor>,
    piped: Vec<String>,
}

impl<'a> ParsedCursors<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            params: Vec::new(),
            cursors: Vec::new(),
            level: 0,
            macro_name: String::new(),
            trim_input: false,
            piped: vec![],
        }
    }

    pub(crate) fn trim(mut self, trim: bool) -> Self {
        self.trim_input = trim;
        self
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

    pub fn piped(mut self, pipe_input: Option<Vec<String>>) -> Self {
        self.piped.extend(pipe_input.unwrap_or_default());
        self
    }

    pub fn is_empty(&self) -> bool {
        self.cursors.is_empty()
    }

    // TODO TT
    // Notify the value that user tried to get
    /// Internal method before get_{type}
    ///
    /// Trim is applied when get is called
    fn get(&'a self, index: usize) -> RadResult<Cow<'a, str>> {
        // Get argument from piped_input
        if index >= self.cursors.len() {
            let idx = index.saturating_sub(self.cursors.len());
            let src = self.piped.get(idx).ok_or(RadError::InvalidExecution(
                "Failed to get argument by index. [Index out of error]".to_string(),
            ))?;

            if self.trim_input {
                return Ok(src.trim().into());
            } else {
                return Ok(src.into());
            }
        }

        let cursor = self.cursors.get(index).ok_or(RadError::InvalidExecution(
            "Failed to get argument by index. [Index out of error]".to_string(),
        ))?;
        match cursor {
            ArgCursor::Reference(star, end) => {
                let mut src = &self.src[*star..*end];
                if self.trim_input {
                    src = src.trim();
                }
                Ok(src.into())
            }
            ArgCursor::Modified(val) => {
                let mut src = std::str::from_utf8(&val[..]).unwrap();
                if self.trim_input {
                    src = src.trim();
                }
                Ok(src.to_string().into())
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

        match expanded.to_arg(&self.params[input.index], None) {
            Ok(Argument::Bool(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidExecution(
                "Failed to get correct argument \
as given type. You should use proper getter for the type"
                    .to_string(),
            )),
        }
    }

    pub fn get_path(&'a self, p: &mut Processor, index: usize) -> RadResult<PathBuf> {
        let input = ExInput::new(&self.macro_name)
            .index(index)
            .level(self.level);
        let expanded: Cow<'a, str> = self.get(input.index)?.to_expanded(p, &input)?.into();
        match expanded.to_arg(&self.params[input.index], None) {
            Ok(Argument::Path(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidExecution(
                "Failed to get correct argument \
as given type. You should use proper getter for the type"
                    .to_string(),
            )),
        }
    }

    pub fn get_uint(&'a self, p: &mut Processor, index: usize) -> RadResult<usize> {
        let input = ExInput::new(&self.macro_name)
            .index(index)
            .level(self.level);
        let expanded: Cow<'a, str> = self.get(input.index)?.to_expanded(p, &input)?.into();
        match expanded.to_arg(&self.params[input.index], None) {
            Ok(Argument::Uint(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidExecution(
                "Failed to get correct argument \
as given type. You should use proper getter for the type"
                    .to_string(),
            )),
        }
    }

    pub fn get_int(&'a self, p: &mut Processor, index: usize) -> RadResult<isize> {
        let input = ExInput::new(&self.macro_name)
            .index(index)
            .level(self.level);
        let expanded: Cow<'a, str> = self.get(input.index)?.to_expanded(p, &input)?.into();
        match expanded.to_arg(&self.params[input.index], None) {
            Ok(Argument::Int(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidExecution(
                "Failed to get correct argument \
as given type. You should use proper getter for the type"
                    .to_string(),
            )),
        }
    }

    pub fn get_float(&'a self, p: &mut Processor, index: usize) -> RadResult<f32> {
        let input = ExInput::new(&self.macro_name)
            .index(index)
            .level(self.level);
        let expanded: Cow<'a, str> = self.get(input.index)?.to_expanded(p, &input)?.into();
        match expanded.to_arg(&self.params[input.index], None) {
            Ok(Argument::Float(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidExecution(
                "Failed to get correct argument \
as given type. You should use proper getter for the type"
                    .to_string(),
            )),
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

    // Currently this is simply a wrapper around text
    pub fn get_regex<'b>(&'a self, p: &'b mut Processor, index: usize) -> RadResult<String> {
        self.get_text(p, index)
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
pub(crate) struct ParsedArguments<'a> {
    args: Vec<Argument<'a>>,
}

impl<'a> ParsedArguments<'a> {
    pub fn empty() -> Self {
        Self { args: Vec::new() }
    }

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
    pub(crate) fn get(&'a self, index: usize) -> RadResult<&Argument<'a>> {
        match self.args.get(index) {
            Some(val) => Ok(val),
            None => Err(crate::RadError::InvalidExecution(
                "Argument index out of range".to_string(),
            )),
        }
    }

    pub fn get_bool(&'a self, index: usize) -> RadResult<bool> {
        match self.args.get(index) {
            Some(Argument::Bool(val)) => Ok(*val),
            _ => Err(crate::RadError::InvalidExecution(format!(
                "Failed to get argument as bool \
. Tried to refer a value {:?}",
                self.args.get(index)
            ))),
        }
    }

    pub fn get_path(&'a self, index: usize) -> RadResult<&'a Path> {
        match self.args.get(index) {
            Some(Argument::Path(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidExecution(format!(
                "Failed to get argument as path \
. Tried to refer a value {:?}",
                self.args.get(index)
            ))),
        }
    }

    pub fn get_text(&'a self, index: usize) -> RadResult<&str> {
        match self.args.get(index) {
            Some(Argument::Text(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidExecution(format!(
                "Failed to get argument as text \
. Tried to refer a value {:?}",
                self.args.get(index)
            ))),
        }
    }

    pub fn get_ctext(&'a self, index: usize) -> RadResult<&str> {
        match self.args.get(index) {
            Some(Argument::Text(val)) => Ok(val.trim()),
            _ => Err(crate::RadError::InvalidExecution(format!(
                "Failed to get argument as compact text \
. Tried to refer a value {:?}",
                self.args.get(index)
            ))),
        }
    }

    pub fn get_regex<'b>(&'a self, index: usize) -> RadResult<&str> {
        match self.args.get(index) {
            Some(Argument::Regex(val)) => Ok(val),
            _ => Err(crate::RadError::InvalidExecution(format!(
                "Failed to get argument as regex \
. Tried to refer a value {:?}",
                self.args.get(index)
            ))),
        }
    }

    pub fn get_uint(&'a self, index: usize) -> RadResult<usize> {
        match self.args.get(index) {
            Some(Argument::Uint(val)) => Ok(*val),
            _ => Err(crate::RadError::InvalidExecution(format!(
                "Failed to get argument as uint \
. Tried to refer a value {:?}",
                self.args.get(index)
            ))),
        }
    }

    pub fn get_int(&'a self, index: usize) -> RadResult<isize> {
        match self.args.get(index) {
            Some(Argument::Int(val)) => Ok(*val),
            _ => Err(crate::RadError::InvalidExecution(format!(
                "Failed to get argument as int \
. Tried to refer a value {:?}",
                self.args.get(index)
            ))),
        }
    }

    pub fn get_float(&'a self, index: usize) -> RadResult<f32> {
        match self.args.get(index) {
            Some(Argument::Float(val)) => Ok(*val),
            _ => Err(crate::RadError::InvalidExecution(format!(
                "Failed to get argument as float \
. Tried to refer a value {:?}",
                self.args.get(index)
            ))),
        }
    }

    pub fn get_enum<T>(&'a self, index: usize, f: fn(&str) -> RadResult<T>) -> RadResult<T> {
        let source = match self.args.get(index) {
            Some(Argument::Text(text)) => Ok(text),
            _ => Err(crate::RadError::InvalidExecution(format!(
                "Failed to get argument as enum \
. Tried to refer a value {:?}",
                self.args.get(index)
            ))),
        }?;

        // Convert to custom type
        f(source)
    }
}

#[derive(Debug)]
pub(crate) enum ArgCursor {
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

    /// Peek last invocation without taking
    pub fn peek_last_invocation<'a>(&'a self, src: &'a str) -> &str {
        let mut ret = match self {
            Self::Reference(s, e) => &src[*s..=*e.min(&(src.len() - 1))],
            Self::Modified(v) => std::str::from_utf8(&v[..]).unwrap(),
        };
        if let Some(rp_index) = ret.rfind(')') {
            let index = (rp_index + 1).min(ret.len() - 1);
            ret = (ret[index..]).trim_start();
        }
        ret
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
