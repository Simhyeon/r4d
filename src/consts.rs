pub const MACRO_START_CHAR: char = '$';
pub const ESCAPE_CHAR: char ='\\';
pub const MAIN_CALLER: &str = "@MAIN@";

#[cfg(windows)]
pub const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
pub const LINE_ENDING: &'static str = "\n";
