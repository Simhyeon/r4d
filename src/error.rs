use thiserror::Error;
use crate::models::WriteOption;
use std::io::Write;
use crate::consts::LINE_ENDING;
use colored::*;

pub struct ErrorLogger {
    last_line_number: u64,
    last_ch_number: u64,
    current_file: String,
    write_option: Option<WriteOption>,
}

impl ErrorLogger{
    pub fn new(write_option: Option<WriteOption>) -> Self {
        Self {
            last_line_number: 0,
            last_ch_number: 0,
            current_file: String::from("stdin"),
            write_option,
        }
    }
    pub fn set_file(&mut self, file: &str) {
        self.current_file = file.to_owned();
    }

    pub fn set_number(&mut self, line_number: u64, ch_number : u64) {
        self.last_line_number = line_number;
        self.last_ch_number = ch_number;
    }

    pub fn elog(&mut self, log : &str) -> Result<(), RadError> {
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(format!("error : {} -> {}:{}:{}{}",log,self.current_file, self.last_line_number, self.last_ch_number,LINE_ENDING).as_bytes())?;
                }
                WriteOption::Stdout => {
                    eprint!(
                        "{}: {} {} --> {}:{}:{}{}",
                        "error".red(),
                        log,
                        LINE_ENDING,
                        self.current_file,
                        self.last_line_number,
                        self.last_ch_number,
                        LINE_ENDING);
                }
            }
        } else {
            // Silent option
            // Do nothing
        }

        Ok(())
    }

    pub fn wlog(&mut self, log : &str) -> Result<(), RadError> {
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(format!("warning : {} -> {}:{}:{}{}",log,self.current_file, self.last_line_number, self.last_ch_number,LINE_ENDING).as_bytes())?;
                }
                WriteOption::Stdout => {
                    eprint!(
                        "{}: {} {} --> {}:{}:{}{}",
                        "warning".yellow(),
                        log,
                        LINE_ENDING,
                        self.current_file,
                        self.last_line_number,
                        self.last_ch_number,
                        LINE_ENDING);
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
    Panic,
}

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
