use crate::utils::RadStr;
use std::env;

use crate::{RadError, RadResult};
use once_cell::sync::Lazy;

pub static PROC_ENV: Lazy<ProcEnv> = Lazy::new(ProcEnv::new);

/// This environmnet changes how processor works
///
/// This environmnet cannot be overriden by design
#[derive(Debug)]
pub struct ProcEnv {
    pub(crate) no_consume: bool,
    pub(crate) no_color_print: bool,
    pub(crate) backtrace: bool,
}

impl ProcEnv {
    pub fn new() -> Self {
        Self {
            no_consume: set_env_safely("RAD_NO_CONSUME"),
            no_color_print: set_env_safely("RAD_NO_COLOR"),
            backtrace: set_env_safely("RAD_BACKTRACE"),
        }
    }
}

/// This environmnet changes how macro works
///
/// This environmnet can be overriden by design
#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct MacEnv {
    pub rad_tab_width: Option<usize>,
    pub fold_space: bool,
    pub fold_reverse: bool,
    pub fold_trim: bool,
    pub map_preserve: bool,
    pub split_for_space: bool,
    pub rotatei_order: bool,
    pub no_negative_index: bool,
    pub disable_map_quote: bool,

    // Feature gated
    #[cfg(feature = "evalexpr")]
    pub retain_formula: bool,
    #[cfg(feature = "evalexpr")]
    pub formula_space: bool,
}

impl MacEnv {
    pub fn new() -> RadResult<Self> {
        let rad_tab_width = match env::var("RAD_TAB_WIDTH") {
            Ok(v) => Some(v.parse::<usize>().map_err(|_| {
                RadError::InvalidMacroEnvironment(
                    "RAD TAB WIDTH should be a unsigned integer.".to_string(),
                )
            })?),
            Err(_) => None,
        };
        Ok(Self {
            rad_tab_width,
            fold_space: set_env_safely("RAD_FOLD_SPACE"),
            fold_reverse: set_env_safely("RAD_FOLD_REVERSE"),
            fold_trim: set_env_safely("RAD_FOLD_TRIM"),
            map_preserve: set_env_safely("RAD_MAP_PRESERVE"),
            split_for_space: set_env_safely("RAD_SPLIT_SPACE"),
            rotatei_order: set_env_safely("RAD_ROTATEI_ORDER"),
            no_negative_index: set_env_safely("RAD_NO_NIN"),
            disable_map_quote: set_env_safely("RAD_NO_MAP_QUOTE"),

            // Feature gated
            #[cfg(feature = "evalexpr")]
            retain_formula: set_env_safely("RAD_RETAIN_FORMULA"),
            #[cfg(feature = "evalexpr")]
            formula_space: set_env_safely("RAD_FORMULA_SPACE"),
        })
    }
}

/// Get env value as boolean with failing
fn set_env_safely(name: &str) -> bool {
    env::var(name)
        .ok()
        .map(|s| s.is_arg_true_infallable())
        .unwrap_or(false)
}
