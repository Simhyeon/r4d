use crate::parser::DefineParser;
use crate::runtime_map::RuntimeMacro;
use crate::{RadError, RadResult};

#[derive(Default)]
pub(crate) struct AnonMap {
    macros: Vec<RuntimeMacro>,
}

impl AnonMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_macro(&mut self, body: &str) -> RadResult<()> {
        let mut full_body = "anon,".to_string();
        full_body.push_str(body);

        let (_, arg, body) = DefineParser::new()
            .parse_define(&full_body)
            .ok_or_else(|| {
                RadError::InvalidMacroDefinition(
                    "Invalid definition for anonymous macro".to_string(),
                )
            })?;
        let rt_macro = RuntimeMacro::new("anon", &arg, &body, false);
        self.macros.push(rt_macro);
        Ok(())
    }

    pub fn get_anon(&self) -> Option<&RuntimeMacro> {
        self.macros.last()
    }

    pub fn clear(&mut self) {
        self.macros.clear();
    }
}
