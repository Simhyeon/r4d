//! Multiple constant variables

use once_cell::sync::Lazy;
use regex::Regex;

/// Text display wrapper
///
/// This can be either simple string or "Color" crate's function
#[cfg(feature = "color")]
pub type ColorDisplayFunc = fn(string: &str, to_file: bool) -> Box<dyn std::fmt::Display>;

/// Static source for lorem lipsum
pub const LOREM_SOURCE: &str = "Lorem ipsum dolor sit amet consectetur adipiscing elit. In rhoncus sapien iaculis sapien congue a dictum urna malesuada. In hac habitasse platea dictumst. Quisque dapibus justo a mollis condimentum sapien ligula aliquam massa in vehicula tellus magna vitae enim. Aliquam mattis ligula in enim congue auctor. Pellentesque at sollicitudin velit. Quisque blandit lobortis turpis at malesuada. Donec vitae luctus mauris. Aenean efficitur risus id tortor blandit laoreet. Vestibulum commodo aliquam sapien. Cras aliquam eget leo iaculis cursus. Morbi iaculis justo sed tellus ultrices aliquet. Nam bibendum ut erat quis. ";

/// Static lorem lipsum vector
pub static LOREM: Lazy<Vec<&'static str>> = Lazy::new(|| LOREM_SOURCE.split(' ').collect());
/// Static lorem lipsum vector's length
pub static LOREM_WIDTH: Lazy<usize> = Lazy::new(|| LOREM.len());

/// Get macro start character
///
/// This return custom character if given so
pub(crate) fn macro_start(custom: Option<char>) -> char {
    if let Some(start) = custom {
        start
    } else {
        MACRO_START_CHAR
    }
}

/// Get comment start chracter
///
/// This return custom character if given so
pub(crate) fn comment_start(custom: Option<char>) -> char {
    if let Some(start) = custom {
        start
    } else {
        COMMENT_CHAR
    }
}

// Platform agonistic consts
/// Default macro character
const MACRO_START_CHAR: char = '$';
/// Default comment character
const COMMENT_CHAR: char = '%';

/// Escape character
pub const ESCAPE_CHAR: char = '\\';
/// Literal start character
pub const LIT_CHAR: char = '*';
/// Default main caller
///
/// This is default for input
pub const MAIN_CALLER: &str = "@MAIN@";

pub const MACRO_SPECIAL_ANON: &str = "_ANON_";
pub const MACRO_SPECIAL_LIPSUM: &str = "_LIPSUM_";

// Numbers
// Macro attributes * ^ = - | ~
// Underscore and reverse slash (\)
// Colon (:) for iterated value
// Exclamation ( ! ) for negation
/// Unallowed regex pattern that is reserved for other purposes
pub static UNALLOWED_CHARS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"[a-zA-Z1-9\\_\*\^\|\(\)-=,:~!]"#).expect("Failed to create regex expression")
});

// Diff related
#[cfg(feature = "debug")]
/// Source file for diff operation
pub const DIFF_SOURCE_FILE: &str = "diff.src";
#[cfg(feature = "debug")]
/// Out file for diff operation
pub const DIFF_OUT_FILE: &str = "diff.out";

// LINE ENDING
#[cfg(windows)]
/// Platform specific line ending
pub const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
/// Platform specific line ending
pub const LINE_ENDING: &str = "\n";

// PATH_SEPARATOR
// On windows this should return double forward slash.
// because only double forward slash is guaranteed to be evaluated as single
// forward slash
#[cfg(windows)]
/// Platform specific path separator
pub const PATH_SEPARATOR: &str = "\\\\";
#[cfg(not(windows))]
/// Platform specific path separator
pub const PATH_SEPARATOR: &str = "/";

#[cfg(feature = "debug")]
/// Debug help message string
pub const RDB_HELP: &str = include_str!("debug_help_message.txt");

/// Empty String aRray
pub const ESR: [&str; 0] = [];

/// Define keyword
pub const DEFINE_KEYWORD: &str = "define";
