use proc_macro::TokenStream;

#[proc_macro]
pub fn function_template(item: TokenStream) -> TokenStream {
    format!("|args : &str, processor : &mut Processor| -> RadResult<Option<String>> {{ 
        {}
}}",item).parse().unwrap()
}

#[proc_macro]
pub fn deterred_template(item: TokenStream) -> TokenStream {
    format!("|args : &str, level: usize, processor : &mut Processor| -> RadResult<Option<String>> {{ 
        {}
}}",item).parse().unwrap()
}

#[proc_macro]
pub fn expand(item: TokenStream) -> TokenStream {
    format!("processor.expand(level,{})", item).parse().unwrap()
}

#[proc_macro]
pub fn split_args(item: TokenStream) -> TokenStream {
    format!("processor.get_split_arguments({}, args)", item).parse().unwrap()
}
