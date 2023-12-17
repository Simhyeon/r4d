//! Macro collection for deterred macros
use crate::common::RadResult;
use crate::consts::ESR;
use crate::extension::{ExtMacroBody, ExtMacroBuilder};
use crate::Processor;
use std::collections::HashMap;
use std::iter::FromIterator;

/// Functino signature for a deterred macro function
pub(crate) type DFunctionMacroType = fn(&str, usize, &mut Processor) -> RadResult<Option<String>>;

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
        // THis needs to be mutable because of wasm configuration
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
                "anon".to_owned(),
                DMacroSign::new(
                    "anon",
                    ["a_macro"],
                    Self::add_anonymous_macro,
                    Some("Create an anonymous macro and return it's name

# Not expanded at all

# Arguments

- a_macro : A macro defintition without name

# Example

$map($anon(a=$a()+),a,b,c)".to_string()),
                ),
            ),
            (
                "consume".to_owned(),
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
                "stream".to_owned(),
                DMacroSign::new(
                    "stream",
                    ["a_macro_name^"],
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
                ),
            ),
            (
                "streaml".to_owned(),
                DMacroSign::new(
                    "streaml",
                    ["a_macro_name^"],
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
                "expand".to_owned(),
                DMacroSign::new(
                    "expand",
                    ["a_literal_expr"],
                    DeterredMacroMap::expand_expression,
                    Some("Expand expression 

# Note

- This will strip a given expression and then expand it.

# Arguments

- a_expr : An expression to expand

# Example

$expand(\\*1,2,3*\\)".to_string()),
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
                "incread".to_owned(),
                DMacroSign::new(
                    "incread",
                    ["a_filename^", "a_raw_mode^+?"],
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
                    ["a_macro_name^", "a_file^"],
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

$readin(file.txt)"
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
        if let Some(ExtMacroBody::Deterred(mac_ref)) = ext.macro_body {
            let sign = DMacroSign::new(&ext.macro_name, &ext.args, mac_ref, ext.macro_desc);
            self.macros.insert(ext.macro_name, sign);
        }
    }
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
