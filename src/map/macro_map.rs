//! Main entry for macro maps ( Local, runtime, deterred, function )

use crate::common::Hygiene;
use crate::common::LocalMacro;
use crate::deterred_map::DeterredMacroMap;
use crate::function_map::FunctionMacroMap;
use crate::runtime_map::{RuntimeMacro, RuntimeMacroMap};
#[cfg(feature = "signature")]
use crate::sigmap::MacroSignature;
use crate::trim;
use crate::utils::Utils;
use crate::MacroType;
use crate::RadResult;
use std::collections::HashMap;

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

    /// Clear runtime macros
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

    /// Check if given macro is deterred macro
    pub fn is_deterred_macro(&self, name: &str) -> bool {
        self.deterred.contains(name)
    }

    /// Check if local macro exists
    pub fn contains_local_macro(&self, macro_name: &str) -> bool {
        self.local.contains_key(macro_name)
    }

    /// Check if macro exists
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
        let mac = RuntimeMacro::new(&trim!(name), &trim!(args), body, false);
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

    /// Rename a macro
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
                // Order is
                // - runtime
                // - deterred
                // - function
                if !self.runtime.rename(macro_name, target_name, hygiene_type)
                    && !self.deterred.rename(macro_name, target_name)
                {
                    self.function.rename(macro_name, target_name);
                }
            }
        }
    }

    /// Append content to a local macro
    pub fn append_local(&mut self, name: &str, target: &str) {
        if let Some(loc) = self.local.get_mut(name) {
            loc.body.push_str(target);
        }
    }

    /// Append content to a macro
    pub fn append(&mut self, name: &str, target: &str, hygiene_type: Hygiene) {
        if self.runtime.contains(name, hygiene_type) {
            self.runtime.append_macro(name, target, hygiene_type);
        }
    }

    /// Replace macro's content
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

    /// Get function signatures
    #[cfg(feature = "signature")]
    pub fn get_function_signatures(&self) -> Vec<MacroSignature> {
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

    /// Get runtime signatures
    #[cfg(feature = "signature")]
    pub fn get_runtime_signatures(&self) -> Vec<MacroSignature> {
        self.runtime
            .macros
            .iter()
            .map(|(_, mac)| MacroSignature::from(mac))
            .collect()
    }
}
