use crate::AuthType;
#[cfg(feature = "cindex")]
use cindex::CIndexError;
#[cfg(feature = "csv")]
use csv::FromUtf8Error;

/// R4d's error type
#[derive(Debug)]
pub enum RadError {
    Exit,
    HookMacroFail(String),
    InvalidConversion(String),
    UnallowedChar(String),
    AssertFail,
    InvalidExecution(String),
    #[cfg(feature = "csv")]
    InvalidString(FromUtf8Error),
    #[cfg(not(feature = "csv"))]
    InvalidString(std::str::Utf8Error),
    InvalidCommandOption(String),
    EnvError(std::env::VarError),
    InvalidMacroName(String),
    InvalidRegex(regex::Error),
    #[cfg(feature = "evalexpr")]
    InvalidFormula(evalexpr::EvalexprError),
    InvalidArgument(String),
    InvalidArgInt(std::num::ParseIntError),
    InvalidArgBoolean(std::str::ParseBoolError),
    InvalidFile(String),
    StdIo(std::io::Error),
    Utf8Err(std::string::FromUtf8Error),
    UnsupportedTableFormat(String),
    #[cfg(feature = "csv")]
    CsvError(csv::Error),
    BincodeError(String),
    PermissionDenied(String, AuthType),
    StrictPanic,
    Panic,
    ManualPanic(String),
    #[cfg(feature = "storage")]
    StorageError(String),
    #[cfg(feature = "cindex")]
    CIndexError(CIndexError),
    UnallowedMacroExecution(String),
}

impl std::fmt::Display for RadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Exit =>"Exited manually".to_string(),
            Self::HookMacroFail(txt) => format!("Hook macro error \n= {}",txt),
            Self::InvalidConversion(txt)=> format!("Invalid conversion \n= {}",txt),
            Self::UnallowedChar(txt)=> format!("Unallowed character \n= {}",txt),
            Self::AssertFail=> "Assert failed".to_string(),
            Self::InvalidExecution(err)=> format!("Invalid execution error \n= {}",err),
            #[cfg(feature = "csv")]
            Self::InvalidString(err) => format!("Invalid byte array conversion to string \n={}",err),
            #[cfg(not(feature = "csv"))]
            Self::InvalidString(err) => format!("Invalid conversion to string \n={}",err),
            Self::InvalidCommandOption(command) => format!("Invalid command option\n= {}",command),
            Self::EnvError(env) => format!("Invalid environment name\n= {}",env),
            Self::InvalidMacroName(name)=> format!("Invalid macro name\n= {}",name),
            Self::InvalidRegex(err) => format!("Failed regex operation\n= {}",err),
            #[cfg(feature = "evalexpr")]
            Self::InvalidFormula(err)=> format!("Invalid formula\n= {}",err),
            Self::InvalidArgument(arg) => format!("Invalid argument\n= {}",arg),
            Self::InvalidArgInt(err)=>format!("Invalid argument type\n= {}",err) ,
            Self::InvalidArgBoolean(err)=> format!("Invalid argument type\n= {}",err),
            Self::InvalidFile(file)=> format!("File,\"{}\", does not exist",file),
            Self::StdIo(err) => format!("Standard IO error\n= {}",err),
            Self::Utf8Err(err) =>format!("Failed to convert to utf8 string\n= {}",err),
            Self::UnsupportedTableFormat(txt)=> format!("Unsupported table format\n= {}",txt),
            #[cfg(feature = "csv")]
            Self::CsvError(err) => format!("Table error\n= {}",err),
            Self::BincodeError(txt)=> format!("Failed frozen operation\n= {}",txt),
            Self::PermissionDenied(txt, atype) => format!("Permission denied for \"{0}\". Use a flag \"-a {1:?}\" to allow this macro.", txt,atype),
            Self::StrictPanic => "Strict error, exiting...".to_string(),
            Self::Panic => "Processor panicked, exiting...".to_string(),
            Self::ManualPanic(txt) => format!("Panic triggered with message\n= {}",txt),
            #[cfg(feature = "storage")]
            Self::StorageError(txt)=> format!("Storage error with message\n= {0}",txt),
            #[cfg(feature = "cindex")]
            Self::CIndexError(err) => err.to_string(),
            Self::UnallowedMacroExecution(txt) => format!("Macro execution is not allowed\n={0}",txt),
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

impl From<std::string::FromUtf8Error> for RadError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Utf8Err(err)
    }
}

#[cfg(feature = "csv")]
impl From<csv::Error> for RadError {
    fn from(err: csv::Error) -> Self {
        Self::CsvError(err)
    }
}

impl From<std::env::VarError> for RadError {
    fn from(err: std::env::VarError) -> Self {
        Self::EnvError(err)
    }
}

#[cfg(feature = "csv")]
impl From<FromUtf8Error> for RadError {
    fn from(err: FromUtf8Error) -> Self {
        Self::InvalidString(err)
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
