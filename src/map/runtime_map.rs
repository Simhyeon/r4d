//! Runtime macro map
//!
//! Runtime macro is defined on runtime.

use crate::{
    common::{ETMap, Hygiene},
    AuthType, Parameter,
};
#[cfg(feature = "rustc_hash")]
use rustc_hash::FxHashMap as HashMap;
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "rustc_hash"))]
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeMacro {
    pub name: String,
    pub params: Vec<Parameter>,
    pub body: String,
    pub desc: Option<String>,
    pub is_static: bool,
    pub required_auth: Vec<AuthType>,
}

impl RuntimeMacro {
    /// Create a new macro
    pub fn new(name: &str) -> Self {
        // Empty args are no args
        RuntimeMacro {
            name: name.trim().to_string(),
            params: vec![],
            body: String::new(),
            desc: None,
            is_static: false,
            required_auth: vec![],
        }
    }

    pub fn is_static(mut self) -> Self {
        self.is_static = true;
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = body.to_string();
        self
    }

    pub fn params(mut self, params: Vec<Parameter>) -> Self {
        self.params = params;
        self
    }
}

impl std::fmt::Display for RuntimeMacro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut inner = self.params.iter().fold(String::new(), |acc, param| {
            acc + &param.arg_type.to_string() + ","
        });
        // This removes last "," character
        inner.pop();
        write!(f, "${}({})", self.name, inner)
    }
}

impl From<&RuntimeMacro> for crate::sigmap::MacroSignature {
    fn from(mac: &RuntimeMacro) -> Self {
        let variant = if mac.is_static {
            crate::sigmap::MacroVariant::Static
        } else {
            crate::sigmap::MacroVariant::Runtime
        };
        Self {
            variant,
            name: mac.name.to_owned(),
            params: mac.params.to_owned(),
            optional: None,
            enum_table: ETMap::default(),
            expr: mac.to_string(),
            desc: mac.desc.clone(),
            return_type: Some(crate::argument::ValueType::Text),
            required_auth: mac.required_auth.clone(),
        }
    }
}

/// Macro map for runtime macros
#[derive(Clone, Debug)]
pub(crate) struct RuntimeMacroMap {
    pub(crate) macros: HashMap<String, RuntimeMacro>,
    pub(crate) volatile: HashMap<String, RuntimeMacro>,
}

impl RuntimeMacroMap {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
            volatile: HashMap::new(),
        }
    }

    pub fn insert_macro(&mut self, mac: RuntimeMacro) {
        self.macros.insert(mac.name.clone(), mac);
    }

    /// Clear runtime macros
    pub fn clear_runtime_macros(&mut self, volatile: bool) {
        if volatile {
            self.volatile.clear();
        } else {
            self.macros.clear();
        }
    }

    /// Check if macro exists
    pub fn contains(&self, key: &str, hygiene_type: Hygiene) -> bool {
        match hygiene_type {
            Hygiene::Aseptic => self.macros.contains_key(key),
            _ => self.macros.contains_key(key) || self.volatile.contains_key(key),
        }
    }

    /// Get macro by name
    pub fn get(&self, key: &str, hygiene_type: Hygiene) -> Option<&RuntimeMacro> {
        match hygiene_type {
            Hygiene::Aseptic => self.macros.get(key),
            _ => {
                let vol_runtime = self.volatile.get(key);

                if vol_runtime.is_none() {
                    self.macros.get(key)
                } else {
                    vol_runtime
                }
            }
        }
    }

    /// Get macro by name as mutable
    pub fn get_mut(&mut self, key: &str, hygiene_type: Hygiene) -> Option<&mut RuntimeMacro> {
        match hygiene_type {
            Hygiene::Aseptic => self.macros.get_mut(key),
            _ => {
                let vol_runtime = self.volatile.get_mut(key);

                if vol_runtime.is_none() {
                    self.macros.get_mut(key)
                } else {
                    vol_runtime
                }
            }
        }
    }

    /// Append a new macro to a map
    pub fn new_macro(&mut self, name: &str, mac: RuntimeMacro, hygiene_type: Hygiene) {
        if hygiene_type == Hygiene::None {
            self.macros.insert(name.to_string(), mac);
        } else {
            // If hygiene, insert into volatile
            self.volatile.insert(name.to_string(), mac);
        }
    }

    /// Remove a macro from a map
    pub fn undefine(&mut self, name: &str, hygiene_type: Hygiene) -> Option<RuntimeMacro> {
        if hygiene_type == Hygiene::None {
            self.macros.remove(name)
        } else {
            // If hygiene, insert into volatile
            self.volatile.remove(name)
        }
    }

    /// Rename a macro
    pub fn rename(&mut self, name: &str, new_name: &str, hygiene_type: Hygiene) -> bool {
        if hygiene_type == Hygiene::None {
            if let Some(mac) = self.macros.remove(name) {
                self.macros.insert(new_name.to_string(), mac);
                return true;
            }
        } else if let Some(mac) = self.volatile.remove(name) {
            self.volatile.insert(new_name.to_string(), mac);
            return true;
        }
        false
    }

    /// Append content to a macro
    pub fn append_macro(&mut self, name: &str, target: &str, hygiene_type: Hygiene) {
        if hygiene_type == Hygiene::None {
            if let Some(mac) = self.macros.get_mut(name) {
                mac.body.push_str(target);
            }
        } else if let Some(mac) = self.volatile.get_mut(name) {
            mac.body.push_str(target);
        }
    }

    /// Replace macro with new name
    pub fn replace_macro(&mut self, name: &str, target: &str, hygiene_type: Hygiene) {
        if hygiene_type == Hygiene::None {
            if let Some(mac) = self.macros.get_mut(name) {
                mac.body = target.to_string();
            }
        } else if let Some(mac) = self.volatile.get_mut(name) {
            mac.body.push_str(target)
        }
    }

    /// Extend map with other hashmap
    pub fn extend_map(&mut self, map: HashMap<String, RuntimeMacro>, hygiene_type: Hygiene) {
        if hygiene_type == Hygiene::None {
            self.macros.extend(map)
        } else {
            self.volatile.extend(map)
        }
    }
}
