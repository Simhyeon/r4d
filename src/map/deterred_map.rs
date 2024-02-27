//! Macro collection for deterred macros
use crate::argument::{ArgType, MacroInput};
use crate::common::RadResult;
use crate::consts::ESR;
use crate::extension::{ExtMacroBody, ExtMacroBuilder};
use crate::{Parameter, Processor};
#[cfg(feature = "rustc_hash")]
use rustc_hash::FxHashMap as HashMap;
#[cfg(not(feature = "rustc_hash"))]
use std::collections::HashMap;

use std::iter::FromIterator;

/// Function signature for a deterred macro function
pub(crate) type DFunctionMacroType =
    fn(MacroInput, usize, &mut Processor) -> RadResult<Option<String>>;

/// Collection map for a deterred macro function
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

    /// Create a new instance with default macros
    pub fn new() -> Self {
        Self::from_iter(IntoIterator::into_iter([
            (
                DMacroSign::new(
                    "include",
                    [(ArgType::Path,"a_filename")],
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
                ).optional(Parameter::new(ArgType::Bool, "a_raw_mode"))
            ),
            (
                DMacroSign::new(
                    "incread",
                    [(ArgType::Path,"a_filename")],
                    Self::incread,
                    Some(
                        "Alwasy include a file as \"read\"

- Include works as bufread in first level and chunk read in nested call.
- Use incread when you need to read on first level.

# NOT Deterred

# AUTH : FIN

# Arguments

- a_filename : A file name to read ( trimmed )
- a_raw_mode : Whehter to escape the read. A default is false [boolean] ( trimmed, optional )

$incread|(file_path)
$-()"
                            .to_string(),
                    ),
                ).optional(Parameter::new(ArgType::Bool, "a_raw_mode"))
            ),
            (
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
                ).optional(Parameter::new(ArgType::Bool, "a_raw_mode"))
            ),
            (
                DMacroSign::new(
                    "mapf",
[(ArgType::Text,"a_macro_name^"),(ArgType::Text, "a_file^"),],
                    Self::map_file,
                    Some(
                        "Execute macro on each lines of a file

# Note : mapf macro doesn't expand lines from a file.

# Auth : FIN

# NOT Deterred

# Arguments

- a_macro_name : A macro name to execute ( trimmed ) 
- a_file       : A file to get lines iterator ( trimmed )

# Example

$define(m,a_src=$a_src()+)
$assert(a+b+c,$mapf(m,file_name.txt))"
                            .to_string(),
                    ),
                )
            ),
            (
                DMacroSign::new(
                    "mapfe",
[(ArgType::Text,"a_expr"),(ArgType::Text,"a_macro_name^"),(ArgType::Text, "a_lines"),],
                    Self::map_file_expr,
                    None,
                )
            ),
            (
                DMacroSign::new(
                    "mapn",
[(ArgType::Text,"a_operation^"),(ArgType::Text,"a_source"),],
                    Self::map_number,
                    None,
                )
            ),
            (
                DMacroSign::new(
                    "readto",
[(ArgType::Text,"a_from_file^"),(ArgType::Text, "a_to_file^"),(ArgType::Text, "a_raw_mode?+^"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "readin",
[(ArgType::Text,"a_file?^"),(ArgType::Text, "a_raw_mode^+?"),],
                    DeterredMacroMap::read_in,
                    Some(
                        "Read from a file as \"Bufread\"

# Auth : FIN

# NOT deterred

# Arguments

- a_file : A file to read from ( trimmed )
- a_raw_mode : Whehter to escape the read. A default is false [boolean] ( trimmed, optional )

# Example

$readin(file.txt)"
                            .to_string(),
                    ),
                )
            ),
            (
                DMacroSign::new(
                    "ifenv",
[(ArgType::Text,"a_env_name^"),(ArgType::Text, "a_if_expr"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "ifenvel",
[(ArgType::Text,"a_env_name^"),(ArgType::Text, "a_if_expr"),(ArgType::Text, "a_else_expr"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "append",
[(ArgType::Text,"a_macro_name^"),(ArgType::Text, "a_content"),(ArgType::Text,"a_trailer+"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "anon",
[(ArgType::Text,"a_macro"),],
                    Self::add_anonymous_macro,
                    Some("Create an anonymous macro and return it's name

# Not expanded at all

# Arguments

- a_macro : A macro defintition without name

# Example

$map($anon(a=$a()+),a,b,c)".to_string()),
                )
            ),
            (
                DMacroSign::new(
                    "consume",
                    ESR,
                    Self::consume,
                    Some("Consume streaming

# Arguments are not expanded at all

# Example

$stream(squash)

abc

clo

fgh

$consume()".to_string()),
                )
            ),
            (
                DMacroSign::new(
                    "EB",
                    ESR,
                    DeterredMacroMap::escape_blanks,
                    Some(
"Escape all following blanks until not. This can only be invoked at first level

# NOT deterred

# Example

$EB()".to_string()),
                )
            ),
            (
                DMacroSign::new(
                    "exec",
[(ArgType::Text,"a_macro_name^"),(ArgType::Text,"a_macro_attribute^"),(ArgType::Text,"a_macro_args"),],
                    DeterredMacroMap::execute_macro,
                    Some("Execute a macro with arguments

# NOT deterred

# Arguments

- a_macro_name      : A macro name to exectue ( trimmed )
- a_macro_attribute : A macro name to exectue ( trimmed )
- a_macro_args      : Arguments to be passed to a macro

# Example

$assert($path(a,b,c),$exec(path,,a,b,c))".to_string()),
                )
            ),
            (
                DMacroSign::new(
                    "fassert",
[(ArgType::Text,"a_expr"),],
                    DeterredMacroMap::assert_fail,
                    Some("Assert succeeds when text expansion yields error

# NOT deterred

# Arguments

- a_expr: An expression to audit

# Example

$fassert($eval(Text is not allowd))".to_string()),
                )
            ),
            (
                DMacroSign::new(
                    "forby",
[(ArgType::Text,"a_body"),(ArgType::Text, "a_sep"),(ArgType::Text,"a_text"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "foreach",
[(ArgType::Text,"a_body"),(ArgType::Text, "a_array^"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "forjoin",
[(ArgType::Text,"a_body"),(ArgType::Text, "a_joined_array^"),],
                    DeterredMacroMap::forjoin,
                    None,
                )
            ),
            (
                DMacroSign::new(
                    "forsp",
[(ArgType::Text,"a_body"),(ArgType::Text, "a_words^"),],
                    DeterredMacroMap::for_space,
                    Some(
                        "Iterate around given words.

An iterated value is bound to macro \":\"
 
# Expansion order

1. a_words : Expanded on time
2. a_body : Split array by comma, and expanded by per item.

# Arguments

- a_body  : A body to be pasted as iterated item
- a_words : Words to iterate ( trimmed )

# Example

".to_string(),
                    ),
                )
            ),
            (
                DMacroSign::new(
                    "forline",
[(ArgType::Text,"a_body"),(ArgType::Text,"a_lines"),],
                    DeterredMacroMap::forline,
                    Some(
"Loop around given lines separated by newline chraracter. 

An iterated value is bound to macro \":\"
 
# Expansion order

1. a_lines : Expanded on time
2. a_body  : Split lines by newline, and expanded by per item.

# Arguments

- a_body  : A body to be pasted as iterated item
- a_lines : Lines to iterate

# Example

$assert(a+b+c+,$forline($:()+,a$nl()b$nl()c))".to_string()),
                )
            ),
            (
                DMacroSign::new(
                    "forcol",
[(ArgType::Text,"a_body"),(ArgType::Text,"a_table"),],
                    DeterredMacroMap::forcol,
                    None,
                )
            ),
            (
                DMacroSign::new(
                    "forloop",
[(ArgType::Text,"a_body"),(ArgType::Text,"a_min^"),(ArgType::Text, "a_max^"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "map",
[(ArgType::Text,"a_expr^"),(ArgType::Text,"a_macro_name^"),(ArgType::Text, "a_cont"),],
                    Self::map,
                    None,
                )
            ),
            (
                DMacroSign::new(
                    "mape",
[(ArgType::Text,"a_start_expr"),(ArgType::Text,"a_end_expr"),(ArgType::Text,"a_macro_name^"),(ArgType::Text, "a_source"),],
                    Self::map_expression,
                    None,
                )
            ),
            (
                DMacroSign::new(
                    "mapa",
[(ArgType::Text,"a_macro_name^"),(ArgType::Text, "a_array"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "mapl",
[(ArgType::Text,"a_macro_name^"),(ArgType::Text, "a_lines"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "maple",
[(ArgType::Text,"a_expr"),(ArgType::Text,"a_macro_name^"),(ArgType::Text, "a_lines"),],
                    Self::map_lines_expr,
                    None,
                )
            ),
            (
                DMacroSign::new(
                    "spread",
[(ArgType::Text,"a_macro_name^"),(ArgType::Text, "a_csv_value^"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "stream",
[(ArgType::Text,"a_macro_name^"),],
                    Self::stream,
                    Some("Stream texts to macro invocaiton

# This is technically a wrapper around relay macro

# Arguments

- a_macro_name : A macro target that following texts are relayed to ( trimmed )

# Example

$stream(squash)

abc

clo

fgh

$consume()".to_string()),
                )
            ),
            (
                DMacroSign::new(
                    "streaml",
[(ArgType::Text,"a_macro_name^"),],
                    Self::stream_by_lines,
                    Some("Stream texts to macro invocaiton but by lines

# This is technically a wrapper around relay macro

# Arguments

- a_macro_name : A macro target that following texts are relayed to ( trimmed )

# Example

$streaml(trim)

abc

clo

fgh

$consume()".to_string()),
                )
            ),
            (
                DMacroSign::new(
                    "if",
[(ArgType::Text,"a_cond?^"),(ArgType::Text, "a_if_expr"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "ifelse",
[(ArgType::Text,"a_cond?^"),(ArgType::Text, "a_if_expr"),(ArgType::Text, "a_else_expr"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "ifdef",
[(ArgType::Text,"a_macro_name^"),(ArgType::Text, "a_if_expr"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "ifdefel",
[(ArgType::Text,"a_macro_name^"),(ArgType::Text, "a_if_expr"),(ArgType::Text, "a_else_expr"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "logm",
[(ArgType::Text,"a_macro_name^"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "que",
[(ArgType::Text,"a_expr"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "queif",
[(ArgType::Text,"a_bool?^"),(ArgType::Text, "a_content"),],
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
                )
            ),
            (
                DMacroSign::new(
                    "expand",
[(ArgType::Text,"a_literal_expr"),],
                    DeterredMacroMap::expand_expression,
                    Some("Expand expression 

# Note

- This will strip a given expression and then expand it.

# Arguments

- a_expr : An expression to expand

# Example

$expand(\\*1,2,3*\\)".to_string()),
                )
            ),
        ]))
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
    pub(crate) fn get_signature(&self, name: &str) -> Option<&DMacroSign> {
        self.macros.get(name)
    }

    /// Check if map contains the name
    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// Undefine a deterred macro
    pub fn undefine(&mut self, name: &str) {
        self.macros.remove(name);
    }

    /// Rename a deterred macro
    pub fn rename(&mut self, name: &str, target: &str) -> bool {
        if let Some(func) = self.macros.remove(name) {
            self.macros.insert(target.to_owned(), func);
            return true;
        }
        false
    }

    /// Add new extension macro as deterred macro
    pub fn new_ext_macro(&mut self, ext: ExtMacroBuilder) {
        // TODO TT
        // if let Some(ExtMacroBody::Deterred(mac_ref)) = ext.macro_body {
        //     let sign = DMacroSign::new(&ext.macro_name, &ext.args, mac_ref, ext.macro_desc);
        //     self.macros.insert(ext.macro_name, sign);
        // }
    }
}

#[derive(Clone)]
pub(crate) struct DMacroSign {
    name: String,
    params: Vec<Parameter>,
    optional: Option<Parameter>,
    pub logic: DFunctionMacroType,
    #[allow(dead_code)]
    desc: Option<String>,
}

impl DMacroSign {
    pub fn new(
        name: &str,
        params: impl IntoIterator<Item = (ArgType, impl AsRef<str>)>,
        logic: DFunctionMacroType,
        desc: Option<String>,
    ) -> Self {
        let params = params
            .into_iter()
            .map(|(t, s)| Parameter {
                name: s.as_ref().to_string(),
                arg_type: t,
            })
            .collect::<Vec<Parameter>>();
        Self {
            name: name.to_owned(),
            params,
            optional: None,
            logic,
            desc,
        }
    }

    pub fn optional(mut self, param: Parameter) -> Self {
        self.optional.replace(param);
        self
    }
}

// ------ REFACTOR

impl std::fmt::Display for DMacroSign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut inner = self.params.iter().fold(String::new(), |acc, param| {
            acc + &param.arg_type.to_string() + ","
        });
        // This removes last "," character
        inner.pop();
        let basic_usage = format!("${}({}", self.name, inner); // Without ending brace
        let ret = write!(f, "{})", basic_usage);
        let sep = if inner.is_empty() { "" } else { ", " };
        if let Some(op) = self.optional.as_ref() {
            write!(f, "  ||  {}{}{}?)", basic_usage, sep, op.arg_type)
        } else {
            ret
        }
    }
}

impl From<&DMacroSign> for crate::sigmap::MacroSignature {
    fn from(ms: &DMacroSign) -> Self {
        Self {
            variant: crate::sigmap::MacroVariant::Deterred,
            name: ms.name.to_owned(),
            params: ms.params.to_owned(),
            optional: ms.optional.clone(),
            expr: ms.to_string(),
            desc: ms.desc.clone(),
        }
    }
}

impl FromIterator<DMacroSign> for DeterredMacroMap {
    fn from_iter<T: IntoIterator<Item = DMacroSign>>(iter: T) -> Self {
        let mut m = HashMap::new();
        for sign in iter {
            m.insert(sign.name.clone(), sign);
        }
        Self { macros: m }
    }
}
