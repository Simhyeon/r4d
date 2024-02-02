//! Common structs, enums for code usage.

use crate::error::RadError;
use std::fmt::Display;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

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
    pub pipe_output: bool,
    pub pipe_input: bool,
    pub yield_literal: bool,
    pub negate_result: bool,
    pub trim_input: bool,
    pub trim_output: bool,
    pub skip_expansion: bool,
}

impl MacroAttribute {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Check if fragment has attribute
    pub(crate) fn has_attribute(&self) -> bool {
        self.pipe_input
            || self.pipe_output
            || self.yield_literal
            || self.trim_output
            || self.trim_input
            || self.negate_result
            || self.skip_expansion
    }
}

impl Display for MacroAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatted = String::new();
        if self.pipe_input {
            formatted.push('-')
        }
        if self.pipe_output {
            formatted.push('|')
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
        if self.negate_result {
            formatted.push('!')
        }
        if self.skip_expansion {
            formatted.push('~')
        }
        write!(f, "{}", formatted)
    }
}

/// Macro framgent that processor saves fragmented information of the mcaro invocation
#[derive(Debug, Default)]
pub(crate) struct MacroFragment {
    pub whole_string: String,
    pub name: String,
    pub args: String,
    // This yield processed_args information which is not needed for normal operation.
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

    /// Check if fragment has attribute
    pub(crate) fn has_attribute(&self) -> bool {
        self.attribute.has_attribute()
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
        let comment_type = match s.to_lowercase().as_str() {
            "none" => Self::None,
            "start" => Self::Start,
            "any" => Self::Any,
            _ => {
                return Err(RadError::InvalidCommandOption(format!(
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
        let var = match text.to_lowercase().as_str() {
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
        let variant = match text.to_lowercase().as_str() {
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
#[derive(PartialEq, Clone, Copy)]
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

/// Type of processing
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ProcessType {
    /// Expand every macros
    Expand,
    /// Freeze definitions
    Freeze,
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
        match s.to_lowercase().as_str() {
            "l" | "left" => Ok(Self::Left),
            "r" | "right" => Ok(Self::Right),
            "c" | "center" => Ok(Self::Center),
            _ => Err(RadError::InvalidCommandOption(format!(
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
