use regex::Regex;

// Get macro start character
pub(crate) fn macro_start(custom: Option<char>) -> char {
    if let Some(start) = custom {
        start
    } else {
        MACRO_START_CHAR
    }
}

// Get comment start chracter
pub(crate) fn comment_start(custom: Option<char>) -> char {
    if let Some(start) = custom {
        start
    } else {
        COMMENT_CHAR
    }
}

// Platform agonistic consts
const MACRO_START_CHAR: char = '$';
const COMMENT_CHAR: char = '%';
pub const ESCAPE_CHAR: char = '\\';
pub const LIT_CHAR: char = '*';
pub const MAIN_CALLER: &str = "@MAIN@";

lazy_static::lazy_static! {
    // Numbers
    // Macro attributes * ^ | +
    // Underscore and reverse slash (\)
    pub static ref UNALLOWED_CHARS: Regex = Regex::new(r#"[a-zA-Z1-9\\_\*\^\|\+\(\)=,]"#).expect("Failed to create regex expression");
}

// Diff related
#[cfg(feature = "debug")]
pub const DIFF_SOURCE_FILE: &str = "diff.src";
#[cfg(feature = "debug")]
pub const DIFF_OUT_FILE: &str = "diff.out";

// Platform specific consts
#[cfg(windows)]
pub const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
pub const LINE_ENDING: &'static str = "\n";

// Option specific consts
#[cfg(feature = "evalexpr")]
pub const ESCAPED_COMMA: &str = "@COMMA@";
#[cfg(feature = "debug")]
pub const RDB_HELP: &'static str = include_str!("debug_help_message.txt");

/// Empty String aRray
pub const ESR: [&str; 0] = [];

pub const DEFINE_KEYWORD: &'static str = "define";
// pub const RAD_READ_CACHE: &'static str = ".rad_read_cache";
