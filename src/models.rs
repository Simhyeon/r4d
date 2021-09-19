use std::collections::HashMap;
use crate::basic::BasicMacro;
use crate::error::RadError;
use crate::utils::Utils;
use serde::{Deserialize, Serialize};
use bincode;

pub enum WriteOption {
    File(std::fs::File),
    Stdout,
}

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

pub struct MacroMap {
    pub basic : BasicMacro,
    pub custom : HashMap<String, MacroRule>,
    pub local : HashMap<String, String>,
}

impl MacroMap {
    /// Creates empty map
    pub fn empty() -> Self {
        Self {
            basic: BasicMacro::empty(),
            custom: HashMap::new(),
            local: HashMap::new(),
        }
    }

    pub fn new() -> Self {
        Self { 
            basic: BasicMacro::new(),
            custom: HashMap::new(),
            local: HashMap::new(),
        }
    }

    // Crate new local macro(argument map)
    pub fn new_local(&mut self, level: usize,name: &str, value: &str) {
        self.local.insert(Utils::local_name(level,name), value.to_owned());
    }

    pub fn clear_local(&mut self) {
        self.local.clear();
    }

    // Empty argument should be treated as no arg
    pub fn register(
        &mut self, 
        name: &str,
        args: &str,
        body: &str,
    ) -> Result<(),RadError> {
        // Trim all whitespaces and newlines from the string
        let mac = MacroRule::new(
            &Utils::trim(name)?, 
            &Utils::trim(args)?, 
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
}

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

    pub fn melt(&mut self, path : &std::path::Path) -> Result<(), RadError> {
        let result = bincode::deserialize::<Self>(&std::fs::read(path)?);
        if let Err(_) = result {
            Err(RadError::BincodeError(format!("Failed to melt the file : {}", path.display())))
        } else {
            self.rules.extend(result.unwrap().rules.into_iter());
            Ok(())
        }
    }

    pub(crate) fn freeze(&self, path: &std::path::Path) -> Result<(), RadError> {
        let result = bincode::serialize(self);
        if let Err(_) = result {
            Err(RadError::BincodeError(format!("Failed to freeze to the file : {}", path.display())))
        } else {
            std::fs::write(path, result.unwrap())?;
            Ok(())
        }
    }
}
