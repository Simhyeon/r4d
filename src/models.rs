use crate::deterred_map::DFunctionMacroType;
use crate::deterred_map::DeterredMacroMap;
use crate::error::RadError;
use crate::function_map::FunctionMacroMap;
use crate::function_map::FunctionMacroType;
use crate::runtime_map::{RuntimeMacro, RuntimeMacroMap};
#[cfg(feature = "signature")]
use crate::sigmap::MacroSignature;
use crate::utils::Utils;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
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
    File(File),
    Variable(&'a mut String),
    Return,
    Terminal,
    Discard,
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

/// Macro map that stores all kinds of macro informations
///
/// Included macro types are
/// - Keyword macro
/// - Basic macro
/// - Runtime macro
/// - Local bound macro
pub(crate) struct MacroMap {
    pub deterred: DeterredMacroMap,
    pub function: FunctionMacroMap,
    pub runtime: RuntimeMacroMap,
    pub local: HashMap<String, LocalMacro>,
}

impl MacroMap {
    /// Creates empty map without default macros
    pub fn empty() -> Self {
        Self {
            deterred: DeterredMacroMap::empty(),
            function: FunctionMacroMap::empty(),
            runtime: RuntimeMacroMap::new(),
            local: HashMap::new(),
        }
    }

    /// Creates default map with default function macros
    pub fn new() -> Self {
        Self {
            deterred: DeterredMacroMap::new(),
            function: FunctionMacroMap::new(),
            runtime: RuntimeMacroMap::new(),
            local: HashMap::new(),
        }
    }

    pub fn clear_runtime_macros(&mut self, volatile: bool) {
        self.runtime.clear_runtime_macros(volatile);
    }

    /// Create a new local macro
    ///
    /// This will override local macro if save value was given.
    pub fn add_local_macro(&mut self, level: usize, name: &str, value: &str) {
        self.local.insert(
            Utils::local_name(level, name),
            LocalMacro::new(level, name.to_owned(), value.to_owned()),
        );
    }

    /// Removes a local macro
    ///
    /// This will try to remove but will do nothing if given macro doesn't exist.
    pub fn remove_local_macro(&mut self, level: usize, name: &str) {
        self.local.remove(&Utils::local_name(level, name));
    }

    /// Clear all local macros
    pub fn clear_local(&mut self) {
        self.local.clear();
    }

    /// Retain only local macros that is smaller or equal to current level
    pub fn clear_lower_locals(&mut self, current_level: usize) {
        self.local.retain(|_, mac| mac.level <= current_level);
    }

    pub fn is_deterred_macro(&self, name: &str) -> bool {
        self.deterred.contains(name)
    }

    pub fn contains_macro(
        &self,
        macro_name: &str,
        macro_type: MacroType,
        hygiene_type: Hygiene,
    ) -> bool {
        match macro_type {
            MacroType::Deterred => self.deterred.contains(macro_name),
            MacroType::Function => self.function.contains(macro_name),
            MacroType::Runtime => self.runtime.contains(macro_name, hygiene_type),
            MacroType::Any => {
                self.function.contains(macro_name)
                    || self.runtime.contains(macro_name, hygiene_type)
                    || self.deterred.contains(macro_name)
            }
        }
    }

    // Empty argument should be treated as no arg
    /// Register a new runtime macro
    pub fn register_runtime(
        &mut self,
        name: &str,
        args: &str,
        body: &str,
        hygiene_type: Hygiene,
    ) -> RadResult<()> {
        // Trim all whitespaces and newlines from the string
        let mac = RuntimeMacro::new(&Utils::trim(name), &Utils::trim(args), body);
        self.runtime.new_macro(name, mac, hygiene_type);
        Ok(())
    }

    /// Undeifne macro
    pub fn undefine(&mut self, macro_name: &str, macro_type: MacroType, hygiene_type: Hygiene) {
        match macro_type {
            MacroType::Deterred => {
                self.deterred.undefine(macro_name);
            }
            MacroType::Function => {
                self.function.undefine(macro_name);
            }
            MacroType::Runtime => {
                self.runtime.undefine(macro_name, hygiene_type);
            }
            MacroType::Any => {
                self.function.undefine(macro_name);
                self.runtime.undefine(macro_name, hygiene_type);
                self.deterred.undefine(macro_name);
            }
        }
    }

    pub fn rename(
        &mut self,
        macro_name: &str,
        target_name: &str,
        macro_type: MacroType,
        hygiene_type: Hygiene,
    ) {
        match macro_type {
            MacroType::Deterred => {
                self.deterred.rename(macro_name, target_name);
            }
            MacroType::Function => {
                self.function.rename(macro_name, target_name);
            }
            MacroType::Runtime => {
                self.runtime.rename(macro_name, target_name, hygiene_type);
            }
            MacroType::Any => {
                self.function.rename(macro_name, target_name);
                self.runtime.rename(macro_name, target_name, hygiene_type);
                self.deterred.rename(macro_name, target_name);
            }
        }
    }

    pub fn append(&mut self, name: &str, target: &str, hygiene_type: Hygiene) {
        if self.runtime.contains(name, hygiene_type) {
            self.runtime.append_macro(name, target, hygiene_type);
        }
    }

    pub fn replace(&mut self, name: &str, target: &str, hygiene_type: Hygiene) -> bool {
        if self.runtime.contains(name, hygiene_type) {
            self.runtime.replace_macro(name, target, hygiene_type);
            true
        } else {
            false
        }
    }

    /// Get a macro signature
    #[cfg(feature = "signature")]
    pub fn get_signature(&self, macro_name: &str) -> Option<MacroSignature> {
        if let Some(mac) = self.runtime.get(macro_name, Hygiene::None) {
            Some(MacroSignature::from(mac))
        } else if let Some(mac) = self.deterred.get_signature(macro_name) {
            Some(MacroSignature::from(mac))
        } else {
            self.function
                .get_signature(macro_name)
                .map(MacroSignature::from)
        }
    }

    /// Get macro signatures object
    #[cfg(feature = "signature")]
    pub fn get_signatures(&self) -> Vec<MacroSignature> {
        let key_iter = self
            .deterred
            .macros
            .iter()
            .map(|(_, sig)| MacroSignature::from(sig));
        let funcm_iter = self
            .function
            .macros
            .iter()
            .map(|(_, sig)| MacroSignature::from(sig));
        let runtime_iter = self
            .runtime
            .macros
            .iter()
            .map(|(_, mac)| MacroSignature::from(mac));
        key_iter.chain(funcm_iter).chain(runtime_iter).collect()
    }

    #[cfg(feature = "signature")]
    pub fn get_default_signatures(&self) -> Vec<MacroSignature> {
        let key_iter = self
            .deterred
            .macros
            .iter()
            .map(|(_, sig)| MacroSignature::from(sig));
        let funcm_iter = self
            .function
            .macros
            .iter()
            .map(|(_, sig)| MacroSignature::from(sig));
        key_iter.chain(funcm_iter).collect()
    }

    #[cfg(feature = "signature")]
    pub fn get_runtime_signatures(&self) -> Vec<MacroSignature> {
        self.runtime
            .macros
            .iter()
            .map(|(_, mac)| MacroSignature::from(mac))
            .collect()
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

/// Result alias for storage operation
///
/// Error is a boxed container for generic error trait. Therefore any kind of errors can be
/// captured by storageresult.
pub type StorageResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Triat for storage interaction
///
/// Rad can utilizes storage to save given input as modified form and extract data from
///
/// # Example
///
/// ```rust
/// use r4d::{RadStorage, RadError, StorageOutput, StorageResult};
///
/// pub struct StorageDemo {
///     content: Vec<String>,
/// }
///
/// impl RadStorage for StorageDemo {
///     fn update(&mut self, args: &[String]) -> StorageResult<()> {
///         if args.is_empty() {
///             return Err(Box::new(RadError::InvalidArgument("Not enough arguments".to_string())));
///         }
///         self.content.push(args[0].clone());
///
///         Ok(())
///     }
///     fn extract(&mut self, serialize: bool) -> StorageResult<Option<StorageOutput>> {
///         let result = if serialize {
///             StorageOutput::Binary(self.content.join(",").as_bytes().to_vec())
///         } else {
///             StorageOutput::Text(self.content.join(","))
///         };
///         Ok(Some(result))
///     }
/// }
/// ```
pub trait RadStorage {
    /// Update storage with given arguments
    fn update(&mut self, args: &[String]) -> StorageResult<()>;
    /// Extract data from storage.
    ///
    /// # Args
    ///
    /// - serialize : whether to serialize storage output or not
    fn extract(&mut self, serialize: bool) -> StorageResult<Option<StorageOutput>>;
}

#[derive(Debug)]
/// Output that storage creates
pub enum StorageOutput {
    /// Binary form of output
    Binary(Vec<u8>),
    /// Text form of output
    Text(String),
}

impl StorageOutput {
    pub(crate) fn into_printable(self) -> String {
        match self {
            Self::Binary(bytes) => format!("{:?}", bytes),
            Self::Text(text) => text,
        }
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
#[derive(PartialEq)]
pub enum ErrorBehaviour {
    Strict,
    Lenient,
    Purge,
}

#[derive(Clone)]
/// Builder struct for extension macros
///
/// This creates an extension macro without going through tedious processor methods interaction.
///
/// Use a template feature to utilizes eaiser extension register.
///
/// # Example
///
/// ```
/// let mut processor = r4d::Processor::new();
/// #[cfg(feature = "template")]
/// processor.add_ext_macro(r4d::ExtMacroBuilder::new("macro_name")
///     .args(&["a1","b2"])
///     .function(r4d::function_template!(
///         let args = r4d::split_args!(2)?;
///         let result = format!("{} + {}", args[0], args[1]);
///         Ok(Some(result))
/// )));
/// ```
pub struct ExtMacroBuilder {
    pub(crate) macro_name: String,
    pub(crate) macro_type: ExtMacroType,
    pub(crate) args: Vec<String>,
    pub(crate) macro_body: Option<ExtMacroBody>,
    pub(crate) macro_desc: Option<String>,
}

impl ExtMacroBuilder {
    /// Creates an empty macro with given macro name
    pub fn new(macro_name: &str) -> Self {
        Self {
            macro_name: macro_name.to_string(),
            macro_type: ExtMacroType::Function,
            // Empty values
            args: vec![],
            macro_body: None,
            macro_desc: None,
        }
    }

    /// Set macro's body type as function
    pub fn function(mut self, func: FunctionMacroType) -> Self {
        self.macro_type = ExtMacroType::Function;
        self.macro_body = Some(ExtMacroBody::Function(func));
        self
    }

    /// Set macro's body type as deterred
    pub fn deterred(mut self, func: DFunctionMacroType) -> Self {
        self.macro_type = ExtMacroType::Deterred;
        self.macro_body = Some(ExtMacroBody::Deterred(func));
        self
    }

    /// Set macro's arguments
    pub fn args(mut self, args: &[impl AsRef<str>]) -> Self {
        self.args = args.iter().map(|a| a.as_ref().to_string()).collect();
        self
    }

    /// Set description of the macro
    pub fn desc(mut self, description: &str) -> Self {
        self.macro_desc.replace(description.to_string());
        self
    }
}

#[derive(Clone)]
pub(crate) enum ExtMacroType {
    Function,
    Deterred,
}

#[derive(Clone)]
pub(crate) enum ExtMacroBody {
    Function(FunctionMacroType),
    Deterred(DFunctionMacroType),
}

/// Types of a macros
///
/// This is intended for processor ext interface but user can use it directly
pub enum MacroType {
    Function,
    Deterred,
    Runtime,
    Any,
}

#[derive(Debug)]
pub struct FileTarget {
    pub(crate) path: PathBuf,
    pub(crate) file: Option<File>,
}

impl FileTarget {
    pub fn empty() -> Self {
        Self {
            path: PathBuf::new(),
            file: None,
        }
    }

    pub fn set_path(&mut self, path: &Path) {
        self.path = path.to_owned();
        self.file = Some(
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
                .unwrap(),
        );
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
