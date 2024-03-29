//! R4d' error types
//!
//! R4d can have many errors because it utilizes multiple functions and
//! external crates at the same time.

use crate::AuthType;
#[cfg(feature = "cindex")]
use cindex::CIndexError;

/// Blank implementation for error trait
impl std::error::Error for RadError {}

/// R4d's error type
#[derive(Debug)]
pub enum RadError {
    Interrupt,
    HookMacroFail(String),
    InvalidConversion(String),
    UnallowedChar(String),
    AssertFail,
    UnsoundExecution(String),
    InvalidExecution(String),
    InvalidCommandOption(String),
    EnvError(std::env::VarError),
    InvalidMacroReference(String),
    NoSuchMacroName(String, Option<String>),
    InvalidMacroDefinition(String),
    InvalidRegex(regex::Error),
    #[cfg(feature = "evalexpr")]
    InvalidFormula(evalexpr::EvalexprError),
    InvalidArgument(String),
    InvalidArgInt(std::num::ParseIntError),
    InvalidArgBoolean(std::str::ParseBoolError),
    InvalidFile(String),
    StdIo(std::io::Error),
    FmtError(std::fmt::Error),
    Utf8Err(std::string::FromUtf8Error),
    UnsupportedTableFormat(String),
    BincodeError(String),
    PermissionDenied(String, AuthType),
    StrictPanic,
    ManualPanic(String),
    StorageError(String),
    #[cfg(feature = "cindex")]
    CIndexError(CIndexError),
    UnallowedMacroExecution(String),
    DcsvError(dcsv::DcsvError),
    #[cfg(feature = "clap")]
    RadoError(String),
}
impl std::fmt::Display for RadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Interrupt => String::from("Process Interrupted"),
            Self::HookMacroFail(txt) => format!("Hook macro error \n= {}", txt),
            Self::InvalidConversion(txt) => format!("Invalid conversion \n= {}", txt),
            Self::UnallowedChar(txt) => format!("Unallowed character \n= {}", txt),
            Self::AssertFail => "Assert failed".to_string(),
            Self::UnsoundExecution(err) => format!("Critical unsound execution error \n= {}", err),
            Self::InvalidExecution(err) => format!("Invalid execution error \n= {}", err),
            Self::InvalidCommandOption(command) => format!("Invalid command option\n= {}", command),
            Self::EnvError(env) => format!("Invalid environment name\n= {}", env),
            Self::InvalidMacroReference(err) => format!("Invalid macro reference\n= {}", err),
            Self::NoSuchMacroName(given, candidate) => match candidate {
                Some(cand) => {
                    format!("No such macro name as \"{given}\", Did you mean \"{cand}\" ?")
                }
                None => {
                    format!("No such macro name as \"{given}\"")
                }
            },
            Self::InvalidMacroDefinition(err) => format!("Invalid macro definition\n= {}", err),
            Self::InvalidRegex(err) => format!("Failed regex operation\n= {}", err),
            #[cfg(feature = "evalexpr")]
            Self::InvalidFormula(err) => format!("Invalid formula\n= {}", err),
            Self::InvalidArgument(arg) => format!("Invalid argument\n= {}", arg),
            Self::InvalidArgInt(err) => format!("Invalid argument type\n= {}", err),
            Self::InvalidArgBoolean(err) => format!("Invalid argument type\n= {}", err),
            Self::InvalidFile(file) => format!("File,\"{}\", does not exist", file),
            Self::StdIo(err) => format!("Standard IO error\n= {}", err),
            Self::FmtError(err) => format!("Formatting error\n= {}", err),
            Self::Utf8Err(err) => format!("Failed to convert to utf8 string\n= {}", err),
            Self::UnsupportedTableFormat(txt) => format!("Unsupported table format\n= {}", txt),
            Self::BincodeError(txt) => format!("Failed frozen operation\n= {}", txt),
            Self::PermissionDenied(txt, atype) => format!(
                "Permission denied for \"{0}\". Use a flag \"-a {1:?}\" to allow this macro.",
                txt, atype
            ),
            Self::StrictPanic => "Every error is panicking in strict mode".to_string(),
            Self::ManualPanic(txt) => format!("Panic triggered with message\n^^^ {} ^^^", txt),
            Self::StorageError(txt) => format!("Storage error with message\n= {0}", txt),
            #[cfg(feature = "cindex")]
            Self::CIndexError(err) => err.to_string(),
            Self::UnallowedMacroExecution(txt) => {
                format!("Macro execution is not allowed\n= {0}", txt)
            }
            Self::DcsvError(err) => format!("{}", err),
            #[cfg(feature = "clap")]
            Self::RadoError(err) => format!("Rado error \n= {}", err),
        };
        write!(f, "{}", text)
    }
}

// ==========
// Start of Convert variations
// <CONVERT>
impl From<regex::Error> for RadError {
    fn from(err: regex::Error) -> Self {
        Self::InvalidRegex(err)
    }
}

impl From<dcsv::DcsvError> for RadError {
    fn from(err: dcsv::DcsvError) -> Self {
        Self::DcsvError(err)
    }
}

#[cfg(feature = "evalexpr")]
impl From<evalexpr::EvalexprError> for RadError {
    fn from(err: evalexpr::EvalexprError) -> Self {
        Self::InvalidFormula(err)
    }
}

impl From<std::num::ParseIntError> for RadError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::InvalidArgInt(err)
    }
}

impl From<std::str::ParseBoolError> for RadError {
    fn from(err: std::str::ParseBoolError) -> Self {
        Self::InvalidArgBoolean(err)
    }
}

impl From<std::io::Error> for RadError {
    fn from(err: std::io::Error) -> Self {
        Self::StdIo(err)
    }
}

impl From<std::fmt::Error> for RadError {
    fn from(err: std::fmt::Error) -> Self {
        Self::FmtError(err)
    }
}

impl From<std::string::FromUtf8Error> for RadError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Utf8Err(err)
    }
}

impl From<std::env::VarError> for RadError {
    fn from(err: std::env::VarError) -> Self {
        Self::EnvError(err)
    }
}

#[cfg(feature = "cindex")]
impl From<CIndexError> for RadError {
    fn from(err: CIndexError) -> Self {
        Self::CIndexError(err)
    }
}
// End of convert variations
// </CONVERT>
// ----------
