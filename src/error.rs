use thiserror::Error;

#[allow(dead_code)]
pub(crate) fn print_error() {
    // TODO
    // Print err to stderr or to file or simply ignore or errors
    // Error in this context, means failure of definition and invocatino
    // rather than argument evaluation error which is a critical abort error
    unimplemented!()
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
