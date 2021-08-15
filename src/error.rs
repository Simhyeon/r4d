use thiserror::Error;

#[derive(Error, Debug)]
pub enum RadError {
    #[error("Failed regex operation : {0}")]
    InvalidRegex(regex::Error),
    #[error("Invalid formula : {0}")]
    InvalidFormula(evalexpr::EvalexprError),
    #[error("Invalid argument : {0}")]
    InvalidArgument(&'static str),
    #[error("Invalid argument type: {0}")]
    InvalidArgType(std::num::ParseIntError),
    #[error("Standard IO error : {0}")]
    StdIo(std::io::Error),
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
        Self::InvalidArgType(err)
    }
}

impl From<std::io::Error> for RadError {
    fn from(err : std::io::Error) -> Self {
        Self::StdIo(err)
    }
}
