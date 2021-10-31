// Platform agonistic consts
pub const MACRO_START_CHAR: char = '$';
pub const ESCAPE_CHAR: char ='\\';
pub const MAIN_CALLER: &str = "@MAIN@";
#[cfg(feature = "debug")]
pub const DIFF_SOURCE_FILE : &str = "diff.src";
#[cfg(feature = "debug")]
pub const DIFF_OUT_FILE : &str = "diff.out";
pub const COMMENT_CHAR : char = '%';

// Platform specific consts

#[cfg(windows)]
pub const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
pub const LINE_ENDING: &'static str = "\n";
pub const LIT_CHAR: char = '*';

// Option specific consts
#[cfg(feature = "evalexpr")]
pub const ESCAPED_COMMA : &str = "@COMMA@";
#[cfg(feature = "debug")]
pub const RDB_HELP: &'static str = include_str!("debug_help_message.txt");
