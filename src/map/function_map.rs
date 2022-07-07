//! # Function macro module
//!
//! Function macro module includes struct and methods related to function macros which are technically function
//! pointers.

use crate::auth::AuthType;
use crate::consts::{ESR, LOREM, LOREM_SOURCE, LOREM_WIDTH, MAIN_CALLER};
use crate::error::RadError;
use crate::formatter::Formatter;
#[cfg(feature = "hook")]
use crate::hookmap::HookType;
use crate::logger::WarningType;
use crate::models::MacroType;
use crate::models::{
    ErrorBehaviour, ExtMacroBody, ExtMacroBuilder, FlowControl, ProcessInput, RadResult,
    RelayTarget,
};
use crate::processor::Processor;
use crate::trim;
use crate::utils::Utils;
use crate::{ArgParser, GreedyState};
#[cfg(feature = "cindex")]
use cindex::OutOption;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Write as _;
#[cfg(not(feature = "wasm"))]
use std::fs::OpenOptions;
use std::io::BufRead;
#[cfg(not(feature = "wasm"))]
use std::io::Write;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(feature = "hook")]
use std::str::FromStr;

lazy_static! {
    static ref CLRF_MATCH: Regex = Regex::new(r#"\r\n"#).unwrap();
    static ref CHOMP_MATCH: Regex = Regex::new(r#"\n\s*\n"#).expect("Failed to crate chomp regex");
    // Thanks stack overflow! SRC : https://stackoverflow.com/questions/12643009/regular-expression-for-floating-point-numbers
    static ref NUM_MATCH: Regex = Regex::new(r#"[+-]?([\d]*[.])?\d+"#).expect("Failed to crate number regex");
}

pub(crate) type FunctionMacroType = fn(&str, &mut Processor) -> RadResult<Option<String>>;

#[derive(Clone)]
pub(crate) struct FunctionMacroMap {
    pub(crate) macros: HashMap<String, FMacroSign>,
}

impl FunctionMacroMap {
    /// Creates empty map
    pub fn empty() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    /// Creates new function macro hashmap
    ///
    /// Optional macros are included only when a feature is enabled
    pub fn new() -> Self {
        // Create hashmap of functions
        #[allow(unused_mut)]
        let mut map = HashMap::from_iter(IntoIterator::into_iter([
            (
                "-".to_owned(),
                FMacroSign::new(
                    "-",
                    ["a_pipe_name?^"],
                    Self::get_pipe,
                    Some(
                        "Get piped value. This truncates original value by default if not configured

# Arguments

- a_pipe_name : Name of pipe ( trimmed, optional )

# Exmaple

$eval|(1+2)
$assert(3,$-())
$nassert(3,$-())
$pipeto(num,5)
$assert(5,$-(num))".to_string(),
                    ),
                ),
            ),
            (
                "append".to_owned(),
                FMacroSign::new(
                    "append",
                    ["a_macro_name^", "a_content"],
                    Self::append,
                    Some(
                        "Append contents to a macro. If macro doesn't exist, yields error

# Arguments
- a_macro_name : a macro name to append to ( trimmed )
- a_content    : contents to be added

# Example

$define(container=Before)
$append(container,$space()After)
$assert($container(),Before After)".to_string(),
                    ),
                ),
            ),
            (
                "arr".to_owned(),
                FMacroSign::new(
                    "arr",
                    ["a_values","a_sep+", "a_retain_expr+"],
                    Self::array,
                    Some("Convert spaced array into a comma-ed array. You can give surplus arguments to configure behaviour.

# Arguments

- a_values      : space separated values
- a_sep         : New separator (optional)
- a_retain_expr : Regex to match from values (optional)

# Example

$static(script,$arr($syscmd(ls),$nl(),\\.sh))
$assert($script(),auto.sh)".to_string()),
                ),
            ),
            (
                "assert".to_owned(),
                FMacroSign::new(
                    "assert",
                    ["a_lvalue", "a_rvalue"],
                    Self::assert,
                    Some("Compare lvalue and rvalue, panics with two values are not equal

# Arguments

- a_lvalue : Left  value to compare
- a_rvalue : Right value to compare

# Example

% Succeed
$assert(1,1)
% Fails
$assert(a,b)".to_string()),
                ),
            ),
            (
                "counter".to_owned(),
                FMacroSign::new(
                    "counter",
                    ["a_macro_name^","a_counter_type^+"],
                    Self::change_counter,
                    Some("Increae/decrease counter macro. Counter's value should be a number and can be negative. 

# Arguments

- a_macro_name   : Macro name to use as counter ( trimmed )
- a_counter_type : Counter opration type. Dfault is plus [ \"plus\", \"minus\" ]

# Example

$define(ct=0)
$counter(ct)
$counter(ct)
$counter(ct,minus)
$assert($ct(),3)

$counter(ct,minus)
$assert($ct(),2)".to_string()),
                ),
            ),
            (
                "ceil".to_owned(),
                FMacroSign::new(
                    "ceil",
                    ["a_number^"],
                    Self::get_ceiling,
                    Some("Get ceiling of a number

# Return : Signed integer

# Arguments

- a_number : Number to get a ceiling from [float] ( trimmed )

# Example

$assert($ceil(0.9),1)
$assert($ceil(3.1),4)".to_string()),
                ),
            ),
            (
                "chomp".to_owned(),
                FMacroSign::new(
                    "chomp",
                    ["a_content"],
                    Self::chomp,
                    Some("Remove duplicate newlines from content

# Arguments

- a_content: Contents to chomp

# Example

$staticr(lines,Upper


Down)
$assert($countl($lines()),4)
$assert($countl($chomp($lines())),3)".to_string()),
                ),
            ),
            (
                "clear".to_owned(),
                FMacroSign::new(
                    "clear",
                    ESR,
                    Self::clear,
                    Some("Clear volatile macros. This macro is intended to be used when hygiene mode is enabled and user wants to clear volatile on right away without waiting.

# Example

$clear()".to_string()),
                ),
            ),
            (
                "comp".to_owned(),
                FMacroSign::new(
                    "comp",
                    ["a_content"],
                    Self::compress,
                    Some("Apply both trim and chomp, or say compress content

# Arguments

- a_content : Content to compress

# Example
$staticr(lines,
    upper
    

    down
)
$assert($countl($lines()),5)
$assert($countl($comp($lines())),3)".to_string()),
                ),
            ),
            (
                "count".to_owned(),
                FMacroSign::new(
                    "count",
                    ["a_array"],
                    Self::count,
                    Some("Get counts of an array

# Return : Unsigned integer

# Arguments

- a_array : An array to get counts from

# Example

$assert($count(a,b,c),3)".to_string()),
                ),
            ),
            (
                "countw".to_owned(),
                FMacroSign::new(
                    "countw",
                    ["a_array"],
                    Self::count_word,
                    Some("Get count of words

# Return : Unsigned integer

# Arguments

- a_array : An array to get word counts from

# Example

$assert($countw(hello world),2)".to_string()),
                ),
            ),
            (
                "countl".to_owned(),
                FMacroSign::new(
                    "countl",
                    ["a_lines"],
                    Self::count_lines,
                    Some("Get counts of lines. This ignores empty line

# Return : Unsigned integer

# Arguments

- a_lines : Lines to get counts from

# Example

$assert(3,$countl(1
2
3))".to_string()),
                ),
            ),
            (
                "dnl".to_owned(),
                FMacroSign::new(
                    "dnl",
                    ESR,
                    Self::deny_newline,
                    Some("Deny next newline. This technically squashes following two consequent line_ending into a single one

# Example

$assert(a$nl()b,a$dnl()

b)".to_string()),
                ),
            ),
            (
                "declare".to_owned(),
                FMacroSign::new(
                    "declare",
                    ["a_macro_names"],
                    Self::declare,
                    Some("Declare multiple variables separated by commas

# Arguments

- a_macro_names: Macro names array

# Example

$declare(first,second)
$assert($first(),$empty())".to_string()),
                ),
            ),
            (
                "docu".to_owned(),
                FMacroSign::new(
                    "docu",
                    ["a_macro_name", "a_doc"],
                    Self::document,
                    Some("Append documents(description) to a macro. You cannot directly retreive documentation from macros but by --man flag.

# Arguments

- a_macro_name : Macro to append documentation
- a_doc        : Document to append

# Example

$define(test=)
$docu(test,This is test macro)".to_string()),
                ),
            ),
            (
                "empty".to_owned(),
                FMacroSign::new(
                    "empty",
                    ESR,
                    Self::print_empty,
                    Some("Print nothing. Used for semantic formatting.

# Example

$assert( ,$empty())".to_string()),
                ),
            ),
            (
                "enl".to_owned(),
                FMacroSign::new(
                    "enl",
                    ESR,
                    Self::escape_newline,
                    Some("Escape following new line

# Example

$assert(ab,a$enl()
b)".to_string()),
                ),
            ),
            (
                "escape".to_owned(),
                FMacroSign::new(
                    "escape",
                    ESR,
                    Self::escape,
                    Some("Escape processing from the invocation.

- NOTE : This flow control only sustains for the processing.

# Example

$escape()".to_string()),
                ),
            ),
            (
                "exit".to_owned(),
                FMacroSign::new(
                    "exit",
                    ESR,
                    Self::exit,
                    Some("Exit processing from invocation

- NOTE : This flow control only sustains for the processing

# Example

$exit()".to_string()),
                ),
            ),
            (
                "input".to_owned(),
                FMacroSign::new(
                    "input",
                    ["a_absolute?^"],
                    Self::print_current_input,
                    Some("Print current file input.

# Return : Path

# Arguments

- a_absolute : Whether to print input path as absolute [boolean] ( trimmed )

# Example

$assert($input(),test)
$assert($input(true),/home/user/dir/test)".to_string()),
                ),
            ),
            (
                "find".to_owned(),
                FMacroSign::new(
                    "find",
                    ["a_expr", "a_source"],
                    Self::find_occurence,
                    Some("Find an occurrence of expression from source. This return boolean value

# Return : Boolean

# Arguments

- a_expr   : Expression to match
- a_source : Source to match for

# Example

$assert(true,$find(^abc,abcde))".to_string()),
                ),
            ),
            (
                "findm".to_owned(),
                FMacroSign::new(
                    "findm",
                    ["a_expr", "a_source"],
                    Self::find_multiple_occurence,
                    Some("Find occurrences of expression from source. This return 0 if there are no occurrences

# Return : Unsigned integer

# Arguments

- a_expr   : Expression to match
- a_source : Source to match for

# Example

$assert(2,$findm(o,hello world))".to_string()),
                ),
            ),
            (
                "floor".to_owned(),
                FMacroSign::new(
                    "floor",
                    ["a_number^"],
                    Self::get_floor,
                    Some("Get floor integer from a given number

# Return : Signed integer

# Arguments

- a_number : Number to get a floor from [float] ( trimmed )

# Example

$assert($floor( 1.9),1)
$assert($floor(-3.1),-4)".to_string()),
                ),
            ),
            (
                "fold".to_owned(),
                FMacroSign::new(
                    "fold",
                    ["a_array"],
                    Self::fold,
                    Some("Fold an array into a single value

# Arguments 

- a_array : An array to fold

# Example

$assert(abc,$fold(a,b,c))".to_string()),
                ),
            ),
            (
                "foldl".to_owned(),
                FMacroSign::new(
                    "foldl",
                    ["a_lines"],
                    Self::fold_line,
                    Some("Fold lines into a single value

# Arguments 

- a_lines : Lines to fold

# Example

$assert(abc,$foldl(a
b
c))".to_string()),
                ),
            ),
            (
                "grep".to_owned(),
                FMacroSign::new(
                    "grep",
                    ["a_expr", "a_content"],
                    Self::grep,
                    Some("Grep text from given content. This returns all lines that matches given expression

# Arguments

- a_expr    : Regex expression to match
- a_content : Content to get matches from

# Example

$assert(2,$countl($grep(Cargo,$syscmd(ls))))".to_string()),
                ),
            ),
            (
                "halt".to_owned(),
                FMacroSign::new(
                    "halt",
                    ESR,
                    Self::halt_relay,
                    Some("Halt relaying

- NOTE : Halt is automatically queued by default. Feed boolean argument to configure this behaviour
- $halt(false) == $halt()

# Example

$define(cont=)
$relay(macro,cont)
12345
$halt()
$assert(12345,$cont^())".to_string()),
                ),
            ),
            (
                "head".to_owned(),
                FMacroSign::new(
                    "head",
                    ["a_count^", "a_content"],
                    Self::head,
                    Some("Crop head texts from given content

# Arguments

- a_count   : Amount of characters to crop [unsigned integer] ( trimmed )
- a_content : Text to crop from

# Example

$assert(Hello~,$head( 6 ,Hello~ World))".to_string()),
                ),
            ),
            (
                "headl".to_owned(),
                FMacroSign::new(
                    "headl",
                    ["a_count^", "a_lines"],
                    Self::head_line,
                    Some("Crop head texts but as lines from given content

# Arguments

- a_count   : Amount of lines to crop [unsigned integer] ( trimmed )
- a_lines   : Lines to crop from

# Example

$assert(2,$countl($headl( 2 ,a
b
c)))".to_string()),
                ),
            ),
            (
                "hygiene".to_owned(),
                FMacroSign::new(
                    "hygiene",
                    ["a_hygiene?"],
                    Self::toggle_hygiene,
                    Some("Toggle hygiene mode. This enables macro hygiene.

- On \"macro\" hygiene, every newly defined runtime macros are all cleared after first level macro invocation.

# Arguments

- a_hygiene : Whether to enable a macro hygiene mode [boolean] (trimmed)

# Example

$hygiene(true)
$define(test=Test)
% test macro is cleared and doesn't exsit
$fassert($test())".to_string()),
                ),
            ),
            (
                "indent".to_owned(),
                FMacroSign::new(
                    "indent",
                    ["a_indenter", "a_lines"],
                    Self::indent_lines,
                    Some("Indent lines with indenter

# Arguments

- a_indenter : Pattern to put before lines
- a_lines    : Liens to prepend indenter

# Example

$assert(
# First
# Second
# Third,
$indent(# ,First
Second
Third))".to_string()),
                ),
            ),
            (
                "index".to_owned(),
                FMacroSign::new(
                    "index",
                    ["a_index^", "a_array"],
                    Self::index_array,
                    Some("Get an indexed value from an array

# Arguments

- a_index : Index to get [Unsigned integer] ( trimmed )
- a_array : Data source to index from

# Example

$assert(ef,$index(2,ab,cd,ef))".to_string()),
                ),
            ),
            (
                "import".to_owned(),
                FMacroSign::new(
                    "import",
                    ["a_file^"],
                    Self::import_frozen_file,
                    Some("Import a frozen file at runtime

- Import always include the macros as non-volatile form, thus never cleared unless accessed as library

# Arguments

- a_file: File name to import from [path] (trimmed) 

# Example

$import(def.r4f)".to_string()),
                ),
            ),
            (
                "join".to_owned(),
                FMacroSign::new(
                    "join",
                    ["a_sep","a_array"],
                    Self::join,
                    Some("Join an array into a single chunk

# Arguments

- a_sep   : Separator used for joining
- a_array : Source to array to join

# Example

$assert(a-b-c,$join(-,a,b,c))".to_string()),
                ),
            ),
            (
                "joinl".to_owned(),
                FMacroSign::new(
                    "joinl",
                    ["a_sep","a_lines"],
                    Self::joinl,
                    Some("Join text lines into a single line

# Arguments

- a_sep   : Separator used for joining
- a_lines : Source lines to join

# Example

$assert(a-b-c,$joinl(-,a
b
c))".to_string()),
                ),
            ),
            (
                "len".to_owned(),
                FMacroSign::new(
                    "len",
                    ["a_string"],
                    Self::len,
                    Some("Get a length of texts. This count utf8 characters not ascii.

# Return : Unsigned integer

# Arguments

- a_string : Text to get length from

# Example

$assert($len(가나다),$len(ABC))".to_string()),
                ),
            ),
            (
                "let".to_owned(),
                FMacroSign::new(
                    "let",
                    ["a_macro_name^", "a_value^"],
                    Self::bind_to_local,
                    Some("Bind a local macro. Every local macro gets removed after first level macro expansion ends.

# Arguments

- a_macro_name : Binding name to make ( trimmed )
- a_value      : A value to bind to which is not trimmed ( trimmed )

# Example

$define(letr_test=
    $let(lc,
        --Bound Value--
    )
    $assert(1,$countl($lc()))
)
$letr_test()
% Cannot access local macro outside the scope
$fassert($lc())".to_string()),
                ),
            ),
            (
                "letr".to_owned(),
                FMacroSign::new(
                    "letr",
                    ["a_macro_name^", "a_value"],
                    Self::bind_to_local_raw,
                    Some("Bind a local macro with raw value. Every local macro gets removed after first level macro expansion ends.

# Arguments

- a_macro_name : Binding name to make ( trimmed )
- a_value      : A value to bind to which is not trimmed

# Example

$define(letr_test=
    $letr(lc,
        --Bound Value--
    )
    $assert(3,$countl($lc()))
)
$letr_test()
% Cannot access local macro outside the scope
$fassert($lc())".to_string()),
                ),
            ),
            (
                "lipsum".to_owned(),
                FMacroSign::new(
                    "lipsum",
                    ["a_word_count^"],
                    Self::lipsum_words,
                    Some("Create a placeholder text. The order of placeholder is always same.

# Arguments

- a_word_count : Word counts of placeholder ( trimmed )

# Example

$assert(Lorem ipsum dolor,$lipsum(3))".to_string()),
                ),
            ),
            (
                "log".to_owned(),
                FMacroSign::new(
                    "log",
                    ["a_msg"],
                    Self::log_message,
                    Some("Log a message to console

# Arguments

- a_msg : Message to log to console

# Example

$log($value_i_want_to_check^())".to_string()),
                ),
            ),
            (
                "lower".to_owned(),
                FMacroSign::new(
                    "lower",
                    ["a_text"],
                    Self::lower,
                    Some("Get lowercase english texts

# Arguments

- a_text: Text to transform

# Example

$assert(abcde,$lower(AbCdE))".to_string()),
                ),
            ),
            (
                "max".to_owned(),
                FMacroSign::new(
                    "max",
                    ["a_array"],
                    Self::get_max,
                    Some("Get max value from a given array
# Arguments

- a_array : Array to get the highest order from

# Example

$assert(eIsBigger,$max(aIsSmall,cIsMiddle,eIsBigger))
$assert(5,$max(1,2,3,4,5))".to_string()),
                ),
            ),
            (
                "min".to_owned(),
                FMacroSign::new(
                    "min",
                    ["a_array"],
                    Self::get_min,
                    Some("Get min value from a given array

 # Arguments

- a_array : Array to get the lowest order from

# Example

$assert(aIsSmall,$min(aIsSmall,cIsMiddle,eIsBigger))
$assert(1,$min(1,2,3,4,5))".to_string()),
                ),
            ),
            (
                "name".to_owned(),
                FMacroSign::new(
                    "name",
                    ["a_path"],
                    Self::get_name,
                    Some("Get a name from a given path including an extension

# Return : path

# Arguments

- a_path : Path to get a name from

# Example

$assert(auto.sh,$name(/path/to/file/auto.sh))".to_string()),
                ),
            ),
            (
                "nassert".to_owned(),
                FMacroSign::new(
                    "nassert",
                    ["a_lvalue", "a_rvalue"],
                    Self::assert_ne,
                    Some("Compare left and right values. Panics if lvalue is \"equal\" to rvalue

# Arguments

- a_lvalue : Left  value
- a_rvalue : Right value

# Example

$nassert(1,2)".to_string()),
                ),
            ),
            (
                "not".to_owned(),
                FMacroSign::new(
                    "not",
                    ["a_boolean?^"],
                    Self::not,
                    Some("Return a negated value of a given boolean. Yield error when given value is not a boolean

# Return : boolean

# Arguments

- a_boolean : Boolean value to negate [boolean] ( trimmed )

# Example

$assert(false,$not(true))
$assert(true,$not(false))
$assert(false,$not(1))
$assert(true,$not(0))".to_string()),
                ),
            ),
            (
                "num".to_owned(),
                FMacroSign::new(
                    "num",
                    ["a_text"],
                    Self::get_number,
                    Some("Extract a number part from given text. If there are multiple numbers, only extract the first
# Arguments

- a_text : Text to extract number from

# Example

$assert(34,$num(34sec))
$assert(30,$num(30k/h for 3 hours))".to_string()),
                ),
            ),
            (
                "nl".to_owned(),
                FMacroSign::new(
                    "nl",
                    ["a_amount+^"],
                    Self::newline,
                    Some("A platform specific newline. It's behaviour can be configured.

# Arguments

- a_amount : Amount of newlines [Unsigned integer] ( trimmed )

# Example

% This may not hold true if a newline is configured by a user
$assert($nl(),
)".to_string()),
                ),
            ),
            (
                "notat".to_owned(),
                FMacroSign::new(
                    "notat",
                    ["a_number^", "a_notation^"],
                    Self::change_notation,
                    Some("Chagne the notaion of a number

# Arguments

- a_number   : A nuber to change notation
- a_notation : A type of notation [\"bin\",\"oct\",\"hex\"] ( trimmed )

# Example                        

$assert(10111,$notat(23,bin))
$assert(27,$notat(23,oct))
$assert(17,$notat(23,hex))".to_string()),
                ),
            ),
            (
                "panic".to_owned(),
                FMacroSign::new(
                    "panic",
                    ["a_msg"],
                    Self::manual_panic,
                    Some("Panics manually with message

# Arguments

- a_msg : A message to print as error

# Example

$panic(This should not be reached)".to_string()),
                ),
            ),
            (
                "parent".to_owned(),
                FMacroSign::new(
                    "parent",
                    ["a_path"],
                    Self::get_parent,
                    Some("Get a parent from a given path. 

- NOTE : This yields error if path is root and will return empty value, not a none value if path is a single node.

# Return : path

# Arguments

- a_path : A Path to extract parent from

# Example 

$fassert($parent(/))
$assert($empty(),$parent(node))
$assert(/first/second,$parent(/first/second/last.txt))".to_string()),
                ),
            ),
            (
                "path".to_owned(),
                FMacroSign::new(
                    "path",
                    ["a_array^"],
                    Self::merge_path,
                    Some("Merge given paths

- This respects a platform path separator
- Path with colliding separator cannot be merged. 
    e.g) a/ + /b cannot be merged

# Return : path

# Arguments

- a_array : Path array to merge

# Example

$assert(a/b,$path(a,b))
$assert(/a/b,$path(/a,b))
$assert(a/b,$path(a/,b))".to_string()),
                ),
            ),
            (
                "pause".to_owned(),
                FMacroSign::new(
                    "pause",
                    ["a_pause?^"],
                    Self::pause,
                    Some("Pause a macro expansion from invocation. Paused processor will only expand $pause(false)

- NOTE : Pause is not a flow control but a processor state, thus the state will sustain for the whole processing.

# Arguments

- a_pause : Whether to pause or not [boolean] ( trimmed )

# Example

$counter(i)
$pause(true)
$counter(i)
$pause(false)

$nassert(2,$i())
$assert(1,$i())".to_string()),
                ),
            ),
            (
                "pipe".to_owned(),
                FMacroSign::new(
                    "pipe",
                    ["a_value"],
                    Self::pipe,
                    Some("Pipe a given value into an unnamed pipe

# Arguments 

- a_value : A value to pipe

# Example

$pipe(Text)
$assert(Text,$-())".to_string()),
                ),
            ),
            (
                "pipeto".to_owned(),
                FMacroSign::new(
                    "pipe",
                    ["a_pipe_name^", "a_value"],
                    Self::pipe_to,
                    Some("Pipe a given value to a named pipe

# Arguments

- a_pipe_name : Name of pipe container ( trimmed )
- a_value     : Value to pipe

# Example

$pipeto(yum,YUM)
$assert($-(yum),YUM)".to_string()),
                ),
            ),
            (
                "prec".to_owned(),
                FMacroSign::new(
                    "prec",
                    ["a_number^", "a_precision^"],
                    Self::prec,
                    Some("Convert a float number with a given precision

# Return : Float

# Arguments

- a_number    : Number to process ( trimmed )
- a_precision : Precision to apply to number ( trimmed )

# Example

$assert(0.30,$prec($eval(0.1 + 0.2),2))".to_string()),
                ),
            ),
            (
                "relay".to_owned(),
                FMacroSign::new(
                    "relay",
                    ["a_target_type^", "a_target^+"],
                    Self::relay,
                    Some("Start relaying to target which redirects all following texts to relay target.

# Auth : FOUT required for relay target \"file\" and \"temp\"

# Arguments

- a_target_type : A type of a relay target [\"macro\",\"file\", \"temp\"] (trimmed)
- a_target      : Name of target. Ignored in temp type

# Example

$relay(file,out.md)$halt()
$relay(macro,container)$halt()
$relay(temp)$halt()".to_string()),
                ),
            ),
            (
                "rev".to_owned(),
                FMacroSign::new(
                    "rev",
                    ["a_array"],
                    Self::reverse_array,
                    Some("Reverse an order of an array

# Arguments 

- a_array : Array to reverse

# Example

$assert(\\*3,2,1*\\,$rev(1,2,3))".to_string()),
                ),
            ),
            (
                "regex".to_owned(),
                FMacroSign::new(
                    "regex",
                    ["a_expr", "a_sub", "a_source"],
                    Self::regex_sub,
                    Some("Apply regular expression substitution to a source

# Arguments

- a_expr   : Regex expression to match
- a_sub    : Text to substitute as
- a_source : Sourec text to operate on

# Example

$assert(Hello Rust,$regex(World,Rust,Hello World))".to_string()),
                ),
            ),
            (
                "regexpr".to_owned(),
                FMacroSign::new(
                    "regexpr",
                    ["a_name", "a_expr"],
                    Self::register_expression,
                    Some("Register an regular expression

- NOTE : Registered name will not be able to matches directly
- Every regex operation creates regex cache, while registered expression will not be cached but saved permanently. Unregistered cache will be cleared if certain capacity reaches.

# Arguments

- a_name : Name of the regex expression. This is not trimmed
- a_epxr : Expression to bind to

# Example

$regexpr(greeting,Hello World)
$assert(true,$find(greeting,Hello World))
$assert(false,$find(greeting,greetings from world))".to_string()),
                ),
            ),
            (
                "rename".to_owned(),
                FMacroSign::new(
                    "rename",
                    ["a_macro_name^", "a_new_name^"],
                    Self::rename_call,
                    Some("Rename a macro with a new name

# Arguments

- a_macro_name : Macro to change name ( trimmed )
- a_new_name   : New macro name to apply ( trimmed )

# Example

$define(test=Test)
$rename(test,demo)
$assert($demo(),Test)".to_string()),
                ),
            ),
            (
                "repeat".to_owned(),
                FMacroSign::new(
                    "repeat",
                    ["a_count^", "a_source"],
                    Self::repeat,
                    Some("Repeat a given source by given counts

# Arguments

- a_count  : Counts of repetition [Unsigned integer] ( trimmed )
- a_source : Source text to repeat

# Example

$assert(R4d
R4d
R4d,$repeat^(3,R4d$nl()))".to_string()),
                ),
            ),
            (
                "repl".to_owned(),
                FMacroSign::new(
                    "repl",
                    ["a_macro_name^", "a_new_value"],
                    Self::replace,
                    Some("Replace a macro's contents with new values

# Arguments

- a_macro_name : Macro name to replace
- a_new_value  : New value to put

# Example

$define(demo=Demo)
$assert(Demo,$demo())
$repl(demo,DOMO)
$assert(DOMO,$demo())".to_string()),
                ),
            ),
            (
                "sep".to_owned(),
                FMacroSign::new(
                    "sep",
                    ["a_sep", "a_array"],
                    Self::separate_array,
                    Some("Separate an array with seperator

# Arguments

- a_sep   : Separator to display
- a_array : Array source to separate

# Example

$assert(3,$countl($sep($nl(),a,b,c)))".to_string()),
                ),
            ),
            (
                "source".to_owned(),
                FMacroSign::new(
                    "source",
                    ["a_file^"],
                    Self::source_static_file,
                    Some("Source an env file. Sourced file is eagerly expanded (As if it was static defined)

Syntax of source-able file is same with .env file

e.g)
demo=DEMO
number=$eval(1+2)

# Arguments

- a_file : File to source ( trimmed )

# Example

$source(def.env)".to_string()),
                ),
            ),
            (
                "sort".to_owned(),
                FMacroSign::new(
                    "sort",
                    ["a_sort_type^","a_array"],
                    Self::sort_array,
                    Some("Sort an array

# Arguments

- a_sort_type : Sor type [\"asec\",\"desc\"] (trimmed)
- a_array     : Array to sort

# Example

$assert(\\*0,1,3,4,6,7,9*\\,$enl()
$sort(asec,3,6,7,4,1,9,0))".to_string()),
                ),
            ),
            (
                "sortl".to_owned(),
                FMacroSign::new(
                    "sortl",
                    ["a_sort_type^","a_lines"],
                    Self::sort_lines,
                    Some("Sort lines

# Arguments

- a_sort_type : Sor type [\"asec\",\"desc\"] (trimmed)
- a_lines     : Lines to sort

# Example

$assert(f$nl()e$nl()d$nl()c,$sortl(desc,f
e
d
c))".to_string()),
                ),
            ),
            (
                "space".to_owned(),
                FMacroSign::new(
                    "space",
                    ["a_amount?^"],
                    Self::space,
                    Some("Print spaces

# Arguments

- a_amount : Amount of spaces [Unsigned integer] ( trimmed )

# Example

$assert(    ,$space(4))".to_string()),
                ),
            ),
            (
                "static".to_owned(),
                FMacroSign::new(
                    "static",
                    ["a_macro_name^", "a_expr^"],
                    Self::define_static,
                    Some("Create a static macro. Static macro is eagerly expanded unlike define

# Arguments

- a_macro_name : Macro to create ( trimmed )
- a_expr       : Expression to bind to ( trimmed )

# Example

$define(ct=0)
$define(ddf=$ct())
$static(stt,$ct())
$counter(ct)
$counter(ct)
$assert(2,$ddf())
$assert(0,$stt())".to_string()),
                ),
            ),
            (
                "staticr".to_owned(),
                FMacroSign::new(
                    "staticr",
                    ["a_macro_name^", "a_value"],
                    Self::define_static_raw,
                    Some("Create a static macro with raw value. Static macro is eagerly expanded unlike define

# Arguments

- a_macro_name : Macro to create ( trimmed )
- a_expr       : Expression to bind to which is not trimmed

# Example

$define(ct=0)
$define(ddf=$ct())
$staticr(stt,$ct() )
$counter(ct)
$counter(ct)
$assert(2,$ddf())
$assert(0 ,$stt())".to_string()),
                ),
            ),
            (
                "sub".to_owned(),
                FMacroSign::new(
                    "sub",
                    ["a_start_index^", "a_end_index^", "a_source"],
                    Self::substring,
                    Some("Get a substring with indices. 

- Out of range index is error
- Substring is calculated as char iterator not a byte iterator
- this operation is technically same with string[start_index..end_index]

# Arguments

- a_start_index : Start substring index [Unsigned integer] (trimmed)
- a_end_index   : End   substring index [Unsigned integer] (trimmed)
- a_source      : Source text get substring from

# Example

$assert(def,$sub(3,5,abcdef))".to_string()),
                ),
            ),
            (
                "surr".to_owned(),
                FMacroSign::new(
                    "surr",
                    ["a_start_pair","a_end_pair","a_content"],
                    Self::surround_with_pair,
                    Some("Surround given content with given pair

# Arguments

- a_start_pair : Start pair
- a_end_pair   : End pair
- a_content    : Text to surround with

# Example

$assert(<div>dividivi dip</div>,$enl()
$surr(<div>,</div>,dividivi dip))".to_string()),
                ),
            ),
            (
                "tab".to_owned(),
                FMacroSign::new(
                    "tab",
                    ["a_amount?^"],
                    Self::tab,
                    Some("Print tab

# Arguments

- a_amount : Amount of tabs to print [Unsigned integer] ( trimmed )

# Example

$tab(2)".to_string()),
                ),
            ),
            (
                "tail".to_owned(),
                FMacroSign::new(
                    "tail",
                    ["a_count^", "a_content"],
                    Self::tail,
                    Some("Get last parts of texts

# Arguments

- a_count   : Amount of characters to crop [unsigned integer] ( trimmed )
- a_content : Text to crop from

# Example

$assert(World,$tail( 5 ,Hello~ World))".to_string()),
                ),
            ),
            (
                "taill".to_owned(),
                FMacroSign::new(
                    "taill",
                    ["a_count^", "a_content"],
                    Self::tail_line,
                    Some("Get last lines of texts

# Arguments

- a_count   : Amount of lines to crop [unsigned integer] ( trimmed )
- a_lines   : Lines to crop from

# Example

$assert(b$nl()c,$taill( 2 ,a
b
c))".to_string()),
                ),
            ),
            (
                "table".to_owned(),
                FMacroSign::new(
                    "table",
                    ["a_table_form^", "a_csv_value^"],
                    Self::table,
                    Some("Construct a formatted table. Available table forms are \"github,html,wikitext\"

# Arguments

- a_table_form : Table format [ \"github\", \"html\", \"wikitext\" ]
- a_csv_value  : Value to convert to table

# Example

$assert=(
	|a|b|c|
	|-|-|-|
	|1|2|3|,$enl()
	$table(github,a,b,c
	1,2,3)
)".to_string()),
                ),
            ),
            (
                "tr".to_owned(),
                FMacroSign::new(
                    "tr",
                    ["a_chars", "a_sub","a_source"],
                    Self::translate,
                    Some("Translate characters. Usage similar to core util tr

# Arguments

- a_chars  : Matching characters
- a_sub    : Substitute characters
- a_source : Source text to apply translation

# Example

$assert(HellO_WOrld,$tr(-how,_HOW,hello-world))".to_string()),
                ),
            ),
            (
                "trim".to_owned(),
                FMacroSign::new(
                    "trim",
                    ["a_text"],
                    Self::trim,
                    Some("Trim text. This removes leading and trailing newlines, tabs and spaces

# Arguments

- a_text : Text to trim

# Example

$assert(Middle,$trim(
    Middle
))".to_string()),
                ),
            ),
            (
                "triml".to_owned(),
                FMacroSign::new(
                    "triml",
                    ["a_content"],
                    Self::triml,
                    Some("Trim values by lines. Trim is applied to each lines

# Arguments

- a_text : Text to trim

# Example

$assert(Upper$nl()Middle$nl()Last,$triml(    Upper
    Middle
          Last))".to_string()),
                ),
            ),
            (
                "trimla".to_owned(),
                FMacroSign::new(
                    "trimla",
                    ["a_trim_option^","a_content"],
                    Self::trimla,
                    Some("Triml with given amount

- Trims by line but with given amount. 
- If given integer it tries to trim blank characters as much as given value
- min trims by minimal amount that can be applied to total lines
- max acts same as triml
- Tab character is calculated as a single character. Don't combine spaces and tabs for this macro

# Arguments

- a_trim_option : Trim behaviour [\"min\", \"max\", Unsigned integer] ( trimmed )

# Example

$trimla(min,$space(1)First
$space(2)Second
$space(3)Third)
% ===
% Equally strips one space
% First
%  Second
%   Third


$trimla(3,$space(2)First
$space(3)Second
$space(5)Third)
% ===
% Equally tries stripping 3 spaces
% First
% Second
%   Third".to_string()),
                ),
            ),
            (
                "undef".to_owned(),
                FMacroSign::new(
                    "undef",
                    ["a_macro_name^"],
                    Self::undefine_call,
                    Some("Undefine a macro

- This undefines all macros that has a given name
- Define macro cannot be undefined
- Undef dodesn't yield error when macro doesn't exist

# Arguments 

- a_macro_name : Name of a macro to undefine

# Example

$define(test=Test)
$undef(test)
$fassert($test())".to_string()),
                ),
            ),
            (
                "unicode".to_owned(),
                FMacroSign::new(
                    "unicode",
                    ["a_value^"],
                    Self::paste_unicode,
                    Some("Creates a unicode character from given hex number without prefix

# Arguments

- a_value : Value to convert to unicode character

# Example

$assert(☺,$unicode(263a))".to_string()),
                ),
            ),
            (
                "upper".to_owned(),
                FMacroSign::new(
                    "upper",
                    ["a_text"],
                    Self::capitalize,
                    Some("Get a uppercase english text

# Arguments

- a_text: Text to transform

# Example

$assert(ABCDE,$upper(aBcDe))".to_string()),
                ),
            ),
            // THis is simply a placeholder
            (
                "define".to_owned(),
                FMacroSign::new(
                    "define",
                    ["a_define_statement"],
                    Self::define_type,
                    Some("Define a macro

# Arguments

Define should follow handful of rules

- Macro name, parameter name should start non number characters.
- Consequent characters for macro names, parameter names can be underscore or any characters except special characters.
- Parameters starts with comma and should be separated by whitespaces
- Macro body starts with equal(=) characters

# Example

$define(test=Test)
$define(demo,a_1 a_2=$a_1() $a_2())
$assert($test(),Test)
$assert(wow cow,$demo(wow,cow))".to_string()),
                ),
            ),
        ]));

        // Auth related macros are speical and has to be segregated from wasm target
        #[cfg(not(feature = "wasm"))]
        {
            map.insert(
                "env".to_owned(),
                FMacroSign::new(
                    "env",
                    ["a_env_name^"],
                    Self::get_env,
                    Some(
                        "Get an environment variable

# Auth : ENV

# Arguments

- a_env_name : Environment variable name to get (trimmed)

# Example

$assert(/home/user/dir,$env(HOME))"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "envset".to_owned(),
                FMacroSign::new(
                    "envset",
                    ["a_env_name^", "a_env_value"],
                    Self::set_env,
                    Some(
                        "Set an environment variable

# Auth : ENV

# Arguments

- a_env_name  : Environment variable name to set (trimmed)
- a_env_value : Value to set

# Example

$envset(HOME,/tmp)"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "abs".to_owned(),
                FMacroSign::new(
                    "abs",
                    ["a_path"],
                    Self::absolute_path,
                    Some(
                        "Get an absolute path. This requires a path to be a real path.

# Auth : FIN

# Return : path

# Arguments

- a_path : Path to make it absolute

# Example

$assert(/home/user/cwd/test.md,$abs(test.md))"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "syscmd".to_owned(),
                FMacroSign::new(
                    "syscmd",
                    ["a_command"],
                    Self::syscmd,
                    Some(
                        "Execute an sysctem command

- Each system command is executed as subprocess of folloiwng platform procedures
- Windows : cmd /C
- *Nix    : sh -c

# Auth : CMD

# Arguments

- a_command : Command to exectute

# Example

$assert(Linux,$syscmd(uname))"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "tempin".to_owned(),
                FMacroSign::new(
                    "tempin",
                    ESR,
                    Self::temp_include,
                    Some(
                        "Include a temporary file

- Default temporary path is folloiwng
- Windows : It depends, but %APPDATA%\\Local\\Temp\\rad.txt can be one
- *nix    : /tmp/rad.txt 

# Auth: FIN

# Example

$tempin()"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "tempout".to_owned(),
                FMacroSign::new(
                    "tempout",
                    ["a_content"],
                    Self::temp_out,
                    Some(
                        "Write to a temporary file

- Default temporary path is folloiwng
- Windows : It depends, but %APPDATA%\\Local\\Temp\\rad.txt can be one
- *nix    : /tmp/rad.txt 

# Auth: FOUT

# Arguments

- a_content : Content to write to temporary file

# Example

$temout(Content)"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "tempto".to_owned(),
                FMacroSign::new(
                    "tempto",
                    ["a_filename^"],
                    Self::set_temp_target,
                    Some(
                        "Change temporary file path

- This macro needs file out permission because it creates temporary file if non-existent

# Auth: FOUT

# Arguments

- a_filename : New temporary file path ( trimmed )

# Example

$tempto(/new/path)"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "temp".to_owned(),
                FMacroSign::new(
                    "temp",
                    ESR,
                    Self::get_temp_path,
                    Some(
                        "Get temp path

- Default temporary path is folloiwng
- Windows : It depends, but %APPDATA%\\Local\\Temp\\rad.txt can be one
- *nix    : /tmp/rad.txt 

# Example

$assert(/tmp/rad.txt,$temp())"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "include".to_owned(),
                FMacroSign::new(
                    "include",
                    ["a_filename^", "a_raw_mode^+?"],
                    Self::include,
                    Some(
                        "Include a file

- Include reads a whole chunk of file into a \"Reader\" and expands
- Use readin or readto if you want buffered behaviour
- If raw mode is enabled include doesn't expand any macros inside the file

# AUTH : FIN

# Arguments

- a_filename : File name to read ( trimmed )
- a_raw_mode : Whehter to escape the read. Default is false [boolean] ( trimmed )

$include(file_path)
$include(file_path, true)"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "fileout".to_owned(),
                FMacroSign::new(
                    "fileout",
                    ["a_filename^", "a_truncate?^", "a_content"],
                    Self::file_out,
                    Some(
                        "Write content to a file

# Auth : FOUT

# Arguments

# Example
"
                        .to_string(),
                    ),
                ),
            );
            map.insert(
                "listdir".to_owned(),
                FMacroSign::new(
                    "listdir",
                    ["a_path+", "a_absolute?^+", "a_delim+"],
                    Self::list_directory_files,
                    Some(
                        "List a directory's files as csv. 

- A default path is a current working directory. 
- A defualt delimiter is comma.

# Auth : FIN

# Arguments

- a_path     : A directory path to list files (optional)
- a_absolute : Whether to print files as absolute form [boolean] (trimmed, optional)
- a_delim    : A delimiter to put between items (optional)

# Example

$assert(15,$count($litdir()))"
                            .to_string(),
                    ),
                ),
            );
        }

        #[cfg(feature = "cindex")]
        {
            map.insert(
                "regcsv".to_owned(),
                FMacroSign::new(
                    "regcsv",
                    ["a_table_name^", "a_table"],
                    Self::cindex_register,
                    Some(
                        "Register a csv table

# Arguments

- a_table_name : Table name to be registered ( trimmed )
- a_table      : Csv data table

# Example

$regcsv(table1,a,b,c
1,2,3)"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "dropcsv".to_owned(),
                FMacroSign::new(
                    "dropcsv",
                    ["a_table_name^"],
                    Self::cindex_drop,
                    Some(
                        "Drop a csv table

# Arguments

- a_table_name : A csv table name to drop ( trimmed )

# Example

$dropcsv(table1)"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "query".to_owned(),
                FMacroSign::new(
                    "query",
                    ["a_query^"],
                    Self::cindex_query,
                    Some(
                        "Query a csv table

- Syntax of the query resembles SQL
- Refer cindex for detailed query syntax

# Arguments

- a_query : Query statement ( trimmed )

# Example

$query(SELECT * FROM TABLE WHERE name == john FLAG PHD)"
                            .to_string(),
                    ),
                ),
            );
        }

        #[cfg(feature = "chrono")]
        {
            map.insert(
                "time".to_owned(),
                FMacroSign::new(
                    "time",
                    ESR,
                    Self::time,
                    Some("Get current time".to_string()),
                ),
            );
            map.insert(
                "date".to_owned(),
                FMacroSign::new(
                    "date",
                    ESR,
                    Self::date,
                    Some("Get current date".to_string()),
                ),
            );
            map.insert(
                "tarray".to_owned(),
                FMacroSign::new(
                    "tarray",
                    ["a_second^"],
                    Self::tarray,
                    Some("Get given time as array".to_string()),
                ),
            );
            map.insert(
                "hms".to_owned(),
                FMacroSign::new(
                    "hms",
                    ["a_second^"],
                    Self::hms,
                    Some("Get given time in hh:mm:ss format".to_string()),
                ),
            );
        }
        #[cfg(feature = "evalexpr")]
        {
            map.insert(
                "eval".to_owned(),
                FMacroSign::new(
                    "eval",
                    ["a_expression"],
                    Self::eval,
                    Some("Evaluate a given expression".to_string()),
                ),
            );
            map.insert(
                "evalk".to_owned(),
                FMacroSign::new(
                    "evalk",
                    ["a_expression"],
                    Self::eval_keep,
                    Some("Evaluate an expression while keeping source texts".to_string()),
                ),
            );
        }
        #[cfg(feature = "textwrap")]
        map.insert(
            "wrap".to_owned(),
            FMacroSign::new(
                "wrap",
                ["a_width^", "a_content"],
                Self::wrap,
                Some("Wrap texts by width".to_string()),
            ),
        );

        #[cfg(feature = "hook")]
        {
            map.insert(
                "hookon".to_owned(),
                FMacroSign::new(
                    "hookon",
                    ["a_macro_type", "a_target_name"],
                    Self::hook_enable,
                    Some("Enable hook".to_string()),
                ),
            );
            map.insert(
                "hookoff".to_owned(),
                FMacroSign::new(
                    "hookoff",
                    ["a_macro_type", "a_target_name"],
                    Self::hook_disable,
                    Some("Disable hook".to_string()),
                ),
            );
        }

        // Storage
        {
            map.insert(
                "update".to_owned(),
                FMacroSign::new(
                    "update",
                    ["a_text"],
                    Self::update_storage,
                    Some("Update storage".to_string()),
                ),
            );
            map.insert(
                "extract".to_owned(),
                FMacroSign::new(
                    "extract",
                    ESR,
                    Self::extract_storage,
                    Some("Extract from storage".to_string()),
                ),
            );
        }

        // Return struct
        Self { macros: map }
    }

    /// Add new macro extension from macro builder
    pub(crate) fn new_ext_macro(&mut self, ext: ExtMacroBuilder) {
        if let Some(ExtMacroBody::Function(mac_ref)) = ext.macro_body {
            let sign = FMacroSign::new(&ext.macro_name, &ext.args, mac_ref, ext.macro_desc);
            self.macros.insert(ext.macro_name, sign);
        }
    }

    /// Check if a given macro exists
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the macro to find
    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// Get function reference by name
    pub fn get_func(&self, name: &str) -> Option<&FunctionMacroType> {
        if let Some(sig) = self.macros.get(name) {
            Some(&sig.logic)
        } else {
            None
        }
    }

    /// Get Function pointer from map
    #[cfg(feature = "signature")]
    pub(crate) fn get_signature(&self, name: &str) -> Option<&FMacroSign> {
        self.macros.get(name)
    }

    /// Undefine a macro
    ///
    /// # Arguments
    ///
    /// * `name` - Macro name to undefine
    pub fn undefine(&mut self, name: &str) {
        self.macros.remove(name);
    }

    /// Rename a macro
    ///
    /// # Arguments
    ///
    /// * `name` - Source macro name to find
    /// * `target` - Target macro name to apply
    pub fn rename(&mut self, name: &str, target: &str) -> bool {
        if let Some(func) = self.macros.remove(name) {
            self.macros.insert(target.to_owned(), func);
            true
        } else {
            false
        }
    }

    // ==========
    // Function Macros
    // ==========
    /// Print out current time
    ///
    /// # Usage
    ///
    /// $time()
    #[cfg(feature = "chrono")]
    fn time(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(format!(
            "{}",
            chrono::offset::Local::now().format("%H:%M:%S")
        )))
    }

    /// Get formattted time from given second
    ///
    /// # Usage
    ///
    /// $tarray(HH:MM:SS)
    #[cfg(feature = "chrono")]
    fn tarray(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let seconds = trim!(&args[0]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a number",
                    args[0]
                ))
            })?;
            let hour = seconds / 3600;
            let minute = seconds % 3600 / 60;
            let second = seconds % 3600 % 60;
            let mut arr = second.to_string();
            if minute != 0 {
                arr.push(',');
                arr.push_str(&minute.to_string());
            }
            if hour != 0 {
                arr.push(',');
                arr.push_str(&hour.to_string());
            }
            Ok(Some(arr))
        } else {
            Err(RadError::InvalidArgument(
                "tarray requires an argument".to_owned(),
            ))
        }
    }

    /// Format time as hms
    ///
    /// # Usage
    ///
    /// $hms(2020)
    #[cfg(feature = "chrono")]
    fn hms(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let seconds = trim!(&args[0]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a number",
                    args[0]
                ))
            })?;
            let hour = seconds / 3600;
            let minute = seconds % 3600 / 60;
            let second = seconds % 3600 % 60;
            let time = format!("{:02}:{:02}:{:02}", hour, minute, second);
            Ok(Some(time))
        } else {
            Err(RadError::InvalidArgument(
                "hms sub requires an argument".to_owned(),
            ))
        }
    }

    /// Print out current date
    ///
    /// # Usage
    ///
    /// $date()
    #[cfg(feature = "chrono")]
    fn date(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(format!(
            "{}",
            chrono::offset::Local::now().format("%Y-%m-%d")
        )))
    }

    /// Substitute the given source with following match expressions
    ///
    /// # Usage
    ///
    /// $regex(expression,substitution,source)
    fn regex_sub(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let match_expr = &args[0];
            let substitution = &args[1];
            let source = &args[2];

            if trim!(match_expr).is_empty() {
                return Err(RadError::InvalidArgument(
                    "Regex expressino cannot be a empty string".to_string(),
                ));
            }

            match p.state.regex_cache.get(match_expr) {
                Some(reg) => Ok(Some(reg.replace_all(source, substitution).to_string())),
                None => {
                    let result = p
                        .state
                        .regex_cache
                        .append(match_expr)?
                        .replace_all(source, substitution);
                    Ok(Some(result.to_string()))
                }
            }
        } else {
            Err(RadError::InvalidArgument(
                "Regex sub requires three arguments".to_owned(),
            ))
        }
    }

    /// Print current file input
    ///
    /// $input()
    fn print_current_input(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        match &p.state.current_input {
            ProcessInput::Stdin => Ok(Some("Stdin".to_string())),
            ProcessInput::File(path) => {
                let args = ArgParser::new().args_to_vec(args, ',', GreedyState::Never);
                if !args.is_empty() && !trim!(&args[0]).is_empty() {
                    let print_absolute = trim!(&args[0]).parse::<bool>().map_err(|_| {
                        RadError::InvalidArgument(
                            "Input's argument should be a boolean value".to_string(),
                        )
                    })?;
                    if print_absolute {
                        return Ok(Some(path.canonicalize()?.display().to_string()));
                    }
                }
                Ok(Some(path.display().to_string()))
            }
        }
    }

    /// Find an occurrence form a source
    ///
    /// # Usage
    ///
    /// $find(regex_match,source)
    fn find_occurence(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let match_expr = &args[0];
            let source = &args[1];

            if trim!(match_expr).is_empty() {
                return Err(RadError::InvalidArgument(
                    "Regex expressino cannot be a empty string".to_string(),
                ));
            }

            match p.state.regex_cache.get(match_expr) {
                Some(reg) => Ok(Some(reg.is_match(source).to_string())),
                None => {
                    let result = p.state.regex_cache.append(match_expr)?.is_match(source);
                    Ok(Some(result.to_string()))
                }
            }
        } else {
            Err(RadError::InvalidArgument(
                "find requires two arguments".to_owned(),
            ))
        }
    }

    /// Find multiple occurrence form a source
    ///
    /// # Usage
    ///
    /// $findm(regex_match,source)
    fn find_multiple_occurence(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let match_expr = &args[0];
            let source = &args[1];

            if trim!(match_expr).is_empty() {
                return Err(RadError::InvalidArgument(
                    "Regex expressino cannot be a empty string".to_string(),
                ));
            }

            match p.state.regex_cache.get(match_expr) {
                Some(reg) => Ok(Some(reg.find_iter(source).count().to_string())),
                None => {
                    let result = p
                        .state
                        .regex_cache
                        .append(match_expr)?
                        .find_iter(source)
                        .count();
                    Ok(Some(result.to_string()))
                }
            }
        } else {
            Err(RadError::InvalidArgument(
                "findm requires two arguments".to_owned(),
            ))
        }
    }

    /// Evaluate given expression
    ///
    /// This returns true, false or evaluated number
    ///
    /// # Usage
    ///
    /// $eval(expression)
    #[cfg(feature = "evalexpr")]
    fn eval(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let formula = &args[0];
            let result = evalexpr::eval(formula)?;
            Ok(Some(result.to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "Eval requires an argument".to_owned(),
            ))
        }
    }

    /// Evaluate given expression but keep original expression
    ///
    /// This returns true, false or evaluated number
    ///
    /// # Usage
    ///
    /// $eval(expression)
    #[cfg(feature = "evalexpr")]
    fn eval_keep(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            // This is the processed raw formula
            let formula = trim!(&args[0]);
            let result = format!("{} = {}", formula, evalexpr::eval(&formula)?);
            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "Eval requires an argument".to_owned(),
            ))
        }
    }

    /// Negate given value
    ///
    /// This returns true, false or evaluated number
    ///
    /// # Usage
    ///
    /// $not(expression)
    fn not(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            // No need to trim right now because is_arg_true trims already
            // Of course, it returns cow so it doesn't create overhead anyway
            let args = &args[0];
            if let Ok(value) = Utils::is_arg_true(args) {
                Ok(Some((!value).to_string()))
            } else {
                Err(RadError::InvalidArgument(format!(
                    "Not requires either true/false or zero/nonzero integer but given \"{}\"",
                    args
                )))
            }
        } else {
            Err(RadError::InvalidArgument(
                "Not requires an argument".to_owned(),
            ))
        }
    }

    /// Trim preceding and trailing whitespaces (' ', '\n', '\t', '\r')
    ///
    /// # Usage
    ///
    /// $trim(expression)
    fn trim(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            Ok(Some(trim!(&args[0]).to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "Trim requires an argument".to_owned(),
            ))
        }
    }

    /// Indent lines
    ///
    /// # Usage
    ///
    /// $indent(*, multi
    /// line
    /// expression
    /// )
    fn indent_lines(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let indenter = &args[0];
            let mut lines = String::new();
            let mut iter = args[1].lines().peekable();
            while let Some(line) = iter.next() {
                if !line.is_empty() {
                    write!(lines, "{}{}", indenter, line)?;
                }
                // Append newline because String.lines() method cuts off all newlines
                if iter.peek().is_some() {
                    lines.push_str(&p.state.newline);
                }
            }
            Ok(Some(lines))
        } else {
            Err(RadError::InvalidArgument(
                "indent requires an argument".to_owned(),
            ))
        }
    }

    /// Trim preceding and trailing whitespaces (' ', '\n', '\t', '\r') but for all lines
    ///
    /// # Usage
    ///
    /// $triml(\t multi
    /// \t line
    /// \t expression
    /// )
    fn triml(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let mut lines = String::new();
            let mut iter = args[0].lines().peekable();
            while let Some(line) = iter.next() {
                lines.push_str(&trim!(line));
                // Append newline because String.lines() method cuts off all newlines
                if iter.peek().is_some() {
                    lines.push_str(&p.state.newline);
                }
            }
            Ok(Some(lines))
        } else {
            Err(RadError::InvalidArgument(
                "Triml requires an argument".to_owned(),
            ))
        }
    }

    /// Trim lines with given amount
    ///
    /// # Usage
    ///
    /// $trimla(min,
    /// \t multi
    /// \t line
    /// \t expression
    /// )
    fn trimla(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let option = &args[0];
            let source = &args[1];
            let mut try_amount = None;
            let min_amount = match option.as_str() {
                "max" => None,
                "min" => {
                    let mut min_amount = usize::MAX;
                    for line in source.lines() {
                        let space_amount = line.len() - line.trim_start().len();
                        if min_amount > space_amount && !line.trim_start().is_empty() {
                            min_amount = space_amount;
                        }
                    }
                    if min_amount == usize::MAX {
                        None
                    } else {
                        Some(min_amount)
                    }
                }
                _ => {
                    try_amount = Some(option.parse::<usize>().map_err(|_| {
                        RadError::InvalidArgument(
                            "Trimla option should be either min,max or number".to_string(),
                        )
                    })?);
                    None
                }
            };

            let mut lines = String::new();
            let mut source_iter = source.lines().peekable();
            while let Some(line) = source_iter.next() {
                if line.trim_start().is_empty() {
                    lines.push_str(line);
                } else {
                    let trimmed = match min_amount {
                        Some(amount) => line[amount..].to_string(),
                        None => match try_amount {
                            Some(amount) => {
                                let space_amount = line.len() - line.trim_start().len();
                                line[amount.min(space_amount)..].to_string()
                            }
                            None => trim!(line).to_string(),
                        },
                    };
                    lines.push_str(&trimmed);
                }
                // Append newline because String.lines() method cuts off all newlines
                if source_iter.peek().is_some() {
                    lines.push_str(&p.state.newline);
                }
            }
            Ok(Some(lines))
        } else {
            Err(RadError::InvalidArgument(
                "Trimla requires two arguments".to_owned(),
            ))
        }
    }

    /// Removes duplicate newlines whithin given input
    ///
    /// # Usage
    ///
    /// $chomp(expression)
    fn chomp(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let source = &args[0];
            // First convert all '\r\n' into '\n' and reformat it into current newline characters
            let lf_converted = &*CLRF_MATCH.replace_all(source, "\n");
            let chomp_result = &*CHOMP_MATCH
                .replace_all(lf_converted, format!("{0}{0}", &processor.state.newline));

            Ok(Some(chomp_result.to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "Chomp requires an argument".to_owned(),
            ))
        }
    }

    /// Both apply trim and chomp to given expression
    ///
    /// # Usage
    ///
    /// $comp(Expression)
    fn compress(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let source = &args[0];
            // Chomp and then compress
            let result = trim!(&FunctionMacroMap::chomp(source, processor)?.unwrap()).to_string();

            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "Compress requires an argument".to_owned(),
            ))
        }
    }

    /// Creates placeholder with given amount of word counts
    ///
    /// # Usage
    ///
    /// $lipsum(Number)
    fn lipsum_words(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let word_count = &args[0];
            if let Ok(count) = trim!(word_count).parse::<usize>() {
                if count <= *LOREM_WIDTH {
                    Ok(Some(LOREM[0..count].join(" ")))
                } else {
                    let mut lorem = String::new();
                    let loop_amount = count / *LOREM_WIDTH;
                    let remnant = count % *LOREM_WIDTH;
                    for _ in 0..loop_amount {
                        lorem.push_str(LOREM_SOURCE);
                    }
                    lorem.push_str(&LOREM[0..remnant].join(" "));
                    Ok(Some(lorem))
                }
            } else {
                Err(RadError::InvalidArgument(format!("Lipsum needs a number bigger or equal to 0 (unsigned integer) but given \"{}\"", word_count)))
            }
        } else {
            Err(RadError::InvalidArgument(
                "Lipsum requires an argument".to_owned(),
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
    fn include(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("include", AuthType::FIN, processor)? {
            return Ok(None);
        }
        let args = ArgParser::new().args_to_vec(args, ',', GreedyState::Never);
        if !args.is_empty() {
            let raw = trim!(&args[0]);
            let mut raw_include = false;
            let mut file_path = PathBuf::from(raw.as_ref());

            // if current input is not stdin and file path is relative
            // Create new file path that starts from current file path
            if let ProcessInput::File(path) = &processor.state.current_input {
                if file_path.is_relative() {
                    // It is ok get parent because any path that has a length can return parent
                    file_path = path.parent().unwrap().join(file_path);
                }
            }

            if file_path.is_file() {
                let canonic = file_path.canonicalize()?;

                Utils::check_include_sanity(processor, &canonic)?;
                // Set sandbox after error checking or it will act starngely
                processor.set_sandbox(true);

                // Optionally enable raw mode
                if args.len() >= 2 {
                    raw_include = Utils::is_arg_true(&args[1])?;

                    // You don't have to backup pause state because include wouldn't be triggered
                    // at the first place, if paused was true
                    if raw_include {
                        processor.state.paused = true;
                    }
                }

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

    /// Repeat given expression about given amount times
    ///
    /// # Usage
    ///
    /// $repeat(count,text)
    fn repeat(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let repeat_count = if let Ok(count) = trim!(&args[0]).parse::<usize>() {
                count
            } else {
                return Err(RadError::InvalidArgument(format!("Repeat needs a number bigger or equal to 0 (unsigned integer) but given \"{}\"", &args[0])));
            };
            let repeat_object = &args[1];
            let mut repeated = String::new();
            for _ in 0..repeat_count {
                repeated.push_str(repeat_object);
            }
            Ok(Some(repeated))
        } else {
            Err(RadError::InvalidArgument(
                "Repeat requires two arguments".to_owned(),
            ))
        }
    }

    /// Call system command
    ///
    /// This calls via 'CMD \C' in windows platform while unix call is operated without any mediation.
    ///
    /// # Usage
    ///
    /// $syscmd(system command -a arguments)
    fn syscmd(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("syscmd", AuthType::CMD, p)? {
            return Ok(None);
        }
        if let Some(args_content) = ArgParser::new().args_with_len(args, 1) {
            let source = &args_content[0];
            let arg_vec = source.split_whitespace().collect::<Vec<&str>>();

            let output = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .arg("/C")
                    .args(arg_vec)
                    .output()
                    .expect("failed to execute process")
                    .stdout
            } else {
                let sys_args = if arg_vec.len() > 1 {
                    &arg_vec[1..]
                } else {
                    &[]
                };
                Command::new(&arg_vec[0])
                    .args(sys_args)
                    .output()
                    .expect("failed to execute process")
                    .stdout
            };

            Ok(Some(String::from_utf8(output)?))
        } else {
            Err(RadError::InvalidArgument(
                "Syscmd requires an argument".to_owned(),
            ))
        }
    }

    /// Undefine a macro
    ///
    /// # Usage
    ///
    /// $undef(macro_name)
    fn undefine_call(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let name = trim!(&args[0]);

            if processor.contains_macro(&name, MacroType::Any) {
                processor.undefine_macro(&name, MacroType::Any);
            } else {
                processor.log_error(&format!(
                    "Macro \"{}\" doesn't exist, therefore cannot undefine",
                    name
                ))?;
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Undefine requires an argument".to_owned(),
            ))
        }
    }

    /// Placeholder for define
    fn define_type(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(None)
    }

    /// Array
    ///
    /// # Usage
    ///
    /// $arr(1 2 3)
    /// $arr(1 2 3, )   // Separator
    /// $arr(1 2 3, ,2) // Separator + regex filter
    fn array(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        let parsed = ArgParser::new().args_to_vec(args, ',', GreedyState::Never);
        if parsed.is_empty() {
            Err(RadError::InvalidArgument(
                "Array requires an argument".to_owned(),
            ))
        } else {
            let separater = if parsed.len() >= 2 {
                &parsed[1] // Use given separater
            } else {
                " "
            }; // Use whitespace as default
            let mut vec = parsed[0].split(separater).collect::<Vec<&str>>();

            // Also given filter argument, then filter with regex expression
            if parsed.len() == 3 {
                let expr = &parsed[2];
                let reg = match p.state.regex_cache.get(expr) {
                    Some(reg) => reg,
                    None => p.state.regex_cache.append(expr)?,
                };
                vec = vec.into_iter().filter(|&item| reg.is_match(item)).collect();
            }

            // Join as csv
            let joined = vec.join(",");

            Ok(Some(joined))
        }
    }

    /// Assert
    ///
    /// # Usage
    ///
    /// $assert(abc,abc)
    fn assert(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            if args[0] == args[1] {
                p.track_assertion(true)?;
                Ok(None)
            } else {
                p.track_assertion(false)?;
                Err(RadError::AssertFail)
            }
        } else {
            Err(RadError::InvalidArgument(
                "Assert requires two arguments".to_owned(),
            ))
        }
    }

    /// Assert not equal
    ///
    /// # Usage
    ///
    /// $nassert(abc,abc)
    fn assert_ne(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            if args[0] != args[1] {
                p.track_assertion(true)?;
                Ok(None)
            } else {
                p.track_assertion(false)?;
                Err(RadError::AssertFail)
            }
        } else {
            Err(RadError::InvalidArgument(
                "Assert_ne requires two arguments".to_owned(),
            ))
        }
    }

    /// Increment Counter
    ///
    /// # Usage
    ///
    /// $counter(name, type)
    fn change_counter(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_to_vec(args, ',', GreedyState::Never);
        if args.is_empty() {
            return Err(RadError::InvalidArgument(
                "counter requires an argument".to_owned(),
            ));
        }
        let counter_name = trim!(&args[0]);
        let counter_type = if args.len() > 1 {
            trim!(&args[1]).to_string()
        } else {
            "plus".to_string()
        };
        // Crate new macro if non-existent
        if !p.contains_macro(&counter_name, MacroType::Runtime) {
            p.add_static_rules(&[(&counter_name, "0")])?;
        }
        let body = p
            .get_runtime_macro_body(&counter_name)?
            .parse::<isize>()
            .map_err(|_| {
                RadError::UnallowedMacroExecution(
                    "You cannot call counter on non-number macro values".to_string(),
                )
            })?;
        match counter_type.to_lowercase().as_ref() {
            "plus" => {
                p.replace_macro(&counter_name, &(body + 1).to_string());
            }
            "minus" => {
                p.replace_macro(&counter_name, &(body - 1).to_string());
            }
            _ => {
                return Err(RadError::InvalidArgument(format!(
                    "Given counter type is not valid \"{}\"",
                    counter_type
                )))
            }
        }
        Ok(None)
    }

    /// Join an array
    ///
    /// # Usage
    ///
    /// $join(" ",a,b,c)
    fn join(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let sep = &args[0];
            let text = &args[1];
            Ok(Some(text.split(',').collect::<Vec<_>>().join(sep)))
        } else {
            Err(RadError::InvalidArgument(
                "join requires two arguments".to_owned(),
            ))
        }
    }

    /// Join lines
    ///
    /// # Usage
    ///
    /// $joinl(" ",text)
    fn joinl(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let sep = &args[0];
            let text = &args[1];
            Ok(Some(text.lines().collect::<Vec<_>>().join(sep)))
        } else {
            Err(RadError::InvalidArgument(
                "joinl requires two arguments".to_owned(),
            ))
        }
    }

    /// Create a table with given format and csv input
    ///
    /// Available formats are 'github', 'wikitext' and 'html'
    ///
    /// # Usage
    ///
    /// $table(github,1,2,3
    /// 4,5,6)
    fn table(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let table_format = trim!(&args[0]); // Either gfm, wikitex, latex, none
            let csv_content = trim!(&args[1]);
            let result = Formatter::csv_to_table(&table_format, &csv_content, &p.state.newline)?;
            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "Table requires two arguments".to_owned(),
            ))
        }
    }

    /// Put value into a temporary stack called pipe
    ///
    /// Piped value can be popped with macro '-'
    ///
    /// # Usage
    ///
    /// $pipe(Value)
    fn pipe(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            processor.state.add_pipe(None, args[0].to_owned());
        }
        Ok(None)
    }

    /// Put value into a temporary stack called pipe
    ///
    /// Piped value can be popped with macro '-'
    ///
    /// # Usage
    ///
    /// $pipeto(Value)
    fn pipe_to(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            processor
                .state
                .add_pipe(Some(&trim!(&args[0])), args[1].to_owned());
        } else {
            return Err(RadError::InvalidArgument(
                "pipeto requires two arguments".to_owned(),
            ));
        }
        Ok(None)
    }

    /// Get environment variable with given name
    ///
    /// # Usage
    ///
    /// $env(SHELL)
    #[cfg(not(feature = "wasm"))]
    fn get_env(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("env", AuthType::ENV, p)? {
            return Ok(None);
        }
        if let Ok(out) = std::env::var(trim!(args).as_ref()) {
            Ok(Some(out))
        } else {
            if p.state.behaviour == ErrorBehaviour::Strict {
                p.log_warning(
                    &format!("Env : \"{}\" is not defined.", args),
                    WarningType::Sanity,
                )?;
            }
            Ok(None)
        }
    }

    /// Set environment variable with given name
    ///
    /// # Usage
    ///
    /// $envset(SHELL,value)
    #[cfg(not(feature = "wasm"))]
    fn set_env(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("envset", AuthType::ENV, p)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let name = trim!(&args[0]);
            let value = &args[1];

            if p.state.behaviour == ErrorBehaviour::Strict && std::env::var(name.as_ref()).is_ok() {
                return Err(RadError::InvalidArgument(format!(
                    "You cannot override environment variable in strict mode. Failed to set \"{}\"",
                    name
                )));
            }

            std::env::set_var(name.as_ref(), value);
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Envset requires two arguments".to_owned(),
            ))
        }
    }

    /// Trigger panic
    fn manual_panic(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Err(RadError::ManualPanic(args.to_string()))
    }

    /// Escape processing
    fn escape(_: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        processor.state.flow_control = FlowControl::Escape;
        Ok(None)
    }

    /// Exit processing
    fn exit(_: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        processor.state.flow_control = FlowControl::Exit;
        Ok(None)
    }

    /// Merge multiple paths into a single path
    ///
    /// This creates platform agonistic path which can be consumed by other macros.
    ///
    /// # Usage
    ///
    /// $path($env(HOME),document,test.docx)
    fn merge_path(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        let vec = ArgParser::new().args_to_vec(args, ',', GreedyState::Never);

        let out = vec
            .iter()
            .map(|s| trim!(s).to_string())
            .collect::<PathBuf>();

        if let Some(value) = out.to_str() {
            Ok(Some(value.to_owned()))
        } else {
            Err(RadError::InvalidArgument(format!(
                "Invalid path : {}",
                out.display()
            )))
        }
    }

    /// Print tab
    ///
    /// # Usage
    ///
    /// $tab()
    fn tab(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        let count = if !args.is_empty() {
            trim!(args)
                .parse::<usize>()
                .map_err(|_| RadError::InvalidArgument("tab requires number".to_string()))?
        } else {
            1
        };

        Ok(Some("\t".repeat(count)))
    }

    /// Yield spaces
    ///
    /// # Usage
    ///
    /// $space()
    fn space(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        let count = if !args.is_empty() {
            trim!(args)
                .parse::<usize>()
                .map_err(|_| RadError::InvalidArgument("space requires number".to_string()))?
        } else {
            1
        };

        Ok(Some(" ".repeat(count)))
    }

    /// Print nothing
    fn print_empty(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(None)
    }

    /// Yield newline according to platform or user option
    ///
    /// # Usage
    ///
    /// $nl()
    fn newline(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        let count = if !args.is_empty() {
            trim!(args)
                .parse::<usize>()
                .map_err(|_| RadError::InvalidArgument("nl requires number".to_string()))?
        } else {
            1
        };

        Ok(Some(p.state.newline.repeat(count)))
    }

    /// deny new line
    ///
    /// # Usage
    ///
    /// $dnl()
    fn deny_newline(_: &str, p: &mut Processor) -> RadResult<Option<String>> {
        p.state.deny_newline = true;
        Ok(None)
    }

    /// escape new line
    ///
    /// # Usage
    ///
    /// $enl()
    fn escape_newline(_: &str, p: &mut Processor) -> RadResult<Option<String>> {
        p.state.escape_newline = true;
        Ok(None)
    }

    /// Get name from given path
    ///
    /// # Usage
    ///
    /// $name(path/file.exe)
    fn get_name(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let path = Path::new(&args[0]);

            if let Some(name) = path.file_name() {
                if let Some(value) = name.to_str() {
                    return Ok(Some(value.to_owned()));
                }
            }
            Err(RadError::InvalidArgument(format!(
                "Invalid path : {}",
                path.display()
            )))
        } else {
            Err(RadError::InvalidArgument(
                "name requires an argument".to_owned(),
            ))
        }
    }

    /// Get absolute path from given path
    ///
    /// # Usage
    ///
    /// $abs(../canonic_path.txt)
    fn absolute_path(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("abs", AuthType::FIN, p)? {
            return Ok(None);
        }

        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let path = std::fs::canonicalize(p.get_current_dir()?.join(&args[0]))?
                .to_str()
                .unwrap()
                .to_owned();
            Ok(Some(path))
        } else {
            Err(RadError::InvalidArgument(
                "Abs requires an argument".to_owned(),
            ))
        }
    }

    /// Get parent from given path
    ///
    /// # Usage
    ///
    /// $parent(path/file.exe)
    fn get_parent(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let path = Path::new(&args[0]);

            if let Some(name) = path.parent() {
                if let Some(value) = name.to_str() {
                    return Ok(Some(value.to_owned()));
                }
            }
            Err(RadError::InvalidArgument(format!(
                "Invalid path : {}",
                path.display()
            )))
        } else {
            Err(RadError::InvalidArgument(
                "parent requires an argument".to_owned(),
            ))
        }
    }

    /// Get pipe value
    ///
    /// # Usage
    ///
    /// $-()
    /// $-(p1)
    fn get_pipe(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        let pipe = if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let name = trim!(&args[0]);
            if name.is_empty() {
                let out = processor.state.get_pipe("-");

                if out.is_none() {
                    processor.log_warning("Empty pipe", WarningType::Sanity)?;
                }

                out
            } else if let Some(pipe) = processor.state.get_pipe(&args[0]) {
                Some(pipe)
            } else {
                processor.log_warning(
                    &format!("Empty named pipe : \"{}\"", args[0]),
                    WarningType::Sanity,
                )?;
                None
            }
        } else {
            // "-" Always exsit, thus safe to unwrap
            let out = processor.state.get_pipe("-").unwrap_or_default();
            Some(out)
        };
        Ok(pipe)
    }

    /// Return a length of the string
    ///
    /// This is O(n) operation.
    /// String.len() function returns byte length not "Character" length
    /// therefore, chars().count() is used
    ///
    /// # Usage
    ///
    /// $len(안녕하세요)
    /// $len(Hello)
    fn len(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(args.chars().count().to_string()))
    }

    /// Rename macro rule to other name
    ///
    /// # Usage
    ///
    /// $rename(name,target)
    fn rename_call(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let target = trim!(&args[0]);
            let new = trim!(&args[1]);

            if processor.contains_macro(&target, MacroType::Any) {
                processor.rename_macro(&target, &new, MacroType::Any);
            } else {
                processor.log_error(&format!(
                    "Macro \"{}\" doesn't exist, therefore cannot rename",
                    target
                ))?;
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Rename requires two arguments".to_owned(),
            ))
        }
    }

    /// Append content to a macro
    ///
    /// Only runtime macros can be appended.
    ///
    /// # Usage
    ///
    /// $append(macro_name,Content)
    fn append(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let name = &args[0];
            let target = &args[1];
            if processor.contains_macro(name, MacroType::Runtime) {
                processor.append_macro(name, target);
            } else {
                processor.log_error(&format!("Macro \"{}\" doesn't exist", name))?;
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Append requires two arguments".to_owned(),
            ))
        }
    }

    /// Translate given char aray into corresponding char array
    ///
    /// # Usage
    ///
    /// $tr(abc,ABC,Source)
    fn translate(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let mut source = args[2].clone();
            let target = &args[0].chars().collect::<Vec<char>>();
            let destination = &args[1].chars().collect::<Vec<char>>();

            if target.len() != destination.len() {
                return Err(RadError::InvalidArgument(format!("Tr's replacment should have same length of texts while given \"{:?}\" and \"{:?}\"", target, destination)));
            }

            for i in 0..target.len() {
                source = source.replace(target[i], &destination[i].to_string());
            }

            Ok(Some(source))
        } else {
            Err(RadError::InvalidArgument(
                "Tr requires three arguments".to_owned(),
            ))
        }
    }

    /// Get a substring(indexed) from given source
    ///
    /// # Usage
    ///
    /// $sub(0,5,GivenString)
    fn substring(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let source = &args[2];

            let mut min: Option<usize> = None;
            let mut max: Option<usize> = None;

            let start = trim!(&args[0]);
            let end = trim!(&args[1]);

            if let Ok(num) = start.parse::<usize>() {
                min.replace(num);
            } else if !start.is_empty() {
                return Err(RadError::InvalidArgument(format!("Sub's min value should be non zero positive integer or empty value but given \"{}\"", start)));
            }

            if let Ok(num) = end.parse::<usize>() {
                max.replace(num);
            } else if !end.is_empty() {
                return Err(RadError::InvalidArgument(format!("Sub's max value should be non zero positive integer or empty value but given \"{}\"", end)));
            }

            Ok(Some(Utils::utf8_substring(source, min, max)))
        } else {
            Err(RadError::InvalidArgument(
                "Sub requires three arguments".to_owned(),
            ))
        }
    }

    /// Save content to temporary file
    ///
    /// # Usage
    ///
    /// $tempout(Content)
    #[cfg(not(feature = "wasm"))]
    fn temp_out(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("tempout", AuthType::FOUT, p)? {
            return Ok(None);
        }

        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let content = &args[0];
            if let Some(file) = p.get_temp_file() {
                file.write_all(content.as_bytes())?;
            } else {
                return Err(RadError::InvalidExecution(
                    "You cannot use temp related macros in environment where fin/fout is not supported".to_owned(),
                ));
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Tempout requires an argument".to_owned(),
            ))
        }
    }

    /// Save content to a file
    ///
    /// # Usage
    ///
    /// $fileout(true,file_name,Content)
    #[cfg(not(feature = "wasm"))]
    fn file_out(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("fileout", AuthType::FOUT, p)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let file_name = trim!(&args[0]);
            let truncate = trim!(&args[1]);
            let content = &args[2];
            if let Ok(truncate) = Utils::is_arg_true(&truncate) {
                let file = std::env::current_dir()?.join(file_name.as_ref());
                let mut target_file = if truncate {
                    OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(file)?
                } else {
                    if !file.is_file() {
                        return Err(RadError::InvalidArgument(format!("Failed to read \"{}\". Fileout without truncate option needs exsiting file",file.display())));
                    }

                    OpenOptions::new().append(true).open(file)?
                };
                target_file.write_all(content.as_bytes())?;
                Ok(None)
            } else {
                Err(RadError::InvalidArgument(format!(
                    "Fileout requires either true/false or zero/nonzero integer but given \"{}\"",
                    truncate
                )))
            }
        } else {
            Err(RadError::InvalidArgument(
                "Fileout requires three argument".to_owned(),
            ))
        }
    }

    /// Get head of given text
    ///
    /// # Usage
    ///
    /// $head(2,Text To extract)
    fn head(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let count = trim!(&args[0]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Head requires positive integer number but got \"{}\"",
                    &args[0]
                ))
            })?;
            let content = &args[1].chars().collect::<Vec<_>>();
            let length = count.min(content.len());

            Ok(Some(content[0..length].iter().collect()))
        } else {
            Err(RadError::InvalidArgument(
                "head requires two argument".to_owned(),
            ))
        }
    }

    /// Get head of given text but for lines
    ///
    /// # Usage
    ///
    /// $headl(2,Text To extract)
    fn head_line(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let count = trim!(&args[0]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Headl requires positive integer number but got \"{}\"",
                    &args[0]
                ))
            })?;
            let lines = Utils::full_lines(args[1].as_bytes())
                .map(|line| line.unwrap())
                .collect::<Vec<String>>();
            let length = count.min(lines.len());

            Ok(Some(lines[0..length].concat()))
        } else {
            Err(RadError::InvalidArgument(
                "headl requires two argument".to_owned(),
            ))
        }
    }

    /// Get tail of given text
    ///
    /// # Usage
    ///
    /// $tail(2,Text To extract)
    fn tail(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let count = trim!(&args[0]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "tail requires positive integer number but got \"{}\"",
                    &args[0]
                ))
            })?;
            let content = &args[1].chars().collect::<Vec<_>>();
            let length = count.min(content.len());

            Ok(Some(
                content[content.len() - length..content.len()]
                    .iter()
                    .collect(),
            ))
        } else {
            Err(RadError::InvalidArgument(
                "tail requires two argument".to_owned(),
            ))
        }
    }

    /// Surround a text with given pair
    ///
    /// # Usage
    ///
    /// $strip(<p>,</p>,content)
    fn surround_with_pair(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let start = &args[0];
            let end = &args[1];
            let content = &args[2];
            Ok(Some(format!("{}{}{}", start, content, end)))
        } else {
            Err(RadError::InvalidArgument(
                "surr requires three arguments".to_owned(),
            ))
        }
    }

    /// Get tail of given text but for lines
    ///
    /// # Usage
    ///
    /// $taill(2,Text To extract)
    fn tail_line(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let count = trim!(&args[0]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "taill requires positive integer number but got \"{}\"",
                    &args[0]
                ))
            })?;
            let lines = Utils::full_lines(args[1].as_bytes())
                .map(|line| line.unwrap())
                .collect::<Vec<String>>();
            let length = count.min(lines.len());

            Ok(Some(lines[lines.len() - length..lines.len()].concat()))
        } else {
            Err(RadError::InvalidArgument(
                "taill requires two argument".to_owned(),
            ))
        }
    }

    /// Sort array
    ///
    /// # Usage
    ///
    /// $sort(asec,1,2,3,4,5)
    fn sort_array(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let order_type = trim!(&args[0]);
            let content = &mut args[1].split(',').collect::<Vec<&str>>();
            match order_type.to_lowercase().as_str() {
                "asec" => content.sort_unstable(),
                "desc" => {
                    content.sort_unstable();
                    content.reverse()
                }
                _ => {
                    return Err(RadError::InvalidArgument(format!(
                        "Sort requires either asec or desc but given \"{}\"",
                        order_type
                    )))
                }
            }

            Ok(Some(content.join(",")))
        } else {
            Err(RadError::InvalidArgument(
                "sort requires two argument".to_owned(),
            ))
        }
    }

    /// Sort lines
    ///
    /// # Usage
    ///
    /// $sortl(asec,Content)
    fn sort_lines(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let order_type = trim!(&args[0]);
            let content = &mut args[1].lines().collect::<Vec<&str>>();
            match order_type.to_lowercase().as_str() {
                "asec" => content.sort_unstable(),
                "desc" => {
                    content.sort_unstable();
                    content.reverse()
                }
                _ => {
                    return Err(RadError::InvalidArgument(format!(
                        "Sortl requires either asec or desc but given \"{}\"",
                        order_type
                    )))
                }
            }

            Ok(Some(content.join(&p.state.newline)))
        } else {
            Err(RadError::InvalidArgument(
                "sortl requires two argument".to_owned(),
            ))
        }
    }

    /// Index array
    ///
    /// # Usage
    ///
    /// $index(1,1,2,3,4,5)
    fn index_array(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let index = trim!(&args[0]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "index requires positive integer number but got \"{}\"",
                    &args[0]
                ))
            })?;
            let content = &mut args[1].split(',').collect::<Vec<&str>>();

            if content.len() <= index {
                return Err(RadError::InvalidArgument(format!(
                    "Index \"{}\" is bigger than content's length \"{}\"",
                    index,
                    content.len()
                )));
            }

            Ok(Some(content[index].to_owned()))
        } else {
            Err(RadError::InvalidArgument(
                "index requires two argument".to_owned(),
            ))
        }
    }

    /// Fold array
    ///
    /// # Usage
    ///
    /// $fold(1,2,3,4,5)
    fn fold(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let content = &mut args[0].split(',').collect::<Vec<&str>>();
            Ok(Some(content.join("")))
        } else {
            Err(RadError::InvalidArgument(
                "fold requires an argument".to_owned(),
            ))
        }
    }

    /// Fold lines
    ///
    /// # Usage
    ///
    /// $foldl(1,1,2,3,4,5)
    fn fold_line(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let content = &mut args[0].lines().collect::<Vec<&str>>();
            Ok(Some(content.join("")))
        } else {
            Err(RadError::InvalidArgument(
                "foldl requires an argument".to_owned(),
            ))
        }
    }

    /// Register expressino
    ///
    /// # Usage
    ///
    /// $regexpr(name,EXPR)
    fn register_expression(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let name = &args[0];
            let expr = &args[1];

            p.state.regex_cache.register(name, expr)?;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "regexpr requires two argument".to_owned(),
            ))
        }
    }

    /// Grep
    ///
    /// # Usage
    ///
    /// $grep(EXPR,CONTENT)
    fn grep(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let expr = &args[0];
            let reg = match p.state.regex_cache.get(expr) {
                Some(reg) => reg,
                None => p.state.regex_cache.append(expr)?,
            };
            let content = args[1].lines();
            let grepped = content
                .filter(|l| reg.is_match(l))
                .collect::<Vec<&str>>()
                .join(&p.state.newline);
            Ok(Some(grepped))
        } else {
            Err(RadError::InvalidArgument(
                "grep requires two argument".to_owned(),
            ))
        }
    }

    /// Count
    ///
    /// # Usage
    ///
    /// $count(1,2,3,4,5)
    fn count(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let array_count = &args[0].split(',').count();
            Ok(Some(array_count.to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "count requires an argument".to_owned(),
            ))
        }
    }

    /// Count words
    ///
    /// # Usage
    ///
    /// $countw(1 2 3 4 5)
    fn count_word(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let array_count = &args[0].split_whitespace().count();
            Ok(Some(array_count.to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "countw requires an argument".to_owned(),
            ))
        }
    }

    /// Count lines
    ///
    /// # Usage
    ///
    /// $countl(CONTENT goes here)
    fn count_lines(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let line_count = &args[0].lines().count();
            Ok(Some(line_count.to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "countl requires an argument".to_owned(),
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
    fn temp_include(_: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("tempin", AuthType::FIN, processor)? {
            return Ok(None);
        }
        let file = processor.get_temp_path().display();
        let chunk = Self::include(&file.to_string(), processor)?;
        Ok(chunk)
    }

    /// Relay all text into given target
    ///
    /// Every text including non macro calls are all sent to relay target
    ///
    /// # Usage
    ///
    /// $relay(type,argument)
    fn relay(args_src: &str, p: &mut Processor) -> RadResult<Option<String>> {
        let args: Vec<&str> = args_src.split(',').collect();
        if args.is_empty() {
            return Err(RadError::InvalidArgument(
                "relay at least requires an argument".to_owned(),
            ));
        }

        p.log_warning(
            &format!("Relaying text content to \"{}\"", args_src),
            WarningType::Security,
        )?;

        let raw_type = trim!(args[0]);
        let target = trim!(args.get(1).unwrap_or(&""));
        let relay_type = match raw_type.as_ref() {
            #[cfg(not(feature = "wasm"))]
            "temp" => {
                if !Utils::is_granted("relay", AuthType::FOUT, p)? {
                    return Ok(None);
                }
                RelayTarget::Temp
            }
            #[cfg(not(feature = "wasm"))]
            "file" => {
                use crate::models::FileTarget;
                if !Utils::is_granted("relay", AuthType::FOUT, p)? {
                    return Ok(None);
                }
                if args.len() == 1 {
                    return Err(RadError::InvalidArgument(
                        "relay requires second argument as file name for file relaying".to_owned(),
                    ));
                }
                let mut file_target = FileTarget::empty();
                file_target.set_path(Path::new(target.as_ref()));
                RelayTarget::File(file_target)
            }
            "macro" => {
                if args.len() == 1 {
                    return Err(RadError::InvalidArgument(
                        "relay requires second argument as macro name for macro relaying"
                            .to_owned(),
                    ));
                }
                if !p.contains_macro(&target, MacroType::Runtime) {
                    return Err(RadError::InvalidMacroName(format!(
                        "Cannot relay to non-exsitent macro or non-runtime macro \"{}\"",
                        target
                    )));
                }
                RelayTarget::Macro(args[1].to_owned())
            }
            _ => {
                return Err(RadError::InvalidArgument(format!(
                    "Given type \"{}\" is not a valid relay target",
                    args[0]
                )))
            }
        };
        p.state.relay.push(relay_type);
        Ok(None)
    }

    /// Disable relaying
    ///
    /// # Usage
    ///
    /// $hold()
    fn halt_relay(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let halt_immediate = if args[0].is_empty() {
                false
            } else {
                trim!(&args[0]).parse::<bool>().map_err(|_| {
                    RadError::InvalidArgument(
                        "Halt's argument should be a boolean value".to_string(),
                    )
                })?
            };
            if halt_immediate {
                // This remove last element from stack
                p.state.relay.pop();
            } else {
                p.insert_queue("$halt(true)");
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "halt requires an argument".to_owned(),
            ))
        }
    }

    /// Set temporary file
    ///
    /// # Usage
    ///
    /// $tempto(file_name)
    #[cfg(not(feature = "wasm"))]
    fn set_temp_target(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("tempto", AuthType::FOUT, processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            processor.set_temp_file(&std::env::temp_dir().join(trim!(&args[0]).as_ref()))?;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Temp requires an argument".to_owned(),
            ))
        }
    }

    /// Get temporary path
    ///
    /// # Usage
    ///
    /// $temp()
    #[cfg(not(feature = "wasm"))]
    fn get_temp_path(_: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(processor.state.temp_target.to_string()))
    }

    /// Get number
    ///
    /// # Usage
    ///
    /// $num(20%)
    fn get_number(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let src = trim!(&args[0]);
            let captured = NUM_MATCH.captures(&src).ok_or_else(|| {
                RadError::InvalidArgument(format!("No digits to extract from \"{}\"", src))
            })?;
            if let Some(num) = captured.get(0) {
                Ok(Some(num.as_str().to_owned()))
            } else {
                Err(RadError::InvalidArgument(format!(
                    "No digits to extract from \"{}\"",
                    src
                )))
            }
        } else {
            Err(RadError::InvalidArgument(
                "num requires an argument".to_owned(),
            ))
        }
    }

    /// Capitalize text
    ///
    /// # Usage
    ///
    /// $upper(hello world)
    fn capitalize(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let src = trim!(&args[0]);
            Ok(Some(src.to_uppercase()))
        } else {
            Err(RadError::InvalidArgument(
                "cap requires an argument".to_owned(),
            ))
        }
    }

    /// Lower text
    ///
    /// # Usage
    ///
    /// $lower(hello world)
    fn lower(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let src = trim!(&args[0]);
            Ok(Some(src.to_lowercase()))
        } else {
            Err(RadError::InvalidArgument(
                "cap requires an argument".to_owned(),
            ))
        }
    }

    /// Log message
    ///
    /// # Usage
    ///
    /// $log(This is a problem)
    fn log_message(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        p.log_message(args)?;
        Ok(None)
    }

    /// Get max value from array
    ///
    /// # Usage
    ///
    /// $max(1,2,3,4,5)
    fn get_max(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let content = trim!(&args[0]);
            if content.is_empty() {
                return Err(RadError::InvalidArgument(
                    "max requires an array to process but given empty value".to_owned(),
                ));
            }
            let max = content.split(',').max().unwrap();
            Ok(Some(max.to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "cap requires an argument".to_owned(),
            ))
        }
    }

    /// Get min value from array
    ///
    /// # Usage
    ///
    /// $min(1,2,3,4,5)
    fn get_min(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let content = trim!(&args[0]);
            if content.is_empty() {
                return Err(RadError::InvalidArgument(
                    "min requires an array to process but given empty value".to_owned(),
                ));
            }
            let max = content.split(',').min().unwrap();
            Ok(Some(max.to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "cap requires an argument".to_owned(),
            ))
        }
    }

    /// Get ceiling value
    ///
    /// # Usage
    ///
    /// $ceiling(1.56)
    fn get_ceiling(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let number = trim!(&args[0]).parse::<f64>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a floating point number",
                    args[0]
                ))
            })?;
            Ok(Some(number.ceil().to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "ceil requires an argument".to_owned(),
            ))
        }
    }

    /// Get floor value
    ///
    /// # Usage
    ///
    /// $floor(1.23)
    fn get_floor(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let number = trim!(&args[0]).parse::<f64>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a floating point number",
                    args[0]
                ))
            })?;
            Ok(Some(number.floor().to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "floor requires an argument".to_owned(),
            ))
        }
    }

    /// Precision
    ///
    /// # Usage
    ///
    /// $prec(1.56,2)
    fn prec(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let number = trim!(&args[0]).parse::<f64>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a floating point number",
                    args[0]
                ))
            })?;
            let precision = trim!(&args[1]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a precision",
                    args[1]
                ))
            })?;
            let decimal_precision = 10.0f64.powi(precision as i32);
            let converted = f64::trunc(number * decimal_precision) / decimal_precision;
            let formatted = format!("{:.1$}", converted, precision);

            Ok(Some(formatted))
        } else {
            Err(RadError::InvalidArgument(
                "ceil requires an argument".to_owned(),
            ))
        }
    }

    /// Reverse array
    ///
    /// # Usage
    ///
    /// $rev(1,2,3,4,5)
    fn reverse_array(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if args.is_empty() {
            Err(RadError::InvalidArgument(
                "rev requires an argument".to_owned(),
            ))
        } else {
            let reversed = args.split(',').rev().collect::<Vec<&str>>().join(",");
            Ok(Some(reversed))
        }
    }

    /// Declare an empty macros
    ///
    /// # Usage
    ///
    /// $declare(n1,n2,n3)
    fn declare(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        let names = ArgParser::new().args_to_vec(args, ',', GreedyState::Never);
        // TODO Create empty macro rules
        let runtime_rules = names
            .iter()
            .map(|name| (trim!(name).to_string(), "", ""))
            .collect::<Vec<(String, &str, &str)>>();

        // Check overriding. Warn or yield error
        for (name, _, _) in runtime_rules.iter() {
            if processor.contains_macro(name, MacroType::Any) {
                if processor.state.behaviour == ErrorBehaviour::Strict {
                    return Err(RadError::InvalidMacroName(format!(
                        "Declaring a macro with a name already existing : \"{}\"",
                        name
                    )));
                } else {
                    processor.log_warning(
                        &format!(
                            "Declaring a macro with a name already existing : \"{}\"",
                            name
                        ),
                        WarningType::Sanity,
                    )?;
                }
            }
        }

        // Add runtime rules
        processor.add_runtime_rules(&runtime_rules)?;
        Ok(None)
    }

    /// Document a macro
    ///
    /// # Usage
    ///
    /// $document(macro,content)
    fn document(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let macro_name = &args[0];
            let content = &args[1];

            // If operation failed
            if !processor.set_documentation(macro_name, content)
                && processor.state.behaviour == ErrorBehaviour::Strict
            {
                processor.log_error(&format!("No such macro \"{}\" to document", macro_name))?;
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Docu requires two argument".to_owned(),
            ))
        }
    }

    /// Declare a local macro
    ///
    /// Local macro gets deleted after macro execution
    ///
    /// # Usage
    ///
    /// $let(name,value)
    fn bind_to_local(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let name = trim!(&args[0]);
            let value = trim!(&args[1]);
            processor.add_new_local_macro(1, &name, &value);
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Let requires two argument".to_owned(),
            ))
        }
    }

    /// Declare a local macro raw
    ///
    /// Local macro gets deleted after macro execution
    ///
    /// # Usage
    ///
    /// $letr(name,value)
    fn bind_to_local_raw(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let name = trim!(&args[0]);
            let value = &args[1];
            processor.add_new_local_macro(1, &name, value);
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Letr requires two argument".to_owned(),
            ))
        }
    }

    /// Clear volatile macros
    fn clear(_: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        processor.clear_volatile();
        Ok(None)
    }

    /// Enable/disable hygiene's macro mode
    ///
    /// # Usage
    ///
    /// $hygiene(true)
    /// $hygiene(false)
    fn toggle_hygiene(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            if let Ok(value) = Utils::is_arg_true(&args[0]) {
                processor.toggle_hygiene(value);
                Ok(None)
            }
            // Failed to evaluate
            else {
                Err(RadError::InvalidArgument(format!(
                    "hygiene requires either true/false or zero/nonzero integer, but given \"{}\"",
                    args[0]
                )))
            }
        } else {
            Err(RadError::InvalidArgument(
                "hygiene requires an argument".to_owned(),
            ))
        }
    }

    /// Pause every macro expansion
    ///
    /// Only other pause call is evaluated
    ///
    /// # Usage
    ///
    /// $pause(true)
    /// $pause(false)
    fn pause(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            if let Ok(value) = Utils::is_arg_true(&args[0]) {
                processor.state.paused = value;
                Ok(None)
            }
            // Failed to evaluate
            else {
                Err(RadError::InvalidArgument(format!(
                    "Pause requires either true/false or zero/nonzero integer, but given \"{}\"",
                    args[0]
                )))
            }
        } else {
            Err(RadError::InvalidArgument(
                "Pause requires an argument".to_owned(),
            ))
        }
    }

    /// Define a static macro
    ///
    /// # Usage
    ///
    /// $static(name,value)
    fn define_static(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let name = trim!(&args[0]);
            let value = trim!(&args[1]);
            // Macro name already exists
            if processor.contains_macro(&name, MacroType::Any) {
                // Strict mode prevents overriding
                // Return error
                if processor.state.behaviour == ErrorBehaviour::Strict {
                    return Err(RadError::InvalidMacroName(format!(
                        "Creating a static macro with a name already existing : \"{}\"",
                        name
                    )));
                } else {
                    // Its warn-able anyway
                    processor.log_warning(
                        &format!(
                            "Creating a static macro with a name already existing : \"{}\"",
                            name
                        ),
                        WarningType::Sanity,
                    )?;
                }
            }
            processor.add_static_rules(&[(&name, &value)])?;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Static requires two argument".to_owned(),
            ))
        }
    }

    /// Define a static macro raw
    ///
    /// # Usage
    ///
    /// $staticr(name,value)
    fn define_static_raw(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let name = trim!(&args[0]);
            let value = &args[1];
            // Macro name already exists
            if processor.contains_macro(&name, MacroType::Any) {
                // Strict mode prevents overriding
                // Return error
                if processor.state.behaviour == ErrorBehaviour::Strict {
                    return Err(RadError::InvalidMacroName(format!(
                        "Creating a static macro with a name already existing : \"{}\"",
                        name
                    )));
                } else {
                    // Its warn-able anyway
                    processor.log_warning(
                        &format!(
                            "Creating a static macro with a name already existing : \"{}\"",
                            name
                        ),
                        WarningType::Sanity,
                    )?;
                }
            }
            processor.add_static_rules(&[(&name, &value)])?;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Staticr requires two argument".to_owned(),
            ))
        }
    }

    /// Separate an array
    ///
    /// # Usage
    ///
    /// $sep( ,1,2,3,4,5)
    fn separate_array(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let separator = &args[0];
            let array = &args[1];
            let mut array = array.split(',');
            let mut splited = String::new();

            if let Some(first) = array.next() {
                splited.push_str(first);

                for item in array {
                    write!(splited, "{}{}", separator, item)?;
                }
            }

            Ok(Some(splited))
        } else {
            Err(RadError::InvalidArgument(
                "sep requires two argument".to_owned(),
            ))
        }
    }

    /// Change a notation of a number
    ///
    /// # Usage
    ///
    /// $notat(23,binary)
    fn change_notation(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let number = trim!(&args[0]);
            let notation = trim!(&args[1]).to_lowercase();
            let format = if let Ok(num) = number.parse::<isize>() {
                match notation.as_str() {
                    "bin" => format!("{:b}", num),
                    "oct" => format!("{:o}", num),
                    "hex" => format!("{:x}", num),
                    _ => {
                        return Err(RadError::InvalidArgument(format!(
                            "Unsupported notation format \"{}\"",
                            notation
                        )))
                    }
                }
            } else {
                return Err(RadError::InvalidArgument(
                    "Notat can only change notation of signed integer ".to_owned(),
                ));
            };
            Ok(Some(format))
        } else {
            Err(RadError::InvalidArgument(
                "Notat requires two arguments".to_owned(),
            ))
        }
    }

    /// Replace value
    ///
    /// # Usage
    ///
    /// $repl(macro,value)
    fn replace(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let name = trim!(&args[0]);
            let target = &args[1];
            if !processor.replace_macro(&name, target) {
                return Err(RadError::InvalidArgument(format!(
                    "{} doesn't exist, thus cannot replace it's content",
                    name
                )));
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Replace requires two arguments".to_owned(),
            ))
        }
    }

    /// Source static file
    ///
    /// Source file's format is mostly equivalent with env.
    /// $source(file_name.renv)
    fn source_static_file(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("source", AuthType::FIN, processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let path = &trim!(&args[0]);
            let path = Path::new(path.as_ref());
            if !path.exists() {
                return Err(RadError::InvalidArgument(format!(
                    "Cannot source non-existent file \"{}\"",
                    path.display()
                )));
            }

            processor.set_sandbox(true);

            let source_lines = std::io::BufReader::new(std::fs::File::open(path)?).lines();
            for (idx, line) in source_lines.enumerate() {
                let line = line?;
                let idx = idx + 1; // 1 starting index is more human friendly
                if let Some((name, body)) = line.split_once('=') {
                    match processor.parse_chunk_args(0, MAIN_CALLER, body) {
                        Ok(body) => processor.add_static_rules(&[(name, body)])?,
                        Err(err) => {
                            processor.log_error(&format!(
                                "Failed to source a file \"{}\" in line \"{}\"",
                                path.display(),
                                idx
                            ))?;
                            return Err(err);
                        }
                    }
                } else {
                    return Err(RadError::InvalidArgument(format!(
                        "Invalid line in source file, line \"{}\" \n = \"{}\"",
                        idx, line
                    )));
                }
            }
            processor.set_sandbox(false);
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "source requires an argument".to_owned(),
            ))
        }
    }

    /// Import a frozen file
    ///
    /// $import(file.r4f)
    fn import_frozen_file(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("import", AuthType::FIN, processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let path = &trim!(&args[0]);
            let path = Path::new(path.as_ref());
            if !path.exists() {
                return Err(RadError::InvalidArgument(format!(
                    "Cannot import from non-existent file \"{}\"",
                    path.display()
                )));
            }
            processor.import_frozen_file(path)?;

            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "import requires an argument".to_owned(),
            ))
        }
    }

    /// List directory files
    ///
    /// $listdir(path, is_abs, delimiter)
    #[cfg(not(feature = "wasm"))]
    fn list_directory_files(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("listdir", AuthType::FIN, processor)? {
            return Ok(None);
        }
        let args = ArgParser::new().args_to_vec(args, ',', GreedyState::Never);
        if args.is_empty() {
            return Err(RadError::InvalidArgument(
                "listdir at least requires an argument".to_owned(),
            ));
        }

        let absolute = match Utils::is_arg_true(&args[0]) {
            Ok(value) => value,
            Err(_) => {
                return Err(RadError::InvalidArgument(format!(
                    "listdir's first argument should be a boolean value but given : \"{}\"",
                    args[0]
                )));
            }
        };

        let path;
        if let Some(val) = args.get(1) {
            path = if val.is_empty() {
                processor.get_current_dir()?
            } else {
                PathBuf::from(val)
            };
            if !path.exists() {
                return Err(RadError::InvalidArgument(format!(
                    "Cannot list non-existent directory \"{}\"",
                    path.display()
                )));
            }
        } else {
            path = processor.get_current_dir()?
        };

        let delim = if let Some(val) = args.get(2) {
            val
        } else {
            ","
        };

        let mut vec = vec![];
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if absolute {
                vec.push(std::fs::canonicalize(entry.path().as_os_str())?);
            } else {
                vec.push(entry.file_name().into());
            }
        }

        let result: Vec<_> = vec
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>();
        Ok(Some(result.join(delim)))
    }

    /// Paste unicode character in place
    /// $unicode
    fn paste_unicode(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let unicode_character = trim!(&args[0]);
            let unicode_hex = u32::from_str_radix(&unicode_character, 16)?;
            Ok(Some(
                char::from_u32(unicode_hex)
                    .ok_or_else(|| {
                        RadError::InvalidArgument(format!(
                            "Invalid unicode value : \"{}\" (as u32)",
                            unicode_hex
                        ))
                    })?
                    .to_string(),
            ))
        } else {
            Err(RadError::InvalidArgument(
                "Unicode requires an argument".to_owned(),
            ))
        }
    }

    // END Default macros
    // ----------
    // START Feature macros

    /// Enable hook
    ///
    /// * Usage
    ///
    /// $hookon(MacroType, macro_name)
    #[cfg(feature = "hook")]
    fn hook_enable(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let hook_type = HookType::from_str(&args[0])?;
            let index = &args[1];
            processor.hook_map.switch_hook(hook_type, index, true)?;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "hookon requires two arguments".to_owned(),
            ))
        }
    }

    /// Disable hook
    ///
    /// * Usage
    ///
    /// $hookoff(MacroType, macro_name)
    #[cfg(feature = "hook")]
    fn hook_disable(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let hook_type = HookType::from_str(&args[0])?;
            let index = &args[1];
            processor.hook_map.switch_hook(hook_type, index, false)?;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "hookoff requires two arguments".to_owned(),
            ))
        }
    }

    /// Wrap text
    ///
    /// * Usage
    ///
    /// $wrap(80, Content goes here)
    #[cfg(feature = "textwrap")]
    fn wrap(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let width = trim!(&args[0]).parse::<usize>()?;
            let content = &args[1];
            let result = textwrap::fill(content, width);
            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "Wrap requires two arguments".to_owned(),
            ))
        }
    }

    fn update_storage(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_to_vec(args, ',', GreedyState::Never);

        // Execute update method for storage
        if let Some(storage) = processor.storage.as_mut() {
            if let Err(err) = storage.update(&args) {
                return Err(RadError::StorageError(format!("Update error : {}", err)));
            }
        } else {
            processor.log_warning("Empty storage, update didn't trigger", WarningType::Sanity)?;
        }
        Ok(None)
    }

    fn extract_storage(_: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        // Execute update method for storage
        if let Some(storage) = processor.storage.as_mut() {
            match storage.extract(false) {
                Err(err) => Err(RadError::StorageError(format!("Update error : {}", err))),
                Ok(value) => {
                    if let Some(output) = value {
                        Ok(Some(output.into_printable()))
                    } else {
                        Ok(None)
                    }
                }
            }
        } else {
            Err(RadError::StorageError(String::from("Empty storage")))
        }
    }

    /// Register a table
    ///
    /// $regcsv(table_name,table_content)
    #[cfg(feature = "cindex")]
    fn cindex_register(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        use cindex::ReaderOption;

        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let table_name = trim!(&args[0]);
            if processor.indexer.contains_table(&table_name) {
                return Err(RadError::InvalidArgument(format!(
                    "Cannot register exsiting table : \"{}\"",
                    args[0]
                )));
            }
            let mut option = ReaderOption::new();
            option.ignore_empty_row = true;
            processor
                .indexer
                .add_table_with_option(&table_name, args[1].as_bytes(), option)?;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "regcsv requires two arguments".to_owned(),
            ))
        }
    }

    /// Drop a table
    ///
    /// $dropcsv(table_name)
    #[cfg(feature = "cindex")]
    fn cindex_drop(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            processor.indexer.drop_table(&trim!(&args[0]));
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "regcsv requires two arguments".to_owned(),
            ))
        }
    }

    /// Execute query from indexer table
    ///
    /// $query(statment)
    #[cfg(feature = "cindex")]
    fn cindex_query(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let mut value = String::new();
            processor
                .indexer
                .index_raw(&trim!(&args[0]), OutOption::Value(&mut value))?;
            Ok(Some(trim!(&value).to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "query requires an argument".to_owned(),
            ))
        }
    }
}

// TODO
// Curently implementation declard logic and signatrue separately.
// Is this ideal?
// Or the whole process should be automated?
// Though I dought the possibility of automation because each logic is so relaxed and hardly follow
// any concrete rules
/// Function Macro signature
#[derive(Clone)]
pub(crate) struct FMacroSign {
    name: String,
    args: Vec<String>,
    pub logic: FunctionMacroType,
    #[allow(dead_code)]
    pub desc: Option<String>,
}

impl FMacroSign {
    pub fn new(
        name: &str,
        args: impl IntoIterator<Item = impl AsRef<str>>,
        logic: FunctionMacroType,
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

impl std::fmt::Display for FMacroSign {
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
impl From<&FMacroSign> for crate::sigmap::MacroSignature {
    fn from(bm: &FMacroSign) -> Self {
        Self {
            variant: crate::sigmap::MacroVariant::Function,
            name: bm.name.to_owned(),
            args: bm.args.to_owned(),
            expr: bm.to_string(),
            desc: bm.desc.clone(),
        }
    }
}
