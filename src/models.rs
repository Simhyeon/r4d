use std::collections::HashMap;
use crate::basic::BasicMacro;
use crate::error::RadError;
use crate::utils::Utils;

#[derive(Clone)]
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

pub struct MacroMap<'a> {
    pub basic : BasicMacro<'a>,
    pub custom : HashMap<String, MacroRule>,
    pub local : HashMap<String, String>,
}

impl<'a> MacroMap<'a> {
    pub fn new() -> Self {
        Self { 
            basic: BasicMacro::new(),
            custom: HashMap::new(),
            local: HashMap::new(),
        }
    }

    // Crate new local macro(argument map)
    pub fn new_local(&mut self, level: usize, caller: &str ,name: &str, value: &str) {
        self.local.insert(Utils::local_name(level, caller, name), value.to_owned());
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
            &Utils::trim(body)?);
        self.custom.insert(name.to_owned(), mac);
        Ok(())
    }
}
