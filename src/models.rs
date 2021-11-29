use std::collections::HashMap;
use std::path::Path;
use crate::{basic::BasicMacro, keyword_map::KeywordMacro};
use crate::error::RadError;
use crate::utils::Utils;
use serde::{Deserialize, Serialize};
use bincode;

pub type RadResult<T> = Result<T, RadError>;

/// State enum value about direction of processed text 
pub enum WriteOption {
    File(std::fs::File),
    Terminal,
    Discard,
}

/// Macro rule of custom macros
#[derive(Clone, Deserialize, Serialize)]
pub struct MacroRule{
    pub name: String,
    pub args: Vec<String>,
    pub body: String,
}

impl MacroRule {
    pub fn new(name: &str, args: &str, body: &str) -> Self {
        // Empty args are no args
        let mut args : Vec<String> = args.split(' ').map(|item| item.to_owned()).collect();
        if args.len() == 1 && args[0] == "" {
            args = vec![]
        }

        MacroRule {  
            name : name.to_owned(),
            args,
            body : body.to_owned(),
        }
    }
}

/// Macro map that stores all kinds of macro informations 
///
/// Included macro types are 
/// - Basic macro
/// - Custom macro
/// - Local bound macro
pub(crate) struct MacroMap {
    pub keyword: KeywordMacro,
    pub basic : BasicMacro,
    pub custom : HashMap<String, MacroRule>,
    pub local : HashMap<String, String>,
}

impl MacroMap {
    /// Creates empty map without default basic macros
    ///
    /// Keyword macro cannot be empty
    pub fn empty() -> Self {
        Self {
            keyword: KeywordMacro::new(),
            basic: BasicMacro::empty(),
            custom: HashMap::new(),
            local: HashMap::new(),
        }
    }

    /// Creates default map with default basic macros
    pub fn new() -> Self {
        Self { 
            keyword: KeywordMacro::new(),
            basic: BasicMacro::new(),
            custom: HashMap::new(),
            local: HashMap::new(),
        }
    }

    /// Create a new local macro
    pub fn new_local(&mut self, level: usize,name: &str, value: &str) {
        self.local.insert(Utils::local_name(level,name), value.to_owned());
    }

    /// Clear all local macros
    pub fn clear_local(&mut self) {
        self.local.clear();
    }

    pub fn is_keyword(&self, name:&str) -> bool {
        self.keyword.contains(name)
    }

    /// Check if macro exists
    pub fn contains(&self, name: &str) -> bool {
        self.basic.contains(name) || self.custom.contains_key(name)
    }

    // Empty argument should be treated as no arg
    /// Register a new custom macro
    pub fn register(
        &mut self, 
        name: &str,
        args: &str,
        body: &str,
    ) -> RadResult<()> {
        // Trim all whitespaces and newlines from the string
        let mac = MacroRule::new(
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

    pub fn rename(&mut self, name: &str, target: &str) {
        if self.basic.contains(name) {
            self.basic.rename(name, target);
        }
        if self.custom.contains_key(name) {
            let rule = self.custom.remove(name).unwrap();
            self.custom.insert(target.to_owned(), rule);
        }
    }

    pub fn append(&mut self, name: &str, target: &str) {
        if self.custom.contains_key(name) {
            let rule = self.custom.get_mut(name).unwrap();
            rule.body.push_str(target);
        }
    }

    pub fn replace(&mut self, name: &str, target: &str) -> bool {
        if self.custom.contains_key(name) {
            let rule = self.custom.get_mut(name).unwrap();
            rule.body = target.to_owned();
            true
        } else {
            false
        }
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
    pub rules : HashMap<String, MacroRule>,
}

impl RuleFile {
    pub fn new(rules : Option<HashMap<String, MacroRule>>) -> Self {
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
            _ => return Err(RadError::InvalidConversion(format!("Diffoption, \"{}\" is not a vliad type", text))),
        };
        Ok(var)
    }
}
