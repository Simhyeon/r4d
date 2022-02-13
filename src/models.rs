use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use crate::{basic_map::BasicMacroMap, keyword_map::KeywordMacroMap};
use crate::error::RadError;
use crate::utils::Utils;
use serde::{Deserialize, Serialize};
#[cfg(feature = "signature")]
use crate::sigmap::MacroSignature;
use bincode;

pub type RadResult<T> = Result<T, RadError>;

/// State enum value about direction of processed text 
pub enum WriteOption<'a> {
    File(std::fs::File),
    Variable(&'a mut String),
    Terminal,
    Discard,
}

/// Custom macro
#[derive(Clone, Deserialize, Serialize)]
pub struct CustomMacro{
    pub name: String,
    pub args: Vec<String>,
    pub body: String,
}

impl CustomMacro {
    pub fn new(name: &str, args: &str, body: &str) -> Self {
        // Empty args are no args
        let mut args : Vec<String> = args.split_whitespace().map(|item| item.to_owned()).collect();
        if args.len() == 1 && args[0] == "" {
            args = vec![]
        }

        CustomMacro {  
            name : name.to_owned(),
            args,
            body : body.to_owned(),
        }
    }
}

impl std::fmt::Display for CustomMacro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut inner = self.args.iter().fold(String::new(),|acc, arg| acc + &arg + ",");
        // This removes last "," character
        inner.pop();
        write!(f,"${}({})", self.name, inner)
    }
}

#[cfg(feature = "signature")]
impl From<&CustomMacro> for crate::sigmap::MacroSignature {
    fn from(mac: &CustomMacro) -> Self {
        Self {
            variant: crate::sigmap::MacroVariant::Custom,
            name: mac.name.to_owned(),
            args: mac.args.to_owned(),
            expr: mac.to_string(),
        }
    }
}

/// Custom macro
#[derive(Clone)]
pub struct LocalMacro{
    pub level: usize,
    pub name: String,
    pub body: String,
}

impl LocalMacro {
    pub fn new(level: usize,name: String, body: String) -> Self {
        Self {  level, name, body }
    }
}

/// Macro map that stores all kinds of macro informations 
///
/// Included macro types are 
/// - Keyword macro
/// - Basic macro
/// - Custom macro
/// - Local bound macro
pub(crate) struct MacroMap {
    pub keyword: KeywordMacroMap,
    pub basic : BasicMacroMap,
    pub custom : HashMap<String, CustomMacro>,
    pub local : HashMap<String, LocalMacro>,
}

impl MacroMap {
    /// Creates empty map without default macros
    pub fn empty() -> Self {
        Self {
            keyword: KeywordMacroMap::empty(),
            basic: BasicMacroMap::empty(),
            custom: HashMap::new(),
            local: HashMap::new(),
        }
    }

    /// Creates default map with default basic macros
    pub fn new() -> Self {
        Self { 
            keyword: KeywordMacroMap::new(),
            basic: BasicMacroMap::new(),
            custom: HashMap::new(),
            local: HashMap::new(),
        }
    }

    pub fn clear_custom_macros(&mut self) {
        self.custom.clear();
    }

    /// Create a new local macro
    /// 
    /// This will override local macro if save value was given.
    pub fn new_local(&mut self, level: usize,name: &str, value: &str) {
        self.local.insert(Utils::local_name(level,name), LocalMacro::new(level, name.to_owned(), value.to_owned()));
    }

    /// Clear all local macros
    pub fn clear_local(&mut self) {
        self.local.clear();
    }
    
    /// Retain only local macros that is smaller or equal to current level 
    pub fn clear_lower_locals(&mut self, current_level: usize) {
        self.local.retain(|_,mac| mac.level <= current_level);
    }

    pub fn is_keyword(&self, name:&str) -> bool {
        self.keyword.contains(name)
    }

    /// Check if macro exits in custom macro
    pub fn contains_custom(&self, name: &str) -> bool {
        self.custom.contains_key(name)
    }

    /// Check if macro exists ( only basic and custom macro )
    pub fn contains_basic_or_custom(&self, name: &str) -> bool {
        self.basic.contains(name) || self.custom.contains_key(name)
    }

    /// Check if macro exists
    pub fn contains_any_macro(&self, name: &str) -> bool {
        self.basic.contains(name) || self.custom.contains_key(name) || self.keyword.contains(name)
    }

    // Empty argument should be treated as no arg
    /// Register a new custom macro
    pub fn register_custom(
        &mut self, 
        name: &str,
        args: &str,
        body: &str,
    ) -> RadResult<()> {
        // Trim all whitespaces and newlines from the string
        let mac = CustomMacro::new(
            &Utils::trim(name), 
            &Utils::trim(args), 
            body);
        self.custom.insert(name.to_owned(), mac);
        Ok(())
    }

    pub fn undefine(&mut self, name: &str) {
        // Return true or false by the definition
        if self.basic.contains(name) {
            self.basic.undefine(name);
        }
        if self.custom.contains_key(name) {
            self.custom.remove(name);
        }
    }

    pub fn undefine_custom(&mut self, name: &str) {
        if self.custom.contains_key(name) {
            self.custom.remove(name);
        }
    }

    pub fn rename(&mut self, name: &str, target: &str) {
        if self.basic.contains(name) {
            self.basic.rename(name, target);
        }
        if self.custom.contains_key(name) {
            let custom = self.custom.remove(name).unwrap();
            self.custom.insert(target.to_owned(), custom);
        }
    }

    pub fn append(&mut self, name: &str, target: &str) {
        if self.custom.contains_key(name) {
            let custom = self.custom.get_mut(name).unwrap();
            custom.body.push_str(target);
        }
    }

    pub fn replace(&mut self, name: &str, target: &str) -> bool {
        if self.custom.contains_key(name) {
            let custom = self.custom.get_mut(name).unwrap();
            custom.body = target.to_owned();
            true
        } else {
            false
        }
    }

    /// Get macro signatures object
    #[cfg(feature = "signature")]
    pub fn get_signatures(&self) -> Vec<MacroSignature> {
        let key_iter = self.keyword.macros
            .iter()
            .map(|(_,sig)| MacroSignature::from(sig));
        let basic_iter = self.basic.macros
            .iter()
            .map(|(_,sig)| MacroSignature::from(sig));
        let custom_iter = self.custom
            .iter()
            .map(|(_,custom)| MacroSignature::from(custom));
        key_iter.chain(basic_iter).chain(custom_iter).collect()
    }

    #[cfg(feature = "signature")]
    pub fn get_default_signatures(&self) -> Vec<MacroSignature> {
        let key_iter = self.keyword.macros
            .iter()
            .map(|(_,sig)| MacroSignature::from(sig));
        let basic_iter = self.basic.macros
            .iter()
            .map(|(_,sig)| MacroSignature::from(sig));
        key_iter.chain(basic_iter).collect()
    }

    #[cfg(feature = "signature")]
    pub fn get_custom_signatures(&self) -> Vec<MacroSignature> {
        self.custom
            .iter()
            .map(|(_,custom)| MacroSignature::from(custom)).collect()
    }
}

/// Struct designed to check unbalanced parenthesis
pub(crate) struct UnbalancedChecker{
    paren: usize,
}

impl UnbalancedChecker {
    pub fn new() -> Self {
        Self {
            paren: 0,
        }
    }
    pub fn check(&mut self, ch: char) -> bool {
        match ch {
            '(' => {
                self.paren = self.paren + 1
            },
            ')' => {
                if self.paren > 0 {self.paren = self.paren - 1; } 
                else {return false; }
            },
            _ => {return true;}
        }
        true
    }
} 

/// Readable, writeable struct that holds information of custom macros
#[derive(Serialize, Deserialize)]
pub struct RuleFile {
    pub rules : HashMap<String, CustomMacro>,
}

impl RuleFile {
    pub fn new(rules : Option<HashMap<String, CustomMacro>>) -> Self {
        if let Some(content) = rules {
            Self {
                rules: content,
            }
        } else {
            Self {
                rules: HashMap::new(),
            }
        }
    }

    /// Read from rule file and make it into hash map
    pub fn melt(&mut self, path : &Path) -> RadResult<()> {
        Utils::is_real_path(path)?;
        let result = bincode::deserialize::<Self>(&std::fs::read(path)?);
        if let Err(_) = result {
            Err(RadError::BincodeError(format!("Failed to melt the file : {}", path.display())))
        } else {
            self.rules.extend(result.unwrap().rules.into_iter());
            Ok(())
        }
    }

    pub fn melt_literal(&mut self, literal : &Vec<u8>) -> RadResult<()> {
        let result = bincode::deserialize::<Self>(literal);
        if let Err(_) = result {
            Err(RadError::BincodeError(format!("Failed to melt the literal value")))
        } else {
            self.rules.extend(result.unwrap().rules.into_iter());
            Ok(())
        }
    }

    /// Convert custom rules into a single binary file
    pub(crate) fn freeze(&self, path: &std::path::Path) -> RadResult<()> {
        let result = bincode::serialize(self);
        if let Err(_) = result {
            Err(RadError::BincodeError(format!("Failed to freeze to the file : {}", path.display())))
        } else {
            if let Err(_) = std::fs::write(path, result.unwrap()) {
                return Err(
                    RadError::InvalidArgument(
                        format!("Failed to create file : {}", path.display())
                    )
                );
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
    pub yield_literal : bool,
    pub trimmed : bool,
}

impl MacroFragment {
    pub fn new() -> Self {
        MacroFragment {
            whole_string : String::new(),
            name : String::new(),
            args : String::new(),
            #[cfg(feature = "debug")]
            processed_args : String::new(),
            pipe: false,
            greedy: false,
            yield_literal : false,
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
    pub(crate) fn from_str(text : &str) -> RadResult<Self> {
        let comment_type = match text.to_lowercase().as_str() {
            "none"  => Self::None,
            "start" => Self::Start,
            "any"   => Self::Any,
            _ => {
                return Err(RadError::InvalidCommandOption(format!("Comment type : \"{}\" is not available.", text)));
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
            _ => return Err(RadError::InvalidConversion(format!("Diffoption, \"{}\" is not a valid type", text))),
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
    Custom,
}

#[cfg(feature = "signature")]
impl SignatureType {
    pub fn from_str(text : &str) -> RadResult<Self> {
        let variant = match text.to_lowercase().as_str() {
            "all" => Self::All,
            "default" => Self::Default,
            "custom" => Self::Custom,
            _ => return Err(RadError::InvalidConversion(format!("\"{}\" is not supported signature type", text)))
        };

        Ok(variant)
    }
}

#[cfg(feature = "storage")]
pub type StorageResult<T> = Result<T, Box<dyn std::error::Error>>;

#[cfg(feature = "storage")]
pub trait RadStorage {
    fn update(&mut self,args : &Vec<String>) -> StorageResult<()>;
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
    File((PathBuf,File)),
    Macro(String),
    Temp,
}

#[derive(Clone,Debug, PartialEq)]
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
