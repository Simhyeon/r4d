use crate::deterred_map::DFunctionMacroType;
use crate::function_map::FunctionMacroType;

#[derive(Clone)]
/// Builder struct for extension macros
///
/// This creates an extension macro without going through tedious processor methods interaction.
///
/// Use a template feature to utilizes eaiser extension register.
///
/// # Example
///
/// ```
/// let mut processor = r4d::Processor::new();
/// #[cfg(feature = "template")]
/// processor.add_ext_macro(r4d::ExtMacroBuilder::new("macro_name")
///     .args(&["a1","b2"])
///     .function(r4d::function_template!(
///         let args = r4d::split_args!(2)?;
///         let result = format!("{} + {}", args[0], args[1]);
///         Ok(Some(result))
/// )));
/// ```
pub struct ExtMacroBuilder {
    pub(crate) macro_name: String,
    pub(crate) macro_type: ExtMacroType,
    pub(crate) args: Vec<String>,
    pub(crate) macro_body: Option<ExtMacroBody>,
    pub(crate) macro_desc: Option<String>,
}

impl ExtMacroBuilder {
    /// Creates an empty macro with given macro name
    pub fn new(macro_name: &str) -> Self {
        Self {
            macro_name: macro_name.to_string(),
            macro_type: ExtMacroType::Function,
            // Empty values
            args: vec![],
            macro_body: None,
            macro_desc: None,
        }
    }

    /// Set macro's body type as function
    pub fn function(mut self, func: FunctionMacroType) -> Self {
        self.macro_type = ExtMacroType::Function;
        self.macro_body = Some(ExtMacroBody::Function(func));
        self
    }

    /// Set macro's body type as deterred
    pub fn deterred(mut self, func: DFunctionMacroType) -> Self {
        self.macro_type = ExtMacroType::Deterred;
        self.macro_body = Some(ExtMacroBody::Deterred(func));
        self
    }

    /// Set macro's arguments
    pub fn args(mut self, args: &[impl AsRef<str>]) -> Self {
        self.args = args.iter().map(|a| a.as_ref().to_string()).collect();
        self
    }

    /// Set description of the macro
    pub fn desc(mut self, description: &str) -> Self {
        self.macro_desc.replace(description.to_string());
        self
    }
}

#[derive(Clone)]
pub(crate) enum ExtMacroType {
    Function,
    Deterred,
}

#[derive(Clone)]
pub(crate) enum ExtMacroBody {
    Function(FunctionMacroType),
    Deterred(DFunctionMacroType),
}
