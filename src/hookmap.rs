use std::collections::HashMap;
use crate::{RadResult, RadError};

#[derive(Debug)]
pub struct HookMap {
    macro_hook : HashMap<String,HookState>,
    char_hook : HashMap<char,HookState>,
}

impl HookMap {
    pub fn new() -> Self {
        Self {
            macro_hook: HashMap::new(),
            char_hook: HashMap::new(),
        }
    }

    // TODO 
    // Make the code more 'DRY' for add_*_count series
    pub fn add_macro_count(&mut self, macro_name: &str) -> Option<String> {
        if let Some(hook_state) = self.macro_hook.get_mut(macro_name) {
            if hook_state.enabled {
                hook_state.current_count += 1; 
                if hook_state.current_count == hook_state.target_count {
                    hook_state.current_count = 0; // reset count
                    if !hook_state.resetable { 
                        hook_state.enabled = false;
                    }
                    return Some(hook_state.target_macro.clone());
                } 
            }
        }
        None
    }

    pub fn add_char_count(&mut self, target: char) -> Option<String> {
        if let Some(hook_state) = self.char_hook.get_mut(&target) {
            if hook_state.enabled {
                hook_state.current_count += 1; 
                if hook_state.current_count == hook_state.target_count {
                    hook_state.current_count = 0; // reset count
                    if !hook_state.resetable { 
                        hook_state.enabled = false;
                    }
                    return Some(hook_state.target_macro.clone());
                } 
            }
        }
        None
    }

    pub fn add_letter_count(&mut self) -> Option<String> {
        if let Some(hook_state) = &mut self.letter_hook {
            if hook_state.enabled {
                hook_state.current_count += 1; 
                if hook_state.current_count == hook_state.target_count {
                    hook_state.current_count = 0; // reset count
                    if !hook_state.resetable { 
                        hook_state.enabled = false;
                    }
                    return Some(hook_state.target_macro.clone());
                } 
            }
        }
        None
    }

    pub fn switch_hook(&mut self, hook_type : HookType, index: &str, switch: bool) -> RadResult<()> {
        match hook_type {
            HookType::Macro => {
                if let Some(state) = self.macro_hook.get_mut(index) {
                    state.enabled = switch
                } else { return Err(RadError::InvalidArgument(format!("Hook trigger \"{}\" is not registered as macro hook", index))) }
            },
            HookType::Char => {
                let index_char = if let Some(ch) = index.chars().next() { ch } 
                else { return Err(RadError::HookMacroFail("Index is empty".to_owned())) };

                if let Some(state) = self.char_hook.get_mut(&index_char) {
                    state.enabled = switch
                } else { return Err(RadError::HookMacroFail(format!("Hook trigger \"{}\" is not registered as character hook", index))) }
            },
            HookType::Letter => {
                if let Some(hook_state) = &mut self.letter_hook { 
                    hook_state.enabled = switch; 
                }
            }
        };
        Ok(())
    }

    pub fn add_hook(&mut self, hook_type : HookType, target: &str, invoke_macro: &str,target_count: usize, resetable : bool) -> RadResult<()> {
        let hook_state = HookState::new(invoke_macro.to_owned(),target_count,resetable);
        match hook_type {
            HookType::Macro => {self.macro_hook.insert(target.to_owned(),hook_state);}
            HookType::Char => {
                let index_char = if let Some(ch) = target.chars().next() { ch } else {  return Err(RadError::HookMacroFail("Index is empty".to_owned())) };
                self.char_hook.insert(index_char,hook_state);
            },
            HookType::Letter => { self.letter_hook.replace(hook_state); } 
        };
        Ok(())
    }

    pub fn del_hook(&mut self, hook_type : HookType, index: &str) -> RadResult<()> {
        match hook_type {
            HookType::Char => {self.macro_hook.remove(index);}
            HookType::Macro => {
                let index_char = if let Some(ch) = index.chars().next() { ch } else {  return Err(RadError::HookMacroFail("Index is empty".to_owned())) };
                self.char_hook.remove(&index_char);
            },
            HookType::Letter => { self.letter_hook = None; }
        };
        Ok(())
    }
}

#[derive(Debug)]
pub enum HookType {
    Macro,
    Char,
    Letter,
}

impl HookType {
    pub fn from_str(hook_type : &str) -> RadResult<Self> {
        let var = match hook_type.to_lowercase().as_str() {
            "macro" => Self::Macro,
            "char" => Self::Char,
            "letter" => Self::Letter,
            _ => return Err(RadError::InvalidConversion(format!("Invalid hook type \"{}\"", hook_type)))
        };

        Ok(var)
    }
}

#[derive(Debug)]
pub struct HookState {
    enabled: bool,
    resetable: bool,
    target_macro: String,
    current_count: usize,
    target_count: usize,
}

impl HookState {
    pub fn new(target: String, target_count: usize, resetable: bool) -> Self {
        Self { 
            target_macro: target,
            enabled: false,
            resetable,
            current_count: 0usize,
            target_count
        }
    }
}
