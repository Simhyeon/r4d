use crate::{Processor, RadError, WriteOption};
use console_error_panic_hook;
use wasm_bindgen::prelude::*;

type WasmResult<T> = Result<T, JsValue>;

impl From<RadError> for JsValue {
    fn from(err: RadError) -> Self {
        JsValue::from_str(&err.to_string())
    }
}

#[wasm_bindgen]
pub struct RadProcessor {
    processor: Processor<'static>,
}

#[wasm_bindgen]
impl RadProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        let mut processor = Processor::new();
        processor.set_write_option(WriteOption::Return);

        Self { processor }
    }

    pub fn process_string(&mut self, src: &str) -> WasmResult<String> {
        let ret = self.processor.from_string(src)?.unwrap_or(String::new());
        Ok(ret)
    }
}
