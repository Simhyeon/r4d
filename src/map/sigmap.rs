//! Signature map module

use crate::argument::ValueType;
use crate::Parameter;
use crate::{common::ETMap, consts::LINE_ENDING};
#[cfg(feature = "rustc_hash")]
use rustc_hash::FxHashMap as HashMap;
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "rustc_hash"))]
use std::collections::HashMap;
use std::iter::FromIterator;

/// Map for macro signatures
#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureMap {
    pub content: HashMap<String, MacroSignature>,
}

impl SignatureMap {
    /// Create a new instance
    pub fn new(sigs: Vec<MacroSignature>) -> Self {
        let sig = HashMap::from_iter(sigs.into_iter().map(|sig| (sig.name.clone(), sig)));
        Self { content: sig }
    }
}

/// Type(variant) of macro
#[derive(Debug, Serialize, Deserialize)]
pub enum MacroVariant {
    Deterred,
    Function,
    Runtime,
    Static,
}

// TODO TT
/// Macro signature struct
#[derive(Debug, Serialize, Deserialize)]
pub struct MacroSignature {
    pub variant: MacroVariant,
    pub name: String,
    pub params: Vec<Parameter>,
    pub optional: Option<Parameter>,
    pub enum_table: ETMap,
    pub expr: String,
    pub desc: Option<String>,
    pub return_type: Option<ValueType>,
}

// TODO TT
// Display value table for such parameters
impl std::fmt::Display for MacroSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params = if self.params.is_empty() {
            "[NONE]".to_owned()
        } else {
            self.params
                .iter()
                .map(|p| format!("'{}' : {}", p.name, p.arg_type))
                .collect::<Vec<_>>()
                .join(", ")
        };
        write!(
            f,
            "Macro Type  : {:#?}
Macro Name  : {}
Parameters  : {}
Return      : {}
Usage       : {}
Description >> 
{}",
            self.variant,
            self.name,
            params
                + &self
                    .optional
                    .as_ref()
                    .map(|p| format!(", '{}'? : {}", p.name, p.arg_type))
                    .unwrap_or_default(),
            self.return_type
                .map(|s| s.to_string())
                .unwrap_or("[NONE]".to_string()),
            self.expr,
            self.desc
                .as_ref()
                .map(|d| d
                    .lines()
                    .map(|line| "    ".to_owned() + line)
                    .collect::<Vec<_>>()
                    .join(LINE_ENDING))
                .unwrap_or_default()
        )
    }
}
