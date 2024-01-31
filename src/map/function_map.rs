//! # Function macro module
//!
//! Function macro module includes struct and methods related to function macros
//! which are technically function pointers.

use crate::common::{MacroAttribute, RadResult};
use crate::consts::ESR;
use crate::extension::{ExtMacroBody, ExtMacroBuilder};
use crate::{man_fun, Processor};
#[cfg(feature = "rustc_hash")]
use rustc_hash::FxHashMap as HashMap;
#[cfg(not(feature = "rustc_hash"))]
use std::collections::HashMap;
use std::iter::FromIterator;

/// Function signature for "function" macro functions
// pub(crate) type FunctionMacroType = fn(&str, &mut Processor) -> RadResult<Option<String>>;
pub(crate) type FunctionMacroType =
    fn(&str, &MacroAttribute, &mut Processor) -> RadResult<Option<String>>;

#[derive(Clone)]
/// Collection map for a "function" macro function
pub(crate) struct FunctionMacroMap {
    pub(crate) macros: HashMap<String, FMacroSign>,
}

impl FunctionMacroMap {
    /// Creates empty map
    pub fn empty() -> Self {
        Self {
            macros: HashMap::default(),
        }
    }

    /// Creates new function macro hashmap
    ///
    /// Optional macros are included only when a feature is enabled
    pub fn new() -> Self {
        // Create hashmap of functions
        let mut map = HashMap::from_iter(IntoIterator::into_iter([
            (
                "-".to_owned(),
                FMacroSign::new(
                    "-",
                    ["a_pipe_name?^"],
                    Self::get_pipe,
                    Some(man_fun!("pipe.r4d")),
                ),
            ),
            (
                "PS".to_owned(),
                FMacroSign::new(
                    "PS",
                    ESR,
                    Self::path_separator,
                    Some(man_fun!("PS.r4d")),
                ),
            ),
            (
                "after".to_owned(),
                FMacroSign::new(
                    "after",
                    ["a_pattern", "a_content"],
                    Self::get_slice_after,
                    Some(man_fun!("after.r4d"))
                ),
            ),
            (
                "apart".to_owned(),
                FMacroSign::new(
                    "apart",
                    ["a_separator", "a_content"],
                    Self::apart_by_separator,
                    Some(man_fun!("apart.r4d"))
                ),
            ),
            (
                "pad".to_owned(),
                FMacroSign::new(
                    "pad",
                    [ "a_type","a_width","a_fill","a_text"],
                    Self::pad_string,
                    Some(man_fun!("pad.r4d"))
                ),
            ),
            (
                "align".to_owned(),
                FMacroSign::new(
                    "align",
                    ["a_separator", "a_lines"],
                    Self::align_by_separator,
                    Some(man_fun!("align.r4d"))
                ),
            ),
            (
                "alignc".to_owned(),
                FMacroSign::new(
                    "alignc",
                    ["a_align_type^","a_content^"],
                    Self::align_columns,
                    Some(man_fun!("alignc.r4d"))
                ),
            ),
            (
                "alignby".to_owned(),
                FMacroSign::new(
                    "alignby",
                    ["a_rules", "a_lines"],
                    Self::align_by_rules,
                    Some(man_fun!("alignby.r4d"))
                ),
            ),
            (
                "gt".to_owned(),
                FMacroSign::new(
                    "gt",
                    ["a_lvalue", "a_rvalue"],
                    Self::greater_than,
                    Some(man_fun!("gt.r4d")),
                ),
            ),
            (
                "gte".to_owned(),
                FMacroSign::new(
                    "gte",
                    ["a_lvalue", "a_rvalue"],
                    Self::greater_than_or_equal,
                    Some(man_fun!("gte.r4d")),
                ),
            ),
            (
                "eq".to_owned(),
                FMacroSign::new(
                    "eq",
                    ["a_lvalue", "a_rvalue"],
                    Self::are_values_equal,
                    Some(man_fun!("eq.r4d")),
                ),
            ),
            (
                "sep".to_owned(),
                FMacroSign::new(
                    "sep",
                    ["a_content"],
                    Self::separate,
                    Some(man_fun!("sep.r4d")),
                ),
            ),
            (
                "rangeu".to_owned(),
                FMacroSign::new(
                    "rangeu",
                    ["a_min^", "a_max^", "a_array"],
                    Self::substring_utf8,
                    Some(man_fun!("range.r4d")),
                ),
            ),
            (
                "rangea".to_owned(),
                FMacroSign::new(
                    "rangea",
                    ["a_min^", "a_max^", "a_array"],
                    Self::range_array,
                    Some(man_fun!("range.r4d")),
                ),
            ),
            (
                "split".to_owned(),
                FMacroSign::new(
                    "split",
                    ["a_sep", "a_text"],
                    Self::split,
                    Some(man_fun!("split.r4d")),
                ),
            ),
            (
                "strip".to_owned(),
                FMacroSign::new(
                    "strip",
                    ["a_count^","a_content"],
                    Self::strip,
                    Some(man_fun!("strip.r4d")),
                ),
            ),
            (
                "striper".to_owned(),
                FMacroSign::new(
                    "striper",
                    ["a_expr^","a_content"],
                    Self::strip_expression_from_rear,
                    Some(man_fun!("striper.r4d")),
                ),
            ),
            (
                "stripf".to_owned(),
                FMacroSign::new(
                    "stripf",
                    ["a_count^","a_content"],
                    Self::stripf,
                    Some(man_fun!("stripf.r4d")),
                ),
            ),
            (
                "stripr".to_owned(),
                FMacroSign::new(
                    "stripr",
                    ["a_count^","a_content"],
                    Self::stripr,
                    Some(man_fun!("stripr.r4d")),
                ),
            ),
            (
                "stripfl".to_owned(),
                FMacroSign::new(
                    "stripfl",
                    ["a_count^","a_content"],
                    Self::stripf_line,
                    Some(man_fun!("stripfl.r4d")),
                ),
            ),
            (
                "striprl".to_owned(),
                FMacroSign::new(
                    "striprl",
                    ["a_count^","a_content"],
                    Self::stripr_line,
                    Some(man_fun!("striprl.r4d")),
                ),
            ),
            (
                "cut".to_owned(),
                FMacroSign::new(
                    "cut",
                    ["a_sep", "a_index","a_text"],
                    Self::split_and_cut,
                    Some(man_fun!("cut.r4d")),
                ),
            ),
            (
                "scut".to_owned(),
                FMacroSign::new(
                    "scut",
                    ["a_index","a_text"],
                    Self::split_whitespace_and_cut,
                    Some(man_fun!("scut.r4d")),
                ),
            ),
            (
                "ssplit".to_owned(),
                FMacroSign::new(
                    "ssplit",
                    ["a_text^"],
                    Self::space_split,
                    Some(man_fun!("ssplit.r4d")),
                ),
            ),
            (
                "squash".to_owned(),
                FMacroSign::new(
                    "squash",
                    ["a_text"],
                    Self::squash,
                    Some(man_fun!("squash.r4d")),
                ),
            ),
            (
                "assert".to_owned(),
                FMacroSign::new(
                    "assert",
                    ["a_lvalue", "a_rvalue"],
                    Self::assert,
                    Some(man_fun!("assert.r4d")),
                ),
            ),
            (
                "capture".to_owned(),
                FMacroSign::new(
                    "capture",
                    ["a_expr", "a_text"],
                    Self::capture,
                    Some(man_fun!("capture.r4d")),
                ),
            ),
            (
                "comma".to_owned(),
                FMacroSign::new(
                    "comma",
                    ESR,
                    Self::print_comma,
                    Some(man_fun!("comma.r4d")),
                ),
            ),
            (
                "comment".to_owned(),
                FMacroSign::new(
                    "comment",
                    ["a_comment_type^"],
                    Self::require_comment,
                    Some(man_fun!("comment.r4d")),
                ),
            ),
            (
                "cond".to_owned(),
                FMacroSign::new(
                    "cond",
                    ["a_text"],
                    Self::condense,
                    Some(man_fun!("cond.r4d")),
                ),
            ),
            (
                "condl".to_owned(),
                FMacroSign::new(
                    "condl",
                    ["a_lines"],
                    Self::condense_by_lines,
                    Some(man_fun!("condl.r4d")),
                ),
            ),
            (
                "counter".to_owned(),
                FMacroSign::new(
                    "counter",
                    ["a_macro_name^","a_counter_type^+"],
                    Self::change_counter,
                    Some(man_fun!("counter.r4d")),
                ),
            ),
            (
                "ceil".to_owned(),
                FMacroSign::new(
                    "ceil",
                    ["a_number^"],
                    Self::get_ceiling,
                    Some(man_fun!("ceil.r4d")),
                ),
            ),
            (
                "chars".to_owned(),
                FMacroSign::new(
                    "chars",
                    ["a_text^"],
                    Self::chars_array,
                    Some(man_fun!("chars.r4d")),
                ),
            ),
            (
                "chomp".to_owned(),
                FMacroSign::new(
                    "chomp",
                    ["a_content"],
                    Self::chomp,
                    Some(man_fun!("chomp.r4d")),
                ),
            ),
            (
                "clear".to_owned(),
                FMacroSign::new(
                    "clear",
                    ESR,
                    Self::clear,
                    Some(man_fun!("clear.r4d")),
                ),
            ),
            (
                "comp".to_owned(),
                FMacroSign::new(
                    "comp",
                    ["a_content"],
                    Self::compress,
                    Some(man_fun!("comp.r4d")),
                ),
            ),
            (
                "count".to_owned(),
                FMacroSign::new(
                    "count",
                    ["a_array"],
                    Self::count,
                    Some(man_fun!("count.r4d")),
                ),
            ),
            (
                "countw".to_owned(),
                FMacroSign::new(
                    "countw",
                    ["a_array"],
                    Self::count_word,
                    Some(man_fun!("countw.r4d")),
                ),
            ),
            (
                "countl".to_owned(),
                FMacroSign::new(
                    "countl",
                    ["a_lines"],
                    Self::count_lines,
                    Some(man_fun!("countl.r4d")),
                ),
            ),
            (
                "dnl".to_owned(),
                FMacroSign::new(
                    "dnl",
                    ESR,
                    Self::deny_newline,
                    Some(man_fun!("dnl.r4d")),
                ),
            ),
            (
                "declare".to_owned(),
                FMacroSign::new(
                    "declare",
                    ["a_macro_names^"],
                    Self::declare,
                    Some(man_fun!("declare.r4d")),
                ),
            ),
            (
                "docu".to_owned(),
                FMacroSign::new(
                    "docu",
                    ["a_macro_name^", "a_doc"],
                    Self::document,
                    Some(man_fun!("docu.r4d")),
                ),
            ),
            (
                "dump".to_owned(),
                FMacroSign::new(
                    "dump",
                    ["a_file_name^"],
                    Self::dump_file_content,
                    Some(man_fun!("dump.r4d")),
                ),
            ),
            (
                "empty".to_owned(),
                FMacroSign::new(
                    "empty",
                    ESR,
                    Self::print_empty,
                    Some(man_fun!("empty.r4d")),
                ),
            ),
            (
                "enl".to_owned(),
                FMacroSign::new(
                    "enl",
                    ESR,
                    Self::escape_newline,
                    Some(man_fun!("enl.r4d"))
                ),
            ),
            (
                "escape".to_owned(),
                FMacroSign::new(
                    "escape",
                    ESR,
                    Self::escape,
                    Some(man_fun!("escape.r4d")),
                ),
            ),
            (
                "exit".to_owned(),
                FMacroSign::new(
                    "exit",
                    ESR,
                    Self::exit,
                    Some(man_fun!("exit.r4d")),
                ),
            ),
            (
                "inner".to_owned(),
                FMacroSign::new(
                    "inner",
                    ["a_rule^","a_count^","a_src"],
                    Self::get_inner,
                    None
                ),
            ),
            (
                "input".to_owned(),
                FMacroSign::new(
                    "input",
                    ["a_absolute?^+"],
                    Self::print_current_input,
                    Some(man_fun!("input.r4d")),
                ),
            ),
            (
                "isempty".to_owned(),
                FMacroSign::new(
                    "isempty",
                    ["a_value"],
                    Self::is_empty,
                    Some(man_fun!("isempty.r4d")),
                ),
            ),
            (
                "insulav".to_owned(),
                FMacroSign::new(
                    "istype",
                    ["a_content"],
                    Self::isolate_vertical,
                    Some(man_fun!("insulav.r4d")),
                ),
            ),
            (
                "insulah".to_owned(),
                FMacroSign::new(
                    "istype",
                    ["a_content"],
                    Self::isolate_horizontal,
                    None,
                    // Some(man_fun!("insluah.r4d")),
                ),
            ),
            (
                "istype".to_owned(),
                FMacroSign::new(
                    "istype",
                    ["a_type^","a_value^"],
                    Self::qualify_value,
                    Some(man_fun!("istype.r4d")),
                ),
            ),
            (
                "iszero".to_owned(),
                FMacroSign::new(
                    "iszero",
                    ["a_value^"],
                    Self::is_zero,
                    Some(man_fun!("iszero.r4d")),
                ),
            ),
            (
                "find".to_owned(),
                FMacroSign::new(
                    "find",
                    ["a_expr", "a_source"],
                    Self::find_occurence,
                    Some(man_fun!("find.r4d")),
                ),
            ),
            (
                "findm".to_owned(),
                FMacroSign::new(
                    "findm",
                    ["a_expr", "a_source"],
                    Self::find_multiple_occurence,
                    Some(man_fun!("findm.r4d")),
                ),
            ),
            (
                "floor".to_owned(),
                FMacroSign::new(
                    "floor",
                    ["a_number^"],
                    Self::get_floor,
                    Some(man_fun!("floor.r4d")),
                ),
            ),
            (
                "fold".to_owned(),
                FMacroSign::new(
                    "fold",
                    ["a_array"],
                    Self::fold,
                    Some(man_fun!("fold.r4d")),
                ),
            ),
            (
                "foldl".to_owned(),
                FMacroSign::new(
                    "foldl",
                    ["a_lines"],
                    Self::fold_line,
                    Some(man_fun!("foldl.r4d")),
                ),
            ),
            (
                "foldlc".to_owned(),
                FMacroSign::new(
                    "foldlc",
                    ["a_count","a_lines"],
                    Self::fold_lines_by_count,
                    None,
                    // Some(),
                ),
            ),
            (
                "foldt".to_owned(),
                FMacroSign::new(
                    "foldt",
                    ["a_lines"],
                    Self::foldt,
                    None,
                ),
            ),
            (
                "foldby".to_owned(),
                FMacroSign::new(
                    "foldby",
                    ["a_separator","a_content"],
                    Self::fold_by,
                    None,
                ),
            ),
            (
                "foldreg".to_owned(),
                FMacroSign::new(
                    "foldreg",
                    ["a_expr","a_lines"],
                    Self::fold_regular_expr,
                    None,
                ),
            ),
            (
                "grep".to_owned(),
                FMacroSign::new(
                    "grep",
                    ["a_expr", "a_array"],
                    Self::grep_array,
                    Some(
"Extract matched items from given array. This returns all items as array

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
                    Some(
"Extract matched lines from given lines. This returns all lines that matches 
a given expression

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

- NOTE : Halt is automatically queued by default. Feed an optional argument to 
configure this behaviour
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

- On \"macro\" hygiene, every newly defined runtime macro is cleared after a 
first level macro invocation.

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
                "indexl".to_owned(),
                FMacroSign::new(
                    "indexl",
                    ["a_index^", "a_lines"],
                    Self::index_lines,
                    Some("Get an indexed line from lines

- A positive integer works as a normal index number
- A negative integer works as an index from end ( -1 == len -1 )

# Arguments

- a_index : An index to get [Signed integer] ( trimmed )
- a_liens : Lines to index from

# Example

$assert(line 2,$indexl(1,line 1$nl()line 2$nl()))".to_string()),
                ),
            ),
            (
                "import".to_owned(),
                FMacroSign::new(
                    "import",
                    ["a_file^"],
                    Self::import_frozen_file,
                    Some("Import a frozen file at runtime

- Import always include the macros as non-volatile form, thus never cleared 
unless accessed from library

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
                    Self::join_lines,
                    None,
//                     Some("Join lines into a single chunk
//
// # Arguments
//
// - a_sep   : A separator used for joining
// - a_array : Source to array to join
//
// # Example
//
// $assert(a-b-c,$join(-,a,b,c))".to_string()),
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
                "ulen".to_owned(),
                FMacroSign::new(
                    "ulen",
                    ["a_string"],
                    Self::unicode_len,
                    Some("Get a unicode length of text.

# Return : Unsigned integer

# Arguments

- a_string : Text to get unicode length

# Example

$assert($ulen(가나다),$ulen(ABCDEF))".to_string()),
                ),
            ),
            (
                "let".to_owned(),
                FMacroSign::new(
                    "let",
                    ["a_macro_name^", "a_value^"],
                    Self::bind_to_local,
                    Some(
"Bind a local macro. Every local macro gets removed after a macro expansion ends

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
                    Some(
"Bind a local macro with raw value. Every local macro gets removed after a macro 
expansion ends.

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
                "lipsumr".to_owned(),
                FMacroSign::new(
                    "lipsumr",
                    ["a_word_count^"],
                    Self::lipsum_repeat,
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

- This prints error in non-breaking manner. Even in strict mode, this doesn't 
trigger a panic.

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
                "lp".to_owned(),
                FMacroSign::new(
                    "lp",
                    ESR,
                    Self::left_parenthesis,
                    Some("Left parenthesis

# Arguments

# Example

$assert(\\(,$lp())".to_string()),
                ),
            ),
            (
                "lt".to_owned(),
                FMacroSign::new(
                    "lt",
                    ["a_lvalue", "a_rvalue"],
                    Self::less_than,
                    Some("Check if lvalue is less than rvalue

# Return : Boolean

# Arguments

- a_lvalue : A left value to compare
- a_rvalue : A right value to compare

# Example

$assert(false,$lt(c,b))
$assert(false,$lt(text,text))".to_string()),
                ),
            ),
            (
                "lte".to_owned(),
                FMacroSign::new(
                    "lte",
                    ["a_lvalue", "a_rvalue"],
                    Self::less_than_or_equal,
                    Some("Check if lvalue is less than or equal to rvalue

# Return : Boolean

# Arguments

- a_lvalue : A left value to compare
- a_rvalue : A right value to compare

# Example

$assert(true,$lte(b,c))
$assert(true,$lte(text,text))".to_string()),
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
                    Some(
"Returns a negated value of a given boolean. Yields error when a given value is 
not a boolean

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
                    Some(
"Extract number parts from given text. If there are multiple numbers, only 
extract the first

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
                    Some(
"Print platform specific newlines. Its behaviour can be configured.

- CRLF is returned on windows
- LF is returned on *nix systems.

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
                "ostype".to_owned(),
                FMacroSign::new(
                    "ostype",
                    ESR,
                    Self::get_os_type,
                    Some("Get operating system type

- R4d only supports windows and *nix systems.
- This return either \"windows\" or \"unix\"

# Example

$assert(unix,$ostype())".to_string()),
                ),
            ),
            (
                "output".to_owned(),
                FMacroSign::new(
                    "require",
                    ["a_output_type^"],
                    Self::require_output,
                    Some(
" Require output type

# Arguments

- a_output_type : A output type to require (trimmed) []

# Example

".to_string()),
                ),
            ),
            (
                "panic".to_owned(),
                FMacroSign::new(
                    "panic",
                    ["a_msg"],
                    Self::manual_panic,
                    Some("Forefully shutdown macro processing

# NOTE

Despite of the macro name, this means panicky state of macro processing not a 
cli program itself. Thus rad gracefully return an error code.

Panic macro's behaviour is not a flowcontrol therefore every input after the 
execution is ignored.

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

- NOTE : This yields an error if a path is a root and will return an empty 
value, but not a none value if a path is a single node.

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
- Users cannot override platform specific separator for this macro.
- Paths with colliding separator cannot be merged.
    e.g) a/ + /b cannot be merged

# Return : path

# Demo

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
                    Some(
"Pause macro expansions from the invocation. Paused processor will only expand 
$pause(false)

- NOTE : Pause is not flow control but a processor state, thus the state will 
sustain for the whole processing.

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
                "percent".to_owned(),
                FMacroSign::new(
                    "percent",
                    ESR,
                    Self::print_percent,
                    Some("percent".to_string()),
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
                "println".to_owned(),
                FMacroSign::new(
                    "println",
                    ["a_message"],
                    Self::print_message,
                    Some("print message -> discard option is ignored for this macro + it has trailing new line for pretty foramtting".to_owned()),
                ),
            ),
            (
                "relay".to_owned(),
                FMacroSign::new(
                    "relay",
                    ["a_target_type^", "a_target^"],
                    Self::relay,
                    Some(
"Start relaying to a target. Relay redirects all following text to the relay 
target. NOTE, relay is not evaluated inside arguments.

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
                "rer".to_owned(),
                FMacroSign::new(
                    "rer",
                    ["a_list_contents"],
                    Self::rearrange,
                    Some("Rearrange order of lists

# Arguments

- a_list_contents : List contents to rearrange

# Example

$assert($rer(8. a
2. b
3. c
1. d)$enl()
,1. a
2. b
3. c
4. d)".to_string()),
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
                "sub".to_owned(),
                FMacroSign::new(
                    "sub",
                    ["a_expr", "a_target", "a_source"],
                    Self::regex_sub,
                    Some("Apply a regular expression substitution to a source

# Arguments

- a_expr   : A regex expression to match
- a_target : Text to substitute as
- a_source : Source text to operate on

# Example

$assert(Hello Rust,$sub(World,Rust,Hello World))".to_string()),
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
- Every regex operation creates regex cache, while registered expression will 
not be cached but saved permanently. Unregistered caches will be cleared if 
certain capacity reaches.

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
                "require".to_owned(),
                FMacroSign::new(
                    "require",
                    ["a_permissions^"],
                    Self::require_permissions,
                    Some(
" Require permissions

# Arguments

- a_permissions : A permission array to require (trimmed) [ \"fin\", \"fout\", \"cmd\", \"env\" ]

# Example

$require(fin,fout)".to_string()),
                ),
            ),
            (
                "rotatel".to_owned(),
                FMacroSign::new(
                    "rotatel",
                    ["a_pattern", "a_orientation", "a_content"],
                    Self::rotatel,
                    None,
                ),
            ),
            (
                "rp".to_owned(),
                FMacroSign::new(
                    "rp",
                    ESR,
                    Self::right_parenthesis,
                    Some("right parenthesis

# Arguments

# Example

$assert(\\),$rp())".to_string()),
                ),
            ),
            (
                "source".to_owned(),
                FMacroSign::new(
                    "source",
                    ["a_file^"],
                    Self::source_static_file,
                    Some(
"Source an env file. The sourced file is eagerly expanded (As if it was static 
defined)

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
                "sortc".to_owned(),
                FMacroSign::new(
                    "sortc",
                    ["a_sort_type^","a_content"],
                    Self::sort_chunk,
                    None,
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
                    Some(
"Create a static macro. A static macro is eagerly expanded unlike define

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
                    Some(
"Create a static macro with raw value. A static macro is eagerly expanded unlike 
define

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
                "strict".to_owned(),
                FMacroSign::new(
                    "strict",
                    ["a_mode^"],
                    Self::require_strict,
                    Some(
"Check strict mode

# Arguments

- a_mode : A mode to require. Empty means strict ( trimmed ) [ \"lenient\", \"purge\" ]

# Example

$strict()
$strict(lenient)".to_string()),
                ),
            ),
            (
                "range".to_owned(),
                FMacroSign::new(
                    "range",
                    ["a_start_index^", "a_end_index^", "a_source"],
                    Self::substring,
                    Some("Get a substring with indices.

- Out of range index is an error
- A substring is calculated as char iterator not a byte iterator
- this operation is technically same with [start_index..end_index]

# Arguments

- a_start_index : A start substring index [signed integer] (trimmed)
- a_end_index   : A end   substring index [signed integer] (trimmed)
- a_source      : Source text get to a substring from

# Example

$assert(def,$range(3,5,abcdef))".to_string()),
                ),
            ),
            (
                "rangel".to_owned(),
                FMacroSign::new(
                    "rangel",
                    ["a_start_index^", "a_end_index^", "a_lines"],
                    Self::range_lines,
                    None
                ),
            ),
            (
                "rangeby".to_owned(),
                FMacroSign::new(
                    "rangeby",
                    ["a_delimeter","a_start_index^", "a_end_index^", "a_lines"],
                    Self::range_pieces,
                    None
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
                "squz".to_owned(),
                FMacroSign::new(
                    "squz",
                    ["a_content"],
                    Self::squeeze_line,
                    None,
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
                    Some(
"Construct a formatted table. Available table forms are \"github,html,wikitext\"

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
                    Some(
"Trim text. This removes leading and trailing newlines, tabs and spaces

# Arguments

- a_text : Text to trim

# Example

$assert(Middle,$trim(
    Middle
))".to_string()),
                ),
            ),
            (
                "trimf".to_owned(),
                FMacroSign::new(
                    "trimf",
                    ["a_text"],
                    Self::trimf,
                    None
                ),
            ),
            (
                "trimr".to_owned(),
                FMacroSign::new(
                    "trimr",
                    ["a_text"],
                    Self::trimr,
                    None
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
- If given an integer, it will try to trim blank characters as much as given 
amount
- min trims by minimal amount that can be applied to total lines
- max acts same as triml
- Tab character is treated as a single character. Don't combine spaces and tabs 
for this macro

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
                "until".to_owned(),
                FMacroSign::new(
                    "until",
                    ["a_pattern", "a_content"],
                    Self::get_slice_until,
                    Some("Get a substring unitl a pattern

# Arguments

- a_pattern : A pattern to find
- a_content : A content to get a sub string

# Example

$assert(Hello,$until($space(),Hello World))".to_string()),
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
- Consequent characters for macro names, parameter names can be underscore or 
any characters except special characters.
- Parameters starts with comma and should be separated by whitespaces
- Macro body starts with equal(=) characters

# Example

$define(test=Test)
$define(demo,a_1 a_2=$a_1() $a_2())
$assert($test(),Test)
$assert(wow cow,$demo(wow,cow))".to_string()),
                ),
            ),
            (
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
            ),
            (
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
            ),
            (
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
            ),
            (
                "exist".to_owned(),
                FMacroSign::new(
                    "exist",
                    ["a_filename^"],
                    Self::file_exists,
                    Some(
                        "Chck if file exists

# Auth : FIN

# Arguments

- a_filename : A file name to audit ( trimmed )

# Example

$exist(file.txt)"
                            .to_string(),
                    ),
                ),
            ),
            (
                "grepf".to_owned(),
                FMacroSign::new(
                    "grepf",
                    ["a_expr", "a_file^"],
                    Self::grep_file,
                    Some(
                        "Extract matched lines from given file. This returns all items as lines

- NOTE : The grep operation is executed on per line and doesn't expand lines

# Arguments

- a_expr  : A regex expression to match
- a_lines : A file get matches from

# Example

$countl($grepf(file.txt))"
                            .to_string(),
                    ),
                ),
            ),
            (
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

# NOTE

- Syscmd's stdout is redirected to rad's input. Which enables inclusion of 
system call's result into a desired output.
- However, due to the inherent feature, you cannot use redirection within 
syscmd's call.
- Therefore code such as $syscmd(ls > file) will not work as expected.

# Auth : CMD

# Arguments

- a_command : A command to exectute

# Example

$assert(Linux,$syscmd(uname))"
                            .to_string(),
                    ),
                ),
            ),
            (
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

$tempout(Content)"
                            .to_string(),
                    ),
                ),
            ),
            (
                "tempto".to_owned(),
                FMacroSign::new(
                    "tempto",
                    ["a_filename^"],
                    Self::set_temp_target,
                    Some(
                        "Change a temporary file path

- NOTE : A temporary file name is merged to a temporary directory. You cannot 
set a temporary file outside of a temporary directory.
- This macro needs FOUT permission because it creates a temporary file if the 
file doesn't exist

# Auth: FOUT

# Arguments

- a_filename : A new temporary file path ( trimmed )

# Example

$tempto(/new/path)"
                            .to_string(),
                    ),
                ),
            ),
            (
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
            ),
            (
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
            ),
            (
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
            ),
            (
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
            ),
            (
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
            ),
        ]));

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

# Auth: FIN

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
        {
            map.insert(
                "eval".to_owned(),
                FMacroSign::new(
                    "eval",
                    ["a_expr"],
                    Self::eval,
                    Some(
                        "Evaluate a given expression

# NOTE

- This macro redirects expression to evalexpr crate
- Append point after number if you want floating point operation.
    e.g. ) 1 / 5 = 0 while 1.0 / 5.0 = 0.2

# Arguments

- a_expr : An expression to evaluate

# Example

$assert(3,$eval(1 + 2))
$assert(true,$eval(\"string\" == \"string\"))"
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
            map.insert(
                "evalf".to_owned(),
                FMacroSign::new(
                    "evalf",
                    ["a_expr"],
                    Self::evalf,
                    Some(
                        "Evaluate a given expression

# NOTE

- This macro redirects expression to evalexpr crate
- Evalf forces floating point form for integers.

# Arguments

- a_expr : An expression to evaluate

# Example"
                            .to_string(),
                    ),
                ),
            );
            map.insert(
                "evalkf".to_owned(),
                FMacroSign::new("evalkf", ["a_expr"], Self::eval_keep_as_float, None),
            );
            map.insert(
                "pie".to_owned(),
                FMacroSign::new("pie", ["a_expr"], Self::pipe_ire, None),
            );
            map.insert(
                "mie".to_owned(),
                FMacroSign::new("mie", ["a_expr"], Self::macro_ire, None),
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
}

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
