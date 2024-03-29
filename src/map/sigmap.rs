//! Signature map module

use crate::consts::LINE_ENDING;
use serde::{Deserialize, Serialize};
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

/// Macro signature struct
#[derive(Debug, Serialize, Deserialize)]
pub struct MacroSignature {
    pub variant: MacroVariant,
    pub name: String,
    pub args: Vec<String>,
    pub expr: String,
    pub desc: Option<String>,
}

impl std::fmt::Display for MacroSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Macro Type  : {:#?}
Macro Name  : {}
Arguments   : {:?}
Usage       : {}
Description >> 
{}",
            self.variant,
            self.name,
            self.args,
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
