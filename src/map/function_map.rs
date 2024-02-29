//! # Function macro module
//!
//! Function macro module includes struct and methods related to function macros
//! which are technically function pointers.

use crate::argument::{MacroInput, ValueType};
use crate::common::*;
use crate::consts::{ESR, RET_ETABLE};
use crate::extension::ExtMacroBuilder;
use crate::{man_fun, Parameter, Processor};
#[cfg(feature = "rustc_hash")]
use rustc_hash::FxHashMap as HashMap;
#[cfg(not(feature = "rustc_hash"))]
use std::collections::HashMap;
use std::iter::FromIterator;

/// Function signature for "function" macro functions
pub(crate) type FunctionMacroType = fn(MacroInput, &mut Processor) -> RadResult<Option<String>>;

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
        let mut map = Self::from_iter(IntoIterator::into_iter([
            (
                FMacroSign::new(
                    "-",
                    [(ValueType::CText,"a_pipe_name")],
                    Self::get_pipe,
                    Some(man_fun!("pipe.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "PS",
                    ESR,
                    Self::path_separator,
                    Some(man_fun!("PS.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "after",
                    [(ValueType::Text,"a_pattern"),(ValueType::Text, "a_content"),],
                    Self::get_slice_after,
                    Some(man_fun!("after.r4d"))
                )
            ),
            (
                FMacroSign::new(
                    "pad",
                    [(ValueType::Enum,"a_type"),(ValueType::Uint,"a_width"),(ValueType::Text,"a_fill"),(ValueType::Text,"a_text"),],
                    Self::pad_string,
                    Some(man_fun!("pad.r4d"))
                ).enum_table(
                    ETable::new("a_type")
                        .candidates(&["l","left","r","right","c","center"])
                )
            ),
            (
                FMacroSign::new(
                    "peel",
                    [(ValueType::Uint,"a_level"),(ValueType::Text,"a_src"),],
                    Self::peel,
                    Some(man_fun!("peel.r4d"))
                )
            ),
            (
                FMacroSign::new(
                    "align",
                    [(ValueType::Enum,"a_align_type"),(ValueType::Text, "a_content"),],
                    Self::align,
                    Some(man_fun!("align.r4d"))
                ).enum_table(
                    ETable::new("a_align_type")
                        .candidates(&[
                            "h", "hierarchy",
                            "l", "left",
                            "r", "Right",
                            "pr", "parralel-right",
                            "pl", "parralel-left" 
                        ])
                )
            ),
            (
                FMacroSign::new(
                    "lineup",
                    [(ValueType::Text,"a_separator"),(ValueType::Text, "a_lines"),],
                    Self::lineup_by_separator,
                    Some(man_fun!("lineup.r4d"))
                )
            ),
            (
                FMacroSign::new(
                    "lineupr",
[(ValueType::Text,"a_separator"),(ValueType::Text, "a_lines"),],
                    Self::lineup_by_separator_match_rear,
                    Some(man_fun!("lineupr.r4d"))
                )
            ),
            (
                FMacroSign::new(
                    "alignc",
                    [(ValueType::Enum,"a_align_type"),(ValueType::Text,"a_content"),],
                    Self::align_columns,
                    Some(man_fun!("alignc.r4d"))
                ).enum_table(
                    ETable::new("a_align_type")
                        .candidates(&[
                            "l", "left",
                            "r", "right", 
                            "c", "center"
                        ])
                )
            ),
            (
                FMacroSign::new(
                    "lineupm",
                    [(ValueType::Text,"a_rules"),(ValueType::Text, "a_lines"),],
                    Self::lineup_by_rules,
                    Some(man_fun!("lineupm.r4d"))
                )
            ),
            (
                FMacroSign::new(
                    "gt",
[(ValueType::Text,"a_lvalue"),(ValueType::Text, "a_rvalue"),],
                    Self::greater_than,
                    Some(man_fun!("gt.r4d")),
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "gte",
[(ValueType::Text,"a_lvalue"),(ValueType::Text, "a_rvalue"),],
                    Self::greater_than_or_equal,
                    Some(man_fun!("gte.r4d")),
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "eq",
                    [(ValueType::Text,"a_lvalue"),(ValueType::Text, "a_rvalue")],
                    Self::are_values_equal,
                    Some(man_fun!("eq.r4d")),
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "sep",
[(ValueType::Text,"a_content"),],
                    Self::separate,
                    Some(man_fun!("sep.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "rangeu",
[(ValueType::Int,"a_min"),(ValueType::Int, "a_max"),(ValueType::Text, "a_array"),],
                    Self::substring_utf8,
                    Some(man_fun!("rangeu.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "rangea",
[(ValueType::Int,"a_min"),(ValueType::Int, "a_max"),(ValueType::Text, "a_array"),],
                    Self::range_array,
                    Some(man_fun!("rangea.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "split",
[(ValueType::Text,"a_sep"),(ValueType::Text, "a_text"),],
                    Self::split,
                    Some(man_fun!("split.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "strip",
[(ValueType::Uint,"a_count"),(ValueType::Text,"a_content"),],
                    Self::strip,
                    Some(man_fun!("strip.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "striper",
[(ValueType::Text,"a_expr"),(ValueType::Text,"a_content"),],
                    Self::strip_expression_from_rear,
                    Some(man_fun!("striper.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "stripf",
[(ValueType::Uint,"a_count"),(ValueType::Text,"a_content"),],
                    Self::stripf,
                    Some(man_fun!("stripf.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "stripr",
[(ValueType::Uint,"a_count"),(ValueType::Text,"a_content"),],
                    Self::stripr,
                    Some(man_fun!("stripr.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "stripfl",
[(ValueType::Uint,"a_count"),(ValueType::Text,"a_content"),],
                    Self::stripf_line,
                    Some(man_fun!("stripfl.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "striprl",
[(ValueType::Uint,"a_count"),(ValueType::Text,"a_content"),],
                    Self::stripr_line,
                    Some(man_fun!("striprl.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "border",
[(ValueType::Text,"a_border_string"),(ValueType::Text,"a_content"),],
                    Self::decorate_border,
                    None,
                )
            ),
            (
                FMacroSign::new(
                    "cut",
[(ValueType::Text,"a_sep"),(ValueType::Uint, "a_index"),(ValueType::Text,"a_text"),],
                    Self::split_and_cut,
                    Some(man_fun!("cut.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "cont",
                    [(ValueType::Enum,"a_op"),(ValueType::Text,"a_arg"),],
                    Self::container,
                    None
                ).enum_table(
                    ETable::new("a_op")
                        .candidates(&[
                            "print",
                            "push",
                            "pop",
                            "clear",
                            "get",
                            "ow",
                            "set",
                            "extend"
                        ])
                )
            ),
            (
                FMacroSign::new(
                    "scut",
[(ValueType::Uint,"a_index"),(ValueType::Text,"a_text"),],
                    Self::split_whitespace_and_cut,
                    Some(man_fun!("scut.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "ssplit",
[(ValueType::Text,"a_text"),],
                    Self::space_split,
                    Some(man_fun!("ssplit.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "squash",
[(ValueType::Text,"a_text"),],
                    Self::squash,
                    Some(man_fun!("squash.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "assert",
[(ValueType::Text,"a_lvalue"),(ValueType::Text, "a_rvalue"),],
                    Self::assert,
                    Some(man_fun!("assert.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "comma",
                    ESR,
                    Self::print_comma,
                    Some(man_fun!("comma.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "coll",
[(ValueType::Text,"a_pat"),(ValueType::Text,"a_lines"),],
                    Self::collapse,
                    None,
                )
            ),
            (
                FMacroSign::new(
                    "comment",
                    [(ValueType::Enum,"a_comment_type"),],
                    Self::require_comment,
                    Some(man_fun!("comment.r4d")),
                ).enum_table(
                    ETable::new("a_comment_type")
                        .candidates(&[
                            "none",
                            "start",
                            "any"
                        ])
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "cond",
                    [(ValueType::Text,"a_text"),],
                    Self::condense,
                    Some(man_fun!("cond.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "condl",
                    [(ValueType::Text,"a_lines"),],
                    Self::condense_by_lines,
                    Some(man_fun!("condl.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "ceil",
                    [(ValueType::Float,"a_number"),],
                    Self::get_ceiling,
                    Some(man_fun!("ceil.r4d")),
                ).ret(ValueType::Int)
            ),
            (
                FMacroSign::new(
                    "chars",
                    [(ValueType::Text,"a_text"),],
                    Self::chars_array,
                    Some(man_fun!("chars.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "chomp",
                    [(ValueType::Text,"a_content"),],
                    Self::chomp,
                    Some(man_fun!("chomp.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "clear",
                    ESR,
                    Self::clear,
                    Some(man_fun!("clear.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "comp",
                    [(ValueType::Text,"a_content"),],
                    Self::compress,
                    Some(man_fun!("comp.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "count",
                    [(ValueType::Text,"a_array"),],
                    Self::count,
                    Some(man_fun!("count.r4d")),
                ).ret(ValueType::Uint)
            ),
            (
                FMacroSign::new(
                    "countw",
                    [(ValueType::Text,"a_array"),],
                    Self::count_word,
                    Some(man_fun!("countw.r4d")),
                ).ret(ValueType::Uint)
            ),
            (
                FMacroSign::new(
                    "countl",
                    [(ValueType::Text,"a_lines"),],
                    Self::count_lines,
                    Some(man_fun!("countl.r4d")),
                ).ret(ValueType::Uint)
            ),
            (
                FMacroSign::new(
                    "dnl",
                    ESR,
                    Self::deny_newline,
                    Some(man_fun!("dnl.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "decl",
                    [(ValueType::CText,"a_macro_names"),],
                    Self::declare,
                    Some(man_fun!("decl.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "docu",
                    [(ValueType::CText,"a_macro_name"),(ValueType::Text, "a_doc"),],
                    Self::document,
                    Some(man_fun!("docu.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "dump",
                    [(ValueType::Text,"a_file_name"),],
                    Self::dump_file_content,
                    Some(man_fun!("dump.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "empty",
                    ESR,
                    Self::print_empty,
                    Some(man_fun!("empty.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "enl",
                    ESR,
                    Self::escape_newline,
                    Some(man_fun!("enl.r4d"))
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "escape",
                    ESR,
                    Self::escape,
                    Some(man_fun!("escape.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "exit",
                    ESR,
                    Self::exit,
                    Some(man_fun!("exit.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "inc",
                    [(ValueType::CText,"a_number")],
                    Self::increase_number,
                    None
                ).optional(Parameter::new(ValueType::Uint,"a_amount"))
            ),
            (
                FMacroSign::new(
                    "dec",
[(ValueType::CText,"a_number"),(ValueType::Uint,"a_amount"),],
                    Self::decrease_number,
                    None
                )
            ),
            (
                FMacroSign::new(
                    "square",
[(ValueType::Text,"a_number"),],
                    Self::square_number,
                    None
                )
            ),
            (
                FMacroSign::new(
                    "cube",
[(ValueType::Text,"a_number"),],
                    Self::cube_number,
                    None
                )
            ),
            (
                FMacroSign::new(
                    "pow",
[(ValueType::Text,"a_number"),(ValueType::Text,"a_exponent"),],
                    Self::power_number,
                    None
                )
            ),
            (
                FMacroSign::new(
                    "sqrt",
[(ValueType::Text,"a_number"),],
                    Self::square_root,
                    None
                )
            ),
            (
                FMacroSign::new(
                    "round",
[(ValueType::Float,"a_number"),],
                    Self::round_number,
                    None
                ).ret(ValueType::Uint)
            ),
            (
                FMacroSign::new(
                    "inner",
                    [(ValueType::CText,"a_rule"),(ValueType::Uint,"a_count"),(ValueType::Text,"a_src"),],
                    Self::get_inner,
                    Some(man_fun!("inner.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "input",
                    ESR,
                    Self::print_current_input,
                    Some(man_fun!("input.r4d")),
                ).optional( Parameter::new(ValueType::Bool,"a_absolute?+"))
                .ret(ValueType::Path)
            ),
            (
                FMacroSign::new(
                    "isempty",
[(ValueType::Text,"a_value"),],
                    Self::is_empty,
                    Some(man_fun!("isempty.r4d")),
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "insulav",
[(ValueType::Text,"a_content"),],
                    Self::isolate_vertical,
                    Some(man_fun!("insulav.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "insulah",
                    [(ValueType::Text,"a_content"),],
                    Self::isolate_horizontal,
                    Some(man_fun!("insulah.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "istype",
[(ValueType::Text,"a_type"),(ValueType::Text,"a_value"),],
                    Self::qualify_value,
                    Some(man_fun!("istype.r4d")),
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "iszero",
[(ValueType::CText,"a_value"),],
                    Self::is_zero,
                    Some(man_fun!("iszero.r4d")),
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "find",
[(ValueType::Text,"a_expr"),(ValueType::Text, "a_source"),],
                    Self::find_occurence,
                    Some(man_fun!("find.r4d")),
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "findm",
[(ValueType::Text,"a_expr"),(ValueType::Text, "a_source"),],
                    Self::find_multiple_occurence,
                    Some(man_fun!("findm.r4d")),
                ).ret(ValueType::Uint)
            ),
            (
                FMacroSign::new(
                    "floor",
[(ValueType::Float,"a_number"),],
                    Self::get_floor,
                    Some(man_fun!("floor.r4d")),
                ).ret(ValueType::Uint)
            ),
            (
                FMacroSign::new(
                    "fold",
[(ValueType::Text,"a_array"),],
                    Self::fold,
                    Some(man_fun!("fold.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "foldl",
[(ValueType::Text,"a_lines"),],
                    Self::fold_line,
                    Some(man_fun!("foldl.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "foldlc",
[(ValueType::Text,"a_count"),(ValueType::Text,"a_lines"),],
                    Self::fold_lines_by_count,
                    Some(man_fun!("foldlc.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "folde",
[(ValueType::Text,"a_start_expr"),(ValueType::Text,"a_end_expr"),(ValueType::Text,"a_lines"),],
                    Self::fold_regular_expr,
                    None,
                )
            ),
            (
                FMacroSign::new(
                    "grep",
[(ValueType::Text,"a_expr"),(ValueType::Text, "a_text"),],
                    Self::grep_expr,
                    Some(man_fun!("grep.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "grepa",
[(ValueType::Text,"a_expr"),(ValueType::Text, "a_array"),],
                    Self::grep_array,
                    Some(man_fun!("grepa.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "grepl",
[(ValueType::Text,"a_expr"),(ValueType::Text, "a_lines"),],
                    Self::grep_lines,
                    Some(man_fun!("grepl.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "halt",
                    ESR,
                    Self::halt_relay,
                    Some(man_fun!("halt.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "head",
[(ValueType::Uint,"a_count"),(ValueType::Text, "a_content"),],
                    Self::head,
                    Some(man_fun!("head.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "headl",
[(ValueType::Uint,"a_count"),(ValueType::Text, "a_lines"),],
                    Self::head_line,
                    Some(man_fun!("headl.r4d"))
                )
            ),
            (
                FMacroSign::new(
                    "hygiene",
[(ValueType::Bool,"a_hygiene"),],
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
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "indentl",
[(ValueType::Text,"a_indenter"),(ValueType::Text, "a_lines"),],
                    Self::indent_lines_before,
                    Some("Indent lines with indenter

# Arguments

- a_indenter : An expression to put before lines
- a_lines    : Lines to prepend indenter

# Example

$assert(
# First
# Second
# Third,
$indentl(# ,First
Second
Third))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "attachl",
[(ValueType::Text,"a_indenter"),(ValueType::Text, "a_lines"),],
                    Self::attach_lines_after,
                    None
                )
            ),
            (
                FMacroSign::new(
                    "index",
[(ValueType::Uint,"a_index"),(ValueType::Text, "a_array"),],
                    Self::index_array,
                    Some("Get an indexed value from an array

- A positive integer works as a normal index number
- A negative integer works as an index from end ( -1 == len -1 )

# Arguments

- a_index : An index to get [Signed integer] ( trimmed )
- a_array : Data source to index from

# Example

$assert(ef,$index(2,ab,cd,ef))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "indexl",
[(ValueType::Uint,"a_index"),(ValueType::Text, "a_lines"),],
                    Self::index_lines,
                    Some("Get an indexed line from lines

- A positive integer works as a normal index number
- A negative integer works as an index from end ( -1 == len -1 )

# Arguments

- a_index : An index to get [Signed integer] ( trimmed )
- a_liens : Lines to index from

# Example

$assert(line 2,$indexl(1,line 1$nl()line 2$nl()))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "import",
[(ValueType::Path,"a_file"),],
                    Self::import_frozen_file,
                    Some("Import a frozen file at runtime

- Import always include the macros as non-volatile form, thus never cleared 
unless accessed from library

# Arguments

- a_file: A file name to import from [path] (trimmed)

# Example

$import(def.r4f)".to_string()),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "join",
[(ValueType::Text,"a_sep"),(ValueType::Text,"a_array"),],
                    Self::join,
                    Some("Join an array into a single chunk with given separator

# Arguments

- a_sep   : A separator used for joining
- a_array : Source to array to join

# Example

$assert(a-b-c,$join(-,a,b,c))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "joinl",
[(ValueType::Text,"a_sep"),(ValueType::Text,"a_lines"),],
                    Self::join_lines,
                    Some("Join lines into a single chunk with given separator

# Note

- use foldl if you want to simply fold everything

# Arguments

- a_sep   : A separator used for joining
- a_array : Source to array to join

# Example

$assert(a-b-c,$joinl(-,a,b,c))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "len",
[(ValueType::Text,"a_string"),],
                    Self::len,
                    Some("Get a length of text. This counts utf8 characters not ascii.

# Return : Unsigned integer

# Arguments

- a_string : Text to get length from

# Example

$assert($len(가나다),$len(ABC))".to_string()),
                ).ret(ValueType::Uint)
            ),
            (
                FMacroSign::new(
                    "ulen",
[(ValueType::Text,"a_string"),],
                    Self::unicode_len,
                    Some(man_fun!("ulen.r4d")),
                ).ret(ValueType::Uint)
            ),
            (
                FMacroSign::new(
                    "let",
[(ValueType::CText,"a_macro_name"),(ValueType::Text, "a_value"),],
                    Self::bind_to_local,
                    Some(man_fun!("let.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "lipsum",
[(ValueType::Uint,"a_word_count"),],
                    Self::lipsum_words,
                    Some(man_fun!("lipsum.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "lipsumr",
[(ValueType::Uint,"a_word_count"),],
                    Self::lipsum_repeat,
                    Some(man_fun!("lipsumr.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "log",
[(ValueType::Text,"a_msg"),],
                    Self::log_message,
                    Some("Log a message to console

# Arguments

- a_msg : A message to log to console

# Example

$log($value_i_want_to_check())".to_string()),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "loge",
[(ValueType::Text,"a_msg"),],
                    Self::log_error_message,
                    Some("Log an error message to console

- This prints error in non-breaking manner. Even in strict mode, this doesn't 
trigger a panic.

# Arguments

- a_msg : An error message to log to console

# Example

$loge(This should not be reached)".to_string()),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "lower",
[(ValueType::Text,"a_text"),],
                    Self::lower,
                    Some(man_fun!("lower.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "lp",
                    ESR,
                    Self::left_parenthesis,
                    Some(man_fun!("lp.r4d"))
                )
            ),
            (
                FMacroSign::new(
                    "lt",
[(ValueType::Text,"a_lvalue"),(ValueType::Text, "a_rvalue"),],
                    Self::less_than,
                    Some("Check if lvalue is less than rvalue

# Return : Boolean

# Arguments

- a_lvalue : A left value to compare
- a_rvalue : A right value to compare

# Example

$assert(false,$lt(c,b))
$assert(false,$lt(text,text))".to_string()),
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "lte",
[(ValueType::Text,"a_lvalue"),(ValueType::Text, "a_rvalue"),],
                    Self::less_than_or_equal,
                    Some("Check if lvalue is less than or equal to rvalue

# Return : Boolean

# Arguments

- a_lvalue : A left value to compare
- a_rvalue : A right value to compare

# Example

$assert(true,$lte(b,c))
$assert(true,$lte(text,text))".to_string()),
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "max",
                    ESR,
                    Self::get_max,
                    Some("Get a max value from a given array

# Arguments

- a_array : An array to get the highest value from

# Example

$assert(eIsBigger,$max(aIsSmall,cIsMiddle,eIsBigger))
$assert(5,$max(1,2,3,4,5))".to_string()),
                ).optional( Parameter::new(ValueType::Text,"a_array"))
            ),
            (
                FMacroSign::new(
                    "min",
                    ESR,
                    Self::get_min,
                    Some("Get a min value from a given array

 # Arguments

- a_array : An array to get the lowest value from

# Example

$assert(aIsSmall,$min(aIsSmall,cIsMiddle,eIsBigger))
$assert(1,$min(1,2,3,4,5))".to_string()),
                ).optional( Parameter::new(ValueType::Text,"a_array"))
            ),
            (
                FMacroSign::new(
                    "name",
[(ValueType::Path,"a_path"),],
                    Self::get_name,
                    Some("Get a name from a given path including an extension

# Return : path

# Arguments

- a_path : A path to get a name from

# Example

$assert(auto.sh,$name(/path/to/file/auto.sh))".to_string()),
                ).ret(ValueType::Path)
            ),
            (
                FMacroSign::new(
                    "nassert",
[(ValueType::Text,"a_lvalue"),(ValueType::Text, "a_rvalue"),],
                    Self::assert_ne,
                    Some("Compare left and right values. Panics if values are equal

# Arguments

- a_lvalue : A left  value
- a_rvalue : A right value

# Example

$nassert(1,2)".to_string()),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "not",
[(ValueType::Bool,"a_boolean"),],
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
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "num",
[(ValueType::Text,"a_text"),],
                    Self::get_number,
                    Some(
"Extract number parts from given text. If there are multiple numbers, only 
extract the first

# Arguments

- a_text : Text to extract number from

# Example

$assert(34,$num(34sec))
$assert(30,$num(30k/h for 3 hours))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "nl",
                    ESR,
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
                ).optional( Parameter::new(ValueType::Text,"a_amount+"))
            ),
            (
                FMacroSign::new(
                    "notat",
                    [(ValueType::Uint,"a_number"),(ValueType::Enum, "a_type"),],
                    Self::change_notation,
                    Some(man_fun!("notat.r4d"))
                ).enum_table(
                    ETable::new("a_type")
                        .candidates(&[
                            "bin",
                            "oct",
                            "hex"
                        ])
                )
            ),
            (
                FMacroSign::new(
                    "ostype",
                    ESR,
                    Self::get_os_type,
                    Some("Get operating system type

- R4d only supports windows and *nix systems.
- This return either \"windows\" or \"unix\"

# Example

$assert(unix,$ostype())".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "require",
                    [(ValueType::Enum,"a_output_type"),],
                    Self::require_output,
                    Some(
" Require output type

# Arguments

- a_output_type : A output type to require (trimmed) []

# Example

".to_string()),
                ).enum_table(
                    ETable::new("a_output_type")
                        .candidates(&[
                            "Terminal",
                            "File",
                            "Discard"
                        ])
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "panic",
[(ValueType::Text,"a_msg"),],
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
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "parent",
[(ValueType::Path,"a_path"),],
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
                ).ret(ValueType::Path)
            ),
            (
                FMacroSign::new(
                    "path",
                    [(ValueType::Text,"a_path")],
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
                    ).optional(Parameter::new(ValueType::Text,"a_sub_path"))
                    .ret(ValueType::Path)
            ),
            (
                FMacroSign::new(
                    "pause",
[(ValueType::Bool,"a_pause"),],
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
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "percent",
                    ESR,
                    Self::print_percent,
                    Some("percent".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "pipe",
[(ValueType::Text,"a_value"),],
                    Self::pipe,
                    Some("Pipe a given value into an unnamed pipe

# Arguments

- a_value : A value to pipe

# Example

$pipe(Text)
$assert(Text,$-())".to_string()),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "pipeto",
[(ValueType::CText,"a_pipe_name"),(ValueType::Text, "a_value"),],
                    Self::pipe_to,
                    Some("Pipe a given value to a named pipe

# Arguments

- a_pipe_name : A name of pipe container ( trimmed )
- a_value     : A value to pipe

# Example

$pipeto(yum,YUM)
$assert($-(yum),YUM)".to_string()),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "prec",
[(ValueType::Float,"a_number"),(ValueType::Uint, "a_precision"),],
                    Self::prec,
                    Some("Convert a float number with given precision

# Return : Float

# Arguments

- a_number    : A number to process ( trimmed )
- a_precision : A precision number to apply to number ( trimmed )

# Example

$assert(0.30,$prec($eval(0.1 + 0.2),2))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "println",
[(ValueType::Text,"a_message"),],
                    Self::print_message,
                    Some("print message -> discard option is ignored for this macro + it has trailing new line for pretty foramtting".to_owned()),
                )
            ),
            (
                FMacroSign::new(
                    "relay",
                    [(ValueType::CText, "a_target"),],
                    Self::relay,
                    Some(man_fun!("relay.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "relayt",
                    [(ValueType::CText, "a_target"),],
                    Self::relayt,
                    None
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "relayf",
                    [(ValueType::CText, "a_target"),],
                    Self::relayf,
                    None
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "reo",
[(ValueType::Text,"a_list_contents"),],
                    Self::reorder,
                    Some("Rearrange order of lists

# Arguments

- a_list_contents : List contents to rearrange

# Example

$assert($reo(8. a
2. b
3. c
1. d)$enl()
,1. a
2. b
3. c
4. d)".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "rev",
[(ValueType::Text,"a_array"),],
                    Self::reverse_array,
                    Some("Reverse order of an array

# Arguments

- a_array : Array to reverse

# Example

$assert(\\*3,2,1*\\,$rev(1,2,3))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "sub",
[(ValueType::Text,"a_expr"),(ValueType::Text, "a_target"),(ValueType::Text, "a_source"),],
                    Self::regex_sub,
                    Some("Apply a regular expression substitution to a source

# Arguments

- a_expr   : A regex expression to match
- a_target : Text to substitute as
- a_source : Source text to operate on

# Example

$assert(Hello Rust,$sub(World,Rust,Hello World))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "addexpr",
[(ValueType::CText,"a_name"),(ValueType::Text, "a_expr"),],
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

$addexpr(greeting,Hello World)
$assert(true,$find(greeting,Hello World))
$assert(false,$find(greeting,greetings from world))".to_string()),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "rename",
[(ValueType::CText,"a_macro_name"),(ValueType::CText, "a_new_name"),],
                    Self::rename_call,
                    Some("Rename a macro with a new name

# Arguments

- a_macro_name : A macro to change name ( trimmed )
- a_new_name   : A new macro name to apply ( trimmed )

# Example

$define(test=Test)
$rename(test,demo)
$assert($demo(),Test)".to_string()),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "repeat",
[(ValueType::Uint,"a_count"),(ValueType::Text, "a_source"),],
                    Self::repeat,
                    Some("Repeat given source by given counts

# Arguments

- a_count  : Counts of repetition [Unsigned integer] ( trimmed )
- a_source : Source text to repeat

# Example

$assert(R4d
R4d
R4d,$repeat(3,R4d$nl()))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "repl",
[(ValueType::CText,"a_macro_name"),(ValueType::Text, "a_new_value"),],
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
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "require",
                    [(ValueType::Enum,"a_permissions"),],
                    Self::require_permissions,
                    Some(man_fun!("require.r4d")),
                ).enum_table(
                    ETable::new("a_permissions")
                        .candidates(&[
                            "env",
                            "fin",
                            "fout",
                            "cmd" 
                        ])
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "rotatel",
                    [(ValueType::Text,"a_pattern"),(ValueType::Enum, "a_orientation"),(ValueType::Text, "a_content"),],
                    Self::rotatel,
                    None,
                ).enum_table(
                    ETable::new("a_orientation")
                        .candidates(&[
                            "Left",
                            "Right",
                            "Center",
                        ])
                )
            ),
            (
                FMacroSign::new(
                    "rotatei",
                    ESR,
                    Self::rotatei,
                    None,
                )
            ),
            (
                FMacroSign::new(
                    "rp",
                    ESR,
                    Self::right_parenthesis,
                    Some(man_fun!("rp.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "source",
[(ValueType::Path,"a_file"),],
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
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "sort",
                    [(ValueType::Enum,"a_sort_type"),(ValueType::Text,"a_array"),],
                    Self::sort_array,
                    Some("Sort an array

# Arguments

- a_sort_type : A sort type [\"asec\",\"desc\"] (trimmed)
- a_array     : An array to sort

# Example

$assert(\\*0,1,3,4,6,7,9*\\,$enl()
$sort(asec,3,6,7,4,1,9,0))".to_string()),
                ).enum_table(
                    ETable::new("a_sort_type")
                        .candidates(&[
                            "a" , "asce",
                            "d" , "desc"
                        ])
                )
            ),
            (
                FMacroSign::new(
                    "sortl",
                    [(ValueType::Enum,"a_sort_type"),(ValueType::Text,"a_lines"),],
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
                ).enum_table(
                    ETable::new("a_sort_type")
                        .candidates(&[
                            "a" , "asce",
                            "d" , "desc"
                        ])
                )
            ),
            (
                FMacroSign::new(
                    "sortc",
                    [(ValueType::Enum,"a_sort_type"),(ValueType::Text,"a_content"),],
                    Self::sort_chunk,
                    None,
                ).enum_table(
                    ETable::new("a_sort_type")
                        .candidates(&[
                            "a" , "asce",
                            "d" , "desc"
                        ])
                )
            ),
            (
                FMacroSign::new(
                    "space",
                    ESR,
                    Self::space,
                    Some(man_fun!("space.r4d")),
                ).optional(Parameter::new(
                    ValueType::Uint,"a_amount"
                ))
            ),
            (
                FMacroSign::new(
                    "static",
                    [(ValueType::CText,"a_macro_name"),(ValueType::Text, "a_expr"),],
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
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "strict",
                    [(ValueType::Enum,"a_mode"),],
                    Self::require_strict,
                    Some(
"Check strict mode

# Arguments

- a_mode : A mode to require. Empty means strict ( trimmed ) [ \"lenient\", \"purge\" ]

# Example

$strict()
$strict(lenient)".to_string()),
                ).enum_table(
                    ETable::new("a_mode")
                        .candidates(&[
                            "leninet",
                            "purge",
                            "strict",
                        ])
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "range",
[(ValueType::CText,"a_start_index"),(ValueType::CText, "a_end_index"),(ValueType::Text, "a_source"),],
                    Self::substring,
                    Some(man_fun!("range.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "rangel",
[(ValueType::CText,"a_start_index"),(ValueType::CText, "a_end_index"),(ValueType::Text, "a_lines"),],
                    Self::range_lines,
                    None
                )
            ),
            (
                FMacroSign::new(
                    "rangeby",
[(ValueType::Text,"a_delimeter"),(ValueType::CText,"a_start_index"),(ValueType::CText, "a_end_index"),(ValueType::Text, "a_lines"),],
                    Self::range_pieces,
                    None
                )
            ),
            (
                FMacroSign::new(
                    "surr",
[(ValueType::Text,"a_start_pair"),(ValueType::Text,"a_end_pair"),(ValueType::Text,"a_content"),],
                    Self::surround_with_pair,
                    Some(man_fun!("surr.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "squz",
[(ValueType::Text,"a_content"),],
                    Self::squeeze_line,
                    Some(man_fun!("squz.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "tab",
                    ESR,
                    Self::print_tab,
                    Some(man_fun!("tab.r4d")),
                ).optional(Parameter::new(ValueType::Uint,"a_amount"))
            ),
            (
                FMacroSign::new(
                    "tail",
[(ValueType::Uint,"a_count"),(ValueType::Text, "a_content"),],
                    Self::tail,
                    Some("Get last parts of texts

# Arguments

- a_count   : Amount of characters to crop [unsigned integer] ( trimmed )
- a_content : Text to crop from

# Example

$assert(World,$tail( 5 ,Hello~ World))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "taill",
[(ValueType::Uint,"a_count"),(ValueType::Text, "a_content"),],
                    Self::tail_line,
                    Some("Get last lines of texts

# Arguments

- a_count   : Amount of lines to crop [unsigned integer] ( trimmed )
- a_lines   : Lines to crop from

# Example

$assert(b$nl()c,$taill( 2 ,a
b
c))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "table",
                    [(ValueType::Enum,"a_table_form"),(ValueType::CText, "a_csv_value"),],
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
                ).enum_table(
                    ETable::new("a_table_form")
                        .candidates(&[
                            "github",
                            "html",
                            "wikitext",
                        ])
                )
            ),
            (
                FMacroSign::new(
                    "tr",
[(ValueType::Text,"a_chars"),(ValueType::Text, "a_sub"),(ValueType::Text,"a_source"),],
                    Self::translate,
                    Some("Translate characters. Usage similar to core util tr

# Arguments

- a_chars  : Matching characters
- a_sub    : Substitute characters
- a_source : Source text to apply translation

# Example

$assert(HellO_WOrld,$tr(-how,_HOW,hello-world))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "trim",
[(ValueType::Text,"a_text"),],
                    Self::trim,
                    Some(man_fun!("trim.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "trimf",
[(ValueType::Text,"a_text"),],
                    Self::trimf,
                    None
                )
            ),
            (
                FMacroSign::new(
                    "trimr",
[(ValueType::Text,"a_text"),],
                    Self::trimr,
                    None
                )
            ),
            (
                FMacroSign::new(
                    "triml",
[(ValueType::Text,"a_content"),],
                    Self::triml,
                    Some("Trim values by lines. Trim is applied to each lines

# Arguments

- a_text : Text to trim

# Example

$assert(Upper$nl()Middle$nl()Last,$triml(    Upper
    Middle
          Last))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "exdent",
[(ValueType::CText,"a_trim_option"),(ValueType::Text,"a_lines"),],
                    Self::exdent,
                    Some("Outdent (exdent) with given amount

- Technically this trims preceding spaces
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

$exdent(min,$space(1)First
$space(2)Second
$space(3)Third)
% ===
% Equally strips one space
% First
%  Second
%   Third


$exdent(3,$space(2)First
$space(3)Second
$space(5)Third)
% ===
% Equally tries stripping 3 spaces
% First
% Second
%   Third".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "undef",
[(ValueType::CText,"a_macro_name"),],
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
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "unicode",
[(ValueType::CText,"a_value"),],
                    Self::paste_unicode,
                    Some("Creates a unicode character from a hex number without prefix

# Arguments

- a_value : A value to convert to a unicode character

# Example

$assert(☺,$unicode(263a))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "until",
[(ValueType::Text,"a_pattern"),(ValueType::Text, "a_content"),],
                    Self::get_slice_until,
                    Some("Get a substring unitl a pattern

# Arguments

- a_pattern : A pattern to find
- a_content : A content to get a sub string

# Example

$assert(Hello,$until($space(),Hello World))".to_string()),
                )
            ),
            (
                FMacroSign::new(
                    "upper",
[(ValueType::Text,"a_text"),],
                    Self::capitalize,
                    Some(man_fun!("upper.r4d"))
                )
            ),
            // THis is a placeholder for documentaion
            (
                FMacroSign::new(
                    "def",
[(ValueType::Text,"a_define_statement"),],
                    Self::define_macro,
                    Some("Define a macro

# Arguments

Define should follow handful of rules

- Macro name, parameter name should start non number characters.
- Consequent characters for macro names, parameter names can be underscore or 
any characters except special characters.
- Parameters starts with comma and should be separated by whitespaces
- Macro body starts with equal(=) characters

# Example

$def(test=Test)
$def(demo,a_1 a_2=$a_1() $a_2())
$assert($test(),Test)
$assert(wow cow,$demo(wow,cow))".to_string()),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "env",
[(ValueType::CText,"a_env_name"),],
                    Self::get_env,
                    Some(
                        "Get an environment variable

# Auth : ENV

# Arguments

- a_env_name : An environment variable name to get (trimmed)

# Example

$assert(/home/user/dir,$env(HOME))"
                            .to_string(),
                    )
                )
            ),
            (
                FMacroSign::new(
                    "setenv",
[(ValueType::CText,"a_env_name"),(ValueType::Text, "a_env_value"),],
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
                    )
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "abs",
[(ValueType::Path,"a_path"),],
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
                    )
                )
            ),
            (
                FMacroSign::new(
                    "exist",
[(ValueType::Path,"a_filename"),],
                    Self::file_exists,
                    Some(
                        "Chck if file exists

# Auth : FIN

# Arguments

- a_filename : A file name to audit ( trimmed )

# Example

$exist(file.txt)"
                            .to_string(),
                    )
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "grepf",
[(ValueType::Text,"a_expr"),(ValueType::Path, "a_file"),],
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
                    )
                )
            ),
            (
                FMacroSign::new(
                    "syscmd",
[(ValueType::CText,"a_command"),],
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
                    )
                )
            ),
            (
                FMacroSign::new(
                    "tempout",
[(ValueType::Text,"a_content"),],
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
                    )
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "tempto",
[(ValueType::Path,"a_filename"),],
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
                    )
                ).no_ret()
            ),
            (
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
                    )
                ).ret(ValueType::Path)
            ),
            (
                FMacroSign::new(
                    "fileout",
[(ValueType::Path,"a_filename"),(ValueType::Bool, "a_truncate"),(ValueType::Text, "a_content"),],
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
                    )
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "update",
[(ValueType::Text,"a_text"),],
                    Self::update_storage,
                    Some(
                        "Update a storage

# Arguments

- a_text : Text to update into a storage

# Example

$update(text to be pushed)"
                            .to_string(),
                    )
                ).no_ret()
            ),
            (
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
                ).no_ret()
            ),
        ]));

        #[cfg(feature = "cindex")]
        {
            map.macros.insert(
                "addcsv".to_string(),
                FMacroSign::new(
                    "addcsv",
                    [
                        (ValueType::CText, "a_table_name"),
                        (ValueType::CText, "a_data"),
                    ],
                    Self::cindex_register,
                    Some(
                        "Register a csv table

- Querying can be only applied to registered table.

# Arguments

- a_table_name : A table name to be registered ( trimmed )
- a_data       : Csv data ( trimmed )

# Example

$addcsv(table1,a,b,c
1,2,3)"
                            .to_string(),
                    ),
                )
                .no_ret(),
            );
            map.macros.insert(
                "dropcsv".to_string(),
                FMacroSign::new(
                    "dropcsv",
                    [(ValueType::CText, "a_table_name")],
                    Self::cindex_drop,
                    Some(
                        "Drop a csv table

# Arguments

- a_table_name : A csv table name to drop ( trimmed )

# Example

$dropcsv(table1)"
                            .to_string(),
                    ),
                )
                .no_ret(),
            );
            map.macros.insert(
                "query".to_string(),
                FMacroSign::new(
                    "query",
                    [(ValueType::CText, "a_query")],
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
            map.macros.insert(
                "time".to_string(),
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
            map.macros.insert(
                "date".to_string(),
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
            map.macros.insert(
                "hms".to_string(),
                FMacroSign::new(
                    "hms",
                    [(ValueType::Uint, "a_second")],
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
            map.macros.insert(
                "ftime".to_string(),
                FMacroSign::new(
                    "ftime",
                    [(ValueType::Path, "a_file")],
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
            map.macros.insert(
                "eval".to_string(),
                FMacroSign::new(
                    "eval",
                    [(ValueType::Text, "a_expr")],
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
            map.macros.insert(
                "evalk".to_string(),
                FMacroSign::new(
                    "evalk",
                    [(ValueType::Text, "a_expr")],
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
            map.macros.insert(
                "evalf".to_string(),
                FMacroSign::new(
                    "evalf",
                    [(ValueType::Text, "a_expr")],
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
            map.macros.insert(
                "evalkf".to_string(),
                FMacroSign::new(
                    "evalkf",
                    [(ValueType::Text, "a_expr")],
                    Self::eval_keep_as_float,
                    None,
                ),
            );
            map.macros.insert(
                "pie".to_string(),
                FMacroSign::new("pie", [(ValueType::Text, "a_expr")], Self::pipe_ire, None),
            );
            map.macros.insert(
                "mie".to_string(),
                FMacroSign::new("mie", [(ValueType::Text, "a_expr")], Self::macro_ire, None),
            );
        }
        map.macros.insert(
            "wrap".to_string(),
            FMacroSign::new(
                "wrap",
                [(ValueType::Uint, "a_width"), (ValueType::Text, "a_text")],
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
            map.macros.insert(
                "hookon".to_string(),
                FMacroSign::new(
                    "hookon",
                    [
                        (ValueType::Enum, "a_macro_type"),
                        (ValueType::CText, "a_target_name"),
                    ],
                    Self::hook_enable,
                    Some("Enable hook which is enabled by library extension".to_string()),
                )
                .enum_table(ETable::new("a_macro_type").candidates(&["macro", "char"])),
            );
            map.macros.insert(
                "hookoff".to_string(),
                FMacroSign::new(
                    "hookoff",
                    [
                        (ValueType::Enum, "a_macro_type"),
                        (ValueType::CText, "a_target_name"),
                    ],
                    Self::hook_disable,
                    Some("Disable hook".to_string()),
                )
                .enum_table(ETable::new("a_macro_type").candidates(&["macro", "char"])),
            );
        }
        map
    }

    /// Add new macro extension from macro builder
    pub(crate) fn new_ext_macro(&mut self, ext: ExtMacroBuilder) {
        // TODO TT
        // if let Some(ExtMacroBody::Function(mac_ref)) = ext.macro_body {
        //     let sign = FMacroSign::new(&ext.macro_name, &ext.args, mac_ref, ext.macro_desc);
        //     self.macros.insert(ext.macro_name, sign);
        // }
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
    params: Vec<Parameter>,
    optional: Option<Parameter>,
    enum_table: ETMap,
    pub logic: FunctionMacroType,
    #[allow(dead_code)]
    pub desc: Option<String>,
    pub ret: Option<ValueType>,
}

impl FMacroSign {
    pub fn new(
        name: &str,
        params: impl IntoIterator<Item = (ValueType, impl AsRef<str>)>,
        logic: FunctionMacroType,
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
            enum_table: ETMap::default(),
            logic,
            desc,
            ret: Some(ValueType::Text),
        }
    }

    pub fn no_ret(mut self) -> Self {
        self.ret = None;
        self
    }

    pub fn ret(mut self, ret_type: ValueType) -> Self {
        self.ret.replace(ret_type);
        self
    }

    pub fn optional(mut self, param: Parameter) -> Self {
        self.optional.replace(param);
        self
    }

    pub fn enum_table(mut self, table: (String, ETable)) -> Self {
        self.enum_table.tables.insert(table.0, table.1);
        self
    }
}

impl From<&FMacroSign> for crate::sigmap::MacroSignature {
    fn from(bm: &FMacroSign) -> Self {
        Self {
            variant: crate::sigmap::MacroVariant::Function,
            name: bm.name.to_owned(),
            params: bm.params.to_owned(),
            optional: bm.optional.clone(),
            enum_table: bm.enum_table.clone(),
            expr: bm.to_string(),
            desc: bm.desc.clone(),
            return_type: bm.ret,
        }
    }
}

impl std::fmt::Display for FMacroSign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut inner = self.params.iter().fold(String::new(), |acc, param| {
            acc + &param.arg_type.to_string() + ","
        });
        // This removes last "," character
        inner.pop();
        let basic_usage = format!("${}({}", self.name, inner); // Without ending brace
        let ret = write!(f, "{})", basic_usage);
        let sep = if inner.is_empty() { "" } else { "," };
        if let Some(op) = self.optional.as_ref() {
            write!(f, " || {}{}{}?)", basic_usage, sep, op.arg_type)
        } else {
            ret
        }
    }
}

impl FromIterator<FMacroSign> for FunctionMacroMap {
    fn from_iter<T: IntoIterator<Item = FMacroSign>>(iter: T) -> Self {
        let mut m = HashMap::new();
        for sign in iter {
            m.insert(sign.name.clone(), sign);
        }
        Self { macros: m }
    }
}
