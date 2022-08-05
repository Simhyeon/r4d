//! Map includes multiple data collection map
//!
//!
//! Macro map is a main regulator with following contents
//!
//! - anon_map
//! - deterred_map
//! - function_map
//! - hook_map
//! - runtime_map
//! - sig_map

pub mod anon_map;
pub mod deterred_map;
pub mod function_map;
mod macro_map;
pub(crate) use macro_map::MacroMap;
#[cfg(feature = "hook")]
pub mod hookmap;
pub mod runtime_map;
#[cfg(feature = "signature")]
pub mod sigmap;
