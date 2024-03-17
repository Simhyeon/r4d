//! Common structs, enums for code usage.

use crate::argument::ValueType;
use crate::error::RadError;
use crate::Parameter;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::File;
use std::fs::OpenOptions;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(feature = "rustc_hash")]
use rustc_hash::FxHashMap as HashMap;
#[cfg(not(feature = "rustc_hash"))]
use std::collections::HashMap;

// Stream related static strings
pub(crate) static STREAM_CONTAINER: &str = "!STREAM_CONTAINER";

/// Genenric result type for every rad operations
///
/// RadResult is a genric result type of T and error of [RadError](RadError)
pub type RadResult<T> = Result<T, RadError>;

/// State enum value about direction of processed text
///
/// - File       : Set file output
/// - Variable   : Set variable to save
/// - Terminal   : Print to terminal
/// - Discard    : Do nothing
pub enum WriteOption<'a> {
    File(FileTarget),
    Variable(&'a mut String),
    Terminal,
    Discard,
}

impl<'a> WriteOption<'a> {
    /// Create a file type writeoption with path and open options
    pub fn file(path: &Path, open_option: OpenOptions) -> RadResult<Self> {
        let file = open_option.open(path).map_err(|_| {
            RadError::InvalidFile(format!("Cannot set write option to {}", path.display()))
        })?;
        Ok(Self::File(FileTarget::from_file(path, file)?))
    }
}

/// Local macro
#[derive(Clone, Debug)]
pub struct LocalMacro {
    pub level: usize,
    pub name: String,
    pub body: String,
}

impl LocalMacro {
    /// Create a new local macro
    pub fn new(level: usize, name: String, body: String) -> Self {
        Self { level, name, body }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MacroAttribute {
    // Macro attributes
    pub pipe_output: PipeOutput,
    pub pipe_input: PipeInput,
    pub negate_result: Negation,
    pub yield_literal: bool,
    pub trim_input: bool,
    pub trim_output: bool,
    pub skip_expansion: bool,
    pub discard_output: bool,
}

impl MacroAttribute {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn set(&mut self, attr: char) -> bool {
        match attr {
            '!' => self.negate_result.set(),
            '|' => self.pipe_output.set(),
            '-' => self.pipe_input.set(),
            '<' => self.trim_input = true,
            '^' => self.trim_output = true,
            '*' => self.yield_literal = true,
            '~' => self.skip_expansion = true,
            _ => return false,
        }
        true
    }

    pub(crate) fn set_from_string(&mut self, attributes: &str) {
        for ch in attributes.chars() {
            self.set(ch);
        }
    }
}

impl Display for MacroAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatted = String::new();

        formatted.push_str(&self.pipe_input.to_string());

        match self.pipe_output {
            PipeOutput::Single => formatted.push('|'),
            PipeOutput::Vector => formatted.push_str("||"),
            PipeOutput::None => (),
        }
        if self.yield_literal {
            formatted.push('*')
        }
        if self.trim_output {
            formatted.push('^')
        }
        if self.trim_input {
            formatted.push('=')
        }
        match self.negate_result {
            Negation::Value => formatted.push('!'),
            Negation::Yield => formatted.push_str("!!"),
            Negation::None => (),
        }
        if self.skip_expansion {
            formatted.push('~')
        }
        if self.discard_output {
            formatted.push_str("!!")
        }
        write!(f, "{}", formatted)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum PipeOutput {
    #[default]
    None,
    Single,
    Vector,
}

impl Display for PipeOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = match self {
            Self::None => "",
            Self::Single => "|",
            Self::Vector => "||",
        };
        write!(f, "{}", ret)
    }
}

impl PipeOutput {
    pub fn set(&mut self) {
        match self {
            Self::None => *self = Self::Single,
            Self::Single => *self = Self::Vector,
            _ => (),
        }
    }
    pub fn is_empty(&self) -> bool {
        *self == Self::None
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PipeInput {
    #[default]
    None,
    Vector,
    Single,
}

impl Display for PipeInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = match self {
            PipeInput::Vector => "-",
            PipeInput::Single => "--",
            PipeInput::None => "",
        };
        write!(f, "{}", ret)
    }
}

impl PipeInput {
    pub fn set(&mut self) {
        match self {
            Self::None => *self = Self::Vector,
            Self::Vector => *self = Self::Single,
            _ => (),
        }
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::None
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Negation {
    #[default]
    None,
    Value,
    Yield,
}

impl Display for Negation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = match self {
            Self::None => "",
            Self::Value => "!",
            Self::Yield => "!!",
        };
        write!(f, "{}", ret)
    }
}

impl Negation {
    pub fn set(&mut self) {
        match self {
            Self::None => *self = Self::Value,
            Self::Value => *self = Self::Yield,
            _ => (),
        }
    }
}

/// Macro framgent that processor saves fragmented information of the mcaro invocation
#[derive(Debug, Default)]
pub(crate) struct MacroFragment {
    pub whole_string: String,
    pub name: String,
    pub args: String,
    // This yield processed_args information which is not needed for normal operation.
    /// Argument that includes piped value
    #[cfg(feature = "debug")]
    pub processed_args: String,
    pub attribute: MacroAttribute,

    // Status varaible
    pub is_processed: bool,
}

impl MacroFragment {
    /// Create a new macro fragment
    pub fn new() -> Self {
        MacroFragment {
            whole_string: String::new(),
            name: String::new(),
            args: String::new(),
            #[cfg(feature = "debug")]
            processed_args: String::new(),
            attribute: MacroAttribute::default(),

            is_processed: false,
        }
    }

    /// Reset all state
    pub(crate) fn clear(&mut self) {
        self.whole_string.clear();
        self.name.clear();
        self.args.clear();
        #[cfg(feature = "debug")]
        self.processed_args.clear();
        self.attribute.clear();
    }

    /// Check if fragment is empty or not
    ///
    /// This also enables user to check if fragment has been cleared or not
    pub(crate) fn is_empty(&self) -> bool {
        self.whole_string.len() == 0
    }
}

/// Comment type
///
/// NoComment is for no comment
/// Start is when comment character should be positioned at start of the line
/// Any is when any position is possible
///
/// * Example
/// ```Text
/// % Sample     -> This is ok for Any,Start
/// Prior % Next -> This is only ok for Any
///
/// ```
#[derive(PartialEq, Debug)]
pub enum CommentType {
    /// Don't enable comment
    None,
    /// Only treat a line as a comment when it starts with comment character
    Start,
    /// Treat any text chunk that starts with comment character
    Any,
}

impl std::str::FromStr for CommentType {
    type Err = RadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let comment_type = match s.trim().to_lowercase().as_str() {
            "none" => Self::None,
            "start" => Self::Start,
            "any" => Self::Any,
            _ => {
                return Err(RadError::InvalidConversion(format!(
                    "Comment type : \"{}\" is not available.",
                    s
                )));
            }
        };
        Ok(comment_type)
    }
}

#[derive(Debug)]
/// Diffing behaviour
pub enum DiffOption {
    /// Do not yield diff
    None,
    /// Diff all texts
    All,
    /// Diff only changes
    Change,
}

impl std::str::FromStr for DiffOption {
    type Err = RadError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let var = match text.trim().to_lowercase().as_str() {
            "none" => Self::None,
            "all" => Self::All,
            "change" => Self::Change,
            _ => {
                return Err(RadError::InvalidConversion(format!(
                    "Diffoption, \"{}\" is not a valid type",
                    text
                )))
            }
        };
        Ok(var)
    }
}

/// Enum that controls processing flow
#[derive(Debug, PartialEq)]
pub enum FlowControl {
    /// No flow control
    None,
    /// Escape following texts
    Escape,
    /// Exit from processing ( Input )
    Exit,
}

/// Signature type
pub enum SignatureType {
    /// Every macros
    All,
    /// Only function macros
    Function,
    /// Only runtime macros
    Runtime,
}

impl SignatureType {
    pub fn from_str(text: &str) -> RadResult<Self> {
        let variant = match text.trim().to_lowercase().as_str() {
            "all" => Self::All,
            "function" => Self::Function,
            "runtime" => Self::Runtime,
            _ => {
                return Err(RadError::InvalidConversion(format!(
                    "\"{}\" is not supported signature type",
                    text
                )))
            }
        };

        Ok(variant)
    }
}

/// Target of relaying
#[derive(Debug)]
pub enum RelayTarget {
    None,
    File(FileTarget),
    Macro(String),
    Temp,
}

/// Process input variant
#[derive(Clone, Debug, PartialEq)]
pub enum ProcessInput {
    /// Standard input
    Stdin,
    /// File input
    File(PathBuf),
}

impl std::fmt::Display for ProcessInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdin => write!(f, "Stdin"),
            Self::File(file) => write!(f, "{}", file.display()),
        }
    }
}

/// Standards of behaviour
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ErrorBehaviour {
    /// No error actually
    Exit,
    /// Every error is a panic
    Strict,
    /// Every error is pasted as is
    Lenient,
    /// Every error is purged
    Purge,
    /// Special behaviour of assertion
    Assert,
    /// Special behaviour of panic
    Interrupt,
}

impl std::str::FromStr for ErrorBehaviour {
    type Err = RadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "lenient" => Ok(ErrorBehaviour::Lenient),
            "purge" => Ok(ErrorBehaviour::Purge),
            "strict" => Ok(ErrorBehaviour::Strict),
            _ => Err(RadError::InvalidArgument(format!(
                "\"{}\" is not a valid error type",
                s
            ))),
        }
    }
}

/// Type of processing
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ProcessType {
    /// Expand every macros
    Expand,
    /// Freeze definitions
    Export,
    /// Dry run mode
    Dry,
}

/// Types of a macros
pub enum MacroType {
    /// Function macro
    Function,
    /// Deterred macro
    Deterred,
    /// Runtime macro
    Runtime,
    /// Any macro
    Any,
}

/// File wrapper which hodls both path and File handle
#[derive(Debug)]
pub struct FileTarget {
    /// Representaion path
    repr: PathBuf,
    /// Real path
    absolute_path: PathBuf,
    /// File handle
    file: File,
}

impl std::fmt::Display for FileTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr.display())
    }
}

impl FileTarget {
    /// Get absolute path
    pub fn path(&self) -> &Path {
        &self.absolute_path
    }

    /// Get representation path
    pub fn name(&self) -> &Path {
        &self.repr
    }

    /// Get inner file struct
    pub fn inner(&mut self) -> &mut File {
        &mut self.file
    }

    /// Create an instance with file
    pub fn from_file(path: &Path, file: File) -> RadResult<Self> {
        Ok(Self {
            repr: path.to_owned(),
            absolute_path: path.canonicalize()?,
            file,
        })
    }

    /// Create an instance without truncate option
    pub fn from_path(path: &Path) -> RadResult<Self> {
        let repr_path = path.to_owned();
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&repr_path)
            .map_err(|_| {
                RadError::InvalidFile(format!("File \"{}\" cannot be opened", repr_path.display()))
            })?;
        Ok(Self {
            repr: repr_path,
            absolute_path: path.canonicalize()?,
            file,
        })
    }

    /// Creat an instance with trucate option
    pub fn with_truncate(path: &Path) -> RadResult<Self> {
        let repr_path = path.to_owned();
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&repr_path)
            .map_err(|_| {
                RadError::InvalidFile(format!("File \"{}\" cannot be opened", repr_path.display()))
            })?;
        Ok(Self {
            repr: repr_path,
            absolute_path: path.canonicalize()?,
            file,
        })
    }
}

#[derive(PartialEq, Clone, Copy)]
/// Hygiene variant
///
/// - None    : No hygiene applied
/// - Macro   : Hygine by per invocation
/// - Input   : Hygiene by per input
/// - Aseptic : No runtime definition or invocation at all.
pub enum Hygiene {
    /// No hygiene applied
    None,
    /// Hygine by per invocation
    Macro,
    /// Hygiene by per input
    Input,
    /// No runtime definition or invocation at all.
    Aseptic,
}

impl FromStr for Hygiene {
    type Err = RadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "macro" => Ok(Self::Macro),
            "input" => Ok(Self::Input),
            "aseptic" => Ok(Self::Aseptic),
            _ => Err(RadError::InvalidConversion(format!(
                "Hygiene type : \"{}\" is not available.",
                s
            ))),
        }
    }
}

// This is mostly useful for include macro
/// Type of container
///
#[derive(Eq, PartialEq)]
pub enum ContainerType {
    /// Container inside arguments
    Argument,
    /// Container that is expanded
    Expand,
    /// D container
    None,
}

#[derive(Eq, PartialEq, Debug)]
pub enum AlignType {
    Left,
    Right,
    Center,
}
impl std::str::FromStr for AlignType {
    type Err = RadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "l" | "left" => Ok(Self::Left),
            "r" | "right" => Ok(Self::Right),
            "c" | "center" => Ok(Self::Center),
            _ => Err(RadError::InvalidConversion(format!(
                "Align type : \"{}\" is not available.",
                s
            ))),
        }
    }
}

impl From<AlignType> for dcsv::CellAlignType {
    fn from(value: AlignType) -> Self {
        match value {
            AlignType::Left => dcsv::CellAlignType::Left,
            AlignType::Center => dcsv::CellAlignType::Center,
            AlignType::Right => dcsv::CellAlignType::Right,
        }
    }
}

#[derive(Debug)]
pub enum VarContOperation {
    Clear,
    Extend,
    Get,
    Pop,
    Print,
    List,
    Push,
    Set,
    Top,
    Len,
}

impl FromStr for VarContOperation {
    type Err = RadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim().to_lowercase().as_str() {
            "print" => Self::Print,
            "push" => Self::Push,
            "pop" => Self::Pop,
            "clear" => Self::Clear,
            "get" => Self::Get,
            "top" => Self::Top,
            "len" => Self::Len,
            "list" => Self::List,
            "set" => Self::Set,
            "extend" => Self::Extend,
            _ => {
                return Err(RadError::InvalidConversion(format!(
                    "{s} is not a valid container operation"
                )));
            }
        })
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum LineUpType {
    Hierarchy,
    Left,
    Right,
    ParralelRight,
    ParralelLeft,
}
impl std::str::FromStr for LineUpType {
    type Err = RadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "h" | "hierarchy" => Ok(Self::Hierarchy),
            "l" | "left" => Ok(Self::Left),
            "r" | "right" => Ok(Self::Right),
            "pr" | "parralel-right" => Ok(Self::ParralelRight),
            "pl" | "parralel-left" => Ok(Self::ParralelLeft),
            _ => Err(RadError::InvalidConversion(format!(
                "Line up type : \"{}\" is not available.",
                s
            ))),
        }
    }
}

pub enum OutputType {
    Terminal,
    File,
    Discard,
}

impl std::str::FromStr for OutputType {
    type Err = RadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = match s.trim().to_lowercase().as_ref() {
            "terminal" => Self::Terminal,
            "file" => Self::File,
            "discard" => Self::Discard,
            _ => {
                return Err(RadError::InvalidArgument(format!(
                    "Given type \"{}\" is not a valid output type",
                    s
                )))
            }
        };
        Ok(t)
    }
}

pub enum OrderType {
    Ascending,
    Descending,
}

impl std::str::FromStr for OrderType {
    type Err = RadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = match s.trim().to_lowercase().as_ref() {
            "a" | "asce" => Self::Ascending,
            "d" | "desc" => Self::Descending,
            _ => {
                return Err(RadError::InvalidArgument(format!(
                    "Given type \"{}\" is not a valid order type",
                    s
                )))
            }
        };
        Ok(t)
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ETMap {
    pub tables: HashMap<String, ETable>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ETable {
    arg_name: String,
    pub candidates: Vec<String>,
}

impl ETable {
    pub fn new(name: &str) -> Self {
        Self {
            arg_name: name.to_string(),
            candidates: Vec::default(),
        }
    }

    pub fn candidates(mut self, cand: &[&str]) -> (String, Self) {
        self.candidates = cand.iter().map(|s| s.to_string()).collect();
        (self.arg_name.clone(), self)
    }
}

#[derive(Debug, Default)]
pub(crate) struct MacroDefinition {
    pub name: String,
    pub params: Vec<Parameter>,
    pub ret: ValueType,
    pub body: String,
}

impl MacroDefinition {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }
}
