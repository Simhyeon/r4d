use std::collections::HashMap;
use crate::basic::BasicMacro;
use crate::error::RadError;

pub struct MacroRule{
    pub name: String,
    pub args: Vec<String>,
    pub body: String,
}

impl MacroRule {
    pub fn new(name: &str, args: &str, body: &str) -> Self {
        MacroRule {  
            name : name.to_owned(),
            args : args.split(' ').map(|item| item.to_owned()).collect(),
            body : body.to_owned(),
        }
    }

    pub fn invoke(&self) -> Result<String, RadError> {
        // TODO
        let result = format!("");
        Ok(result)
    }
}

pub struct MacroMap<'a> {
    basic : BasicMacro<'a>,
    map : HashMap<String, MacroRule>,
}

impl<'a> MacroMap<'a> {
    pub fn new() -> Self {
        Self { 
            basic: BasicMacro::new(),
            map: HashMap::new() 
        }
    }

    pub fn register(
        &mut self, 
        name: &str,
        args: &str,
        body: &str,
    ) -> Result<(),RadError> {
        let mac = MacroRule::new(name, args, body);
        self.map.insert(name.to_owned(), mac);
        Ok(())
    }

    pub fn evaluate(&mut self, name: &str, args: &str) -> Result<Option<String>, RadError> {
        if self.basic.contains(name) {
            let result = self.basic.call(name, args)?;
            Ok(Some(result))
        } else if self.map.contains_key(name) {
            let mac = self.map.get(name).unwrap();
            Ok(Some(mac.invoke()?))
        } else {
            Ok(None)
        }
    }
}
