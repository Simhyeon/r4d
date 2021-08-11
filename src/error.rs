use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError{
    #[error("Unkown option was given")]
    Unknown,
}

#[derive(Error, Debug)]
pub enum BasicError{
    #[error("Invalid regex statement : {0}")]
    InvalidRegex(regex::Error),
    #[error("Invalid formula : {0}")]
    InvalidFormula(evalexpr::EvalexprError),
    #[error("Invalid argument : {0}")]
    InvalidArgument(&'static str),
    #[error("Invalid argument type: {0}")]
    InvalidArgType(std::num::ParseIntError)
}

impl From<regex::Error> for BasicError {
    fn from(err : regex::Error) -> Self {
        Self::InvalidRegex(err)
    }
}

impl From<evalexpr::EvalexprError> for BasicError {
    fn from(err : evalexpr::EvalexprError) -> Self {
        Self::InvalidFormula(err)
    }
}

impl From<std::num::ParseIntError> for BasicError {
    fn from(err : std::num::ParseIntError) -> Self {
        Self::InvalidArgType(err)
    }
}

#[derive(Error, Debug)]
pub enum MainError{
    #[error("Command line error of : {0}")]
    Cli(CliError),
    #[error("Basic macro call failed with error : {0}")]
    Basic(BasicError)
}

impl From<CliError> for MainError {
    fn from(err : CliError) -> Self {
        Self::Cli(err)
    }
}

impl From<BasicError> for MainError {
    fn from(err : BasicError) -> Self {
        Self::Basic(err)
    }
}
