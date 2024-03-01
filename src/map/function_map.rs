//! # Function macro module
//!
//! Function macro module includes struct and methods related to function macros
//! which are technically function pointers.

use crate::argument::{MacroInput, ValueType};
use crate::common::*;
use crate::consts::ESR;
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
                    ESR,
                    Self::get_pipe,
                    Some(man_fun!("pipe.r4d")),
                ).optional(Parameter::new(ValueType::CText,"a_pipe_name"))
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
                    Some(man_fun!("hygiene.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "indentl",
[(ValueType::Text,"a_indenter"),(ValueType::Text, "a_lines"),],
                    Self::indent_lines_before,
                    Some(man_fun!("indentl.r4d")),
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
                    Some(man_fun!("index.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "indexl",
[(ValueType::Uint,"a_index"),(ValueType::Text, "a_lines"),],
                    Self::index_lines,
                    Some(man_fun!("indexl.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "import",
[(ValueType::Path,"a_file"),],
                    Self::import_frozen_file,
                    Some(man_fun!("import.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "join",
[(ValueType::Text,"a_sep"),(ValueType::Text,"a_array"),],
                    Self::join,
                    Some(man_fun!("join.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "joinl",
[(ValueType::Text,"a_sep"),(ValueType::Text,"a_lines"),],
                    Self::join_lines,
                    Some(man_fun!("joinl.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "len",
[(ValueType::Text,"a_string"),],
                    Self::len,
                    Some(man_fun!("len.r4d")),
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
                    Some(man_fun!("log.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "loge",
[(ValueType::Text,"a_msg"),],
                    Self::log_error_message,
                    Some(man_fun!("loge.r4d")),
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
                    Some(man_fun!("lt.r4d")),
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "lte",
[(ValueType::Text,"a_lvalue"),(ValueType::Text, "a_rvalue"),],
                    Self::less_than_or_equal,
                    Some(man_fun!("lte.r4d")),
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "max",
                    ESR,
                    Self::get_max,
                    Some(man_fun!("max.r4d")),
                ).optional( Parameter::new(ValueType::Text,"a_array"))
            ),
            (
                FMacroSign::new(
                    "min",
                    ESR,
                    Self::get_min,
                    Some(man_fun!("min.r4d")),
                ).optional( Parameter::new(ValueType::Text,"a_array"))
            ),
            (
                FMacroSign::new(
                    "name",
[(ValueType::Path,"a_path"),],
                    Self::get_name,
                    Some(man_fun!("name.r4d")),
                ).ret(ValueType::Path)
            ),
            (
                FMacroSign::new(
                    "nassert",
[(ValueType::Text,"a_lvalue"),(ValueType::Text, "a_rvalue"),],
                    Self::assert_ne,
                    Some(man_fun!("nassert.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "not",
[(ValueType::Bool,"a_boolean"),],
                    Self::not,
                    Some(man_fun!("not.r4d")),
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "num",
[(ValueType::Text,"a_text"),],
                    Self::get_number,
                    Some(man_fun!("num.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "nl",
                    ESR,
                    Self::newline,
                    Some(man_fun!("nl.r4d")),
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
                    Some(man_fun!("ostype.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "output",
                    [(ValueType::Enum,"a_output_type"),],
                    Self::require_output,
                    Some(man_fun!("output.r4d")),
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
                    Some(man_fun!("panic.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "parent",
[(ValueType::Path,"a_path"),],
                    Self::get_parent,
                    Some(man_fun!("parent.r4d")),
                ).ret(ValueType::Path)
            ),
            (
                FMacroSign::new(
                    "path",
                    [(ValueType::Text,"a_path")],
                    Self::merge_path,
                    Some(man_fun!("path.r4d")),
                    ).optional(Parameter::new(ValueType::Text,"a_sub_path"))
                    .ret(ValueType::Path)
            ),
            (
                FMacroSign::new(
                    "pause",
[(ValueType::Bool,"a_pause"),],
                    Self::pause,
                    Some(man_fun!("pause.r4d")),
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
                    Some(man_fun!("pipe.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "pipeto",
[(ValueType::CText,"a_pipe_name"),(ValueType::Text, "a_value"),],
                    Self::pipe_to,
                    Some(man_fun!("pipeto.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "prec",
[(ValueType::Float,"a_number"),(ValueType::Uint, "a_precision"),],
                    Self::prec,
                    Some(man_fun!("prec.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "println",
[(ValueType::Text,"a_message"),],
                    Self::print_message,
                    Some("print message -> discard option is ignored for this macro + it has trailing new line for pretty foramtting".to_string()),
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
                    Some(man_fun!("reo.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "rev",
[(ValueType::Text,"a_array"),],
                    Self::reverse_array,
                    Some(man_fun!("rev.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "sub",
[(ValueType::Text,"a_expr"),(ValueType::Text, "a_target"),(ValueType::Text, "a_source"),],
                    Self::regex_sub,
                    Some(man_fun!("sub.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "addexpr",
[(ValueType::CText,"a_name"),(ValueType::Text, "a_expr"),],
                    Self::register_expression,
                    Some(man_fun!("addexpr.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "rename",
[(ValueType::CText,"a_macro_name"),(ValueType::CText, "a_new_name"),],
                    Self::rename_call,
                    Some(man_fun!("rename.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "repeat",
[(ValueType::Uint,"a_count"),(ValueType::Text, "a_source"),],
                    Self::repeat,
                    Some(man_fun!("repeat.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "repl",
[(ValueType::CText,"a_macro_name"),(ValueType::Text, "a_new_value"),],
                    Self::replace,
                    Some(man_fun!("repl.r4d")),
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
                    Some(man_fun!("source.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "sort",
                    [(ValueType::Enum,"a_sort_type"),(ValueType::Text,"a_array"),],
                    Self::sort_array,
                    Some(man_fun!("sort.r4d")),
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
                    Some(man_fun!("sortl.r4d")),
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
                    Some(man_fun!("static.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "strict",
                    [(ValueType::Enum,"a_mode"),],
                    Self::require_strict,
                    Some(man_fun!("strict.r4d")),
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
                    Some(man_fun!("tail.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "taill",
[(ValueType::Uint,"a_count"),(ValueType::Text, "a_content"),],
                    Self::tail_line,
                    Some(man_fun!("taill.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "table",
                    [(ValueType::Enum,"a_table_form"),(ValueType::CText, "a_csv_value"),],
                    Self::table,
                    Some(man_fun!("table.r4d")),
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
                    Some(man_fun!("tr.r4d")),
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
                    Some(man_fun!("triml.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "exdent",
[(ValueType::CText,"a_trim_option"),(ValueType::Text,"a_lines"),],
                    Self::exdent,
                    Some(man_fun!("exdent.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "undef",
[(ValueType::CText,"a_macro_name"),],
                    Self::undefine_call,
                    Some(man_fun!("undef.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "unicode",
[(ValueType::CText,"a_value"),],
                    Self::paste_unicode,
                    Some(man_fun!("unicode.r4d")),
                )
            ),
            (
                FMacroSign::new(
                    "until",
[(ValueType::Text,"a_pattern"),(ValueType::Text, "a_content"),],
                    Self::get_slice_until,
                    Some(man_fun!("until.r4d")),
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
                    Some(man_fun!("def.r4d")),
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "env",
[(ValueType::CText,"a_env_name"),],
                    Self::get_env,
                    Some(man_fun!("env.r4d"))
                )
            ),
            (
                FMacroSign::new(
                    "setenv",
[(ValueType::CText,"a_env_name"),(ValueType::Text, "a_env_value"),],
                    Self::set_env,
                    Some(man_fun!("setenv.r4d"))
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "abs",
[(ValueType::Path,"a_path"),],
                    Self::absolute_path,
                    Some(man_fun!("abs.r4d"))
                )
            ),
            (
                FMacroSign::new(
                    "exist",
[(ValueType::Path,"a_filename"),],
                    Self::file_exists,
                    Some(man_fun!("exist.r4d"))
                ).ret(ValueType::Bool)
            ),
            (
                FMacroSign::new(
                    "grepf",
                    [(ValueType::Text,"a_expr"),(ValueType::Path, "a_file"),],
                    Self::grep_file,
                    Some(man_fun!("grepf.r4d"))
                )
            ),
            (
                FMacroSign::new(
                    "syscmd",
[(ValueType::CText,"a_command"),],
                    Self::syscmd,
                    Some(man_fun!("syscmd.r4d"))
                )
            ),
            (
                FMacroSign::new(
                    "tempout",
[(ValueType::Text,"a_content"),],
                    Self::temp_out,
                    Some(man_fun!("tempout.r4d"))
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "tempto",
[(ValueType::Path,"a_filename"),],
                    Self::set_temp_target,
                    Some(man_fun!("tempto.r4d"))
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "temp",
                    ESR,
                    Self::get_temp_path,
                    Some(man_fun!("temp.r4d"))
                ).ret(ValueType::Path)
            ),
            (
                FMacroSign::new(
                    "fileout",
[(ValueType::Path,"a_filename"),(ValueType::Bool, "a_truncate"),(ValueType::Text, "a_content"),],
                    Self::file_out,
                    Some(man_fun!("fileout.r4d"))
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "update",
[(ValueType::Text,"a_text"),],
                    Self::update_storage,
                    Some(man_fun!("update.r4d"))
                ).no_ret()
            ),
            (
                FMacroSign::new(
                    "extract",
                    ESR,
                    Self::extract_storage,
                    Some(man_fun!("extract.r4d")),
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
                    Some(man_fun!("addcsv.r4d")),
                )
                .no_ret(),
            );
            map.macros.insert(
                "dropcsv".to_string(),
                FMacroSign::new(
                    "dropcsv",
                    [(ValueType::CText, "a_table_name")],
                    Self::cindex_drop,
                    Some(man_fun!("dropcsv.r4d")),
                )
                .no_ret(),
            );
            map.macros.insert(
                "query".to_string(),
                FMacroSign::new(
                    "query",
                    [(ValueType::CText, "a_query")],
                    Self::cindex_query,
                    Some(man_fun!("query.r4d")),
                ),
            );
        }

        #[cfg(feature = "chrono")]
        {
            map.macros.insert(
                "time".to_string(),
                FMacroSign::new("time", ESR, Self::time, Some(man_fun!("time.r4d"))),
            );
            map.macros.insert(
                "date".to_string(),
                FMacroSign::new("date", ESR, Self::date, Some(man_fun!("date.r4d"))),
            );
            map.macros.insert(
                "hms".to_string(),
                FMacroSign::new(
                    "hms",
                    [(ValueType::Uint, "a_second")],
                    Self::hms,
                    Some(man_fun!("hms.r4d")),
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
                    Some(man_fun!("ftime.r4d")),
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
                    Some(man_fun!("eval.r4d")),
                ),
            );
            map.macros.insert(
                "evalk".to_string(),
                FMacroSign::new(
                    "evalk",
                    [(ValueType::Text, "a_expr")],
                    Self::eval_keep,
                    Some(man_fun!("evalk.r4d")),
                ),
            );
            map.macros.insert(
                "evalf".to_string(),
                FMacroSign::new("evalf", [(ValueType::Text, "a_expr")], Self::evalf, None),
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
                Some(man_fun!("wrap.r4d")),
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
