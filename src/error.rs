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
