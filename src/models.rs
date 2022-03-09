use crate::runtime_map::{RuntimeMacroMap, RuntimeMacro};
use crate::error::RadError;
#[cfg(feature = "signature")]
use crate::sigmap::MacroSignature;
use crate::utils::Utils;
use crate::{function_map::FunctionMacroMap, deterred_map::DeterredMacroMap};
use bincode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use crate::function_map::FunctionMacroType;
use crate::deterred_map::DFunctionMacroType;

pub type RadResult<T> = Result<T, RadError>;

/// State enum value about direction of processed text
pub enum WriteOption<'a> {
    File(std::fs::File),
    Variable(&'a mut String),
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

    pub fn contains_macro(&self, macro_name:&str, macro_type: MacroType, volatile: bool) -> bool {
        match macro_type {
            MacroType::Deterred => self.deterred.contains(macro_name),
            MacroType::Function => self.function.contains(macro_name),
            MacroType::Runtime => self.runtime.contains(macro_name, volatile),
            MacroType::Any => self.function.contains(macro_name) || self.runtime.contains(macro_name, volatile) || self.deterred.contains(macro_name),
        }
    }

    // Empty argument should be treated as no arg
    /// Register a new runtime macro
    pub fn register_runtime(&mut self, name: &str, args: &str, body: &str, volatile: bool) -> RadResult<()> {
        // Trim all whitespaces and newlines from the string
        let mac = RuntimeMacro::new(&Utils::trim(name), &Utils::trim(args), body);
        self.runtime.new_macro(name, mac, volatile);
        Ok(())
    }

    pub fn register_runtime_as_volatile(&mut self, name: &str, args: &str, body: &str, volatile: bool) -> RadResult<()> {
        // Trim all whitespaces and newlines from the string
        let mac = RuntimeMacro::new(&Utils::trim(name), &Utils::trim(args), body);
        self.runtime.new_macro(name, mac, volatile);
        Ok(())
    }

    /// Undeifne macro
    pub fn undefine(&mut self, macro_name: &str, macro_type: MacroType, volatile: bool) {
        match macro_type{
            MacroType::Deterred => {self.deterred.undefine(macro_name);}
            MacroType::Function => {self.function.undefine(macro_name);}
            MacroType::Runtime => {self.runtime.undefine(macro_name, volatile);}
            MacroType::Any => {
                self.function.undefine(macro_name);
                self.runtime.undefine(macro_name, volatile);
                self.deterred.undefine(macro_name);
            }
        }
    }

    pub fn rename(&mut self, macro_name: &str, target_name: &str, macro_type: MacroType, volatile: bool) {
        match macro_type{
            MacroType::Deterred => {self.deterred.rename(macro_name,target_name);}
            MacroType::Function => {self.function.rename(macro_name,target_name);}
            MacroType::Runtime => {self.runtime.rename(macro_name,target_name, volatile);}
            MacroType::Any => {
                self.function.rename(macro_name,target_name);
                self.runtime.rename(macro_name,target_name, volatile);
                self.deterred.rename(macro_name,target_name);
            }
        }
    }

    pub fn append(&mut self, name: &str, target: &str, volatile: bool) {
        if self.runtime.contains(name, volatile) {
            self.runtime.append_macro(name, target, volatile);
        }
    }

    pub fn replace(&mut self, name: &str, target: &str, volatile: bool) -> bool {
        if self.runtime.contains(name, volatile) {
            self.runtime.replace_macro(name, target, volatile);
            true
        } else {
            false
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
            '(' => self.paren = self.paren + 1,
            ')' => {
                if self.paren > 0 {
                    self.paren = self.paren - 1;
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
        if let Err(_) = result {
            Err(RadError::BincodeError(format!(
                "Failed to melt the file : {}",
                path.display()
            )))
        } else {
            self.rules.extend(result.unwrap().rules.into_iter());
            Ok(())
        }
    }

    pub fn melt_literal(&mut self, literal: &Vec<u8>) -> RadResult<()> {
        let result = bincode::deserialize::<Self>(literal);
        if let Err(_) = result {
            Err(RadError::BincodeError(format!(
                "Failed to melt the literal value"
            )))
        } else {
            self.rules.extend(result.unwrap().rules.into_iter());
            Ok(())
        }
    }

    /// Convert runtime rules into a single binary file
    pub(crate) fn freeze(&self, path: &std::path::Path) -> RadResult<()> {
        let result = bincode::serialize(self);
        if let Err(_) = result {
            Err(RadError::BincodeError(format!(
                "Failed to freeze to the file : {}",
                path.display()
            )))
        } else {
            if let Err(_) = std::fs::write(path, result.unwrap()) {
                return Err(RadError::InvalidArgument(format!(
                    "Failed to create file : {}",
                    path.display()
                )));
            }
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
        self.trimmed = false;
    }

    /// Check if fragment is empty or not
    ///
    /// This also enables user to check if fragment has been cleared or not
    pub(crate) fn is_empty(&self) -> bool {
        self.whole_string.len() == 0
    }

    pub(crate) fn has_attribute(&self) -> bool {
        self.pipe || self.greedy || self.yield_literal || self.trimmed
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
    None,
    Start,
    Any,
}

impl CommentType {
    pub(crate) fn from_str(text: &str) -> RadResult<Self> {
        let comment_type = match text.to_lowercase().as_str() {
            "none" => Self::None,
            "start" => Self::Start,
            "any" => Self::Any,
            _ => {
                return Err(RadError::InvalidCommandOption(format!(
                    "Comment type : \"{}\" is not available.",
                    text
                )));
            }
        };
        Ok(comment_type)
    }
}

#[derive(Debug)]
pub enum DiffOption {
    None,
    All,
    Change,
}

impl DiffOption {
    pub fn from_str(text: &str) -> RadResult<Self> {
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

#[cfg(feature = "storage")]
pub type StorageResult<T> = Result<T, Box<dyn std::error::Error>>;

#[cfg(feature = "storage")]
pub trait RadStorage {
    fn update(&mut self, args: &Vec<String>) -> StorageResult<()>;
    fn extract(&mut self, serialize: bool) -> StorageResult<Option<StorageOutput>>;
}

#[cfg(feature = "storage")]
#[derive(Debug)]
pub enum StorageOutput {
    Binary(Vec<u8>),
    Text(String),
}

#[cfg(feature = "storage")]
impl StorageOutput {
    pub(crate) fn into_printable(&self) -> String {
        match self {
            Self::Binary(bytes) => format!("{:?}", bytes),
            Self::Text(text) => text.to_owned(),
        }
    }
}

pub enum RelayTarget {
    None,
    File((PathBuf, File)),
    Macro(String),
    Temp,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProcessInput {
    Stdin,
    File(PathBuf),
}

impl ToString for ProcessInput {
    fn to_string(&self) -> String {
        match self {
            Self::Stdin => "Stdin".to_owned(),
            Self::File(file) => file.display().to_string(),
        }
    }
}

/// Standards of behaviour
#[derive(PartialEq)]
pub enum Behaviour {
    Strict,
    Leninet,
    Purge,
    Nopanic,
}

#[derive(Clone)]
pub enum ExtMacroBody {
    Function(FunctionMacroType),
    Deterred(DFunctionMacroType),
}

#[derive(Clone)]
pub struct ExtMacroBuilder {
    pub(crate) macro_name: String,
    pub(crate) macro_type: ExtMacroType,
    pub(crate) args: Vec<String>,
    pub(crate) macro_body: Option<ExtMacroBody>,
}

impl ExtMacroBuilder {
    pub fn new(macro_name: &str, macro_type: ExtMacroType) -> Self {
        Self {
            macro_name: macro_name.to_string(),
            macro_type,
            // Empty values
            args: vec![],
            macro_body: None,
        }
    }

    pub fn args(mut self, args: &Vec<impl AsRef<str>>) -> Self {
        self.args = args.iter().map(|a| a.as_ref().to_string()).collect();
        self
    }

    pub fn body(mut self, body: ExtMacroBody) -> Self {
        self.macro_body.replace(body);
        self
    }
}

#[derive(Clone)]
pub enum ExtMacroType {
    Function,
    Deterred,
}

/// Intended for processor ext interface
pub enum MacroType {
    Function,
    Deterred,
    Runtime,
    Any
}
