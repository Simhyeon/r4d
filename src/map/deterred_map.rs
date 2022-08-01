#[cfg(not(feature = "wasm"))]
use crate::auth::AuthType;
#[cfg(not(feature = "wasm"))]
use crate::common::{ContainerType, FileTarget, FlowControl, RelayTarget};
use crate::common::{ErrorBehaviour, MacroType, RadResult};
use crate::consts::ESR;
use crate::extension::{ExtMacroBody, ExtMacroBuilder};
use crate::formatter::Formatter;
use crate::parser::GreedyState;
use crate::utils::Utils;
use crate::ArgParser;
use crate::{trim, Processor, RadError};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::FromIterator;
#[cfg(not(feature = "wasm"))]
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
                "append".to_owned(),
                DMacroSign::new(
                    "append",
                    ["a_macro_name^", "a_content","a_trailer+"],
                    Self::append,
                    Some(
"Append contents to a macro. If the macro doesn't exist, yields error

- If given a \"trailer\", the macro checks if target macro has a trailer and 
append if not.
- If a macro body is empty, trailer is not appended

# NOT deterred

# Arguments

- a_macro_name : a macro name to append to ( trimmed )
- a_content    : contents to be added
- a_trailer    : A trailer to append before content ( Optional ) 

# Example

$define(container=Before)
$append(container,$space()After)
$assert($container(),Before After)

$define(arr=)
$append(arr,first,$comma())
$append(arr,second,$comma())
$assert($arr(),first,second)".to_string(),
                    ),
                ),
            ),
            (
                "grepmap".to_owned(),
                DMacroSign::new(
                    "grepmap",
                    ["a_grep_type^","a_expr", "a_macro_name^", "a_text"],
                    DeterredMacroMap::grep_map,
                    Some(
"Capture expressions and apply a macro to each captured expression

# Auth : FIN

# Note

- If grep type is file, grep operation is executed on per line.
- If grep type is text, grep operation is executed one whole text.

# Arguments

- a_grep_type  : Grep type to execute. [\"text\", \"file\" ]
- a_expr       : An expression to match
- a_macro_name : A macro name to execute on each captured string
- a_text       : Source text to find expressions

# Example

$define(ss,a_text=$sub(2,,$a_text())$nl())
$assert(c$nl()d$nl()e,$grepmap^(text,ab.,ss,abc abd abe))".to_string()),
                ),
            ),
            (
                "EB".to_owned(),
                DMacroSign::new(
                    "EB",
                    ESR,
                    DeterredMacroMap::escape_blanks,
                    Some(
"Escape all following blanks until not. This can only be invoked at first level

# NOT deterred

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

# NOT deterred

# Arguments

- a_macro_name : A macro name to exectue ( trimmed )
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
                    Some("Assert succeeds when text expansion yields error

# NOT deterred

# Arguments

- a_expr: An expression to audit

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

# Expansion order

1. a_sep  : Expanded on time
2. a_text : Expanded on time
3. a_body : Split text by separator, and expanded by per item.

# Arguments

- a_body : A body to be pasted as iterated item
- a_sep  : A separator to split a text
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
                    ["a_body", "a_array^"],
                    DeterredMacroMap::foreach,
                    Some(
                        "Iterate around given array.

An iterated value is bound to macro \":\"
 
# Expansion order

1. a_array : Expanded on time
2. a_body : Split array by comma, and expanded by per item.

# Arguments

- a_body  : A body to be pasted as iterated item
- a_array : An array to iterate ( trimmed )

# Example

$assert(a+b+c+,$foreach($:()+,a,b,c))".to_string(),
                    ),
                ),
            ),
            (
                "forline".to_owned(),
                DMacroSign::new(
                    "forline",
                    ["a_body","a_lines^"],
                    DeterredMacroMap::forline,
                    Some(
"Loop around given lines separated by newline chraracter. 

An iterated value is bound to macro \":\"
 
# Expansion order

1. a_lines : Expanded on time
2. a_body  : Split lines by newline, and expanded by per item. ( trimmed )

# Arguments

- a_body  : A body to be pasted as iterated item
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

An iterated value is bound to macro \":\" 

# Expansion order


1. a_min  : Expanded on time
2. a_max  : Expanded on time
3. a_body : Creates a range of numbers from min+max pair, and expanded by per 
item.

# Arguments

- a_body : A body to be pasted as iterated item
- a_min  : A start index ( trimmed )
- a_max  : A end index ( trimmed )

# Example

$assert(1+2+3+,$forloop($:()+,1,3))".to_string()),
                ),
            ),
            (
                "map".to_owned(),
                DMacroSign::new(
                    "map",
                    ["a_macro_name^", "a_array"],
                    Self::map_array,
                    Some(
"Execute macro on each array item

# NOT Deterred

# Arguments

- a_macro_name : A macro name to execute ( trimmed ) 
- a_array      : An array to iterate

# Example

$define(m,a_src=$a_src()+)
$assert(a+b+c,$map(m,a,b,c))".to_string()),
                ),
            ),
            (
                "mapl".to_owned(),
                DMacroSign::new(
                    "mapl",
                    ["a_macro_name^", "a_lines"],
                    Self::map_lines,
                    Some(
"Execute macro on each line

# NOT Deterred

# Arguments

- a_macro_name : A macro name to execute ( trimmed ) 
- a_lines      : A lines to iterate

# Example

$define(m,a_src=$a_src()+)
$assert(a+b+c,$mapl(m,a$nl()b$nl()c$nl()))".to_string()),
                ),
            ),
            (
                "spread".to_owned(),
                DMacroSign::new(
                    "spread",
                    ["a_macro_name^", "a_csv_value^"],
                    Self::spread_data,
                    Some(
"Execute a macro multiple times with given data chunk. Each csv line represents 
an argument for a macro

# NOT Deterred

# Arguments

- a_macro_name : A macro name to execute ( trimmed ) 
- a_csv_value  : Arguments table ( trimmed )

# Example

$assert=(
    text******
    ***text***
    ******text,
    $spread=(
        align,
        left,10,*,text
        center,10,*,text
        right,10,*,text
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
"Check condition and then execute an expression if the condition is true

# Expansion order

1. a_cond    : Expanded on time
2. a_if_expr : Only when a_cond is true

# Arguments

- a_cond    : A condition to audit ( trimmed )
- a_if_expr : An expression to expand if the condition is true

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
"Check condition and execute a different expression by the condition

# Expansion order

1. a_cond      : Expanded on time
2. a_if_expr   : Only when a_cond is true
3. a_else_expr : Only when a_cond is false

# Arguments

- a_cond      : A condition to audit ( trimmed )
- a_if_expr   : An expression to expand if the condition is \"true\"
- a_else_expr : An expression to expand if the condition is \"false\"

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
                    Some("Execute an expression if macro is defined

# Expansion order

1. a_macro_name : Expanded on time
2. a_if_expr    : Only when a_macro_name is defined

# Arguments

- a_macro_name : A macro name to check ( trimmed )
- a_if_expr    : An expression to expand if the macro is defined

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
                    Some(
"Execute an expression by whether macro is defined or not

# Expansion order

1. a_macro_name : Expanded on time
2. a_if_expr    : Only when a_macro_name is defined
3. a_else_expr  : Only when a_macro_name is NOT defined

# Arguments

- a_macro_name : A macro name to check ( trimmed )
- a_if_expr    : An expression to expand if the macro is defined
- a_else_epxr  : An expression to expand if the macro is NOT defined

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
                    Some(
"Log a macro information. Either print a macro body of a local or a runtime 
macro.

# NOT deterred

# Arguments

- a_macro_name : A macro name to log (trimmed)

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
                    Some(
"Que an expression. Queued expressions are expanded when the macro finishes

Use a que macro when macros do operations that do not return a string AND you 
need to make sure the operation should happen only after all string manipulation 
ended. Halt is queued by default.

Que does not evalute inner contents and simply put expression into a queue.

# NO expansion at all

# Arguments

- a_expr : An expression to queue

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

Use a que macro when macros do operations that do not return a string AND you 
need to make sure the operation should happen only after all string manipulation 
ended. Halt is queued by default.

Que does not evalute inner contents and simply put expression into a queue.

# Expansion order

1. a_bool : Expanded on time
2. a_expr : NEVER expanded

# Arguments

- a_bool : A condition [boolean] ( trimmed )
- a_expr : An expression to queue

# Example

$ifque(true,halt(false))".to_string()),
                ),
            ),
            (
                "strip".to_owned(),
                DMacroSign::new(
                    "strip",
                    ["a_literal_expr"],
                    DeterredMacroMap::strip_expression,
                    Some("Strip literal expression and then expand 

# Expansion order

1. a_literal_expr : After a pair of quote was striped

# Arguments

- a_literal_expr : An expression to strip

# Example

$strip(\\*1,2,3*\\)".to_string()),
                ),
            ),
        ]));
        // Auth realted macros should be segregated from wasm target
        #[cfg(not(feature = "wasm"))]
        {
            map.insert(
                "include".to_owned(),
                DMacroSign::new(
                    "include",
                    ["a_filename^", "a_raw_mode^+?"],
                    Self::include,
                    Some(
                        "Include a file

- Include works as bufread in first level and chunk read in nested call.
- Use readin if you want to enforce bufread
- If raw mode is enabled include doesn't expand any macros inside the file

# NOT Deterred

# AUTH : FIN

# Arguments

- a_filename : A file name to read ( trimmed )
- a_raw_mode : Whehter to escape the read. A default is false [boolean] ( trimmed, optional )

$include(file_path)
$include(file_path, true)"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "tempin".to_owned(),
                DMacroSign::new(
                    "tempin",
                    ESR,
                    Self::temp_include,
                    Some(
                        "Include a temporary file

- A default temporary path is folloiwng
- Windows : It depends, but %APPDATA%\\Local\\Temp\\rad.txt can be one
- *nix    : /tmp/rad.txt

# NOT Deterred

# Auth: FIN

# Example

$tempin()"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "mapf".to_owned(),
                DMacroSign::new(
                    "mapf",
                    ["a_macro_name^", "a_file"],
                    Self::map_file,
                    Some(
                        "Execute macro on each lines of a file

# Note : mapf macro doesn't expand lines from a file.

# Auth : FIN

# NOT Deterred

# Arguments

- a_macro_name : A macro name to execute ( trimmed ) 
- a_file       : A file to get lines iterator

# Example

$define(m,a_src=$a_src()+)
$assert(a+b+c,$mapf(m,file_name.txt))"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "readto".to_owned(),
                DMacroSign::new(
                    "readto",
                    ["a_from_file^", "a_to_file^", "a_raw_mode?+^"],
                    DeterredMacroMap::read_to,
                    Some(
                        "Read from a file as bufread and paste into a file

# Auth : FIN + FOUT

# NOT deterred

# Arguments

- a_from_file : A file to read from ( trimmed )
- a_to_file   : A file to paste into ( trimmed )
- a_raw_mode : Whehter to escape the read. A default is false [boolean] ( trimmed, optional )

# Example

$readto(from.txt,into.txt)"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "readin".to_owned(),
                DMacroSign::new(
                    "readin",
                    ["a_file?^", "a_raw_mode^+?"],
                    DeterredMacroMap::read_in,
                    Some(
                        "Read from a file as \"Bufread\"

# Auth : FIN

# NOT deterred

# Arguments

- a_file : A file to read from ( trimmed )
- a_raw_mode : Whehter to escape the read. A default is false [boolean] ( trimmed, optional )

# Example

$readto(from.txt,into.txt)"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "ifenv".to_owned(),
                DMacroSign::new(
                    "ifenv",
                    ["a_env_name^", "a_if_expr"],
                    DeterredMacroMap::ifenv,
                    Some(
                        "Execute an expression if an environment variable is set

# Auth : ENV

# Expansion order

1. a_env_name : Expanded on time
2. a_if_expr  : Only when env_name is defined

# Arguments

- a_env_name   : An environment variable to check ( trimmed )
- a_if_expr    : An expression to expand if env exists

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
                        "Execute an expression by whether environment variable is set or not

# Auth : ENV

# Expansion order

1. a_env_name   : Expanded on time
2. a_if_expr    : Only when env_name is defined
3. a_else_expr  : Only when env_name is NOT defined


# Arguments

- a_env_name   : An environment variable to check ( trimmed )
- a_if_expr    : An expression to expand if env exists
- a_else_expr  : An expression to expand if env doesn't exist

# Example

$assert(I'm alive,$ifenvel(HOME,I'm alive,I'm dead))
$assert(I'm dead,$ifenvel(EMOH,I'm alive,I'm dead))"
                            .to_string(),
                    ),
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

    /// Append content to a macro
    ///
    /// This is deterred because it needs level for local macro indexing
    ///
    /// Runtime + local macros can be appended.
    ///
    /// # Usage
    ///
    /// $append(macro_name,Content,tailer)
    fn append(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        let args = ap.args_to_vec(args, ',', GreedyState::Never);
        ap.set_strip(true);
        if args.len() >= 2 {
            let name =
                processor.parse_and_strip(&mut ap, level, "append", trim!(&args[0]).as_ref())?;
            let target = processor.parse_and_strip(&mut ap, level, "append", &args[1])?;
            let mut trailer = None;

            if args.len() >= 3 {
                trailer = Some(processor.parse_and_strip(&mut ap, level, "append", &args[2])?);
            }

            if let Some(name) = processor.contains_local_macro(level, &name) {
                if let Some(tt) = trailer {
                    let body = processor.get_local_macro_body(&name)?;
                    if !body.ends_with(&tt) && !body.is_empty() {
                        processor.append_local_macro(&name, &format!("{}{}", tt, target));
                        return Ok(None);
                    }
                }
                processor.append_local_macro(&name, &target);
            } else if processor.contains_macro(&name, MacroType::Runtime) {
                // Append to runtime
                if let Some(tt) = trailer {
                    let body = processor.get_runtime_macro_body(&name)?;
                    if !body.ends_with(&tt) && !body.is_empty() {
                        processor.append_macro(&name, &format!("{}{}", tt, target));
                        return Ok(None);
                    }
                }
                processor.append_macro(&name, &target);
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "Macro \"{}\" doesn't exist",
                    name
                )));
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Append at least requires two arguments".to_owned(),
            ))
        }
    }

    /// Apply map on array
    ///
    /// # Usage
    ///
    /// $map(macro_name,array)
    fn map_array(args: &str, level: usize, p: &mut Processor) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let macro_name = trim!(&args[0]);
            let src = p.parse_and_strip(&mut ap, level, "map", &args[1])?;
            let array = src.split(',');

            let mut acc = String::new();
            for item in array {
                acc.push_str(
                    &p.execute_macro(level, "map", &macro_name, item)?
                        .unwrap_or_default(),
                );
            }
            Ok(Some(acc))
        } else {
            Err(RadError::InvalidArgument(
                "map requires two arguments".to_owned(),
            ))
        }
    }

    /// Apply map on lines
    ///
    /// # Usage
    ///
    /// $mapl(macro_name,lines)
    fn map_lines(args: &str, level: usize, p: &mut Processor) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let macro_name = trim!(&args[0]);
            let src = p.parse_and_strip(&mut ap, level, "mapl", &args[1])?;
            let lines = src.lines();

            let mut acc = String::new();
            for item in lines {
                acc.push_str(
                    &p.execute_macro(level, "mapl", &macro_name, item)?
                        .unwrap_or_default(),
                );
            }
            Ok(Some(acc))
        } else {
            Err(RadError::InvalidArgument(
                "mapl requires two arguments".to_owned(),
            ))
        }
    }

    /// Apply map on file lines
    ///
    /// # Usage
    ///
    /// $mapf(macro_name,file_name)
    #[cfg(not(feature = "wasm"))]
    fn map_file(args: &str, level: usize, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("grepmap", AuthType::FIN, p)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let macro_name = trim!(&args[0]);
            let file = BufReader::new(std::fs::File::open(
                p.parse_and_strip(&mut ap, level, "mapf", &args[1])?,
            )?)
            .lines();

            let mut acc = String::new();
            for line in file {
                let line = line?;
                acc.push_str(
                    &p.execute_macro(level, "mapf", &macro_name, &line)?
                        .unwrap_or_default(),
                );
            }
            Ok(Some(acc))
        } else {
            Err(RadError::InvalidArgument(
                "mapf requires two arguments".to_owned(),
            ))
        }
    }

    /// Apply maps on captured expressions
    ///
    /// # Usage
    ///
    /// $grepmap(type,expr,macro,text)
    fn grep_map(args: &str, level: usize, p: &mut Processor) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 4) {
            ap.set_strip(true);
            let grep_type = &args[0];
            let match_expr = &args[1];
            let macro_name = trim!(&args[2]);
            let source = &args[3];

            let bufread = match grep_type.to_lowercase().as_str() {
                #[cfg(not(feature = "wasm"))]
                "file" => {
                    if !Utils::is_granted("grepmap", AuthType::FIN, p)? {
                        return Ok(None);
                    }
                    true
                }
                "text" => false,
                _ => {
                    return Err(RadError::InvalidArgument(format!(
                        "{} is not a valid grep type",
                        grep_type
                    )))
                }
            };

            if bufread && !std::path::Path::new(source).exists() {
                return Err(RadError::InvalidArgument(format!(
                    "Cannot find a file \"{}\" ",
                    source
                )));
            }

            if match_expr.is_empty() {
                return Err(RadError::InvalidArgument(
                    "Regex expression cannot be an empty string".to_string(),
                ));
            }

            let mut res = String::new();

            // If this regex is not cloned, "capture" should collect captured string into a allocated
            // vector. Which is generaly worse for performance.
            let reg = p.try_get_or_insert_regex(match_expr)?.clone();

            if !bufread {
                for cap in reg.captures_iter(source) {
                    let captured = cap.get(0).map_or("", |m| m.as_str());
                    let expanded = p
                        .execute_macro(level, "grepmap", &macro_name, captured)?
                        .unwrap_or_default();
                    res.push_str(&expanded);
                }
            } else {
                let lines = BufReader::new(File::open(std::path::Path::new(source))?).lines();

                for line in lines {
                    let line = line?;
                    for cap in reg.captures_iter(&line) {
                        let captured = cap.get(0).map_or("", |m| m.as_str());
                        let expanded = p
                            .execute_macro(level, "grepmap", &macro_name, captured)?
                            .unwrap_or_default();
                        res.push_str(&expanded);
                    }
                }
            }

            Ok(Some(res))
        } else {
            Err(RadError::InvalidArgument(
                "grepamp requires four arguments".to_owned(),
            ))
        }
    }

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
            let sep = &processor.parse_and_strip(&mut ap, level, "forby", &args[1])?;
            let loopable = &processor.parse_and_strip(&mut ap, level, "forby", &args[2])?;
            for (count, value) in loopable.split_terminator(sep).enumerate() {
                // This overrides value
                processor.add_new_local_macro(level, "a_LN", &count.to_string());
                processor.add_new_local_macro(level, ":", value);
                let result = &processor.parse_and_strip(&mut ap, level, "forby", body)?;

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
            let loop_src = processor.parse_and_strip(&mut ap, level, "foreach", &args[1])?;
            let loopable = trim!(&loop_src);
            for (count, value) in loopable.as_ref().split(',').enumerate() {
                // This overrides value
                processor.add_new_local_macro(level, "a_LN", &count.to_string());
                processor.add_new_local_macro(level, ":", value);
                let result = &processor.parse_and_strip(&mut ap, level, "foreach", body)?;

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
            let loop_src = processor.parse_and_strip(&mut ap, level, "forline", &args[1])?;
            let loopable = trim!(&loop_src);
            let mut count = 1;
            for value in loopable.lines() {
                // This overrides value
                processor.add_new_local_macro(level, "a_LN", &count.to_string());
                processor.add_new_local_macro(level, ":", value);
                let result = processor.parse_and_strip(&mut ap, level, "forline", body)?;
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
            let min_src =
                trim!(&processor.parse_and_strip(&mut ap, level, "forloop", &args[1])?).to_string();
            let max_src =
                trim!(&processor.parse_and_strip(&mut ap, level, "forloop", &args[2])?).to_string();

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
                result = processor.parse_and_strip(&mut ap, level, "forloop", body)?;

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
        let macro_name = trim!(&p.parse_and_strip(&mut ap, level, "logm", args)?).to_string();
        let body = if let Some(name) = p.contains_local_macro(level, &macro_name) {
            p.get_local_macro_body(&name)?.to_string()
        } else if let Ok(body) = p.get_runtime_macro_body(&macro_name) {
            body.to_string()
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
            let boolean = &processor.parse_and_strip(&mut ap, level, "if", &args[0])?;

            // Given condition is true
            let cond = Utils::is_arg_true(boolean);
            if let Ok(cond) = cond {
                if cond {
                    let if_expr = processor.parse_and_strip(&mut ap, level, "if", &args[1])?;
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

            let boolean = &processor.parse_and_strip(&mut ap, level, "ifelse", &args[0])?;

            // Given condition is true
            let cond = Utils::is_arg_true(boolean);
            if let Ok(cond) = cond {
                if cond {
                    let if_expr = processor.parse_and_strip(&mut ap, level, "ifelse", &args[1])?;
                    return Ok(Some(if_expr));
                }
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "Ifelse requires either true/false or zero/nonzero integer but given \"{}\"",
                    boolean
                )));
            }

            // Else state
            let else_expr = processor.parse_and_strip(&mut ap, level, "ifelse", &args[2])?;
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

            let name =
                trim!(&processor.parse_and_strip(&mut ap, level, "ifdef", &args[0])?).to_string();

            let boolean = processor.contains_macro(&name, MacroType::Any);
            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_and_strip(&mut ap, level, "ifdef", &args[1])?;
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
    /// $ifdefel(macro_name,expr,expr2)
    fn ifdefel(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 3) {
            ap.set_strip(true);

            let name =
                trim!(&processor.parse_and_strip(&mut ap, level, "ifdefel", &args[0])?).to_string();

            let boolean = processor.contains_macro(&name, MacroType::Any);
            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_and_strip(&mut ap, level, "ifdefel", &args[1])?;
                Ok(Some(if_expr))
            } else {
                let else_expr = processor.parse_and_strip(&mut ap, level, "ifdefel", &args[2])?;
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

            let name =
                trim!(&processor.parse_and_strip(&mut ap, level, "ifenv", &args[0])?).to_string();

            let boolean = std::env::var(name).is_ok();

            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_and_strip(&mut ap, level, "ifenv", &args[1])?;
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

            let name =
                trim!(&processor.parse_and_strip(&mut ap, level, "ifenvel", &args[0])?).to_string();

            let boolean = std::env::var(name).is_ok();

            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_and_strip(&mut ap, level, "ifenvel", &args[1])?;
                Ok(Some(if_expr))
            } else {
                let else_expr = processor.parse_and_strip(&mut ap, level, "ifenvel", &args[2])?;
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
        let result = processor.parse_and_strip(&mut ap, level, "fassert", args);
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
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let boolean = &processor.parse_and_strip(&mut ap, level, "ifque", &args[0])?;
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
    #[cfg(not(feature = "wasm"))]
    fn read_to(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        // Needs both permission
        if !Utils::is_granted("readto", AuthType::FIN, processor)?
            || !Utils::is_granted("readto", AuthType::FOUT, processor)?
        {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let file_path = PathBuf::from(processor.parse_and_strip(
                &mut ap,
                level,
                "readto",
                trim!(&args[0]).as_ref(),
            )?);
            let to_path = PathBuf::from(processor.parse_and_strip(
                &mut ap,
                level,
                "readto",
                trim!(&args[1]).as_ref(),
            )?);
            if file_path == to_path {
                return Err(RadError::InvalidArgument(format!(
                    "readto cannot read from and into a same file \"{}\"",
                    file_path.display()
                )));
            }
            let mut raw_include = false;
            if file_path.is_file() {
                let canonic = file_path.canonicalize()?;
                Utils::check_file_sanity(processor, &canonic)?;

                // Check path sanity if to_path exists
                if to_path.exists() {
                    Utils::check_file_sanity(processor, &to_path.canonicalize()?)?;
                }
                // Set sandbox after error checking or it will act starngely
                processor.set_sandbox(true);

                // Optionally enable raw mode
                if args.len() >= 3 {
                    raw_include = Utils::is_arg_true(&processor.parse_and_strip(
                        &mut ap,
                        level,
                        "readto",
                        trim!(&args[2]).as_ref(),
                    )?)?;

                    // You don't have to backup pause state because include wouldn't be triggered
                    // at the first place, if paused was true
                    if raw_include {
                        processor.state.flow_control = FlowControl::Escape;
                    }
                }

                let file_target = FileTarget::from_path(&to_path)?;
                processor.state.relay.push(RelayTarget::File(file_target));

                // Create chunk
                let chunk = processor.process_file_as_chunk(&file_path, ContainerType::Expand)?;

                // Reset flow control per processing
                if processor.state.flow_control != FlowControl::None {
                    processor.reset_flow_control();
                }
                if raw_include {
                    processor.state.flow_control = FlowControl::None; // Recover state
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
    #[cfg(not(feature = "wasm"))]
    fn read_in(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        use crate::WarningType;

        if !Utils::is_granted("readin", AuthType::FIN, processor)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 1) {
            ap.set_strip(true);
            let file_path = PathBuf::from(processor.parse_and_strip(
                &mut ap,
                level,
                "readin",
                trim!(&args[0]).as_ref(),
            )?);
            let mut raw_include = false;
            if file_path.is_file() {
                let canonic = file_path.canonicalize()?;
                Utils::check_file_sanity(processor, &canonic)?;
                // Set sandbox after error checking or it will act starngely
                processor.set_sandbox(true);

                // Optionally enable raw mode
                if args.len() >= 2 {
                    raw_include = Utils::is_arg_true(&processor.parse_and_strip(
                        &mut ap,
                        level,
                        "readin",
                        trim!(&args[1]).as_ref(),
                    )?)?;

                    // You don't have to backup pause state because include wouldn't be triggered
                    // at the first place, if paused was true
                    if raw_include {
                        processor.state.flow_control = FlowControl::Escape;
                    }
                }

                if let Some(relay) = processor.state.relay.last() {
                    processor.log_warning(
                        &format!("Read file's content will be relayed to \"{:?}\"", relay),
                        WarningType::Sanity,
                    )?;
                }

                // Create chunk
                let chunk = processor.process_file(&file_path)?;

                // Reset flow control per processing
                if processor.state.flow_control != FlowControl::None {
                    processor.reset_flow_control();
                }
                if raw_include {
                    processor.state.flow_control = FlowControl::None;
                }
                processor.set_sandbox(false);
                processor.state.input_stack.remove(&canonic); // Collect stack
                Ok(chunk)
            } else {
                Err(RadError::InvalidArgument(format!(
                    "readin cannot read non-file \"{}\"",
                    file_path.display()
                )))
            }
        } else {
            Err(RadError::InvalidArgument(
                "readin requires an argument".to_owned(),
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
                trim!(&processor.parse_and_strip(&mut ap, level, "exec", &args[0])?).to_string();
            let args = processor.parse_and_strip(&mut ap, level, "exec", &args[1])?;
            let result = processor
                .execute_macro(level, "exec", &macro_name, &args)?
                .unwrap_or_default();
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

            let expanded_name = &processor.parse_and_strip(&mut ap, level, "spread", &args[0])?;
            let expanded_data = &processor.parse_and_strip(&mut ap, level, "spread", &args[1])?;
            let macro_name = trim!(expanded_name);
            let macro_data = trim!(expanded_data);

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
            let result = processor.parse_and_strip(&mut ap, level, "spread", &result)?;

            // Set custom prompt log to indicate user thatn from macro doesn't support
            // debugging inside macro expansion
            #[cfg(feature = "debug")]
            {
                use crate::debugger::DebugSwitch;
                processor.set_debug(original);
                match processor.get_debug_switch() {
                    DebugSwitch::StepMacro | DebugSwitch::NextMacro => {
                        processor.set_prompt("\"Spread macro\"")
                    }
                    _ => (),
                }
            }

            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "spread requires two arguments".to_owned(),
            ))
        }
    }

    /// Paste given file's content
    ///
    /// Every macros within the file is also expanded
    ///
    /// Include read file's content into a single string and print out.
    /// This enables ergonomic process of macro execution. If you want file
    /// inclusion to happen as bufstream, use read instead.
    ///
    /// # Usage
    ///
    /// $include(path)
    #[cfg(not(feature = "wasm"))]
    fn include(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("include", AuthType::FIN, processor)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        let args = ap.args_to_vec(args, ',', GreedyState::Never);
        ap.set_strip(true);
        if !args.is_empty() {
            let raw_file = &processor.parse_and_strip(&mut ap, level, "include", &args[0])?;
            let mut raw_include = false;
            let file_path = PathBuf::from(trim!(raw_file).as_ref());

            if file_path.is_file() {
                let canonic = file_path.canonicalize()?;

                Utils::check_file_sanity(processor, &canonic)?;
                // Set sandbox after error checking or it will act starngely
                processor.set_sandbox(true);

                // Optionally enable raw mode
                if args.len() >= 2 {
                    raw_include = Utils::is_arg_true(
                        &processor.parse_and_strip(&mut ap, level, "include", &args[1])?,
                    )?;

                    // You don't have to backup pause state because include wouldn't be triggered
                    // at the first place, if paused was true
                    if raw_include {
                        processor.state.flow_control = FlowControl::Escape;
                    }
                }

                let container_type = if level != 1 {
                    ContainerType::Argument
                } else {
                    ContainerType::Expand
                };
                // Create chunk
                let chunk = processor.process_file_as_chunk(&file_path, container_type)?;

                // Reset flow control per processing
                if processor.state.flow_control != FlowControl::None {
                    processor.reset_flow_control();
                }
                if raw_include {
                    processor.state.flow_control = FlowControl::None;
                }
                processor.set_sandbox(false);
                processor.state.input_stack.remove(&canonic); // Collect stack
                Ok(chunk)
            } else {
                let formatted = format!(
                    "File path : \"{}\" doesn't exist or not a file",
                    file_path.display()
                );
                Err(RadError::InvalidArgument(formatted))
            }
        } else {
            Err(RadError::InvalidArgument(
                "Include requires an argument".to_owned(),
            ))
        }
    }

    /// Include but for temporary file
    ///
    /// This reads file's content into memory. Use read macro if streamed write is needed.
    ///
    /// # Usage
    ///
    /// $tempin()
    #[cfg(not(feature = "wasm"))]
    fn temp_include(_: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("tempin", AuthType::FIN, processor)? {
            return Ok(None);
        }
        let file = processor.get_temp_path().display();
        let chunk = Self::include(&file.to_string(), level, processor)?;
        Ok(chunk)
    }

    #[allow(unused_variables)]
    #[cfg(debug_assertions)]
    #[allow(dead_code)]
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
