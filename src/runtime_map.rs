use crate::{Processor, RadResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime macro
#[derive(Clone, Deserialize, Serialize)]
pub struct RuntimeMacro {
    pub name: String,
    pub args: Vec<String>,
    pub body: String,
}

impl RuntimeMacro {
    pub fn new(name: &str, args: &str, body: &str) -> Self {
        // Empty args are no args
        let mut args: Vec<String> = args
            .split_whitespace()
            .map(|item| item.to_owned())
            .collect();
        if args.len() == 1 && args[0] == "" {
            args = vec![]
        }

        RuntimeMacro {
            name: name.to_owned(),
            args,
            body: body.to_owned(),
        }
    }
}

impl std::fmt::Display for RuntimeMacro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut inner = self
            .args
            .iter()
            .fold(String::new(), |acc, arg| acc + &arg + ",");
        // This removes last "," character
        inner.pop();
        write!(f, "${}({})", self.name, inner)
    }
}

#[cfg(feature = "signature")]
impl From<&RuntimeMacro> for crate::sigmap::MacroSignature {
    fn from(mac: &RuntimeMacro) -> Self {
        Self {
            variant: crate::sigmap::MacroVariant::Custom,
            name: mac.name.to_owned(),
            args: mac.args.to_owned(),
            expr: mac.to_string(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct RuntimeMacroMap {
    pub(crate) macros: HashMap<String, RuntimeMacro>,
}

impl RuntimeMacroMap {
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    pub fn clear_runtime_macros(&mut self) {
        self.macros.clear();
    }

    pub fn contains(&self, key: &str) -> bool {
        self.macros.contains_key(key)
    }

    pub fn new_macro(&mut self, name: &str, mac: RuntimeMacro) {
        self.macros.insert(name.to_string(), mac);
    }

    pub fn undefine(&mut self, name: &str) -> Option<RuntimeMacro> {
        self.macros.remove(name)
    }

    pub fn rename(&mut self, name: &str,new_name: &str) {
        if let Some(mac) = self.macros.remove(name) {
            self.macros.insert(new_name.to_string(), mac);
        }
    }

    pub fn append_macro(&mut self, name: &str, target: &str) {
        if let Some(mac) = self.macros.get_mut(name) {
            mac.body.push_str(target)
        }
    }

    pub fn replace_macro(&mut self, name: &str, target: &str) {
        if let Some(mac) = self.macros.get_mut(name) {
            mac.body = target.to_string();
        }
    }

    pub fn extend_map(&mut self, map: HashMap<String, RuntimeMacro>) {
        self.macros.extend(map)
    }
}
