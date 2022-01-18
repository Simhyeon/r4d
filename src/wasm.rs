use wasm_bindgen::prelude::*;
use crate::error::RadError;

// JS methods
#[wasm_bindgen]
extern "C" {
    //console.error
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log(l : &str);
    //console.error
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn log_error(e: &JsValue);
}


type WasmResult<T> = Result<T, JsValue>;

impl From<RadError> for JsValue {
    fn from(err : RadError) -> Self {
        JsValue::from_str(&err.to_string())
    }
}

#[wasm_bindgen]
pub fn process(option: &ProcessOption) -> JsValue {
    let processor: Processor = Processor::new();
}

#[wasm_bindgen]
pub struct ProcessOption {

}
