use crate::utils::RadStr;
use std::env;

use crate::{RadError, RadResult};
use once_cell::sync::Lazy;

pub static PROC_ENV: Lazy<ProcEnv> = Lazy::new(ProcEnv::new);

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

#[derive(Default, Debug)]
pub(crate) struct MacEnv {
    pub rad_tab_width: Option<usize>,
    pub fold_space: bool,
    pub fold_reverse: bool,
    pub fold_trim: bool,
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
