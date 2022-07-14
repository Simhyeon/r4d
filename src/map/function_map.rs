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
#[cfg(not(feature = "wasm"))]
use std::process::Command;
#[cfg(feature = "hook")]
use std::str::FromStr;

const ALIGN_TYPES: [&str; 3] = ["left", "right", "center"];

lazy_static! {
    static ref CLRF_MATCH: Regex = Regex::new(r#"\r\n"#).unwrap();
    static ref CHOMP_MATCH: Regex = Regex::new(r#"\n\s*\n"#).expect("Failed to create chomp regex");
    // Thanks stack overflow! SRC : https://stackoverflow.com/questions/12643009/regular-expression-for-floating-point-numbers
    static ref NUM_MATCH: Regex = Regex::new(r#"[+-]?([\d]*[.])?\d+"#).expect("Failed to create number regex");
    static ref TWO_NL_MATCH: Regex = Regex::new(r#"\n\s*\n"#).expect("Failed to create tow nl regex");
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

- a_pipe_name : A name of pipe ( trimmed, optional )

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
                "align".to_owned(),
                FMacroSign::new(
                    "align",
                    ["a_type^","a_width^","a_fill^", "a_text"],
                    Self::align,
                    Some(
                        "Align texts with character filler

# Arguments

- a_type  : Types of alignment [\"Left\", \"right\", \"center\"] ( trimmed )
- a_width : Total width of aligned chunk [ Unsigned integer ] ( trimmed )
- a_fill  : A character to fill ( trimmed )
- a_text  : Text to align

# Example

$assert(Hello---,$align(left  ,8,-,Hello))
$assert(---Hello,$align(right ,8,-,Hello))
$assert(--Hello-,$align(center,8,-,Hello))".to_string(),
                    ),
                ),
            ),
            (
                "cmp".to_owned(),
                FMacroSign::new(
                    "cmp",
                    ["a_lvalue", "a_rvalue"],
                    Self::compare_values,
                    Some("Compares left value and right value

- This returns true if lvalue is \"bigger\" than rvalue
- This returns false if lvalue is \"smaller or equal\" to rvalue

# Return : Boolean

# Arguments

- a_lvalue : A left value to compare
- a_rvalue : A right value to cmpare

# Example

$assert(true,$cmp(c,b))
$assert(false,$cmp(text,text))".to_string()),
                ),
            ),
            (
                "eq".to_owned(),
                FMacroSign::new(
                    "eq",
                    ["a_lvalue", "a_rvalue"],
                    Self::are_values_equal,
                    Some("Check if given values are same

# Return : Boolean

# Arguments

- a_lvalue : A left value to compare
- a_rvalue : A right value to cmpare

# Example

$assert(false,$eq(a,b))
$assert(true,$eq(23,23))".to_string()),
                ),
            ),
            (
                "slice".to_owned(),
                FMacroSign::new(
                    "slice",
                    ["a_min^", "a_max^", "a_array"],
                    Self::slice,
                    Some("Get a slice from an aray

# Arguments

- a_min   : A start index ( trimmed )
- a_max   : A end index ( trimmed )
- a_array : An array to process

# Example

$assert(\\*2,3*\\,$slice(1,2,\\*1,2,3,4,5,6*\\))".to_string()),
                ),
            ),
            (
                "split".to_owned(),
                FMacroSign::new(
                    "spilt",
                    ["a_sep", "a_text"],
                    Self::split,
                    Some("Split text into an array

# Arguments

- a_sep  : A separator string
- a_text : Text to spilt

# Example

$assert(\\*a,b,c*\\,$split(/,a/b/c))".to_string()),
                ),
            ),
            (
                "ssplit".to_owned(),
                FMacroSign::new(
                    "sspilt",
                    ["a_text^"],
                    Self::space_split,
                    Some("Split text with space separator into an array

- This macro split text by one or more blank characters ( space, tab, newline )

# Arguments

- a_text : Text to spilt ( trimmed )

# Example

$assert(6,$count($ssplit(I have  some    spaces   in between)))".to_string()),
                ),
            ),
            (
                "squash".to_owned(),
                FMacroSign::new(
                    "squash",
                    ["a_text"],
                    Self::squash,
                    Some("Squash text by trimming all empty newlines

# Arguments

- a_text : Text to squash

# Example

$assert(a$nl()b,$squash(
a

b
))".to_string()),
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
                "comma".to_owned(),
                FMacroSign::new(
                    "comma",
                    ESR,
                    Self::print_comma,
                    Some("Print a comma

# Example

$assert(\\*,*\\,$comma())".to_string()),
                ),
            ),
            (
                "counter".to_owned(),
                FMacroSign::new(
                    "counter",
                    ["a_macro_name^","a_counter_type^+"],
                    Self::change_counter,
                    Some("Increae/decrease counter macro.

- Counter macro is automatically defined if the macro doesn't exist
- Counter's value should be a number and can be negative.

# Arguments

- a_macro_name   : A macro name to use as counter ( trimmed )
- a_counter_type : A counter opration type. Dfault is plus [ \"plus\", \"minus\" ] ( trimmed )

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

- a_number : A number to get a ceiling from [float] ( trimmed )

# Example

$assert($ceil(0.9),1)
$assert($ceil(3.1),4)".to_string()),
                ),
            ),
            (
                "chars".to_owned(),
                FMacroSign::new(
                    "chars",
                    ["a_text^"],
                    Self::chars_array,
                    Some("Get a characters array from text

# Arguments

- a_text : Text to get a chars array from ( trimmed )

# Example

$assert(\\*a,b,c,d,e*\\$chars(abcde))".to_string()),
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
                    Some("Clear volatile macros. This macro is intended to be used when hygiene mode is enabled and user wants to clear volatiles immediately without waiting.

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
                    Some("Apply both trim and chomp (compress) to contents

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
                    Some("Get counts of lines.

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
                    Some("Deny a next newline. This technically squashes following two consequent line_ending into a single one

- dnl doesn't deny right next newlie but a newline after a newline.

# Example

$assert(a$nl()b,a$dnl()

b)".to_string()),
                ),
            ),
            (
                "declare".to_owned(),
                FMacroSign::new(
                    "declare",
                    ["a_macro_names^"],
                    Self::declare,
                    Some("Declare multiple variables separated by commas

# Arguments

- a_macro_names: Macro names array ( trimmed )

# Example

$declare(first,second)
$assert($first(),$empty())".to_string()),
                ),
            ),
            (
                "docu".to_owned(),
                FMacroSign::new(
                    "docu",
                    ["a_macro_name^", "a_doc"],
                    Self::document,
                    Some("Append documents(description) to a macro. You cannot directly retreive documentation from macros but by --man flag.

# Arguments

- a_macro_name : A macro to append documentation ( trimmed )
- a_doc        : Documents to append

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
                    Some("Print empty string. Used for semantic formatting.

# Example

$assert(,$empty())".to_string()),
                ),
            ),
            (
                "enl".to_owned(),
                FMacroSign::new(
                    "enl",
                    ESR,
                    Self::escape_newline,
                    Some("Escape a following newline

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

- NOTE : This flow control only sustains for the input.

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
                    Some("Exit processing from the invocation

- NOTE : This flow control only sustains for the input

# Example

$exit()".to_string()),
                ),
            ),
            (
                "input".to_owned(),
                FMacroSign::new(
                    "input",
                    ["a_absolute?^+"],
                    Self::print_current_input,
                    Some("Print a current file input.

# Return : Path

# Arguments

- a_absolute : Whether to print an input path as absolute. Default is false [boolean] ( trimmed, optional )

# Example

$assert($input(),test)
$assert($input(true),/home/user/dir/test)".to_string()),
                ),
            ),
            (
                "isempty".to_owned(),
                FMacroSign::new(
                    "isempty",
                    ["a_value"],
                    Self::is_empty,
                    Some("Check if a given value is empty

# Return : Boolean

# Arguments

- a_value : Value to qualify

# Example

$assert(true,$isempty())
$assert(false,$isempty( ))".to_string()),
                ),
            ),
            (
                "istype".to_owned(),
                FMacroSign::new(
                    "istype",
                    ["a_value^","a_type^"],
                    Self::qualify_value,
                    Some("Check if a given value is a type

# Return : Boolean

# Arguments

- a_value : Value to qualify ( trimmed )
- a_type  : Type of qualification [\"uint\",\"int\",\"float\",\"bool\"] ( trimmed )

# Example

$assert(true,$istype(  0,  uint))
$assert(false,$istype(-1,  uint))
$assert(true,$istype(  0,  int))
$assert(false,$istype(-0.1,uint))
$assert(true,$istype( -1,  int))
$assert(false,$istype( 0.1,int))
$assert(true,$istype( -0.1,float))
$assert(true,$istype( -0,  float))
$assert(true,$istype(  0,  float))
$assert(true,$istype(  0,  bool))
$assert(true,$istype(  1,  bool))".to_string()),
                ),
            ),
            (
                "iszero".to_owned(),
                FMacroSign::new(
                    "iszero",
                    ["a_value^"],
                    Self::is_zero,
                    Some("Check if a given value is a zero

# Return : Boolean

# Arguments

- a_value : Value to qualify

# Example

$assert(true,$iszero(0))
$assert(false,$iszero(1))".to_string()),
                ),
            ),
            (
                "find".to_owned(),
                FMacroSign::new(
                    "find",
                    ["a_expr", "a_source"],
                    Self::find_occurence,
                    Some("Check an occurrence of expression from source.

# Return : Boolean

# Arguments

- a_expr   : An expression to match
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
                    Some("Get occurrences of expression from source. This returns 0 if there are no occurrences.

# Return : Unsigned integer

# Arguments

- a_expr   : An expression to match
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
                    Some("Get a floor integer from a given number

# Return : Signed integer

# Arguments

- a_number : A number to get a floor from [float] ( trimmed )

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
                    ["a_expr", "a_array"],
                    Self::grep_array,
                    Some("Extract matched items from given array. This returns all items as array

# Arguments

- a_expr  : A regex expression to match
- a_lines : An array to get matches from

# Example

$assert(\\*a,b,c*\\,$grep([a-z],a,b,c,1,2))".to_string()),
                ),
            ),
            (
                "grepl".to_owned(),
                FMacroSign::new(
                    "grepl",
                    ["a_expr", "a_lines"],
                    Self::grep_lines,
                    Some("Extract matched lines from given lines. This returns all lines that matches given expression

# Arguments

- a_expr  : A regex expression to match
- a_lines : Lines to get matches from

# Example

$assert(2,$countl($grepl(Cargo,$syscmd(ls))))".to_string()),
                ),
            ),
            (
                "halt".to_owned(),
                FMacroSign::new(
                    "halt",
                    ESR,
                    Self::halt_relay,
                    Some("Halt relaying

- NOTE : Halt is automatically queued by default. Feed an optional argument to configure this behaviour
- $halt(false) == $halt()
- use $halt(true) to immediately halt

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
                    ["a_hygiene?^"],
                    Self::toggle_hygiene,
                    Some("Toggle hygiene mode. This enables macro hygiene.

- On \"macro\" hygiene, every newly defined runtime macro is cleared after a first level macro invocation.

# Arguments

- a_hygiene : Whether to enable macro hygiene mode [boolean] (trimmed)

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

- a_indenter : An expression to put before lines
- a_lines    : Lines to prepend indenter

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

- A positive integer works as a normal index number
- A negative integer works as an index from end ( -1 == len -1 )

# Arguments

- a_index : An index to get [Signed integer] ( trimmed )
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

- Import always include the macros as non-volatile form, thus never cleared unless accessed from library

# Arguments

- a_file: A file name to import from [path] (trimmed)

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

- a_sep   : A separator used for joining
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

- a_sep   : A separator used for joining
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
                    Some("Get a length of text. This counts utf8 characters not ascii.

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
                    Some("Bind a local macro. Every local macro gets removed after a first level macro expansion ends.

# Arguments

- a_macro_name : A macro name to create ( trimmed )
- a_value      : A value to bind to ( trimmed )

# Example

$define(let_test=
    $let(lc,
        --Bound Value--
    )
    $assert(1,$countl($lc()))
)
$let_test()
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
                    Some("Bind a local macro with raw value. Every local macro gets removed after a first level macro expansion ends.

# Arguments

- a_macro_name : A macro name to make ( trimmed )
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
                    Some("Create placeholder text. The order of placeholder is always same.

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

- a_msg : A message to log to console

# Example

$log($value_i_want_to_check^())".to_string()),
                ),
            ),
            (
                "loge".to_owned(),
                FMacroSign::new(
                    "loge",
                    ["a_msg"],
                    Self::log_error_message,
                    Some("Log an error message to console

- This prints error in non-breaking manner. Even in strict mode, this doesn't occur a panick.

# Arguments

- a_msg : An error message to log to console

# Example

$loge(This should not be reached)".to_string()),
                ),
            ),
            (
                "lower".to_owned(),
                FMacroSign::new(
                    "lower",
                    ["a_text"],
                    Self::lower,
                    Some("Get lowercase english text

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
                    Some("Get a max value from a given array

# Arguments

- a_array : An array to get the highest value from

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
                    Some("Get a min value from a given array

 # Arguments

- a_array : An array to get the lowest value from

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

- a_path : A path to get a name from

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
                    Some("Compare left and right values. Panics if values are equal

# Arguments

- a_lvalue : A left  value
- a_rvalue : A right value

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
                    Some("Returns a negated value of a given boolean. Yields error when a given value is not a boolean

# Return : boolean

# Arguments

- a_boolean : A boolean value to negate [boolean] ( trimmed )

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
                    Some("Extract number parts from given text. If there are multiple numbers, only extract the first

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
                    Some("Print platform specific newlines. Its behaviour can be configured.

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
                    ["a_number^", "a_type^"],
                    Self::change_notation,
                    Some("Chagne notation of a number

# Arguments

- a_number   : A number to change notation
- a_type     : A type of notation [\"bin\",\"oct\",\"hex\"] ( trimmed )

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
                    Some("Panics manually with a message

# Arguments

- a_msg : A message to print as an error

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

- NOTE : This yields an error if a path is a root and will return an empty value, but not a none value if a path is a single node.

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
- Paths with colliding separator cannot be merged.
    e.g) a/ + /b cannot be merged

# Return : path

# Arguments

- a_array : A path array to merge ( trimmed )

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

- NOTE : Pause is not flow control but a processor state, thus the state will sustain for the whole processing.

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
                    "pipeto",
                    ["a_pipe_name^", "a_value"],
                    Self::pipe_to,
                    Some("Pipe a given value to a named pipe

# Arguments

- a_pipe_name : A name of pipe container ( trimmed )
- a_value     : A value to pipe

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
                    Some("Convert a float number with given precision

# Return : Float

# Arguments

- a_number    : A number to process ( trimmed )
- a_precision : A precision number to apply to number ( trimmed )

# Example

$assert(0.30,$prec($eval(0.1 + 0.2),2))".to_string()),
                ),
            ),
            (
                "relay".to_owned(),
                FMacroSign::new(
                    "relay",
                    ["a_target_type^", "a_target^"],
                    Self::relay,
                    Some("Start relaying to a target. Relay redirects all following text to the relay target.

# Auth : FOUT is required for relay target \"file\" and \"temp\"

# Arguments

- a_target_type : A type of a relay target [\"macro\",\"file\", \"temp\"] (trimmed)
- a_target      : A name of a target. Ignored in temp type ( trimmed )

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
                    Some("Reverse order of an array

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
                    Some("Apply a regular expression substitution to a source

# Arguments

- a_expr   : A regex expression to match
- a_sub    : Text to substitute as
- a_source : Source text to operate on

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
                    Some("Register a regular expression

- NOTE : A registered name will not be able to be matched directly
- Every regex operation creates regex cache, while registered expression will not be cached but saved permanently. Unregistered caches will be cleared if certain capacity reaches.

# Arguments

- a_name : A name of the regex expression. This is not trimmed
- a_epxr : An expression to bind to

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

- a_macro_name : A macro to change name ( trimmed )
- a_new_name   : A new macro name to apply ( trimmed )

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
                    Some("Repeat given source by given counts

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

- a_macro_name : A macro name to replace ( trimmed )
- a_new_value  : A new value to put

# Example

$define(demo=Demo)
$assert(Demo,$demo())
$repl(demo,DOMO)
$assert(DOMO,$demo())".to_string()),
                ),
            ),
            (
                "source".to_owned(),
                FMacroSign::new(
                    "source",
                    ["a_file^"],
                    Self::source_static_file,
                    Some("Source an env file. The sourced file is eagerly expanded (As if it was static defined)

Syntax of source-able file is same with .env file

e.g)
demo=DEMO
number=$eval(1+2)

# Arguments

- a_file : A file to source ( trimmed )

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

- a_sort_type : A sort type [\"asec\",\"desc\"] (trimmed)
- a_array     : An array to sort

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

- a_sort_type : A sort type [\"asec\",\"desc\"] (trimmed)
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
                    Some("Create a static macro. A static macro is eagerly expanded unlike define

# Arguments

- a_macro_name : A macro to create ( trimmed )
- a_expr       : An expression to bind to ( trimmed )

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
                    Some("Create a static macro with raw value. A static macro is eagerly expanded unlike define

# Arguments

- a_macro_name : A macro to create ( trimmed )
- a_expr       : An expression to bind to which is not trimmed

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

- Out of range index is an error
- A substring is calculated as char iterator not a byte iterator
- this operation is technically same with [start_index..end_index]

# Arguments

- a_start_index : A start substring index [Unsigned integer] (trimmed)
- a_end_index   : A end   substring index [Unsigned integer] (trimmed)
- a_source      : Source text get to a substring from

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
                    Some("Surround given contents with a given pair

# Arguments

- a_start_pair : A start pair
- a_end_pair   : A end pair
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
                    Self::print_tab,
                    Some("Print tabs

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

- a_table_form : A table format [ \"github\", \"html\", \"wikitext\" ] ( trimmed )
- a_csv_value  : A value to convert to table ( trimmed )

# Example

$assert=(
    |a|b|c|
    |-|-|-|
    |1|2|3|,$enl()
    $table(github,a,b,
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
                    ["a_trim_option^","a_lines"],
                    Self::trimla,
                    Some("Triml with given amount

- Trims by line but with given amount.
- If given an integer, it will try to trim blank characters as much as given amount
- min trims by minimal amount that can be applied to total lines
- max acts same as triml
- Tab character is treated as a single character. Don't combine spaces and tabs for this macro

# Arguments

- a_trim_option : A trim behaviour [\"min\", \"max\", Unsigned integer] ( trimmed )
- a_lines       : Lines to trim

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
- \"Define\" macro cannot be undefined
- Undef doesn't yield error when a macro doesn't exist

# Arguments

- a_macro_name : A name of a macro to undefine ( trimmed )

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
                    Some("Creates a unicode character from a hex number without prefix

# Arguments

- a_value : A value to convert to a unicode character

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
                    Some("Get uppercase english text

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

- a_env_name : An environment variable name to get (trimmed)

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

- a_env_name  : An environment variable name to set (trimmed)
- a_env_value : A value to set

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
                    ["a_path^"],
                    Self::absolute_path,
                    Some(
                        "Get an absolute path. This requires a path to be a real path.

# Auth : FIN

# Return : path

# Arguments

- a_path : A path to make it absolute ( trimmed )

# Example

$assert(/home/user/cwd/test.md,$abs(test.md))"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "grepf".to_owned(),
                FMacroSign::new(
                    "grepf",
                    ["a_expr", "a_file^"],
                    Self::grep_file,
                    Some(
                        "Extract matched lines from given file. This returns all items as lines

- NOTE : The grep operation is executed on per line

# Arguments

- a_expr  : A regex expression to match
- a_lines : A file get matches from

# Example

$countl($grepf(file.txt))"
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
                        "Execute a sysctem command

- Each system command is executed as subprocess of folloiwng platform procedures
- Windows : cmd /C
- *Nix    : sh -c

# Auth : CMD

# Arguments

- a_command : A command to exectute

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

- A default temporary path is folloiwng
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

- A default temporary path is folloiwng
- Windows : It depends, but %APPDATA%\\Local\\Temp\\rad.txt can be one
- *nix    : /tmp/rad.txt

# Auth: FOUT

# Arguments

- a_content : Content to write to a temporary file

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
                        "Change a temporary file path

- NOTE : A temporary file name is merged to a temporary directory. You cannot set a temporary file outside of a temporary directory.
- This macro needs FOUT permission because it creates a temporary file if the file doesn't exist

# Auth: FOUT

# Arguments

- a_filename : A new temporary file path ( trimmed )

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
                        "Get a temporary file path

- A default temporary path is folloiwng
- Windows : It depends, but %APPDATA%\\Local\\Temp\\rad.txt can be one
- *nix    : /tmp/rad.txt

# Auth: FIN

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

- a_filename : A file name to read ( trimmed )
- a_raw_mode : Whehter to escape the read. A default is false [boolean] ( trimmed,optional )

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

- a_filename : A file name to write ( trimmed )
- a_truncate : Whether to truncate before writing [boolean] ( trimmed )
- a_content  : Content to write to the file

# Example

$fileout(/tmp/some_file.txt,true,Hello World)"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "listdir".to_owned(),
                FMacroSign::new(
                    "listdir",
                    ["a_path^+", "a_absolute?^+", "a_delim+"],
                    Self::list_directory_files,
                    Some(
                        "List a directory's files as csv.

- A default path is a current working directory.
- A defualt delimiter is comma.

# Auth : FIN

# Arguments

- a_path     : A directory path to list files (optional, trimmed)
- a_absolute : Whether to print files as absolute form [boolean] (trimmed, optional)
- a_delim    : A delimiter to put between items (optional)

# Example

$assert(15,$count($listdir()))
$listdir(/tmp,true)
$listdir(/tmp,true,|)"
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
                    ["a_table_name^", "a_data^"],
                    Self::cindex_register,
                    Some(
                        "Register a csv table

- Querying can be only applied to registered table.

# Arguments

- a_table_name : A table name to be registered ( trimmed )
- a_data       : Csv data ( trimmed )

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

- a_query : A query statement ( trimmed )

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
                    Some(
                        "Get current local time

# Example

% HH:mm:ss
$time()"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "date".to_owned(),
                FMacroSign::new(
                    "date",
                    ESR,
                    Self::date,
                    Some(
                        "Get current local date without timezone

# Example

% yyyy-MM-dd
$date()"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "hms".to_owned(),
                FMacroSign::new(
                    "hms",
                    ["a_second^"],
                    Self::hms,
                    Some(
                        "Get given sesconds in hh:mm:ss format

# Arguments

- a_second : Seconds to convert ( trimmed )

# Example

$assert(00:33:40,$hms(2020))"
                            .to_string(),
                    ),
                ),
            );
        }
        #[cfg(not(feature = "wasm"))]
        #[cfg(feature = "chrono")]
        {
            map.insert(
                "ftime".to_owned(),
                FMacroSign::new(
                    "ftime",
                    ["a_file"],
                    Self::get_file_time,
                    Some(
                        "Get a file's last modified time.

# Arguments

- a_file : A file to get last modified time ( trimmed )

# Example

$ftime(some_file.txt)
% 2022-07-07 19:07:06"
                            .to_string(),
                    ),
                ),
            );
        }
        #[cfg(feature = "evalexpr")]
        {
            map.insert(
                "eval".to_owned(),
                FMacroSign::new(
                    "eval",
                    ["a_expr"],
                    Self::eval,
                    Some(
                        "Evaluate a given expression

- This macro redirects expression to evalexpr crate

# Arguments

- a_expr : An expression to evaluate

# Example

$assert(3,$eval(1 + 2))"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "evalk".to_owned(),
                FMacroSign::new(
                    "evalk",
                    ["a_expr"],
                    Self::eval_keep,
                    Some(
                        "Evaluate an expression while keeping source text

- This macro redirects an expression to evalexpr crate

# Arguments

- a_expr : An expression to evaluate

# Example

$assert(1 + 2 = 3,$evalk(1 + 2 ))"
                            .to_string(),
                    ),
                ),
            );
        }
        #[cfg(feature = "textwrap")]
        map.insert(
            "wrap".to_owned(),
            FMacroSign::new(
                "wrap",
                ["a_width^", "a_text"],
                Self::wrap,
                Some(
                    "Wrap text by width

# Arguments

- a_width : A width(chars) of given texts ( trimmed )
- a_text  : Text to wrap

# Example

$assert(\\*Lorem ipsum
dolor sit amet,
consectetur
adipiscing elit. In
rhoncus*\\,$wrap(20,$lipsum(10)))"
                        .to_string(),
                ),
            ),
        );

        #[cfg(feature = "hook")]
        {
            map.insert(
                "hookon".to_owned(),
                FMacroSign::new(
                    "hookon",
                    ["a_macro_type^", "a_target_name^"],
                    Self::hook_enable,
                    Some("Enable hook which is enabled by library extension".to_string()),
                ),
            );
            map.insert(
                "hookoff".to_owned(),
                FMacroSign::new(
                    "hookoff",
                    ["a_macro_type^", "a_target_name^"],
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
                    Some(
                        "Update a storage

# Arguments

- a_text : Text to update into a storage

# Example

$update(text to be pushed)"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "extract".to_owned(),
                FMacroSign::new(
                    "extract",
                    ESR,
                    Self::extract_storage,
                    Some(
                        "Extract from a storage

# Example

$extract()"
                            .to_string(),
                    ),
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

            if match_expr.is_empty() {
                return Err(RadError::InvalidArgument(
                    "Regex expression cannot be a empty string".to_string(),
                ));
            }

            let reg = p.try_get_or_insert_regex(match_expr)?;
            Ok(Some(reg.replace_all(source, substitution).to_string()))
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

    /// Get a last modified time from a file
    ///
    /// # Usage
    ///
    /// $ftime(file_name.txt)
    #[cfg(not(feature = "wasm"))]
    #[cfg(feature = "chrono")]
    fn get_file_time(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("ftime", AuthType::FIN, processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let file = trim!(&args[0]);
            let path = Path::new(file.as_ref());
            if !path.exists() {
                return Err(RadError::InvalidArgument(format!(
                    "Cannot get a filetime from a non-existent file : \"{}\"",
                    path.display()
                )));
            }
            let time: chrono::DateTime<chrono::Utc> = std::fs::metadata(path)?.modified()?.into();
            Ok(Some(time.format("%Y-%m-%d %H:%m:%S").to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "ftime requires an argument".to_owned(),
            ))
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

            if match_expr.is_empty() {
                return Err(RadError::InvalidArgument(
                    "Regex expression cannot be a empty string".to_string(),
                ));
            }

            let reg = p.try_get_or_insert_regex(match_expr)?;
            Ok(Some(reg.is_match(source).to_string()))
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

            if match_expr.is_empty() {
                return Err(RadError::InvalidArgument(
                    "Regex expression cannot be a empty string".to_string(),
                ));
            }

            let reg = p.try_get_or_insert_regex(match_expr)?;
            Ok(Some(reg.find_iter(source).count().to_string()))
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
            let formula = &args[0];
            let result = format!("{}= {}", formula, evalexpr::eval(formula)?);
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
            let option = trim!(&args[0]);
            let source = &args[1];
            let mut try_amount = None;
            let min_amount = match option.as_ref() {
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
            let raw_file = trim!(&args[0]);
            let mut raw_include = false;
            let file_path = PathBuf::from(raw_file.as_ref());

            if file_path.is_file() {
                let canonic = file_path.canonicalize()?;

                Utils::check_file_sanity(processor, &canonic)?;
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
    #[cfg(not(feature = "wasm"))]
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

    /// Squash
    ///
    /// # Usage
    ///
    /// $squash(/,a/b/c)
    fn squash(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let text = trim!(&args[0]);
            let new_text = TWO_NL_MATCH.replace_all(&text, &p.state.newline);

            Ok(Some(new_text.to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "Squash requires an argument".to_owned(),
            ))
        }
    }

    /// Split
    ///
    /// # Usage
    ///
    /// $split(/,a/b/c)
    fn split(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let sep = &args[0];
            let text = &args[1];

            let mut result = text.split(sep).fold(String::new(), |mut acc, v| {
                write!(acc, "{},", v).unwrap();
                acc
            });
            result.pop();
            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "Split requires two arguments".to_owned(),
            ))
        }
    }

    /// Ssplit
    ///
    /// # Usage
    ///
    /// $ssplit(a/b/c)
    fn space_split(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let text = trim!(&args[0]);

            let mut result = text.split_whitespace().fold(String::new(), |mut acc, v| {
                write!(acc, "{},", v).unwrap();
                acc
            });
            result.pop();
            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "Ssplit requires an argument".to_owned(),
            ))
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
    fn manual_panic(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        p.state.behaviour = ErrorBehaviour::Interrupt;
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
    fn print_tab(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        let count = if !args.is_empty() {
            trim!(args)
                .parse::<usize>()
                .map_err(|_| RadError::InvalidArgument("tab requires number".to_string()))?
        } else {
            1
        };

        Ok(Some("\t".repeat(count)))
    }

    /// Print a literal comma
    ///
    /// # Usage
    ///
    /// $comma()
    fn print_comma(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(",".to_string()))
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
    #[cfg(not(feature = "wasm"))]
    fn absolute_path(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("abs", AuthType::FIN, p)? {
            return Ok(None);
        }

        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let path = std::fs::canonicalize(p.get_current_dir()?.join(trim!(&args[0]).as_ref()))?
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

    /// Ailgn texts
    ///
    /// # Usage
    ///
    /// $align(center,10,a,Content)
    fn align(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 4) {
            let align_type = trim!(&args[0]).to_lowercase();

            if ALIGN_TYPES
                .iter()
                .filter(|&&x| x == align_type.as_str())
                .count()
                == 0
            {
                return Err(RadError::InvalidArgument(format!(
                    "Align type should be among left, right or center but given {}",
                    align_type
                )));
            }

            let width = trim!(&args[1]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Align requires positive integer number as width but got \"{}\"",
                    &args[1]
                ))
            })?;
            let filler: &str = args[2].as_ref();
            let text = trim!(&args[3]);
            let filler_char: String;

            if filler.is_empty() {
                return Err(RadError::InvalidArgument(
                    "Filler cannot be empty".to_string(),
                ));
            }

            let next_char = if filler == " " {
                Some(' ')
            } else {
                filler.chars().next()
            };

            if let Some(ch) = next_char {
                if ch == '\r' || ch == '\n' {
                    return Err(RadError::InvalidArgument(
                        "Filler cannot be a newline character".to_string(),
                    ));
                } else {
                    filler_char = ch.to_string();
                }
            } else {
                return Err(RadError::InvalidArgument(
                    "Filler should be an valid utf8 character".to_string(),
                ));
            }

            let text_length = text.chars().count();
            if width < text_length {
                return Err(RadError::InvalidArgument(
                    "Width should be bigger than source texts".to_string(),
                ));
            }

            let space_count = width - text_length;

            let formatted = match align_type.as_str() {
                "left" => format!("{0}{1}", text, &filler_char.repeat(space_count)),
                "right" => format!("{1}{0}", text, &filler_char.repeat(space_count)),
                "center" => {
                    let right_sp = space_count / 2;
                    let left_sp = space_count - right_sp;
                    format!(
                        "{1}{0}{2}",
                        text,
                        &filler_char.repeat(left_sp),
                        &filler_char.repeat(right_sp)
                    )
                }
                _ => unreachable!(),
            };

            Ok(Some(formatted))
        } else {
            Err(RadError::InvalidArgument(
                "Align requires four arguments".to_owned(),
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
    /// $fileout(file_name,true,Content)
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
                // This doesn't use canonicalize, because fileout can write file to non-existent
                // file. Thus canonicalize can possibly yield error
                let path = std::env::current_dir()?.join(file_name.as_ref());
                if path.exists() && !path.is_file() {
                    return Err(RadError::InvalidArgument(format!(
                        "Failed to write \"{}\". Fileout cannot write to a directory",
                        path.display()
                    )));
                }
                if path.exists() {
                    Utils::check_file_sanity(p, &path)?;
                }
                let mut target_file = if truncate {
                    OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(path)?
                } else {
                    if !path.exists() {
                        return Err(RadError::InvalidArgument(format!("Failed to write \"{}\". Fileout without truncate option needs exsiting non-directory file",path.display())));
                    }

                    OpenOptions::new().append(true).open(path)?
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
                "Fileout requires three arguments".to_owned(),
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
                "head requires two arguments".to_owned(),
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
                "headl requires two arguments".to_owned(),
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
                "tail requires two arguments".to_owned(),
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
                "taill requires two arguments".to_owned(),
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
                "sort requires two arguments".to_owned(),
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
                "sortl requires two arguments".to_owned(),
            ))
        }
    }

    // [1 2 3]
    //  0 1 2
    //  -3-2-1

    /// Index array
    ///
    /// # Usage
    ///
    /// $index(1,1,2,3,4,5)
    fn index_array(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let content = &mut args[1].split(',').collect::<Vec<&str>>();
            let index = trim!(&args[0]).parse::<isize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "index requires to be an integer but got \"{}\"",
                    &args[0]
                ))
            })?;

            if index >= content.len() as isize || index < -(content.len() as isize) {
                return Err(RadError::InvalidArgument(format!(
                    "Index out of range. Given index is \"{}\" but array length is \"{}\"",
                    index,
                    content.len()
                )));
            }

            let final_index = if index < 0 {
                content.len() + index as usize
            } else {
                index.max(0) as usize
            };

            if content.len() <= final_index {
                return Err(RadError::InvalidArgument(format!(
                    "Index out of range. Given index is \"{}\" but array length is \"{}\"",
                    index,
                    content.len()
                )));
            }

            Ok(Some(content[final_index].to_owned()))
        } else {
            Err(RadError::InvalidArgument(
                "index requires two arguments".to_owned(),
            ))
        }
    }

    /// Get a sliced array
    ///
    /// # Usage
    ///
    /// $slice(1,2,1,2,3,4,5)
    fn slice(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let mut min: Option<usize> = None;
            let mut max: Option<usize> = None;

            let start_src = trim!(&args[0]);
            let end_src = trim!(&args[1]);

            if let Ok(num) = start_src.parse::<usize>() {
                min.replace(num);
            } else if !start_src.is_empty() {
                return Err(RadError::InvalidArgument(format!("Silce's min value should be non zero positive integer or empty value but given \"{}\"", start_src)));
            }

            if let Ok(num) = end_src.parse::<usize>() {
                max.replace(num);
            } else if !end_src.is_empty() {
                return Err(RadError::InvalidArgument(format!("Slice's max value should be non zero positive integer or empty value but given \"{}\"", end_src)));
            }

            let content = &args[2].split(',').collect::<Vec<_>>();

            Ok(Some(
                content[min.unwrap_or(0)..=max.unwrap_or(content.len() - 1)].join(","),
            ))
        } else {
            Err(RadError::InvalidArgument(
                "Slice requires three arguments".to_owned(),
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
                "regexpr requires two arguments".to_owned(),
            ))
        }
    }

    /// Grep
    ///
    /// # Usage
    ///
    /// $grep(EXPR,CONTENT)
    fn grep_array(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let expr = &args[0];
            let reg = p.try_get_or_insert_regex(expr)?;
            let content = args[1].split(',').collect::<Vec<_>>();
            let grepped = content
                .iter()
                .filter(|l| reg.is_match(l))
                .copied()
                .collect::<Vec<&str>>()
                .join(",");
            Ok(Some(grepped))
        } else {
            Err(RadError::InvalidArgument(
                "grep requires two arguments".to_owned(),
            ))
        }
    }

    /// Grepl
    ///
    /// # Usage
    ///
    /// $grepl(EXPR,CONTENT)
    fn grep_lines(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let expr = &args[0];
            let reg = p.try_get_or_insert_regex(expr)?;
            let content = args[1].lines();
            let grepped = content
                .filter(|l| reg.is_match(l))
                .collect::<Vec<&str>>()
                .join(&p.state.newline);
            Ok(Some(grepped))
        } else {
            Err(RadError::InvalidArgument(
                "grepl requires two arguments".to_owned(),
            ))
        }
    }

    /// Grepf
    ///
    /// # Usage
    ///
    /// $grepf(EXPR,CONTENT)
    fn grep_file(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("grepf", AuthType::FIN, p)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let file = trim!(&args[1]);
            let path = Path::new(file.as_ref());

            if path.exists() {
                let canonic = path.canonicalize()?;
                Utils::check_file_sanity(p, &canonic)?;
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "grepf requires a real file to read from but \"{}\" doesn't exist",
                    file
                )));
            };

            let expr = &args[0];
            let reg = p.try_get_or_insert_regex(expr)?;
            let file_stream = std::fs::File::open(path)?;
            let reader = std::io::BufReader::new(file_stream);

            let mut vec = vec![];
            for line in reader.lines() {
                let line = line?;
                if reg.is_match(&line) {
                    vec.push(line);
                }
            }

            Ok(Some(vec.join(&p.state.newline)))
        } else {
            Err(RadError::InvalidArgument(
                "grep requires two arguments".to_owned(),
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
        let args = ArgParser::new().args_to_vec(args_src, ',', GreedyState::Never);
        if args.is_empty() {
            return Err(RadError::InvalidArgument(
                "relay at least requires an argument".to_owned(),
            ));
        }

        p.log_warning(
            &format!("Relaying text content to \"{}\"", args_src),
            WarningType::Security,
        )?;

        let raw_type = trim!(&args[0]);
        let target = if let Some(t) = args.get(1) {
            trim!(t).to_string()
        } else {
            String::new()
        };
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
                let file_target = FileTarget::with_truncate(Path::new(&target))?;
                RelayTarget::File(file_target)
            }
            "macro" => {
                if target.is_empty() {
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
    /// This forcefully merge paths
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
            let path = &std::env::temp_dir().join(trim!(&args[0]).as_ref());
            Utils::check_file_sanity(processor, path)?;
            processor.set_temp_file(path)?;
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
        if !Utils::is_granted("temp", AuthType::FIN, processor)? {
            return Ok(None);
        }
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
        let args = ArgParser::new().strip(args);
        p.log_message(&args)?;
        Ok(None)
    }

    /// Log error message
    ///
    /// # Usage
    ///
    /// $loge(This is a problem)
    fn log_error_message(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().strip(args);
        p.print_error(&args)?;
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
            let macro_name = trim!(&args[0]);
            let content = &args[1];

            // If operation failed
            if !processor.set_documentation(&macro_name, content)
                && processor.state.behaviour == ErrorBehaviour::Strict
            {
                processor.log_error(&format!("No such macro \"{}\" to document", macro_name))?;
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Docu requires two arguments".to_owned(),
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
                "Let requires two arguments".to_owned(),
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
                "Letr requires two arguments".to_owned(),
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
                "Static requires two arguments".to_owned(),
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
                "Staticr requires two arguments".to_owned(),
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

    /// cmp : is lvalue bigger than rvalue
    ///
    /// # Usage
    ///
    /// $cmp(lvalue, rvalue)
    fn compare_values(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let lvalue = &args[0];
            let rvalue = &args[1];
            Ok(Some((lvalue > rvalue).to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "cmp requires two arguments".to_owned(),
            ))
        }
    }

    /// eq : are values equal
    ///
    /// # Usage
    ///
    /// $eq(lvalue, rvalue)
    fn are_values_equal(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let lvalue = &args[0];
            let rvalue = &args[1];
            Ok(Some(lvalue.eq(rvalue).to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "cmp requires two arguments".to_owned(),
            ))
        }
    }

    /// isempty : Check if value is empty
    ///
    /// # Usage
    ///
    /// $isempty(value)
    fn is_empty(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let value = &args[0];
            Ok(Some(value.is_empty().to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "isempty requires an argument".to_owned(),
            ))
        }
    }

    /// iszero : Check if value is zero
    ///
    /// # Usage
    ///
    /// $iszero(value)
    fn is_zero(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let value = trim!(&args[0]);
            Ok(Some(value.as_ref().eq("0").to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "iszero requires an argument".to_owned(),
            ))
        }
    }

    /// istype : Qualify a value
    ///
    /// # Usage
    ///
    /// $istype(value,type)
    fn qualify_value(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let value = trim!(&args[0]);
            let qtype = trim!(&args[1]);
            let qualified = match qtype.to_lowercase().as_str() {
                "uint" => value.parse::<usize>().is_ok(),
                "int" => value.parse::<isize>().is_ok(),
                "float" => value.parse::<f64>().is_ok(),
                "bool" => Utils::is_arg_true(&value).is_ok(),
                _ => {
                    return Err(RadError::InvalidArgument(format!(
                        "Given type \"{}\" is not valid",
                        &qtype
                    )));
                }
            };
            Ok(Some(qualified.to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "istype requires two arguments".to_owned(),
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

        let absolute = if let Some(val) = args.get(1) {
            match Utils::is_arg_true(val) {
                Ok(value) => value,
                Err(_) => {
                    return Err(RadError::InvalidArgument(format!(
                        "listdir's second argument should be a boolean value but given : \"{}\"",
                        args[0]
                    )));
                }
            }
        } else {
            false
        };

        let path;
        if let Some(val) = args.get(0) {
            path = if val.is_empty() {
                processor.get_current_dir()?
            } else {
                PathBuf::from(trim!(val).as_ref())
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

    /// Get characters array
    ///
    /// $chars(abcde)
    fn chars_array(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let arg = trim!(&args[0]);
            let mut chars = arg.as_ref().chars().fold(String::new(), |mut acc, ch| {
                write!(acc, "{},", ch).unwrap();
                acc
            });
            chars.pop();
            Ok(Some(chars))
        } else {
            Err(RadError::InvalidArgument(
                "chars requires an argument".to_owned(),
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
            let hook_type = HookType::from_str(&trim!(&args[0]))?;
            let index = trim!(&args[1]);
            processor.hook_map.switch_hook(hook_type, &index, true)?;
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
            let hook_type = HookType::from_str(&trim!(&args[0]))?;
            let index = trim!(&args[1]);
            processor.hook_map.switch_hook(hook_type, &index, false)?;
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
            processor.indexer.add_table_with_option(
                &table_name,
                trim!(&args[1]).as_bytes(),
                option,
            )?;
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
