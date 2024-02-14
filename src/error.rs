//! R4d' error types
//!
//! R4d can have many errors because it utilizes multiple functions and
//! external crates at the same time.

use crate::AuthType;
#[cfg(feature = "cindex")]
use cindex::CIndexError;

/// Blank implementation for error trait
impl std::error::Error for RadError {}

// TODO
// Should this error message respect newline for os types?
/// R4d's error type
#[derive(Debug)]
pub enum RadError {
    // ----------
    // Following errors are the errors that macro logics and yield.
    // In other words, macros cannot yield other logics other than followings.
    // Yet other logics ( processing, debugging etc. ) can yield any error from this enum.
    /// Failure of assertion
    AssertFail,
    /// Error behaviour which was specifically enforced by a user.
    UnsoundExecution(String),
    /// Invalid exeuction means macro processing has failed to achieve expected result.
    InvalidExecution(String),
    /// Invalid argument means argument is invalid before processing
    ///
    /// This error can also mean "ingredients" associated with macro argument is tot valid
    InvalidArgument(String),
    /// Macro is not availble for execution due to variadic situation
    UnallowedMacroExecution(String),
    /// Error when macro with such name doesn't exist
    ///
    /// This error types accepts similar macro name and display it to user.
    NoSuchMacroName(String, Option<String>),
    /// Error when processor fails to define a macro
    InvalidMacroDefinition(String),
    /// Used by panic macro
    ///
    /// This error is recaptured as Interrupt by processor
    ManualPanic(String),
    // ----------

    // ----------
    // <GENERAL>
    /// Given macro environment is not valid
    InvalidMacroEnvironment(String),
    /// Conversion of env::VarError
    EnvError(std::env::VarError),
    /// Error when processor fails get reference of
    ///
    /// This error indicates that given macro name doesn't fit the standard of the processor
    /// requires. Mostly empty macro name but also includes when processor interanl changed without
    /// proper care or attribute is wrong etc...
    InvalidMacroReference(String),
    /// Error on file operation
    InvalidFile(String),
    /// Storage operation failed
    StorageError(String),
    /// Hook failure
    HookMacroFail(String),
    /// When raw string input fails conver to specific type
    ///
    /// This should not be directly invoked from macro logics but from implemented methods.
    InvalidConversion(String),
    /// Invalid character while processing
    UnallowedChar(String),
    /// Required permission was not authorized to user
    PermissionDenied(String, AuthType),
    /// Strict panic
    StrictPanic,
    // </GENERAL>

    // <Preocedure flow related>
    /// Invoked by exit macro
    SaneExit,
    /// Invoked by panic macro
    Interrupt,
    // </Procedure>

    // <BIN>
    /// Error behaviour occured from command line binary
    #[cfg(feature = "clap")]
    InvalidCommandOption(String),
    // </BIN>

    // Crate specific || Conversion
    // Namely,
    // <MISC>
    /// Conversion of regex error
    InvalidRegex(regex::Error),
    /// Conversion of evalexpr error
    InvalidFormula(evalexpr::EvalexprError),
    StdIo(std::io::Error),
    FmtError(std::fmt::Error),
    Utf8Err(std::string::FromUtf8Error),
    BincodeError(String),
    #[cfg(feature = "cindex")]
    CIndexError(CIndexError),
    DcsvError(dcsv::DcsvError),
    // </MISC>
}
impl std::fmt::Display for RadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::SaneExit => String::from("Process finished"),
            Self::Interrupt => String::from("Process Interrupted"),
            Self::HookMacroFail(txt) => format!("Hook macro error \n= {}", txt),
            Self::InvalidConversion(txt) => format!("Invalid conversion \n= {}", txt),
            Self::UnallowedChar(txt) => format!("Unallowed character \n= {}", txt),
            Self::AssertFail => "Assert failed".to_string(),
            Self::UnsoundExecution(err) => format!("Critical unsound execution error \n= {}", err),
            Self::InvalidExecution(err) => format!("Invalid execution error \n= {}", err),
            Self::InvalidMacroEnvironment(err) => format!("Invalid macro environment\n= {}", err),
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
            Self::InvalidFormula(err) => format!("Invalid formula\n= {}", err),
            Self::InvalidArgument(arg) => format!("Invalid argument\n= {}", arg),
            Self::InvalidFile(file) => format!("File,\"{}\", does not exist", file),
            Self::StdIo(err) => format!("Standard IO error\n= {}", err),
            Self::FmtError(err) => format!("Formatting error\n= {}", err),
            Self::Utf8Err(err) => format!("Failed to convert to utf8 string\n= {}", err),
            Self::BincodeError(txt) => format!("Failed frozen operation\n= {}", txt),
            Self::PermissionDenied(txt, atype) => format!(
                "Permission denied for \"{0}\". Use a flag \"-a {1:?}\" to allow this macro.",
                txt, atype
            ),
            Self::StrictPanic => "Every error is panicking in strict mode".to_string(),
            Self::ManualPanic(txt) => format!("Panic triggered with message\n \"{}\"", txt),
            Self::StorageError(txt) => format!("Storage error with message\n= {0}", txt),
            #[cfg(feature = "cindex")]
            Self::CIndexError(err) => err.to_string(),
            Self::UnallowedMacroExecution(txt) => {
                format!("Macro execution is not allowed\n= {0}", txt)
            }
            Self::DcsvError(err) => format!("{}", err),
            #[cfg(feature = "clap")]
            Self::InvalidCommandOption(command) => format!("Invalid command option\n= {}", command),
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

impl From<evalexpr::EvalexprError> for RadError {
    fn from(err: evalexpr::EvalexprError) -> Self {
        Self::InvalidFormula(err)
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
