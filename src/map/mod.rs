//! Map includes multiple data collection map
//!
//! - macro map
//! - function_map
//! - deterred_map

pub mod deterred_map;
pub mod function_map;
mod macro_map;
pub(crate) use macro_map::MacroMap;
#[cfg(feature = "hook")]
pub mod hookmap;
pub mod runtime_map;
#[cfg(feature = "signature")]
pub mod sigmap;
