use thiserror::Error;
use crate::models::WriteOption;
use std::io::Write;
use crate::consts::LINE_ENDING;
use colored::*;

pub struct ErrorLogger {
    write_option: Option<WriteOption>,
}

impl ErrorLogger{
    pub fn new(write_option: Option<WriteOption>) -> Self {
        Self {
            write_option,
        }
    }

    pub fn elog(&mut self, log : &str) -> Result<(), RadError> {
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(format!("{}{}",log,LINE_ENDING).as_bytes())?;
                }
                WriteOption::Stdout => {
                    eprintln!("{}", log.red());
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
    #[error("Failed regex operation : {0}")]
    InvalidRegex(regex::Error),
    #[error("Invalid formula : {0}")]
    InvalidFormula(evalexpr::EvalexprError),
    #[error("Invalid argument : {0}")]
    InvalidArgument(&'static str),
    #[error("Invalid argument type: {0}")]
    InvalidArgInt(std::num::ParseIntError),
    #[error("Invalid argument type: {0}")]
    InvalidArgBoolean(std::str::ParseBoolError),
    #[error("Standard IO error : {0}")]
    StdIo(std::io::Error),
    #[error("Failed to convert to utf8 string : {0}")]
    Utf8Err(std::string::FromUtf8Error),
    #[error("Unsupported table format : {0}")]
    UnsupportedTableFormat(String),
    #[error("Table error : {0}")]
    CsvError(csv::Error),
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
