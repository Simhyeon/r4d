use thiserror::Error;
use crate::models::WriteOption;
use std::io::Write;
use crate::consts::LINE_ENDING;
use colored::*;

/// Logger that controls whether to print error, warning or not and where to yield the error and warnings.
pub struct ErrorLogger {
    line_number: usize,
    char_number: usize,
    last_line_number: usize,
    last_char_number: usize,
    current_file: String,
    write_option: Option<WriteOption>,
    error_count: usize,
    warning_count: usize,
}
/// Struct specifically exists for error loggers backup information
pub struct LoggerLines {
    line_number: usize,
    char_number: usize,
    last_line_number: usize,
    last_char_number: usize,
}

impl ErrorLogger{
    pub fn new(write_option: Option<WriteOption>) -> Self {
        Self {
            line_number: 0,
            char_number: 0,
            last_line_number: 0,
            last_char_number: 0,
            current_file: String::from("stdin"),
            write_option,
            error_count:0,
            warning_count:0,
        }
    }

    /// Backup current line information into a struct
    pub fn backup_lines(&self) -> LoggerLines {
        LoggerLines { line_number: self.line_number, char_number: self.char_number, last_line_number: self.last_line_number, last_char_number: self.last_char_number }
    }

    /// Recover backuped line information from a struct
    pub fn recover_lines(&mut self, logger_lines: LoggerLines) {
        self.line_number =          logger_lines.line_number;
        self.char_number =          logger_lines.char_number;
        self.last_line_number =     logger_lines.last_line_number;
        self.last_char_number =     logger_lines.last_char_number;
    }

    /// Set file's logging information
    pub fn set_file(&mut self, file: &str) {
        self.current_file = file.to_owned();
        self.line_number = 0;
        self.char_number = 0;
        self.last_line_number = 0;
        self.last_char_number = 0;
    }

    /// Increase line number
    pub fn add_line_number(&mut self) {
        self.line_number = self.line_number + 1;
    }
    /// Increase char number
    pub fn add_char_number(&mut self) {
        self.char_number = self.char_number + 1;
    }
    /// Reset char number
    pub fn reset_char_number(&mut self) {
        self.char_number = 0;
    }
    /// Freeze line and char number for logging
    pub fn freeze_number(&mut self) {
        self.last_line_number = self.line_number;
        self.last_char_number = self.char_number;
    }

    pub fn elog_panic(&mut self, log: &str, error: RadError) -> Result<(), RadError> {
        self.elog(log)?;

        Err(error)
    }

    /// Log error
    pub fn elog(&mut self, log : &str) -> Result<(), RadError> {
        self.error_count = self.error_count + 1; 
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(format!("error : {} -> {}:{}:{}{}",log,self.current_file, self.line_number, self.char_number,LINE_ENDING).as_bytes())?;
                }
                WriteOption::Stdout => {
                    eprint!(
                        "{}: {} {} --> {}:{}:{}{}",
                        "error".red(),
                        log,
                        LINE_ENDING,
                        self.current_file,
                        self.last_line_number,
                        self.last_char_number,
                        LINE_ENDING);
                }
            }
        } else {
            // Silent option
            // Do nothing
        }

        Ok(())
    }

    /// Log warning
    pub fn wlog(&mut self, log : &str) -> Result<(), RadError> {
        self.warning_count = self.warning_count + 1; 
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(format!("warning : {} -> {}:{}:{}{}",log,self.current_file, self.line_number, self.char_number,LINE_ENDING).as_bytes())?;
                }
                WriteOption::Stdout => {
                    eprintln!(
                        "{}: {} {} --> {}:{}:{}",
                        "warning".yellow(),
                        log,
                        LINE_ENDING,
                        self.current_file,
                        self.last_line_number,
                        self.last_char_number);
                }
            }
        } else {
            // Silent option
            // Do nothing
        }

        Ok(())
    }

    pub fn print_result(&mut self) -> Result<(), RadError> {
        if let Some(option) = &mut self.write_option {
            // No warning or error
            if self.error_count == 0 && self.warning_count == 0 {
                return Ok(())
            }
    
            // There is either error or warning
            let error_result = format!("{}: found {} errors","error".red(), self.error_count);
            let warning_result = format!("{}: found {} warnings","warning".yellow(), self.warning_count);
            match option {
                WriteOption::File(file) => {
                    if self.error_count > 0 {file.write_all(error_result.as_bytes())?;}
                    if self.warning_count > 0 {file.write_all(warning_result.as_bytes())?;}
                }
                WriteOption::Stdout => {
                    if self.error_count > 0 { eprintln!("{}",error_result);}
                    if self.warning_count > 0 {eprintln!("{}",warning_result);}
                }
            }
        } else {
            // Silent option
            // Do nothing
        }

        Ok(())
    }
} 

#[derive(Error, Debug)]
pub enum RadError {
    #[error("Invalid command option\n= {0}")]
    InvalidCommandOption(String),
    #[error("Invalid environment name\n= {0}")]
    EnvError(std::env::VarError),
    #[error("Failed regex operation\n= {0}")]
    InvalidRegex(regex::Error),
    #[error("Invalid formula\n= {0}")]
    InvalidFormula(evalexpr::EvalexprError),
    #[error("Invalid argument\n= {0}")]
    InvalidArgument(String),
    #[error("Invalid argument type\n= {0}")]
    InvalidArgInt(std::num::ParseIntError),
    #[error("Invalid argument type\n= {0}")]
    InvalidArgBoolean(std::str::ParseBoolError),
    #[error("Standard IO error\n= {0}")]
    StdIo(std::io::Error),
    #[error("Failed to convert to utf8 string\n= {0}")]
    Utf8Err(std::string::FromUtf8Error),
    #[error("Unsupported table format\n= {0}")]
    UnsupportedTableFormat(String),
    #[error("Table error\n= {0}")]
    CsvError(csv::Error),
    #[error("Failed frozen operation\n= {0}")]
    BincodeError(String),
    #[error("Processor panicked, exiting...")]
    StrictPanic,
    #[error("Processor panicked, exiting...")]
    Panic,
}

// ==========
// -->> Convert variations
impl From<regex::Error> for RadError {
    fn from(err : regex::Error) -> Self {
        Self::InvalidRegex(err)
    }
}

impl From<evalexpr::EvalexprError> for RadError {
    fn from(err : evalexpr::EvalexprError) -> Self {
        Self::InvalidFormula(err)
    }
}

impl From<std::num::ParseIntError> for RadError {
    fn from(err : std::num::ParseIntError) -> Self {
        Self::InvalidArgInt(err)
    }
}

impl From<std::str::ParseBoolError> for RadError {
    fn from(err : std::str::ParseBoolError) -> Self {
        Self::InvalidArgBoolean(err)
    }
}

impl From<std::io::Error> for RadError {
    fn from(err : std::io::Error) -> Self {
        Self::StdIo(err)
    }
}

impl From <std::string::FromUtf8Error> for RadError {
    fn from(err : std::string::FromUtf8Error) -> Self {
        Self::Utf8Err(err)
    }
}

impl From <csv::Error> for RadError {
    fn from(err : csv::Error) -> Self {
        Self::CsvError(err)
    }
}

impl From <std::env::VarError> for RadError {
    fn from(err : std::env::VarError) -> Self {
        Self::EnvError(err)
    }
}
