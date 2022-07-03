use std::collections::HashMap;
use std::iter::FromIterator;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureMap {
    pub content: HashMap<String, MacroSignature>,
}

impl SignatureMap {
    pub fn new(sigs: Vec<MacroSignature>) -> Self {
        let sig = HashMap::from_iter(sigs.into_iter().map(|sig| (sig.name.clone(), sig)));
        Self { content: sig }
    }
}

// TODO
// use serde::ser::SerializeStruct;
// Placeholder for manual implementation of Serialize
//impl Serialize for SignatureMap {
//fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//where
//S: serde::Serializer {
//let mut state = serializer.serialize_struct("SignatureMap", 1)?;
//state.serialize_field("object", &self.object)?;
//state.end()
//}
//}

/// Type(variant) of macro
#[derive(Debug, Serialize, Deserialize)]
pub enum MacroVariant {
    Deterred,
    Function,
    Runtime,
}

/// Macro signature
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
Description : {}",
            self.variant,
            self.name,
            self.args,
            self.expr,
            self.desc.as_ref().unwrap_or(&String::new()) // This is ugly...
        )
    }
}
