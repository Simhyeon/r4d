use std::collections::HashMap;
use crate::models::Invocation;
use crate::error::RadError;
use crate::basic::BasicMacro;

pub struct Processor {
    pub invoke_map : HashMap<String, Invocation>,
}

impl Processor {
    pub fn new(invocations: Vec<Invocation>) -> Self {
        Self { invoke_map: HashMap::new() }
    }

    pub fn register(
        &mut self, 
        name: &str,
        args: &str,
        body: &str
    ) -> Result<(),RadError> {
        let mac = Invocation::new(name, args, body);
        self.invoke_map.insert(name.to_owned(), mac);
        Ok(())
    }

    pub fn evaluate(&mut self, name: &str, args: &str) -> Result<Option<String>, RadError> {
        if self.invoke_map.contains_key(name) {
            let mac = self.invoke_map.get(name).unwrap();
            Ok(Some(mac.invoke()?))
        } else {
            Ok(None)
        }
    }
}
