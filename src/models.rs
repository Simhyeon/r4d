use crate::error::RadError;

pub struct Invocation{
    pub name: String,
    pub args: Vec<String>,
    pub body: String,
}

impl Invocation {
    pub fn new(name: &str, args: &str, body: &str) -> Self {
        Self {  
            name : name.to_owned(),
            args : args.split(',').map(|item| item.to_owned()).collect(),
            body : body.to_owned(),
        }
    }

    pub fn invoke(&self) -> Result<String, RadError> {

        Ok(String::new())
    }
}
