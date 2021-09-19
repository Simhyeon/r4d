use thiserror::Error;

/// R4d's error type
#[derive(Error, Debug)]
pub enum RadError {
    #[error("Invalid command option\n= {0}")]
    InvalidCommandOption(String),
    #[error("Invalid environment name\n= {0}")]
    EnvError(std::env::VarError),
    #[error("Failed regex operation\n= {0}")]
    InvalidRegex(regex::Error),
    #[cfg(feature = "evalexpr")]
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
    #[cfg(feature = "csv")]
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

#[cfg(feature = "evalexpr")]
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

#[cfg(feature = "csv")]
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
