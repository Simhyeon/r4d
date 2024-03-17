use crate::runtime_map::RuntimeMacro;
use crate::utils::Utils;
use crate::RadResult;

#[derive(Default)]
pub(crate) struct AnonMap {
    macros: Vec<RuntimeMacro>,
}

impl AnonMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_macro(&mut self, body_src: &str) -> RadResult<()> {
        let mut full_body = "anon,".to_string();
        full_body.push_str(body_src);

        let def = Utils::split_macro_definition(full_body.as_str())?;
        let rt_macro = RuntimeMacro::new("anon").body(def.body).params(def.params);
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
