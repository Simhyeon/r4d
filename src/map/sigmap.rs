//! Signature map module

use crate::argument::ValueType;
use crate::{common::ETMap, consts::LINE_ENDING};
use crate::{AuthType, Parameter};
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
    pub name: String,
    pub variant: MacroVariant,
    pub params: Vec<Parameter>,
    pub optional: Option<Parameter>,
    pub optional_multiple: bool,
    pub enum_table: ETMap,
    pub return_type: Option<ValueType>,
    pub required_auth: Vec<AuthType>,
    pub desc: Option<String>,
}

// TODO TT
// Display value table for such parameters
impl std::fmt::Display for MacroSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Parameter construction
        let params = if self.params.is_empty() {
            "[NONE]".to_owned()
        } else {
            self.params
                .iter()
                .map(|p| format!("'{}' : {}", p.name, p.arg_type))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let duplicate = self.params.last().map(|s| s.name.as_ref()).unwrap_or(".")
            == self
                .optional
                .as_ref()
                .map(|s| s.name.as_ref())
                .unwrap_or(".");

        // Optional construction
        let optional = &self
            .optional
            .as_ref()
            .map(|p| {
                if self.optional_multiple {
                    if duplicate {
                        " ... ".to_string()
                    } else {
                        format!(", '[{}'? : {} ... ]", p.name, p.arg_type)
                    }
                } else {
                    format!(", '{}'? : {}", p.name, p.arg_type)
                }
            })
            .unwrap_or_default();

        let mut inner = self
            .params
            .iter()
            .fold(String::new(), |acc, param| acc + &param.name + ",");

        // This removes last "," character
        inner.pop();
        let expr = if let Some(opt) = self.optional.as_ref() {
            // Optional
            if self.optional_multiple {
                if duplicate {
                    format!("${}({}, ... )", self.name, inner)
                } else {
                    let basic_usage = format!("${}({}", self.name, inner);
                    format!("{}) || {},{}?)", basic_usage, basic_usage, opt.name)
                }
            } else {
                let basic_usage = format!("${}({}", self.name, inner);
                format!("{}) || {},{}?)", basic_usage, basic_usage, opt.name)
            }
        } else {
            // No optional
            format!("${}({})", self.name, inner) // Without ending brace
        };

        write!(
            f,
            "Macro Type  : {:#?}
Macro Name  : {}
Parameters  : {}
Return      : {}
Usage       : {}
Required    : {:?}
Description >> 
{}",
            self.variant,
            self.name,
            params + optional,
            self.return_type
                .map(|s| s.to_string())
                .unwrap_or("[NONE]".to_string()),
            expr,
            self.required_auth,
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
