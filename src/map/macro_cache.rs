use std::collections::HashMap;

use crate::MacroType;

use super::{
    deterred_map::DFunctionMacroType, function_map::FunctionMacroType, runtime_map::RuntimeMacro,
};

pub enum Cache {
    Function(FunctionMacroType),
    Deterred(DFunctionMacroType),
    Runtime(RuntimeMacro),
}

impl Cache {
    pub fn new_runtime_cache(cache: RuntimeMacro) -> Self {
        Self::Runtime(cache)
    }
    pub fn new_function_cache(cache: FunctionMacroType) -> Self {
        Self::Function(cache)
    }
    pub fn new_deterred_cache(cache: DFunctionMacroType) -> Self {
        Self::Deterred(cache)
    }
}

pub struct MacroCache {
    pub map: HashMap<String, Cache>,
}

impl MacroCache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add_cache(&mut self, name: String, cache: Cache) {
        self.map.insert(name, cache);
    }

    pub fn get_with_type(&mut self, macro_name: &str, macro_type: MacroType) -> Option<Cache> {
        let mac = self.map.remove(macro_name);

        // If none, return none
        if mac.is_none() {
            return mac;
        }

        let mac = mac.unwrap();

        match macro_type {
            MacroType::Any => Some(mac),
            MacroType::Function => {
                if let Cache::Function(_) = mac {
                    Some(mac)
                } else {
                    None
                }
            }
            MacroType::Deterred => {
                if let Cache::Deterred(_) = mac {
                    Some(mac)
                } else {
                    None
                }
            }
            MacroType::Runtime => {
                if let Cache::Runtime(_) = mac {
                    Some(mac)
                } else {
                    None
                }
            }
        }
    }

    pub fn get(&mut self, macro_name: &str) -> Option<Cache> {
        self.map.remove(macro_name)
    }
}
