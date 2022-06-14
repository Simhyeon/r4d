use regex::Regex;

#[cfg(feature = "color")]
pub type ColorDisplayFunc = Option<fn(string: &str) -> Box<dyn std::fmt::Display>>;

pub const LOREM_SOURCE: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In rhoncus sapien iaculis sapien congue, a dictum urna malesuada. In hac habitasse platea dictumst. Quisque dapibus, justo a mollis condimentum, sapien ligula aliquam massa, in vehicula tellus magna vitae enim. Aliquam mattis ligula in enim congue auctor. Pellentesque at sollicitudin velit. Quisque blandit lobortis turpis at malesuada. Donec vitae luctus mauris. Aenean efficitur risus id tortor blandit laoreet. Vestibulum commodo aliquam sapien. Cras aliquam eget leo iaculis cursus. Morbi iaculis justo sed tellus ultrices aliquet. Nam bibendum ut erat quis. ";

lazy_static::lazy_static! {
    pub static ref LOREM: Vec<&'static str> = LOREM_SOURCE.split(' ').collect();
    pub static ref LOREM_WIDTH: usize = LOREM.len();
}

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
    pub static ref UNALLOWED_CHARS: Regex = Regex::new(r#"[a-zA-Z1-9\\_\*\^\|\(\)=,]"#).expect("Failed to create regex expression");
}

// Diff related
#[cfg(feature = "debug")]
pub const DIFF_SOURCE_FILE: &str = "diff.src";
#[cfg(feature = "debug")]
pub const DIFF_OUT_FILE: &str = "diff.out";

// Read cache
pub const READ_CACHE: &str = ".R4D_READ_CACHE";

// Platform specific consts
#[cfg(windows)]
pub const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
pub const LINE_ENDING: &str = "\n";

#[cfg(feature = "debug")]
pub const RDB_HELP: &str = include_str!("debug_help_message.txt");

/// Empty String aRray
pub const ESR: [&str; 0] = [];

pub const DEFINE_KEYWORD: &str = "define";
// pub const RAD_READ_CACHE: &str = ".rad_read_cache";
