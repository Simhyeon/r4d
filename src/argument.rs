use std::borrow::Cow;
use std::fmt::Display;
use std::path::PathBuf;

use crate::common::MacroAttribute;
use crate::RadError;
use crate::RadResult;
use crate::RadStr;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub(crate) struct MacroInput<'a> {
    pub params: Vec<Parameter>,
    pub attr: MacroAttribute,
    pub args: &'a str,
}

impl<'a> MacroInput<'a> {
    pub fn new(args: &'a str) -> Self {
        Self {
            params: Vec::new(),
            attr: MacroAttribute::default(),
            args,
        }
    }

    pub fn parameter(mut self, params: &[Parameter]) -> Self {
        self.params = params.to_vec();
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
pub(crate) struct Parameter {
    pub name: String,
    pub arg_type: ArgType,
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {:#?}", self.name, self.arg_type)
    }
}

pub(crate) trait Argable<'a> {
    fn to_arg(self, arg_type: ArgType) -> RadResult<Argument<'a>>;
}

impl<'a> Argable<'a> for Cow<'a, str> {
    fn to_arg(self, arg_type: ArgType) -> RadResult<Argument<'a>> {
        let arg = match arg_type {
            ArgType::Bool => Argument::Bool(self.is_arg_true()?),
            ArgType::Int => Argument::Int(self.trim().parse::<isize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a number",
                    self
                ))
            })?),
            ArgType::Uint => Argument::Uint(self.trim().parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a positive number",
                    self
                ))
            })?),
            ArgType::Float => Argument::Float(self.trim().parse::<f32>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a floating point number",
                    self
                ))
            })?),
            ArgType::Path | ArgType::File => Argument::Path(PathBuf::from(self.as_ref())),
            ArgType::CText | ArgType::Text | ArgType::Enum => Argument::Text(self),
        };
        Ok(arg)
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
    File,
    Float,
    Int,
    Path,
    Text,
    Uint,
}

#[derive(Debug)]
pub struct ParsedArguments<'a> {
    args: Vec<Argument<'a>>,
}

impl<'a> ParsedArguments<'a> {
    pub fn with_args(args: Vec<Argument<'a>>) -> Self {
        Self { args }
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

    pub fn get_custom<T>(&'a self, index: usize, f: fn(&str) -> RadResult<T>) -> RadResult<T> {
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
