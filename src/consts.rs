pub const MACRO_START_CHAR: char = '$';
pub const ESCAPE_CHAR: char ='\\';
pub const MAIN_CALLER: &str = "@MAIN@";
pub const ESCAPED_COMMA : &str = "@COMMA@";

#[cfg(windows)]
pub const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
pub const LINE_ENDING: &'static str = "\n";

pub const TEMP_FILE: &'static str = "radtemp.txt";

pub const LIT_CHAR: char = '*';
