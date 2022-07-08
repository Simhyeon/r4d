use crate::consts::ESR;
use crate::formatter::Formatter;
use crate::models::ErrorBehaviour;
use crate::models::FileTarget;
use crate::models::FlowControl;
use crate::models::MacroType;
use crate::models::RadResult;
use crate::models::RelayTarget;
use crate::models::{ExtMacroBody, ExtMacroBuilder};
use crate::parser::GreedyState;
use crate::trim;
use crate::utils::Utils;
use crate::ArgParser;
use crate::Processor;
use crate::{AuthType, RadError};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::PathBuf;

pub(crate) type DFunctionMacroType = fn(&str, usize, &mut Processor) -> RadResult<Option<String>>;

#[derive(Clone)]
pub struct DeterredMacroMap {
    pub(crate) macros: HashMap<String, DMacroSign>,
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
                "EB".to_owned(),
                DMacroSign::new(
                    "EB",
                    ESR,
                    DeterredMacroMap::escape_blanks,
                    Some("Escape following all blanks until not. This can only be invoked at first level

# Example

$EB()".to_string()),
                ),
            ),
            (
                "exec".to_owned(),
                DMacroSign::new(
                    "exec",
                    ["a_macro_name^", "a_macro_args"],
                    DeterredMacroMap::execute_macro,
                    Some("Execute a macro with arguments

# Arguments

- a_macro_name : Macro name to exectue ( trimmed )
- a_macro_args : Arguments to be passed to a macro

# Example

$assert($path(a,b,c),$exec(path,a,b,c))".to_string()),
                ),
            ),
            (
                "fassert".to_owned(),
                DMacroSign::new(
                    "fassert",
                    ["a_expr"],
                    DeterredMacroMap::assert_fail,
                    Some("Assert succeedes when text expansion yields error

# Arguments

- a_expr: Expression to audit

# Example

$fassert($eval(Text is not allowd))".to_string()),
                ),
            ),
            (
                "forby".to_owned(),
                DMacroSign::new(
                    "forby",
                    ["a_body", "a_sep","a_text"],
                    DeterredMacroMap::forby,
                    Some(
                        "Iterate around text separated by separator.

Iterated value is bound to macro \":\"

# Arguments

- a_body : Body to be pasted as iterated item
- a_sep  : Separator to split a text
- a_text : Text to split by separator

# Example

$assert(a+b+c+,$forby($:()+,-,a-b-c))".to_string(),
                    ),
                ),
            ),
            (
                "foreach".to_owned(),
                DMacroSign::new(
                    "foreach",
                    ["a_body", "a_array"],
                    DeterredMacroMap::foreach,
                    Some(
                        "Iterate around given array.

Iterated value is bound to macro \":\"
 
# Arguments

- a_body  : Body to be pasted as iterated item
- a_array : An array to iterate

# Example

$assert(a+b+c+,$foreach($:()+,a,b,c))".to_string(),
                    ),
                ),
            ),
            (
                "forline".to_owned(),
                DMacroSign::new(
                    "forline",
                    ["a_body","a_lines"],
                    DeterredMacroMap::forline,
                    Some("Loop around given lines separated by newline chraracter. 

Iterated value is bound to macro \":\"
 
# Arguments

- a_body  : Body to be pasted as iterated item
- a_lines : Lines to iterate

# Example

$assert(a+b+c+,$forline($:()+,a$nl()b$nl()c))".to_string()),
                ),
            ),
            (
                "forloop".to_owned(),
                DMacroSign::new(
                    "forloop",
                    ["a_body","a_min^", "a_max^"],
                    DeterredMacroMap::forloop,
                    Some("Iterate around given range (min,max). 

Iterated value is bound to macro \":\" 

# Arguments

- a_body : Body to be pasted as iterated item
- a_min  : Start index ( trimmed )
- a_max  : End index ( trimmed )

# Example

$assert(1+2+3+,$forloop($:()+,1,3))".to_string()),
                ),
            ),
            (
                "spread".to_owned(),
                DMacroSign::new(
                    "spread",
                    ["a_macro_name^", "a_csv_value^"],
                    Self::spread_data,
                    Some("Execute a macro multiple times with given data chunk. Each csv line represent arguments for a macro

# Arguments

- a_macro_name : Macro name to execute ( trimmed ) 
- a_csv_value  : Arguments table ( trimmed )

# Example

$assert=(
	text------
	---text---
	------text,
	$spread=(
		align,
		left,10,-,text
		center,10,-,text
		right,10,-,text
	)
)".to_string()),
                ),
            ),
            (
                "if".to_owned(),
                DMacroSign::new(
                    "if",
                    ["a_cond?^", "a_if_expr"],
                    DeterredMacroMap::if_cond,
                    Some(
                        "Check condition and then execute the expression if the condition is true

# Arguments

- a_cond    : Condition ( trimmed )
- a_if_expr : Expression to expand if condition is true

# Example

$assert(I'm true,$if(true,I'm true))".to_string(),
                    ),
                ),
            ),
            (
                "ifelse".to_owned(),
                DMacroSign::new(
                    "ifelse",
                    ["a_cond?^", "a_if_expr", "a_else_expr"],
                    DeterredMacroMap::ifelse,
                    Some(
                        "Check condition and execute different expressions by the condition

# Arguments

- a_cond      : Condition ( trimmed )
- a_if_expr   : Expression to expand if condition is true
- a_else_expr : Expression to expand if condition is false

# Example

$assert(I'm true,$ifelse(true,I'm true,I'm false))
$assert(I'm false,$ifelse(false,I'm true,I'm false))".to_string(),
                    ),
                ),
            ),
            (
                "ifdef".to_owned(),
                DMacroSign::new(
                    "ifdef",
                    ["a_macro_name^", "a_if_expr"],
                    DeterredMacroMap::ifdef,
                    Some("Execute expression if macro is defined

# Arguments

- a_macro_name : Macro name to check ( trimmed )
- a_if_expr    : Expression to expand if macro is defined

# Example

$assert(I'm defined,$ifdef(define,I'm defined))".to_string()),
                ),
            ),
            (
                "ifdefel".to_owned(),
                DMacroSign::new(
                    "ifdefel",
                    ["a_macro_name^", "a_if_expr", "a_else_expr"],
                    DeterredMacroMap::ifdefel,
                    Some("Execute expressions whether macro is defined or not

# Arguments

- a_macro_name : Macro name to check ( trimmed )
- a_if_expr    : Expression to expand if macro is defined
- a_else_epxr  : Expression to expand if macro is NOT defined

# Example

$assert(I'm defined,$ifdefel(define,I'm defined,I'm NOT defined))
$assert(I'm NOT defined,$ifdefel(defuo,I'm defined,I'm NOT defined))".to_string()),
                ),
            ),
            (
                "logm".to_owned(),
                DMacroSign::new(
                    "logm",
                    ["a_macro_name^"],
                    Self::log_macro_info,
                    Some("Log a macro information. Either print macro body of local or runtime macros.

# Arguments

- a_macro_name : Macro name to log (trimmed)

# Example

$define(test=Test)
$logm(test)".to_string()),
                ),
            ),
            (
                "que".to_owned(),
                DMacroSign::new(
                    "que",
                    ["a_expr"],
                    DeterredMacroMap::queue_content,
                    Some("Que expressions. Queued expressions are expanded when the macro finishes

Use que macro when a macro does operations that do not return a string AND you need to make sure the operation should happen only after all string manipulation ended. Halt is queued by default.

Que does not evalute inner contents and simply put expression into a queue.

# Arguments

- a_expr : Expression to queue

# Example

$que(halt(false))".to_string()),
                ),
            ),
            (
                "ifque".to_owned(),
                DMacroSign::new(
                    "ifque",
                    ["a_bool?^", "a_content"],
                    DeterredMacroMap::if_queue_content,
                    Some("If true, then queue expressions

Use que macro when a macro does operations that do not return a string AND you need to make sure the operation should happen only after all string manipulation ended. Halt is queued by default.

Que does not evalute inner contents and simply put expression into a queue.

# Arguments

- a_bool : Condition [boolean] ( trimmed )
- a_expr : Expression to queue

# Example

$ifque(true,halt(false))".to_string()),
                ),
            ),
            (
                "readto".to_owned(),
                DMacroSign::new(
                    "readto",
                    ["a_from_file^", "a_to_file^"],
                    DeterredMacroMap::read_to,
                    Some("Read from a file and paste into a file

Readto can be only executed on first level therefore readto cannot be used inside other macros

# Arguments

- a_from_file : File to read from ( trimmed )
- a_to_file   : File to paste into ( trimmed )

# Example

$readto(from.txt,into.txt)".to_string()),
                ),
            ),
            (
                "readin".to_owned(),
                DMacroSign::new(
                    "readin",
                    ["a_file?^"],
                    DeterredMacroMap::read_in,
                    Some("Read from a file

Readin can be only executed on first level therefore readin cannot be used inside other macros

# Arguments

- a_file : File to read from ( trimmed )

# Example

$readto(from.txt,into.txt)".to_string()),
                ),
            ),
            (
                "strip".to_owned(),
                DMacroSign::new(
                    "strip",
                    ["a_literal_expr"],
                    DeterredMacroMap::strip_expression,
                    Some("Strip literal expression and then expand 

# Arguments

- a_literal_expr : Expression to strip

# Example

$strip(\\*1,2,3*\\)".to_string()),
                ),
            ),
        ]));
        // Auth realted macros should be segregated from wasm target
        #[cfg(not(feature = "wasm"))]
        {
            map.insert(
                "ifenv".to_owned(),
                DMacroSign::new(
                    "ifenv",
                    ["a_env_name^", "a_if_expr"],
                    DeterredMacroMap::ifenv,
                    Some(
                        "Execute expression if environment variable is set

# Auth : ENV

# Arguments

- a_env_name   : Environment variable ( trimmed )
- a_if_expr    : Expression to expand if env exists

# Example

$assert(I'm alive,$ifenv(HOME,I'm alive))"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "ifenvel".to_owned(),
                DMacroSign::new(
                    "ifenvel",
                    ["a_env_name^", "a_if_expr", "a_else_expr"],
                    DeterredMacroMap::ifenvel,
                    Some(
                        "Execute expression by whether environment variable is set or not

# Auth : ENV

# Arguments

- a_env_name   : Environment variable ( trimmed )
- a_if_expr    : Expression to expand if env exists
- a_else_expr  : Expression to expand if env doesn't exist

# Example

$assert(I'm alive,$ifenvel(HOME,I'm alive,I'm dead))
$assert(I'm dead,$ifenvel(EMOH,I'm alive,I'm dead))"
                            .to_string(),
                    ),
                ),
            );
        }
        // Test method
        #[cfg(debug_assertions)]
        {
            map.insert(
                "test".to_owned(),
                DMacroSign::new(
                    "test",
                    ESR,
                    Self::test_logics,
                    Some("Debugging".to_string()),
                ),
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

    /// Get Function pointer from map
    #[cfg(feature = "signature")]
    pub(crate) fn get_signature(&self, name: &str) -> Option<&DMacroSign> {
        self.macros.get(name)
    }

    /// Check if map contains the name
    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    pub fn undefine(&mut self, name: &str) {
        self.macros.remove(name);
    }

    pub fn rename(&mut self, name: &str, target: &str) -> bool {
        if let Some(func) = self.macros.remove(name) {
            self.macros.insert(target.to_owned(), func);
            return true;
        }
        false
    }

    pub fn new_ext_macro(&mut self, ext: ExtMacroBuilder) {
        if let Some(ExtMacroBody::Deterred(mac_ref)) = ext.macro_body {
            let sign = DMacroSign::new(&ext.macro_name, &ext.args, mac_ref, ext.macro_desc);
            self.macros.insert(ext.macro_name, sign);
        }
    }

    // ----------
    // Keyword Macros start

    /// Loop around given values which is separated by given separator
    ///
    /// # Usage
    ///
    /// $forby($:(),-,a-b-c)
    fn forby(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 3) {
            ap.set_strip(true);
            let mut sums = String::new();
            let body = &args[0];
            let sep = &processor.parse_and_strip(&mut ap, level, &args[1])?;
            let loopable = &processor.parse_and_strip(&mut ap, level, &args[2])?;
            for (count, value) in loopable.split(sep).enumerate() {
                // This overrides value
                processor.add_new_local_macro(level, "a_LN", &count.to_string());
                processor.add_new_local_macro(level, ":", value);
                let result = &processor.parse_and_strip(&mut ap, level, body)?;

                sums.push_str(result);
            }

            // Clear local macro
            processor.remove_local_macro(level, ":");

            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument(
                "Foreach requires two argument".to_owned(),
            ))
        }
    }

    /// Loop around given values and substitute iterators  with the value
    ///
    /// # Usage
    ///
    /// $foreach($:(),a,b,c)
    fn foreach(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let mut sums = String::new();
            let body = &args[0];
            let loopable = &processor.parse_and_strip(&mut ap, level, &args[1])?;
            for (count, value) in loopable.split(',').enumerate() {
                // This overrides value
                processor.add_new_local_macro(level, "a_LN", &count.to_string());

                processor.add_new_local_macro(level, ":", value);
                let result = &processor.parse_and_strip(&mut ap, level, body)?;

                sums.push_str(result);
            }

            // Clear local macro
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
    /// $forline($:(),Content)
    fn forline(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let mut sums = String::new();
            let body = &args[0];
            let loopable = &processor.parse_and_strip(&mut ap, level, &args[1])?;
            let mut count = 1;
            for value in loopable.lines() {
                // This overrides value
                processor.add_new_local_macro(level, "a_LN", &count.to_string());
                processor.add_new_local_macro(level, ":", value);
                let result = processor.parse_and_strip(&mut ap, level, body)?;
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
    /// $forloop($:(),1,5)
    fn forloop(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 3) {
            ap.set_strip(true);
            let mut sums = String::new();

            let body = &args[0];
            let min_src = trim!(&processor.parse_and_strip(&mut ap, level, &args[1])?).to_string();
            let max_src = trim!(&processor.parse_and_strip(&mut ap, level, &args[2])?).to_string();

            let min = if let Ok(num) = min_src.parse::<usize>() {
                num
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "Forloop's min value should be non zero positive integer but given {}",
                    min_src
                )));
            };
            let max = if let Ok(num) = max_src.parse::<usize>() {
                num
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "Forloop's max value should be non zero positive integer but given \"{}\"",
                    max_src
                )));
            };
            let mut result: String;
            for value in min..=max {
                processor.add_new_local_macro(level, ":", &value.to_string());
                result = processor.parse_and_strip(&mut ap, level, body)?;

                sums.push_str(&result);
                result.clear();
            }

            // Clear local macro
            processor.remove_local_macro(level, ":");

            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument(
                "Forloop requires two argument".to_owned(),
            ))
        }
    }

    /// Log macro information
    ///
    /// # Usage
    ///
    /// $logm(mac)
    fn log_macro_info(args: &str, level: usize, p: &mut Processor) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let macro_name = trim!(&p.parse_and_strip(&mut ap, level, args)?).to_string();
        let body = if let Ok(body) = p.get_local_macro_body(level, &macro_name) {
            trim!(body).to_string()
        } else if let Ok(body) = p.get_runtime_macro_body(&macro_name) {
            trim!(body).to_string()
        } else {
            return Err(RadError::InvalidArgument(format!(
                "Macro \"{}\" doesn't exist",
                &macro_name
            )));
        };
        p.log_message(&body)?;
        Ok(None)
    }

    /// Print content according to given condition
    ///
    /// # Usage
    ///
    /// $if(evaluation, ifstate)
    fn if_cond(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let boolean = &processor.parse_and_strip(&mut ap, level, &args[0])?;

            // Given condition is true
            let cond = Utils::is_arg_true(boolean);
            if let Ok(cond) = cond {
                if cond {
                    let if_expr = processor.parse_and_strip(&mut ap, level, &args[1])?;
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
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 3) {
            ap.set_strip(true);

            let boolean = &processor.parse_and_strip(&mut ap, level, &args[0])?;

            // Given condition is true
            let cond = Utils::is_arg_true(boolean);
            if let Ok(cond) = cond {
                if cond {
                    let if_expr = processor.parse_and_strip(&mut ap, level, &args[1])?;
                    return Ok(Some(if_expr));
                }
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "Ifelse requires either true/false or zero/nonzero integer but given \"{}\"",
                    boolean
                )));
            }

            // Else state
            let else_expr = processor.parse_and_strip(&mut ap, level, &args[2])?;
            Ok(Some(else_expr))
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
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);

            let name = trim!(&processor.parse_and_strip(&mut ap, level, &args[0])?).to_string();

            let boolean = processor.contains_macro(&name, MacroType::Any);
            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_and_strip(&mut ap, level, &args[1])?;
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
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 3) {
            ap.set_strip(true);

            let name = trim!(&processor.parse_and_strip(&mut ap, level, &args[0])?).to_string();

            let boolean = processor.contains_macro(&name, MacroType::Any);
            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_and_strip(&mut ap, level, &args[1])?;
                Ok(Some(if_expr))
            } else {
                let else_expr = processor.parse_and_strip(&mut ap, level, &args[2])?;
                Ok(Some(else_expr))
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
    #[cfg(not(feature = "wasm"))]
    fn ifenv(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("ifenv", AuthType::ENV, processor)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);

            let name = trim!(&processor.parse_and_strip(&mut ap, level, &args[0])?).to_string();

            let boolean = std::env::var(name).is_ok();

            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_and_strip(&mut ap, level, &args[1])?;
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
    #[cfg(not(feature = "wasm"))]
    fn ifenvel(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("ifenvel", AuthType::ENV, processor)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 3) {
            ap.set_strip(true);

            let name = trim!(&processor.parse_and_strip(&mut ap, level, &args[0])?).to_string();

            let boolean = std::env::var(name).is_ok();

            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_and_strip(&mut ap, level, &args[1])?;
                Ok(Some(if_expr))
            } else {
                let else_expr = processor.parse_and_strip(&mut ap, level, &args[2])?;
                Ok(Some(else_expr))
            }
        } else {
            Err(RadError::InvalidArgument(
                "ifenvel requires three arguments".to_owned(),
            ))
        }
    }

    /// Strip literal expression
    ///
    /// This strip expression and then expand it
    ///
    /// # Usage
    ///
    /// $strip(\*expression*\)
    fn strip_expression(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().strip(args);
        let result = processor.parse_chunk_args(level, "", &args)?;

        Ok(if result.is_empty() {
            None
        } else {
            Some(result)
        })
    }

    /// Assert fail
    ///
    /// This has to be deterred macro because it's value should be evaluated later
    ///
    /// # Usage
    ///
    /// $fassert(abc)
    fn assert_fail(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let backup = processor.state.behaviour;
        processor.state.behaviour = ErrorBehaviour::Assert;

        let mut ap = ArgParser::new().no_strip();
        let result = processor.parse_and_strip(&mut ap, level, &args);
        processor.state.behaviour = backup;
        if result.is_err() {
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
        processor.insert_queue(&args);
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
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let boolean = &processor.parse_and_strip(&mut ap, level, &args[0])?;
            let cond = Utils::is_arg_true(boolean)?;
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

    fn escape_blanks(
        _: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if level != 1 {
            return Err(RadError::UnallowedMacroExecution(
                "\"EB\" is only available on first level".to_string(),
            ));
        }
        processor.state.lexor_escape_blanks = true;
        Ok(None)
    }

    /// Read to
    ///
    /// # Usage
    ///
    /// $readto(file_a,file_b)
    fn read_to(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        // Needs both permission
        if !Utils::is_granted("readto", AuthType::FIN, processor)?
            || !Utils::is_granted("readto", AuthType::FOUT, processor)?
        {
            return Ok(None);
        }
        if level != 1 {
            return Err(RadError::UnallowedMacroExecution(
                "Readto doesn't support nested buf read".to_string(),
            ));
        }
        let mut ap = ArgParser::new().no_strip();
        let args = ap.args_to_vec(args, ',', GreedyState::Never);
        ap.set_strip(true);
        if args.len() >= 2 {
            let file_path = PathBuf::from(processor.parse_and_strip(
                &mut ap,
                level,
                trim!(&args[0]).as_ref(),
            )?);
            let to_path = PathBuf::from(processor.parse_and_strip(
                &mut ap,
                level,
                trim!(&args[1]).as_ref(),
            )?);
            let mut raw_include = false;
            if file_path.is_file() {
                let canonic = file_path.canonicalize()?;
                Utils::check_include_sanity(processor, &canonic)?;

                // Check path sanity if to_path exists
                if to_path.exists() {
                    Utils::check_include_sanity(processor, &to_path.canonicalize()?)?;
                }
                // Set sandbox after error checking or it will act starngely
                processor.set_sandbox(true);

                // Optionally enable raw mode
                if args.len() >= 3 {
                    raw_include = Utils::is_arg_true(&processor.parse_and_strip(
                        &mut ap,
                        level,
                        trim!(&args[2]).as_ref(),
                    )?)?;

                    // You don't have to backup pause state because include wouldn't be triggered
                    // at the first place, if paused was true
                    if raw_include {
                        processor.state.paused = true;
                    }
                }

                let mut file_target = FileTarget::empty();
                file_target.set_path(&to_path);
                processor.state.relay.push(RelayTarget::File(file_target));

                // Create chunk
                let chunk = processor.process_file_as_chunk(&file_path)?;

                // Reset flow control per processing
                if processor.state.flow_control != FlowControl::None {
                    processor.reset_flow_control();
                }
                if raw_include {
                    processor.state.paused = false; // Recover paused state
                }
                processor.set_sandbox(false);
                processor.state.input_stack.remove(&canonic); // Collect stack
                processor.state.relay.pop(); // Pop relay
                Ok(chunk)
            } else {
                Err(RadError::InvalidArgument(format!(
                    "readto cannot read non-file \"{}\"",
                    file_path.display()
                )))
            }
        } else {
            Err(RadError::InvalidArgument(
                "readto requires two argument".to_owned(),
            ))
        }
    }

    /// Read in
    ///
    /// # Usage
    ///
    /// $readin(file_a)
    fn read_in(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("readin", AuthType::FIN, processor)? {
            return Ok(None);
        }
        if level != 1 {
            return Err(RadError::UnallowedMacroExecution(
                "Readin doesn't support nested buf read".to_string(),
            ));
        }
        let mut ap = ArgParser::new().no_strip();
        let args = ArgParser::new().args_to_vec(args, ',', GreedyState::Never);
        ap.set_strip(true);
        if !args.is_empty() {
            let file_path = PathBuf::from(processor.parse_and_strip(
                &mut ap,
                level,
                trim!(&args[0]).as_ref(),
            )?);
            let mut raw_include = false;
            if file_path.is_file() {
                let canonic = file_path.canonicalize()?;
                Utils::check_include_sanity(processor, &canonic)?;
                // Set sandbox after error checking or it will act starngely
                processor.set_sandbox(true);

                // Optionally enable raw mode
                if args.len() >= 2 {
                    raw_include = Utils::is_arg_true(&processor.parse_and_strip(
                        &mut ap,
                        level,
                        trim!(&args[1]).as_ref(),
                    )?)?;

                    // You don't have to backup pause state because include wouldn't be triggered
                    // at the first place, if paused was true
                    if raw_include {
                        processor.state.paused = true;
                    }
                }

                // Create chunk
                let chunk = processor.process_file(&file_path)?;

                // Reset flow control per processing
                if processor.state.flow_control != FlowControl::None {
                    processor.reset_flow_control();
                }
                if raw_include {
                    processor.state.paused = false; // Recover paused state
                }
                processor.set_sandbox(false);
                processor.state.input_stack.remove(&canonic); // Collect stack
                Ok(chunk)
            } else {
                Err(RadError::InvalidArgument(format!(
                    "readto cannot read non-file \"{}\"",
                    file_path.display()
                )))
            }
        } else {
            Err(RadError::InvalidArgument(
                "readto requires two argument".to_owned(),
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
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);

            let macro_name =
                trim!(&processor.parse_and_strip(&mut ap, level, &args[0])?).to_string();
            if !processor.contains_macro(&macro_name, MacroType::Any) {
                return Err(RadError::InvalidArgument(format!(
                    "Macro \"{}\" doesn't exist",
                    macro_name
                )));
            }
            let args = &args[1];
            let result =
                processor.parse_and_strip(&mut ap, level, &format!("${}({})", macro_name, args))?;
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
    fn spread_data(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);

            let macro_name = trim!(&args[0]);
            // Trimming data might be very costly operation
            // Plus, it is already trimmed by csv crate.
            let macro_data = trim!(&args[1]);

            let result =
                Formatter::csv_to_macros(&macro_name, &macro_data, &processor.state.newline)?;

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
            let result = processor.parse_and_strip(&mut ap, level, &result)?;

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

    #[allow(unused_variables)]
    #[cfg(debug_assertions)]
    fn test_logics(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().no_strip().args_with_len(args, 3) {
            //processor.log_message(&args[0]);
            //processor.log_message(&args[1]);
            //processor.log_message(&args[2]);
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Insufficient argumetns for test".to_owned(),
            ))
        }
    }
    // Keyword macros end
    // ----------
}

/// Keyword Macro signature
#[derive(Clone)]
pub(crate) struct DMacroSign {
    name: String,
    args: Vec<String>,
    pub logic: DFunctionMacroType,
    #[allow(dead_code)]
    desc: Option<String>,
}

impl DMacroSign {
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

impl std::fmt::Display for DMacroSign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut inner = self
            .args
            .iter()
            .fold(String::new(), |acc, arg| acc + arg + ",");
        // This removes last "," character
        inner.pop();
        write!(f, "${}({})", self.name, inner)
    }
}

#[cfg(feature = "signature")]
impl From<&DMacroSign> for crate::sigmap::MacroSignature {
    fn from(ms: &DMacroSign) -> Self {
        Self {
            variant: crate::sigmap::MacroVariant::Deterred,
            name: ms.name.to_owned(),
            args: ms.args.to_owned(),
            expr: ms.to_string(),
            desc: ms.desc.clone(),
        }
    }
}
