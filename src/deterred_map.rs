use crate::arg_parser::ArgParser;
use crate::models::MacroType;
use crate::models::RadResult;
use crate::models::{ExtMacroBody, ExtMacroBuilder};
use crate::utils::Utils;
use crate::Processor;
use crate::{AuthType, RadError};
use std::collections::HashMap;
use std::iter::FromIterator;
#[cfg(feature = "csv")]
use crate::formatter::Formatter;

pub(crate) type DFunctionMacroType = fn(&str, usize, &mut Processor) -> RadResult<Option<String>>;

#[derive(Clone)]
pub struct DeterredMacroMap {
    pub(crate) macros: HashMap<String, KMacroSign>,
}

impl DeterredMacroMap {
    /// Creates empty map
    pub fn empty() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    pub fn new() -> Self {
        let mut map = HashMap::from_iter(IntoIterator::into_iter([
            (
                "exec".to_owned(),
                KMacroSign::new(
                    "exec",
                    ["macro_name", "macro_args"],
                    DeterredMacroMap::execute_macro,
                    None,
                ),
            ),
            (
                "fassert".to_owned(),
                KMacroSign::new(
                    "fassert",
                    ["a_lvalue", "a_rvalue"],
                    DeterredMacroMap::assert_fail,
                    None,
                ),
            ),
            (
                "foreach".to_owned(),
                KMacroSign::new(
                    "foreach",
                    ["a_array", "a_body"],
                    DeterredMacroMap::foreach,
                    None,
                ),
            ),
            (
                "forline".to_owned(),
                KMacroSign::new(
                    "forline",
                    ["a_iterable", "a_body"],
                    DeterredMacroMap::forline,
                    None,
                ),
            ),
            (
                "forloop".to_owned(),
                KMacroSign::new(
                    "forloop",
                    ["a_min", "a_max", "a_body"],
                    DeterredMacroMap::forloop,
                    None,
                ),
            ),
            (
                "if".to_owned(),
                KMacroSign::new(
                    "if",
                    ["a_boolean", "a_if_expr"],
                    DeterredMacroMap::if_cond,
                    None,
                ),
            ),
            (
                "ifelse".to_owned(),
                KMacroSign::new(
                    "ifelse",
                    ["a_boolean", "a_if_expr", "a_else_expr"],
                    DeterredMacroMap::ifelse,
                    None,
                ),
            ),
            (
                "ifdef".to_owned(),
                KMacroSign::new(
                    "ifdef",
                    ["a_macro_name", "a_if_expr"],
                    DeterredMacroMap::ifdef,
                    None,
                ),
            ),
            (
                "ifdefel".to_owned(),
                KMacroSign::new(
                    "ifdefel",
                    ["a_macro_name", "a_if_expr", "a_else_expr"],
                    DeterredMacroMap::ifdefel,
                    None,
                ),
            ),
            (
                "que".to_owned(),
                KMacroSign::new("que", ["a_content"], DeterredMacroMap::queue_content, None),
            ),
            (
                "ifque".to_owned(),
                KMacroSign::new(
                    "ifque",
                    ["a_bool", "a_content"],
                    DeterredMacroMap::if_queue_content,
                    None,
                ),
            ),
        ]));
        // Auth realted macros should be segregated from wasm target
        #[cfg(not(feature = "wasm"))]
        {
            map.insert(
                "ifenv".to_owned(),
                KMacroSign::new(
                    "ifenv",
                    ["a_env_name", "a_if_expr"],
                    DeterredMacroMap::ifenv,
                    None,
                ),
            );
            map.insert(
                "ifenvel".to_owned(),
                KMacroSign::new(
                    "ifenvel",
                    ["a_env_name", "a_if_expr", "a_else_expr"],
                    DeterredMacroMap::ifenvel,
                    None,
                ),
            );
        }
        // Optional macros
        #[cfg(feature = "csv")]
        {
            map.insert(
                "from".to_owned(),
                KMacroSign::new(
                    "from",
                    ["a_macro_name", "a_csv_value"],
                    Self::from_data,
                    None,
                ),
            );
        }

        #[cfg(feature = "evalexpr")]
        {
            map.insert(
                "ieval".to_owned(),
                KMacroSign::new("ieval", ["a_macro","a_expression"], Self::eval_inplace, None),
            );
        }

        Self { macros: map }
    }

    /// Get Function pointer from map
    pub fn get_deterred_macro(&self, name: &str) -> Option<&DFunctionMacroType> {
        if let Some(mac) = self.macros.get(name) {
            Some(&mac.logic)
        } else {
            None
        }
    }

    /// Check if map contains the name
    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    pub fn undefine(&mut self, name: &str) {
        self.macros.remove(name);
    }

    pub fn rename(&mut self, name: &str, target: &str) {
        let func = self.macros.remove(name).unwrap();
        self.macros.insert(target.to_owned(), func);
    }

    pub fn new_ext_macro(&mut self, ext: ExtMacroBuilder) {
        if let Some(ExtMacroBody::Deterred(mac_ref)) = ext.macro_body {
            let sign = KMacroSign::new(&ext.macro_name, &ext.args, mac_ref, ext.macro_desc);
            self.macros.insert(ext.macro_name, sign);
        }
    }

    // ----------
    // Keyword Macros start

    /// Loop around given values and substitute iterators  with the value
    ///
    /// # Usage
    ///
    /// $foreach(\*a,b,c*\,$:)
    fn foreach(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let mut sums = String::new();
            let loopable = &processor.parse_chunk_args(level, "", &args[0])?;
            let mut count = 0;
            for value in loopable.split(',') {
                // This overrides value
                processor.add_new_local_macro(level, "a_LN", &count.to_string());

                let result : String;
                #[cfg(feature = "for_macro")]
                {
                    processor.add_new_local_macro(level, ":", value);
                    result = processor.parse_chunk_args(level, "", &args[1])?;
                }

                #[cfg(not(feature = "for_macro"))]
                { result = processor.parse_chunk_args(level, "", &args[1].replace("$:", &value.to_string()))?; }

                sums.push_str(&result);
                count += 1;
            }

            // Clear local macro
            #[cfg(feature = "for_macro")]
            processor.remove_local_macro(level, ":");

            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument(
                "Foreach requires two argument".to_owned(),
            ))
        }
    }

    /// Loop around given values split by new line and substitute iterators  with the value
    ///
    /// # Usage
    ///
    /// $forline(TTT,$:)
    fn forline(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let mut sums = String::new();
            let loopable = &processor.parse_chunk_args(level, "", &args[0])?;
            let mut count = 1;
            for value in loopable.lines() {
                // This overrides value
                processor.add_new_local_macro(level, "a_LN", &count.to_string());
                let result =
                    processor.parse_chunk_args(level, "", &args[1].replace("$:", value))?;
                sums.push_str(&result);
                count += 1;
            }
            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument(
                "Forline requires two argument".to_owned(),
            ))
        }
    }

    /// For loop around given min, max value and finally substitue iterators with value
    ///
    /// # Usage
    ///
    /// $forloop(1,5,$:)
    fn forloop(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let mut sums = String::new();

            let min_src = processor.parse_chunk_args(level, "", &Utils::trim(&args[0]))?;
            let max_src = processor.parse_chunk_args(level, "", &Utils::trim(&args[1]))?;

            let min: usize;
            let max: usize;
            if let Ok(num) = min_src.parse::<usize>() {
                min = num;
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "Forloop's min value should be non zero positive integer but given {}",
                    min_src
                )));
            }
            if let Ok(num) = max_src.parse::<usize>() {
                max = num
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "Forloop's max value should be non zero positive integer but given \"{}\"",
                    max_src
                )));
            }
            let mut result: String;
            for value in min..=max {
                #[cfg(feature = "for_macro")]
                {
                    processor.add_new_local_macro(level, ":", &value.to_string());
                    result = processor.parse_chunk_args(
                        level,
                        "",
                        &args[2],
                    )?;
                }
                #[cfg(not(feature = "for_macro"))]
                {
                    result = processor.parse_chunk_args(
                        level,
                        "",
                        &args[2].replace("$:", &value.to_string())
                    )?;
                }

                sums.push_str(&result);
                result.clear();
            }

            // Clear local macro
            #[cfg(feature = "for_macro")]
            processor.remove_local_macro(level, ":");

            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument(
                "Forloop requires two argument".to_owned(),
            ))
        }
    }

    /// Print content according to given condition
    ///
    /// # Usage
    ///
    /// $if(evaluation, ifstate)
    fn if_cond(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let boolean = &processor.parse_chunk_args(level, "", &args[0])?;

            // Given condition is true
            let cond = Utils::is_arg_true(&Utils::trim(boolean));
            if let Ok(cond) = cond {
                if cond {
                    let if_expr = processor.parse_chunk_args(level, "", &args[1])?;
                    return Ok(Some(if_expr));
                }
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "If requires either true/false or zero/nonzero integer but given \"{}\"",
                    boolean
                )));
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "if requires two arguments".to_owned(),
            ))
        }
    }

    /// Print content according to given condition
    ///
    /// # Usage
    ///
    /// $ifelse(evaluation, \*ifstate*\, \*elsestate*\)
    fn ifelse(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let boolean = &processor.parse_chunk_args(level, "", &args[0])?;

            // Given condition is true
            let cond = Utils::is_arg_true(&Utils::trim(boolean));
            if let Ok(cond) = cond {
                if cond {
                    let if_expr = processor.parse_chunk_args(level, "", &args[1])?;
                    return Ok(Some(if_expr));
                }
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "Ifelse requires either true/false or zero/nonzero integer but given \"{}\"",
                    boolean
                )));
            }

            // Else state
            let else_expr = processor.parse_chunk_args(level, "", &args[2])?;
            return Ok(Some(else_expr));
        } else {
            Err(RadError::InvalidArgument(
                "ifelse requires three argument".to_owned(),
            ))
        }
    }

    /// If macro exists, then execute expresion
    ///
    /// # Usage
    ///
    /// $ifdef(macro_name, expr)
    fn ifdef(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let name = processor.parse_chunk_args(level, "", &Utils::trim(&args[0]))?;

            let boolean = processor.contains_macro(&name, MacroType::Any);
            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_chunk_args(level, "", &args[1])?;
                return Ok(Some(if_expr));
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "ifdef requires two arguments".to_owned(),
            ))
        }
    }

    /// If macro exists, then execute expresion else exectue another
    ///
    /// # Usage
    ///
    /// $ifdefelse(macro_name,expr,expr2)
    fn ifdefel(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let name = processor.parse_chunk_args(level, "", &Utils::trim(&args[0]))?;

            let boolean = processor.contains_macro(&name, MacroType::Any);
            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_chunk_args(level, "", &args[1])?;
                return Ok(Some(if_expr));
            } else {
                let else_expr = processor.parse_chunk_args(level, "", &args[2])?;
                return Ok(Some(else_expr));
            }
        } else {
            Err(RadError::InvalidArgument(
                "ifdefel requires three arguments".to_owned(),
            ))
        }
    }

    /// If env exists, then execute expresion
    ///
    /// # Usage
    ///
    /// $ifenv(env_name, expr)
    fn ifenv(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("ifenv", AuthType::ENV, processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let name = processor.parse_chunk_args(level, "", &Utils::trim(&args[0]))?;

            let boolean = if let Ok(_) = std::env::var(name) {
                true
            } else {
                false
            };

            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_chunk_args(level, "", &args[1])?;
                return Ok(Some(if_expr));
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "ifenv requires two arguments".to_owned(),
            ))
        }
    }

    /// If env exists, then execute expresion else execute another
    ///
    /// # Usage
    ///
    /// $ifenvel(env_name,expr,expr2)
    fn ifenvel(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("ifenvel", AuthType::ENV, processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let name = processor.parse_chunk_args(level, "", &Utils::trim(&args[0]))?;

            let boolean = if let Ok(_) = std::env::var(name) {
                true
            } else {
                false
            };

            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_chunk_args(level, "", &args[1])?;
                return Ok(Some(if_expr));
            } else {
                let else_expr = processor.parse_chunk_args(level, "", &args[2])?;
                return Ok(Some(else_expr));
            }
        } else {
            Err(RadError::InvalidArgument(
                "ifenvel requires three arguments".to_owned(),
            ))
        }
    }

    /// Assert fail
    ///
    /// This has to be deterred macro because it's value should be evaluated later
    ///
    /// # Usage
    ///
    /// $fassert(abc,abc)
    fn assert_fail(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let result = processor.parse_chunk_args(level, "", args);
        if let Err(_) = result {
            processor.track_assertion(true)?;
            Ok(None)
        } else {
            processor.track_assertion(false)?;
            Err(RadError::AssertFail)
        }
    }

    /// Queue processing
    ///
    /// # Usage
    ///
    /// $que(Sentence to process)
    fn queue_content(args: &str, _: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        processor.insert_queue(args);
        Ok(None)
    }

    /// Queue processing
    ///
    /// # Usage
    ///
    /// $ifque(true,Sentence to process)
    fn if_queue_content(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let boolean = &processor.parse_chunk_args(level, "", &args[0])?;
            let cond = Utils::is_arg_true(&boolean)?;
            if cond {
                processor.insert_queue(&args[1]);
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "ifque requires two argument".to_owned(),
            ))
        }
    }

    /// Execute macro
    ///
    /// # Usage
    ///
    /// $exec(macro_name,macro_args)
    fn execute_macro(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let macro_name = &processor.parse_chunk_args(level, "", &args[0])?;
            if !processor.contains_macro(&macro_name, MacroType::Any) {
                return Err(RadError::InvalidArgument(
                        format!("Macro \"{}\" doesn't exist", macro_name)
                ));
            }
            let args = &args[1];
            let result = processor.parse_chunk_args(level, "", &format!("${}({})",macro_name, args))?;
            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "exec requires two argument".to_owned(),
            ))
        }
    }

    /// Create multiple macro executions from given csv value
    ///
    /// # Usage
    ///
    /// $from(macro_name,\*1,2,3
    /// 4,5,6*\)
    ///
    /// $from+(macro_name,
    /// 1,2,3
    /// 4,5,6
    /// )
    #[cfg(feature = "csv")]
    fn from_data(args: &str,level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let macro_name = Utils::trim(&args[0]);
            // Trimming data might be very costly operation
            // Plus, it is already trimmed by csv crate.
            let macro_data = &args[1];

            let result = Formatter::csv_to_macros(&macro_name, macro_data, &processor.state.newline)?;

            // TODO
            // This behaviour might can be improved
            // Disable debugging for nested macro expansion
            #[cfg(feature = "debug")]
            let original = processor.is_debug();

            // Now this might look strange,
            // "Why not just enclose two lines with curly brackets?"
            // The answer is such appraoch somehow doesn't work.
            // Compiler cannot deduce the variable original and will yield error on
            // processor.debug(original)
            #[cfg(feature = "debug")]
            processor.set_debug(false);

            // Parse macros
            let result = processor.parse_chunk_args(level, "", &result)?;

            // Set custom prompt log to indicate user thatn from macro doesn't support
            // debugging inside macro expansion
            #[cfg(feature = "debug")]
            {
                use crate::debugger::DebugSwitch;
                processor.set_debug(original);
                match processor.get_debug_switch() {
                    DebugSwitch::StepMacro | DebugSwitch::NextMacro => {
                        processor.set_prompt_log("\"From macro\"")
                    }
                    _ => (),
                }
            }

            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "From requires two arguments".to_owned(),
            ))
        }
    }

    /// Evaluate given macro with given expression
    ///
    /// # Usage
    ///
    /// $ieval(macro,expression)
    #[cfg(feature = "evalexpr")]
    fn eval_inplace(args: &str,level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            // This is the processed raw formula
            let macro_name = Utils::trim(&args[0]);
            if !processor.contains_macro(&macro_name, MacroType::Runtime) {
                return Err(RadError::InvalidArgument(format!("Macro \"{}\" doesn't exist", macro_name)));
            }

            let expr = Utils::trim(&args[1]);
            let chunk = format!("$eval( ${}() {} )",macro_name, expr);
            let result = processor.parse_chunk_args(level, "", &chunk)?;

            processor.replace_macro(&macro_name, &result);
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Ieval requires two arguments".to_owned(),
            ))
        }
    }


    // Keyword macros end
    // ----------
}

/// Keyword Macro signature
#[derive(Clone)]
pub(crate) struct KMacroSign {
    name: String,
    args: Vec<String>,
    pub logic: DFunctionMacroType,
    #[allow(dead_code)]
    desc: Option<String>,
}

impl KMacroSign {
    pub fn new(
        name: &str,
        args: impl IntoIterator<Item = impl AsRef<str>>,
        logic: DFunctionMacroType,
        desc: Option<String>,
    ) -> Self {
        let args = args
            .into_iter()
            .map(|s| s.as_ref().to_owned())
            .collect::<Vec<String>>();
        Self {
            name: name.to_owned(),
            args,
            logic,
            desc,
        }
    }
}

impl std::fmt::Display for KMacroSign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut inner = self
            .args
            .iter()
            .fold(String::new(), |acc, arg| acc + &arg + ",");
        // This removes last "," character
        inner.pop();
        write!(f, "${}({})", self.name, inner)
    }
}

#[cfg(feature = "signature")]
impl From<&KMacroSign> for crate::sigmap::MacroSignature {
    fn from(ms: &KMacroSign) -> Self {
        Self {
            variant: crate::sigmap::MacroVariant::Deterred,
            name: ms.name.to_owned(),
            args: ms.args.to_owned(),
            expr: ms.to_string(),
            desc: ms.desc.clone(),
        }
    }
}
