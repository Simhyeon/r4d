use std::collections::HashMap;
use std::iter::FromIterator;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureMap {
    pub object: HashMap<String, MacroSignature>,
}

impl SignatureMap {
    pub fn new(sigs: Vec<MacroSignature>) -> Self {
        let sig = HashMap::from_iter(sigs.into_iter().map(|sig| (sig.name.clone(), sig)));
        Self { object: sig }
    }
}

/// Type(variant) of macro
#[derive(Debug, Serialize, Deserialize)]
pub enum MacroVariant {
    Keyword,
    Basic,
    Custom,
}

/// Macro signature
#[derive(Debug, Serialize, Deserialize)]
pub struct MacroSignature {
    pub variant: MacroVariant,
    pub name: String,
    pub args: Vec<String>,
    pub expr: String,
}
