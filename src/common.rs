use crate::error::RadError;
use crate::runtime_map::RuntimeMacro;
use crate::utils::Utils;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

/// Genenric result type for every rad operations
///
/// RadResult is a genric result type of T and error of [RadError](RadError)
pub type RadResult<T> = Result<T, RadError>;

/// State enum value about direction of processed text
///
/// - File       : Set file output
/// - Variable   : Set variable to save
/// - Return     : Return otuput directly ( logger ignores this variant )
/// - Terminal   : Print to terminal
/// - Discard    : Do nothing
pub enum WriteOption<'a> {
    File(FileTarget),
    Variable(&'a mut String),
    Return,
    Terminal,
    Discard,
}

impl<'a> WriteOption<'a> {
    pub fn file(path: &Path, open_option: OpenOptions) -> RadResult<Self> {
        let file = open_option.open(path).map_err(|_| {
            RadError::InvalidFile(format!("Cannot set write option to {}", path.display()))
        })?;
        Ok(Self::File(FileTarget::from_file(path, file)?))
    }
}

/// Local macro
#[derive(Clone)]
pub struct LocalMacro {
    pub level: usize,
    pub name: String,
    pub body: String,
}

impl LocalMacro {
    pub fn new(level: usize, name: String, body: String) -> Self {
        Self { level, name, body }
    }
}

/// Struct designed to check unbalanced parenthesis
pub(crate) struct UnbalancedChecker {
    paren: usize,
}

impl UnbalancedChecker {
    pub fn new() -> Self {
        Self { paren: 0 }
    }
    pub fn check(&mut self, ch: char) -> bool {
        match ch {
            '(' => self.paren += 1,
            ')' => {
                if self.paren > 0 {
                    self.paren -= 1;
                } else {
                    return false;
                }
            }
            _ => {
                return true;
            }
        }
        true
    }
}

/// Readable, writeable struct that holds information of runtime macros
#[derive(Serialize, Deserialize)]
pub struct RuleFile {
    pub rules: HashMap<String, RuntimeMacro>,
}

impl RuleFile {
    pub fn new(rules: Option<HashMap<String, RuntimeMacro>>) -> Self {
        if let Some(content) = rules {
            Self { rules: content }
        } else {
            Self {
                rules: HashMap::new(),
            }
        }
    }

    /// Read from rule file and make it into hash map
    pub fn melt(&mut self, path: &Path) -> RadResult<()> {
        Utils::is_real_path(path)?;
        let result = bincode::deserialize::<Self>(&std::fs::read(path)?);
        if let Err(err) = result {
            Err(RadError::BincodeError(format!(
                "Failed to melt the file : {} \n {}",
                path.display(),
                err
            )))
        } else {
            self.rules.extend(result.unwrap().rules.into_iter());
            Ok(())
        }
    }

    pub fn melt_literal(&mut self, literal: &[u8]) -> RadResult<()> {
        let result = bincode::deserialize::<Self>(literal);
        if let Ok(rule_file) = result {
            self.rules.extend(rule_file.rules.into_iter());
            Ok(())
        } else {
            Err(RadError::BincodeError(
                "Failed to melt the literal value".to_string(),
            ))
        }
    }

    /// Convert runtime rules into a single binary file
    pub(crate) fn freeze(&self, path: &std::path::Path) -> RadResult<()> {
        let result = bincode::serialize(self);
        if result.is_err() {
            Err(RadError::BincodeError(format!(
                "Failed to freeze to a file : {}",
                path.display()
            )))
        } else if std::fs::write(path, result.unwrap()).is_err() {
            Err(RadError::InvalidArgument(format!(
                "Failed to create a file : {}",
                path.display()
            )))
        } else {
            Ok(())
        }
    }

    pub(crate) fn serialize(&self) -> RadResult<Vec<u8>> {
        let result = bincode::serialize(self);
        if result.is_err() {
            return Err(RadError::BincodeError(
                "Failed to serialize a rule".to_string(),
            ));
        }
        Ok(result.unwrap())
    }
}

/// Macro framgent that processor saves fragmented information of the mcaro invocation
#[derive(Debug)]
pub(crate) struct MacroFragment {
    pub whole_string: String,
    pub name: String,
    pub args: String,
    // This yield processed_args information which is not needed for normal operation.
    #[cfg(feature = "debug")]
    pub processed_args: String,

    // Macro attributes
    pub pipe: bool,
    pub greedy: bool,
    pub yield_literal: bool,
    pub trim_input: bool,
    pub trimmed: bool,

    // Status varaible
    pub is_processed: bool,
}

impl MacroFragment {
    pub fn new() -> Self {
        MacroFragment {
            whole_string: String::new(),
            name: String::new(),
            args: String::new(),
            #[cfg(feature = "debug")]
            processed_args: String::new(),
            pipe: false,
            greedy: false,
            yield_literal: false,
            trim_input: false,
            trimmed: false,

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
        self.pipe = false;
        self.greedy = false;
        self.yield_literal = false;
        self.trim_input = false;
        self.trimmed = false;
    }

    /// Check if fragment is empty or not
    ///
    /// This also enables user to check if fragment has been cleared or not
    pub(crate) fn is_empty(&self) -> bool {
        self.whole_string.len() == 0
    }

    pub(crate) fn has_attribute(&self) -> bool {
        self.pipe || self.greedy || self.yield_literal || self.trimmed || self.trim_input
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
    None,
    Escape,
    Exit,
}

#[cfg(feature = "signature")]
pub enum SignatureType {
    All,
    Default,
    Runtime,
}

#[cfg(feature = "signature")]
impl SignatureType {
    pub fn from_str(text: &str) -> RadResult<Self> {
        let variant = match text.to_lowercase().as_str() {
            "all" => Self::All,
            "default" => Self::Default,
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

#[derive(Debug)]
pub enum RelayTarget {
    None,
    File(FileTarget),
    Macro(String),
    #[cfg(not(feature = "wasm"))]
    Temp,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProcessInput {
    Stdin,
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
}

/// Types of a macros
pub enum MacroType {
    Function,
    Deterred,
    Runtime,
    Any,
}

#[derive(Debug)]
pub struct FileTarget {
    /// Representaion
    repr: PathBuf,
    /// Real path
    absolute_path: PathBuf,
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

/// Cache for regex compilation
pub(crate) struct RegexCache {
    cache: HashMap<String, Regex>,
    register: HashMap<String, Regex>,
}

impl RegexCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            register: HashMap::new(),
        }
    }

    /// Check if cache contains a key
    pub fn contains(&self, name: &str) -> bool {
        self.cache.contains_key(name)
    }

    /// Register a regex
    ///
    /// Registered regex is not cleared
    pub fn register(&mut self, name: &str, source: &str) -> RadResult<()> {
        self.cache.insert(name.to_string(), Regex::new(source)?);
        Ok(())
    }

    /// Append a regex to cache
    pub fn append(&mut self, src: &str) -> RadResult<&Regex> {
        // Set hard capacity of 100
        if self.cache.len() > 100 {
            self.cache.clear();
        }
        self.cache.insert(src.to_string(), Regex::new(src)?);
        Ok(self.get(src).unwrap())
    }

    pub fn get(&self, src: &str) -> Option<&Regex> {
        if self.register.get(src).is_some() {
            self.register.get(src)
        } else {
            self.cache.get(src)
        }
    }
}
