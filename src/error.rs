use thiserror::Error;
use crate::models::WriteOption;
use std::io::Write;
use crate::consts::LINE_ENDING;
use colored::*;

pub struct ErrorLogger {
    last_line_number: usize,
    last_ch_number: usize,
    current_file: String,
    write_option: Option<WriteOption>,
    error_count: usize,
    warning_count: usize,
}

impl ErrorLogger{
    pub fn new(write_option: Option<WriteOption>) -> Self {
        Self {
            last_line_number: 0,
            last_ch_number: 0,
            current_file: String::from("stdin"),
            write_option,
            error_count:0,
            warning_count:0,
        }
    }
    pub fn set_file(&mut self, file: &str) {
        self.current_file = file.to_owned();
    }

    pub fn set_number(&mut self, line_number: usize, ch_number : usize) {
        self.last_line_number = line_number;
        self.last_ch_number = ch_number;
    }

    pub fn elog(&mut self, log : &str) -> Result<(), RadError> {
        self.error_count = self.error_count + 1; 
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
        self.warning_count = self.warning_count + 1; 
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(format!("warning : {} -> {}:{}:{}{}",log,self.current_file, self.last_line_number, self.last_ch_number,LINE_ENDING).as_bytes())?;
                }
                WriteOption::Stdout => {
                    eprintln!(
                        "{}: {} {} --> {}:{}:{}",
                        "warning".yellow(),
                        log,
                        LINE_ENDING,
                        self.current_file,
                        self.last_line_number,
                        self.last_ch_number);
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
    #[error("{}: Invalid command option\n= {0}", "error".red())]
    InvalidCommandOption(String),
    #[error("{}: Invalid environment name\n= {0}", "error".red())]
    EnvError(std::env::VarError),
    #[error("{}: Failed regex operation\n= {0}", "error".red())]
    InvalidRegex(regex::Error),
    #[error("{}: Invalid formula\n= {0}", "error".red())]
    InvalidFormula(evalexpr::EvalexprError),
    #[error("{}: Invalid argument\n= {0}", "error".red())]
    InvalidArgument(String),
    #[error("{}: Invalid argument type\n= {0}", "error".red())]
    InvalidArgInt(std::num::ParseIntError),
    #[error("{}: Invalid argument type\n= {0}", "error".red())]
    InvalidArgBoolean(std::str::ParseBoolError),
    #[error("{}: Standard IO error\n= {0}", "error".red())]
    StdIo(std::io::Error),
    #[error("{}: Failed to convert to utf8 string\n= {0}", "error".red())]
    Utf8Err(std::string::FromUtf8Error),
    #[error("{}: Unsupported table format\n= {0}", "error".red())]
    UnsupportedTableFormat(String),
    #[error("{}: Table error\n= {0}", "error".red())]
    CsvError(csv::Error),
    #[error("{}: Failed frozen operation\n= {0}", "error".red())]
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
