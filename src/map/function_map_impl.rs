use super::function_map::FunctionMacroMap;

use crate::argument::MacroInput;
use crate::auth::{AuthState, AuthType};
use crate::common::VarContOperation;
use crate::common::{
    AlignType, ErrorBehaviour, FlowControl, LineUpType, MacroType, OrderType, OutputType,
    ProcessInput, RadResult, RelayTarget,
};
use crate::consts::{
    LOREM, LOREM_SOURCE, LOREM_WIDTH, MACRO_SPECIAL_LIPSUM, MAIN_CALLER, PATH_SEPARATOR,
};
use crate::parser::{ArgParser, SplitVariant};

use crate::env::MacEnv;
use crate::error::RadError;
use crate::formatter::Formatter;
#[cfg(feature = "hook")]
use crate::hookmap::HookType;
use crate::logger::WarningType;
use crate::utils::{RadStr, RegCow, Utils, NUM_MATCH, UNUM_MATCH};
use crate::{CommentType, WriteOption};
use crate::{Hygiene, Processor};
#[cfg(feature = "cindex")]
use cindex::OutOption;
#[cfg(feature = "evalexpr")]
use evalexpr::eval;
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use unicode_width::UnicodeWidthStr;

static ISOLATION_SURR_SPACE: [char; 1] = ['='];
static ISOLATION_SINGLE_SPACE: [char; 3] = [',', ':', ';'];
static ISOLATION_CHARS: [char; 6] = ['(', ')', '[', ']', '{', '}'];
static ISOLATION_CHARS_OPENING: [char; 3] = ['(', '[', '{'];
static ISOLATION_CHARS_CLOSING: [char; 3] = [')', ']', '}'];

static BYTE_CHARS_OPENING: [u8; 3] = [b'(', b'[', b'{'];
static BYTE_CHARS_CLOSING: [u8; 3] = [b')', b']', b'}'];

/// Regex for leading and following spaces
static LSPA: Lazy<Regex> = Lazy::new(|| Regex::new(r"(^[^\S\r\n]+)").unwrap());
static FSPA: Lazy<Regex> = Lazy::new(|| Regex::new(r"([^\S\r\n]+$)").unwrap());
// ----------
// rer related regexes
//
// 1. leading space & tabs
// 2. Numbers ( could be multiple )
// 3. Any character except space, tab, newline
// This regex consists of two groups
// 1. Leading spaces which represents nested level
// 2. Index characters which acts an index
static BLANKHASH_MATCH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(^[\s\t]*)\d+([^\d\s]+)\s+"#).expect("Failed to create blank regex")
});

// This is similar to blankhash but for replacing purpose
static REPLACER_MATCH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(^[\s\t]*)(\d+)([^\d\s]+)(\s+)"#).expect("Failed to create replacer regex")
});
// ----------

/// Two lines match
static TWO_NL_MATCH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(\n|\r\n)\s*(\n|\r\n)"#).expect("Failed to create tow nl regex"));

// TODO TT
// /// Path separator match
// static PATH_MATCH: Lazy<Regex> =
//     Lazy::new(|| Regex::new(r#"(\\|/)"#).expect("Failed to create path separator matches"));

// Macros implemnation
impl FunctionMacroMap {
    // ==========
    // Function Macros
    // ==========
    /// Print out current time
    ///
    /// # Usage
    ///
    /// $time()
    #[cfg(feature = "chrono")]
    pub(crate) fn time(_: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn hms(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let seconds = args.get_uint(0)?;
        let hour = seconds / 3600;
        let minute = seconds % 3600 / 60;
        let second = seconds % 3600 % 60;
        let time = format!("{:02}:{:02}:{:02}", hour, minute, second);
        Ok(Some(time))
    }

    /// Print out current date
    ///
    /// # Usage
    ///
    /// $date()
    #[cfg(feature = "chrono")]
    pub(crate) fn date(_: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(format!(
            "{}",
            chrono::offset::Local::now().format("%Y-%m-%d")
        )))
    }

    /// Substitute the given source with following match expressions
    ///
    /// # Usage
    ///
    /// $sub(expression,substitution,source)
    pub(crate) fn regex_sub(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let match_expr = args.get_text(0)?;
        let substitution = args.get_text(1)?;
        let source = args.get_text(2)?;

        if match_expr.is_empty() {
            return Err(RadError::InvalidArgument(
                "Regex expression cannot be an empty string".to_string(),
            ));
        }

        let reg = p.try_get_or_create_regex(match_expr)?;
        let ret = reg.replace_all(source, substitution).to_string();
        p.insert_regex(match_expr, reg.to_option())?;
        Ok(Some(ret))
    }

    /// Print current file input
    ///
    /// $input()
    pub(crate) fn print_current_input(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        match &p.state.current_input {
            ProcessInput::Stdin => Ok(Some("Stdin".to_string())),
            ProcessInput::File(path) => {
                let args = ArgParser::new().args_with_len(input)?;
                if !args.is_empty() {
                    let print_absolute = args.get_bool(0)?;
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
    #[cfg(feature = "chrono")]
    pub(crate) fn get_file_time(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let path = args.get_path(0)?;
        if !path.exists() {
            return Err(RadError::InvalidArgument(format!(
                "Cannot get a filetime from a non-existent file : \"{}\"",
                path.display()
            )));
        }
        let time: chrono::DateTime<chrono::Utc> = std::fs::metadata(path)?.modified()?.into();
        Ok(Some(time.format("%Y-%m-%d %H:%m:%S").to_string()))
    }

    /// Find an occurrence form a source
    ///
    /// # Usage
    ///
    /// $find(regex_match,source)
    pub(crate) fn find_occurence(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let match_expr = args.get_text(0)?;
        let source = args.get_text(1)?;

        if match_expr.is_empty() {
            return Err(RadError::InvalidArgument(
                "Regex expression cannot be an empty string".to_string(),
            ));
        }

        let reg = p.try_get_or_create_regex(match_expr)?;
        let ret = reg.is_match(source).to_string();
        p.insert_regex(match_expr, reg.to_option())?;
        Ok(Some(ret))
    }

    /// Find multiple occurrence form a source
    ///
    /// # Usage
    ///
    /// $findm(regex_match,source)
    pub(crate) fn find_multiple_occurence(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;
        let match_expr = args.get_text(0)?;
        let source = args.get_text(1)?;

        if match_expr.is_empty() {
            return Err(RadError::InvalidArgument(
                "Regex expression cannot be an empty string".to_string(),
            ));
        }

        let reg = p.try_get_or_create_regex(match_expr)?;
        let ret = reg.find_iter(source).count().to_string();
        p.insert_regex(match_expr, reg.to_option())?;
        Ok(Some(ret))
    }

    /// Evaluate given expression
    ///
    /// This returns true, false or evaluated number
    ///
    /// # Usage
    ///
    /// $eval(expression)
    #[cfg(feature = "evalexpr")]
    pub(crate) fn eval(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let formula = args.get_text(0)?;
        let mut result = evalexpr::eval(formula)?.to_string();
        if p.env.retain_formula {
            if p.env.formula_space {
                result.insert(0, ' ');
            }
            result.insert(0, '=');
            result.insert_str(0, formula);
        }
        Ok(Some(result))
    }

    /// Evaluate given expression but force floating point
    ///
    /// This returns true, false or evaluated number
    ///
    /// # Usage
    ///
    /// $evalf(expression)
    #[cfg(feature = "evalexpr")]
    pub(crate) fn evalf(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let formula = args.get_text(0)?;
        let mut new_formula = String::with_capacity(formula.len());
        let mut last_match = 0;
        for caps in UNUM_MATCH.captures_iter(formula) {
            let m = caps.get(0).unwrap();
            new_formula.push_str(&formula[last_match..m.start()]);
            if let Ok(num) = m.as_str().parse::<usize>() {
                new_formula.push_str(&(num.to_string() + ".0"));
            } else {
                new_formula.push_str(m.as_str());
            }
            last_match = m.end();
        }
        new_formula.push_str(&formula[last_match..]);

        let mut result = evalexpr::eval(&new_formula)?.to_string();
        if p.env.retain_formula {
            if p.env.formula_space {
                result.insert(0, ' ');
            }
            result.insert(0, '=');
            result.insert_str(0, formula);
        }
        Ok(Some(result))
    }

    /// Pipe in replace evaluation
    ///
    /// # Usage
    ///
    /// $pie(expression)
    #[cfg(feature = "evalexpr")]
    pub(crate) fn pipe_ire(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let mut formula = args.get_text(0)?.to_string();
        let pipe = p.state.get_pipe("-", true).unwrap_or("".to_string());
        let replaced = if formula.contains('p') {
            formula.replace('p', &pipe)
        } else {
            formula.insert_str(0, &pipe);
            formula
        };
        let result = evalexpr::eval(&replaced)?;
        p.state.add_pipe(None, result.to_string());
        Ok(None)
    }

    /// Macro in replace evaluation
    ///
    /// # Usage
    ///
    /// $mie(macro,expression)
    #[cfg(feature = "evalexpr")]
    pub(crate) fn macro_ire(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let macro_name = args.get_ctext(0)?;
        let mut formula = args.get_text(1)?.to_string();
        let body = p.get_runtime_macro_body(macro_name)?;
        let replaced = if formula.contains('m') {
            formula.replace('m', body)
        } else {
            formula.insert_str(0, body);
            formula
        };
        let result = evalexpr::eval(&replaced)?;
        p.replace_macro(macro_name, &result.to_string());
        Ok(None)
    }

    /// Negate given value
    ///
    /// This returns true, false or evaluated number
    ///
    /// # Usage
    ///
    /// $not(expression)
    pub(crate) fn not(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        // No need to trim right now because is_arg_true trims already
        // Of course, it returns cow so it doesn't create overhead anyway
        let args = args.get_bool(0)?;
        Ok(Some((!args).to_string()))
    }

    // TODO TT
    // This is optioanl
    /// Container macro
    ///
    /// # Usage
    ///
    /// $cont(operation,argument: Optional)
    pub(crate) fn container(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;
        let op = args.get_enum(0, VarContOperation::from_str)?;
        let var = args.get_text(1).unwrap_or("");
        let ret = match op {
            VarContOperation::Pop => p.var_container.pop(),
            VarContOperation::Push => {
                p.var_container.push(var.to_string());
                None
            }
            VarContOperation::Clear => {
                p.var_container.clear();
                None
            }
            VarContOperation::Print => Some(format!("{:?}", p.var_container)),
            VarContOperation::List => p.var_container.join(",").into(),
            VarContOperation::Top => p.var_container.last().cloned(),
            VarContOperation::Len => p.var_container.len().to_string().into(),
            VarContOperation::Get => p
                .var_container
                .get(var.trim().parse::<usize>().map_err(|_| {
                    RadError::InvalidArgument(format!(
                        "Cannt index a container with invalid integer \"{}\"",
                        var
                    ))
                })?)
                .cloned(),
            VarContOperation::Set => {
                match p.var_container.last_mut() {
                    Some(v) => *v = var.to_string(),
                    None => {
                        return Err(RadError::InvalidExecution(
                            "Set has failed because current container is empty".to_string(),
                        ))
                    }
                }
                None
            }
            VarContOperation::Extend => {
                if var.is_empty() {
                    p.log_warning(
                        "No content was given to extend for container",
                        WarningType::Sanity,
                    )?;
                }

                // It is required to utilize split by charater because other operations require
                // unsplitted argument input.
                for item in var.split(',') {
                    p.var_container.push(item.to_string());
                }
                None
            }
        };

        Ok(ret)
    }

    /// Trim preceding and trailing whitespaces (' ', '\n', '\t', '\r')
    ///
    /// # Usage
    ///
    /// $trim(expression)
    pub(crate) fn trim(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;
        Ok(Some(args.get_ctext(0)?.to_string()))
    }

    /// Trim preceding whitespaces (' ', '\n', '\t', '\r')
    ///
    /// # Usage
    ///
    /// $trimf(expression)
    pub(crate) fn trimf(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        Ok(Some(args.get_text(0)?.trim_start().to_string()))
    }

    /// Trim trailing whitespaces (' ', '\n', '\t', '\r')
    ///
    /// # Usage
    ///
    /// $trimr(expression)
    pub(crate) fn trimr(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let content = args.get_text(0)?;
        let trailer = if content.ends_with('\n') {
            "\n"
        } else if content.ends_with("\r\n") {
            "\r\n"
        } else {
            ""
        };
        let mut content = content.trim_end().to_string();
        content.push_str(trailer);
        Ok(Some(content))
    }

    /// Get inner text from given src
    ///
    /// This doesn't support utf-8 character but only ASCII
    pub(crate) fn get_inner(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;
        let rule = args.get_ctext(0)?.as_bytes();
        if rule.len() != 2 {
            return Err(RadError::InvalidArgument(format!(
                "Inner rule should consists of two ascii characters but given {}",
                args.get_text(0)?
            )));
        }
        let (rs, re) = (rule[0], rule[1]);
        let target_count = args.get_uint(1)?;
        let src = args.get_text(2)?;
        let mut cursors: Vec<InnerCursor> = vec![];
        let mut opened_count = 0usize; // This only gets increased and don't return to 0
        let mut current_count = 0usize; // This goes to 0
        for (idx, ch) in src.bytes().enumerate() {
            // Start ch match
            if ch == rs {
                if current_count == 0 {
                    // Match start first
                    opened_count += 1;
                    current_count += 1;
                    cursors.push(InnerCursor {
                        start_index: idx,
                        end_index: idx,
                        level: opened_count,
                    });
                } else if ch == re {
                    // Update count
                    // opened_count -= 1;
                    // End match
                    if target_count == cursors.last().map(|s| s.level).unwrap_or(0) {
                        let start = cursors.last().unwrap().start_index + 1;
                        return Ok(Some(src[start..idx].to_string()));
                    }

                    cursors.pop();
                    current_count -= 1;
                } else {
                    // Nested content
                    opened_count += 1;
                    current_count += 1;
                    cursors.push(InnerCursor {
                        start_index: idx,
                        end_index: idx,
                        level: opened_count,
                    });
                }
            } else if ch == re && current_count > 0 {
                // End ch match
                // End match
                if target_count == cursors.last().map(|s| s.level).unwrap_or(0) {
                    let start = cursors.last().unwrap().start_index + 1;
                    return Ok(Some(src[start..idx].to_string()));
                }
                cursors.pop();
                current_count -= 1;
            }
            if current_count > 0 {
                if let Some(cur) = cursors.last_mut() {
                    cur.end_index = idx;
                }
            }
        }

        for cur in cursors.iter().rev() {
            if cur.level == target_count {
                let start = cur.start_index + 1;
                let end = cur.end_index;
                return Ok(Some(src[start..end].to_string()));
            }
        }

        Err(RadError::InvalidArgument(format!(
            "Given source does not have corresponding pair \"{}\" with given count \"{}\"",
            args.get_ctext(0)?,
            target_count,
        )))
    }

    /// Indent lines before
    ///
    /// # Usage
    ///
    /// $indentl(*, multi
    /// line
    /// expression
    /// )
    pub(crate) fn indent_lines_before(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let indenter = args.get_text(0)?;
        let indented = args
            .get_text(1)?
            .full_lines()
            .map(|line| {
                if !line.is_empty() {
                    let mut l = line.to_string();
                    l.insert_str(0, indenter);
                    Cow::from(l)
                } else {
                    Cow::from("")
                }
            })
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v);
                acc
            });

        Ok(Some(indented))
    }

    /// Attach content after lines
    ///
    /// # Usage
    ///
    /// $attachl(*, multi
    /// line
    /// expression
    /// )
    pub(crate) fn attach_lines_after(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let indenter = args.get_text(0)?;
        let indented = args
            .get_text(1)?
            .full_lines()
            .map(|line| {
                if !line.is_empty() {
                    let line_end = line.get_line_ending().len();
                    let mut l = line.to_string();
                    l.insert_str(line.len() - line_end, indenter);
                    Cow::from(l)
                } else {
                    Cow::from("")
                }
            })
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v);
                acc
            });

        Ok(Some(indented))
    }

    /// Trim preceding and trailing whitespaces (' ', '\n', '\t', '\r') but for all lines
    ///
    /// # Usage
    ///
    /// $triml(\t multi
    /// \t line
    /// \t expression
    /// )
    pub(crate) fn triml(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let lines = args.get_text(0)?.trim_each_lines();
        Ok(Some(lines))
    }

    /// Trim lines ( exdent ) with given amount
    ///
    /// # Usage
    ///
    /// $exdent(min,
    /// \t multi
    /// \t line
    /// \t expression
    /// )
    pub(crate) fn exdent(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let option = args.get_ctext(0)?;
        let source = args.get_text(1)?;
        let mut try_amount = None;
        let min_amount = match option {
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
            v => {
                try_amount = Some(option.parse::<usize>().map_err(|_| {
                    RadError::InvalidArgument(format!(
                        "Exdent option should be either min,max or number gut given \"{}\"",
                        v
                    ))
                })?);
                None
            }
        };

        let mut lines = String::new();
        let source_iter = source.full_lines().peekable();
        for line in source_iter {
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
                        None => line.trim_start().to_string(),
                    },
                };
                lines.push_str(&trimmed);
            }
        }
        Ok(Some(lines))
    }

    /// Collapse lines into a singe line
    ///
    /// This macro eagerly agggreate contents
    ///
    /// # Usage
    ///
    /// $coll(// a $nl() // b)
    pub(crate) fn collapse(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let mut fmt = String::new();
        let mut cont = String::new();
        let pattern = args.get_text(0)?;
        let src = args.get_text(1)?;
        // NOTE
        // I'm trying to implement no-neseter pattern
        for line in src.full_lines() {
            // Collaps-able line
            if let Some((leading, following)) = line.split_once(pattern) {
                if !leading.trim().is_empty() {
                    fmt.push_str(&std::mem::take(&mut cont));
                    continue;
                }
                // Empty leading characters
                // Add whole line
                if cont.is_empty() {
                    cont.push_str(line);
                    continue;
                }

                // Add partial line
                Utils::pop_newline(&mut cont);
                cont.push_str(following);
                continue;
            }
            // Non collaps-able line
            if !cont.is_empty() {
                fmt.push_str(&std::mem::take(&mut cont));
            }
            fmt.push_str(line);
        }

        // Final aggregation
        if !cont.is_empty() {
            fmt.push_str(&std::mem::take(&mut cont));
        }

        Ok(Some(fmt))
    }

    /// Removes duplicate newlines whithin given input
    ///
    /// # Usage
    ///
    /// $chomp(expression)
    pub(crate) fn chomp(input: MacroInput, processor: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let source = args.get_text(0)?;

        let chomp_result = Self::chomp_inner(source, &processor.state.newline);

        Ok(Some(chomp_result.to_string()))
    }

    #[inline]
    fn chomp_inner<'a>(src: &'a str, newline: &'a str) -> Cow<'a, str> {
        TWO_NL_MATCH.replace_all(src, newline.repeat(2))
    }

    /// Both apply trim and chomp to given expression
    ///
    /// # Usage
    ///
    /// $comp(Expression)
    pub(crate) fn compress(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let source = args.get_text(0)?;
        // Chomp and then compress
        let result = Self::chomp_inner(source, &processor.state.newline)
            .trim()
            .to_string();

        Ok(Some(result))
    }

    /// Creates placeholder with given amount of word counts
    ///
    /// # Usage
    ///
    /// $lipsum(Number)
    pub(crate) fn lipsum_words(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let count = args.get_uint(0)?;
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
    }

    /// Creates placeholder with given amount of word counts but for repeated purposes.
    ///
    /// # Usage
    ///
    /// $lipsumr(Number)
    pub(crate) fn lipsum_repeat(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let count = args.get_uint(0)?;
        let mut current_index = match p.get_runtime_macro_body(MACRO_SPECIAL_LIPSUM) {
            Ok(value) => value.parse::<usize>().unwrap(),
            Err(_) => {
                p.add_static_rules(&[(MACRO_SPECIAL_LIPSUM, "0")])?;
                0usize
            }
        };

        if current_index + count <= *LOREM_WIDTH - 1 {
            let mut fin = current_index + count;
            if fin == *LOREM_WIDTH {
                fin = 0;
            }
            // Renew current index in macro
            p.replace_macro(MACRO_SPECIAL_LIPSUM, &fin.to_string());

            Ok(Some(
                LOREM[current_index..=current_index + count - 1].join(" "),
            ))
        } else {
            let mut lorem = String::new();
            let mut rem = count;

            // While there are words to print
            while rem != 0 {
                // Try print until end
                lorem.push_str(
                    &LOREM[current_index..=(current_index + rem - 1).min(*LOREM_WIDTH - 1)]
                        .join(" "),
                );
                // Get "possible" printed count of words
                let printed = if current_index + rem > *LOREM_WIDTH {
                    *LOREM_WIDTH - current_index
                } else {
                    rem
                };
                if rem >= printed {
                    // Not yet final
                    rem -= printed;
                    current_index += printed;
                    if current_index >= *LOREM_WIDTH {
                        current_index = 0;
                    }
                    p.replace_macro(MACRO_SPECIAL_LIPSUM, &current_index.to_string());
                    lorem.push(' ');
                } else {
                    current_index += printed - 1;
                    if current_index >= *LOREM_WIDTH {
                        current_index = 0;
                    }
                    // Final
                    p.replace_macro(MACRO_SPECIAL_LIPSUM, &current_index.to_string());
                    break;
                }
            }
            Ok(Some(lorem))
        }
    }

    /// Repeat given expression about given amount times
    ///
    /// # Usage
    ///
    /// $repeat(count,text)
    pub(crate) fn repeat(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let repeat_count = args.get_uint(0)?;
        let repeat_object = args.get_text(1)?;
        let mut repeated = String::new();
        for _ in 0..repeat_count {
            repeated.push_str(repeat_object);
        }
        Ok(Some(repeated))
    }

    /// Call shell command
    ///
    /// This calls via 'CMD \C' in windows platform and 'sh -c' in *nix system
    ///
    /// # Usage
    ///
    /// $shell(system command -a arguments)
    pub(crate) fn shell_command(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let source = args.get_ctext(0)?;
        let arg_vec = Utils::get_whitespace_split_retain_quote_rule(source);
        let args_ref = arg_vec.iter().map(|s| s.as_ref()).collect::<Vec<_>>();

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .arg("/C")
                .args(args_ref)
                .output()
                .expect("failed to execute process")
        } else {
            let sys_args = if arg_vec.len() > 1 {
                &args_ref[1..]
            } else {
                &[]
            };
            Command::new(args_ref[0])
                .args(sys_args)
                .output()
                .expect("failed to execute process")
        };

        if output.status.success() {
            Ok(Some(String::from_utf8(output.stdout)?))
        } else {
            let error_message = String::from_utf8(output.stderr)?;
            if p.state.behaviour == ErrorBehaviour::Strict {
                Err(RadError::InvalidExecution(format!(
                    "Command \"{}\" failed with message : {}\"{}\"",
                    source, p.state.newline, error_message
                )))
            } else {
                Ok(Some(error_message))
            }
        }
    }

    /// Undefine a macro
    ///
    /// # Usage
    ///
    /// $undef(macro_name)
    pub(crate) fn undefine_call(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new()
            .split(SplitVariant::Always)
            .args_with_len(input)?;

        (0..args.len())
            .map(|idx| {
                let name = args.get_ctext(idx)?;
                Self::try_undefine_macro(name, processor)?;
                Ok(())
            })
            .collect::<RadResult<Vec<_>>>()?;

        Ok(None)
    }

    #[inline]
    fn try_undefine_macro(name: &str, processor: &mut Processor) -> RadResult<()> {
        if processor.contains_macro(name, MacroType::Any) {
            processor.undefine_macro(name, MacroType::Any);
        } else {
            if processor.state.behaviour == ErrorBehaviour::Strict {
                return Err(RadError::UnsoundExecution(format!(
                    "Macro \"{}\" doesn't exist, therefore cannot undefine",
                    name,
                )));
            }

            processor.log_error(&format!(
                "Macro \"{}\" doesn't exist, therefore cannot undefine",
                name,
            ))?;
        }
        Ok(())
    }

    /// Placeholder for define
    pub(crate) fn define_macro(_: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(None)
    }

    /// Squash
    ///
    /// # Usage
    ///
    /// $squash(/,a/b/c)
    pub(crate) fn squash(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let text = args.get_ctext(0)?;
        let new_text = TWO_NL_MATCH.replace_all(text, &p.state.newline);

        Ok(Some(new_text.to_string()))
    }

    /// Split
    ///
    /// # Usage
    ///
    /// $split(/,a/b/c)
    pub(crate) fn split(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let sep = args.get_text(0)?;
        let text = args.get_text(1)?;
        let delimiter = if p.env.split_for_space { ' ' } else { ',' };

        let mut result = text
            .split_terminator(sep)
            .fold(String::new(), |mut acc, v| {
                acc.push_str(v);
                acc.push(delimiter);
                acc
            });
        result.pop();
        Ok(Some(result))
    }

    /// Split by whitespaces and cut
    ///
    /// # Usage
    ///
    /// $scut(0,a/b/c)
    pub(crate) fn split_whitespace_and_cut(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let split = &mut args.get_text(1)?.split_whitespace();
        let len = split.clone().count();

        let index = args.get_int(0)?;

        if index >= len as isize || index < -(len as isize) {
            return Err(RadError::InvalidArgument(format!(
                "Index out of range. Given index is \"{}\" but array length is \"{}\"",
                index, len
            )));
        }

        let final_index = if index < 0 {
            (len as isize + index) as usize
        } else {
            index.max(0) as usize
        };

        if len <= final_index {
            return Err(RadError::InvalidArgument(format!(
                "Index out of range. Given index is \"{}\" but array length is \"{}\"",
                index, len
            )));
        }
        let result = split.nth(final_index).unwrap().to_string();
        Ok(Some(result))
    }

    /// Split and cut
    ///
    /// # Usage
    ///
    /// $cut(/,a/b/c)
    pub(crate) fn split_and_cut(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let sep = args.get_text(0)?;
        let mut split = args.get_text(2)?.split_terminator(sep);
        let len = split.clone().count();

        let index = args.get_int(1)?;

        if index >= len as isize || index < -(len as isize) {
            return Err(RadError::InvalidArgument(format!(
                "Index out of range. Given index is \"{}\" but array length is \"{}\"",
                index, len
            )));
        }

        let final_index = if index < 0 {
            (len as isize + index) as usize
        } else {
            index.max(0) as usize
        };

        if len <= final_index {
            return Err(RadError::InvalidArgument(format!(
                "Index out of range. Given index is \"{}\" but array length is \"{}\"",
                index, len
            )));
        }
        let result = split.nth(final_index).unwrap().to_string();
        Ok(Some(result))
    }

    /// Split whitespaces
    ///
    /// # Usage
    ///
    /// $ssplit(a/b/c)
    pub(crate) fn space_split(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let text = args.get_ctext(0)?;
        let delimiter = if p.env.split_for_space { ' ' } else { ',' };

        let mut result = text.split_whitespace().fold(String::new(), |mut acc, v| {
            acc.push_str(v);
            acc.push(delimiter);
            acc
        });
        result.pop();
        Ok(Some(result))
    }

    /// Assert
    ///
    /// # Usage
    ///
    /// $assert(abc,abc)
    pub(crate) fn assert(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        if args.get_text(0)? == args.get_text(1)? {
            p.track_assertion(true)?;
            Ok(None)
        } else {
            p.track_assertion(false)?;
            Err(RadError::AssertFail)
        }
    }

    /// Assert not equal
    ///
    /// # Usage
    ///
    /// $nassert(abc,abc)
    pub(crate) fn assert_ne(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        if args.get_text(0)? != args.get_text(1)? {
            p.track_assertion(true)?;
            Ok(None)
        } else {
            p.track_assertion(false)?;
            Err(RadError::AssertFail)
        }
    }

    /// Join an array
    ///
    /// # Usage
    ///
    /// $join(" ",a,b,c)
    pub(crate) fn join(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new()
            .split(SplitVariant::Always)
            .args_with_len(input)?;

        let sep = args.get_text(0)?;
        let out = (1..args.len())
            .map(|idx| args.get_text(idx))
            .collect::<RadResult<Vec<&str>>>()?;

        let ret = out.join(sep);

        Ok(Some(ret))
    }

    /// Join lines
    ///
    /// # Usage
    ///
    /// $joinl(" ",a\nb\nc\n)
    pub(crate) fn join_lines(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let sep = args.get_text(0)?;
        let text = args.get_text(1)?;
        let join = text.lines().fold(String::new(), |mut acc, s| {
            acc.push_str(s);
            acc.push_str(sep);
            acc
        });
        Ok(join.strip_suffix(sep).map(|s| s.to_owned()))
    }

    /// Create a table with given format and csv input
    ///
    /// Available formats are 'github', 'wikitext' and 'html'
    ///
    /// # Usage
    ///
    /// $table(github,1,2,3
    /// 4,5,6)
    pub(crate) fn table(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let table_format = args.get_ctext(0)?; // Either gfm, wikitex, latex, none
        let csv_content = args.get_ctext(1)?;
        let result = Formatter::csv_to_table(table_format, csv_content, &p.state.newline)?;
        Ok(Some(result))
    }

    /// Put value into a temporary stack called pipe
    ///
    /// Piped value can be popped with macro '-'
    ///
    /// # Usage
    ///
    /// $pipe(Value)
    pub(crate) fn pipe(input: MacroInput, processor: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        processor
            .state
            .add_pipe(None, args.get_text(0)?.to_string());
        Ok(None)
    }

    /// Put value into a temporary stack called pipe
    ///
    /// Piped value can be popped with macro '-'
    ///
    /// # Usage
    ///
    /// $pipeto(Value)
    pub(crate) fn pipe_to(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        processor
            .state
            .add_pipe(Some(args.get_ctext(0)?), args.get_text(1)?.to_string());
        Ok(None)
    }

    /// Peel enclosed value
    ///
    /// # Usage
    ///
    /// $peel(level,Value)
    pub(crate) fn peel(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let target_level = args.get_uint(0)?;
        let src = args.get_text(1)?;
        let mut chunk_index = 0;
        let mut paren_index = 0;
        let mut level = 0;
        let mut previous = &b' ';
        let result: String;

        for (idx, ch) in src.as_bytes().iter().enumerate() {
            if level == 0 && previous.is_ascii_whitespace() && !ch.is_ascii_whitespace() {
                chunk_index = idx;
            }
            if BYTE_CHARS_OPENING.contains(ch) {
                level += 1;
                if target_level == level {
                    paren_index = idx;
                }
            } else if BYTE_CHARS_CLOSING.contains(ch) {
                if target_level == level {
                    result = src[..chunk_index].to_string() + &src[paren_index + 1..idx];
                    return Ok(Some(result));
                }

                level -= 1;
            }
            // Start ch match
            previous = ch;
        }

        if paren_index != 0 {
            result = src[..chunk_index].to_string() + &src[paren_index + 1..];
            return Ok(Some(result));
        }

        Ok(Some(src.to_string()))
    }

    /// Get environment variable with given name
    ///
    /// # Usage
    ///
    /// $env(SHELL)
    pub(crate) fn get_env(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;
        let env = args.get_ctext(0)?;
        if let Ok(out) = std::env::var(env) {
            Ok(Some(out))
        } else {
            if p.state.behaviour == ErrorBehaviour::Strict {
                p.log_warning(
                    &format!("ENV : \"{}\" is not defined.", env),
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
    pub(crate) fn set_env(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let name = args.get_ctext(0)?;
        let value = args.get_text(1)?;

        if p.state.behaviour == ErrorBehaviour::Strict && std::env::var(name).is_ok() {
            return Err(RadError::UnsoundExecution(format!(
                "You cannot override environment variable in strict mode. Failed to set \"{}\"",
                name
            )));
        }

        std::env::set_var(name, value);
        Ok(None)
    }

    /// Trigger panic
    pub(crate) fn manual_panic(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        p.state.behaviour = ErrorBehaviour::Interrupt;
        Err(RadError::ManualPanic(input.args.to_string()))
    }

    /// Escape processing
    pub(crate) fn escape(_: MacroInput, processor: &mut Processor) -> RadResult<Option<String>> {
        processor.state.flow_control = FlowControl::Escape;
        Ok(None)
    }

    /// Exit processing
    pub(crate) fn exit(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;
        if let Ok(true) = args.get_bool(0) {
            p.state.behaviour = ErrorBehaviour::Exit;
            return Err(RadError::SaneExit);
        }
        p.state.flow_control = FlowControl::Exit;
        Ok(None)
    }

    /// Merge multiple paths into a single path
    ///
    /// This creates platform agonistic path which can be consumed by other macros.
    ///
    /// # Usage
    ///
    /// $path($env(HOME),document,test.docx)
    pub(crate) fn merge_path(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let vec = ArgParser::new()
            .split(SplitVariant::Always)
            .args_with_len(input)?;

        let out = (0..vec.len())
            .map(|idx| {
                let value = vec.get_path(idx)?;
                Ok(value)
            })
            .collect::<RadResult<PathBuf>>()?;

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
    /// This prints spaces by tab_width amount if RAD_TAB_WIDTH is set as value
    ///
    /// If not, it prints tab
    ///
    /// # Usage
    ///
    /// $tab()
    pub(crate) fn print_tab(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;
        let count = if let Ok(c) = args.get_uint(0) { c } else { 1 };

        let tab_width = p.env.rad_tab_width;
        match tab_width {
            Some(value) => {
                let tab = " ".repeat(value);
                Ok(Some(tab.repeat(count)))
            }
            None => Ok(Some("\t".repeat(count))),
        }
    }

    /// Print a literal percent
    ///
    /// # Usage
    ///
    /// $percent()
    pub(crate) fn print_percent(_: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some('%'.to_string()))
    }

    /// Print a literal comma
    ///
    /// # Usage
    ///
    /// $comma()
    pub(crate) fn print_comma(_: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(','.to_string()))
    }

    /// Yield spaces
    ///
    /// # Usage
    ///
    /// $space()
    pub(crate) fn space(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;
        let count = if let Ok(c) = args.get_uint(0) { c } else { 1 };

        Ok(Some(" ".repeat(count)))
    }

    /// Path separator
    ///
    /// # Usage
    ///
    /// $PS()
    pub(crate) fn path_separator(_: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(PATH_SEPARATOR.to_string()))
    }

    /// Print nothing
    ///
    /// $empty()
    pub(crate) fn print_empty(_: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(None)
    }

    /// Yield newline according to platform or user option
    ///
    /// # Usage
    ///
    /// $nl()
    pub(crate) fn newline(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;
        let count = if let Ok(c) = args.get_uint(0) { c } else { 1 };

        Ok(Some(p.state.newline.repeat(count)))
    }

    /// deny new line
    ///
    /// # Usage
    ///
    /// $dnl()
    pub(crate) fn deny_newline(_: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        p.state.deny_newline = true;
        Ok(None)
    }

    /// escape new line
    ///
    /// # Usage
    ///
    /// $enl()
    pub(crate) fn escape_newline(_: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        p.state.escape_newline = true;
        Ok(None)
    }

    /// Get name from given path
    ///
    /// # Usage
    ///
    /// $name(path/file.exe)
    pub(crate) fn get_name(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let path = args.get_path(0)?;

        if let Some(name) = path.file_name() {
            if let Some(value) = name.to_str() {
                return Ok(Some(value.to_owned()));
            }
        }
        Err(RadError::InvalidExecution(format!(
            "Invalid path : {}",
            path.display()
        )))
    }

    /// Check if file exists
    ///
    /// # Usage
    ///
    /// $exist(../canonic_path.txt)
    pub(crate) fn file_exists(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let boolean = args.get_path(0)?.exists();
        Ok(Some(boolean.to_string()))
    }

    /// Get absolute path from given path
    ///
    /// # Usage
    ///
    /// $abs(../canonic_path.txt)
    pub(crate) fn absolute_path(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let path = std::fs::canonicalize(p.get_current_dir()?.join(args.get_path(0)?))?
            .to_str()
            .unwrap()
            .to_owned();
        Ok(Some(path))
    }

    /// Get parent from given path
    ///
    /// # Usage
    ///
    /// $parent(path/file.exe)
    pub(crate) fn get_parent(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let path = args.get_path(0)?;

        if let Some(name) = path.parent() {
            if let Some(value) = name.to_str() {
                return Ok(Some(value.to_owned()));
            }
        }
        Err(RadError::InvalidExecution(format!(
            "Invalid path : {}",
            path.display()
        )))
    }

    /// Get pipe value
    ///
    /// # Usage
    ///
    /// $-()
    /// $-(p1)
    pub(crate) fn get_pipe(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;
        let pipe = if let Ok(name) = args.get_ctext(0) {
            if name.is_empty() {
                let out = processor.state.get_pipe("-", false);

                if out.is_none() {
                    processor.log_warning("Empty pipe", WarningType::Sanity)?;
                }

                out
            } else if let Some(pipe) = processor.state.get_pipe(args.get_text(0)?, false) {
                Some(pipe)
            } else {
                processor.log_warning(
                    &format!("Empty named pipe : \"{}\"", args.get_text(0)?),
                    WarningType::Sanity,
                )?;
                None
            }
        } else {
            // "-" Always exsit, thus safe to unwrap
            let out = processor.state.get_pipe("-", false).unwrap_or_default();
            if out.is_empty() {
                processor.log_warning("Empty pipe", WarningType::Sanity)?;
            }
            Some(out)
        };
        Ok(pipe)
    }

    /// Print left parenthesis
    ///
    /// # Usage
    ///
    /// $lp()
    pub(crate) fn left_parenthesis(_: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some("(".to_string()))
    }

    /// Print right parenthesis
    ///
    /// # Usage
    ///
    /// $rp()
    pub(crate) fn right_parenthesis(_: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(")".to_string()))
    }

    /// Rotate lines which is separated by pattern
    ///
    /// # Usage
    ///
    /// $rotatel(//,left,Content)
    pub(crate) fn rotatel(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        use std::fmt::Write;
        let args = ArgParser::new().args_with_len(input)?;

        let pattern = args.get_text(0)?;
        let orientation = args.get_enum(1, AlignType::from_str)?;
        let source = args.get_text(2)?;

        let mut result = String::new();
        let mut extracted = String::new();
        // Leading blank spaces from the line itself.
        let mut line_preceding_blank;
        for line in source.full_lines() {
            let line_ending = line.get_line_ending_always(&p.state.newline);
            if let Some((leading, following)) = line.split_once(pattern) {
                // Don't "rotate" for pattern starting line
                if leading.trim().is_empty() {
                    write!(result, "{line}")?;
                    continue;
                }

                extracted.clear();
                let leader_pattern = if orientation == AlignType::Center {
                    ""
                } else {
                    pattern
                };
                write!(extracted, "{}{}", leader_pattern, following.trim())?;
                line_preceding_blank = LSPA
                    .find(leading)
                    .map(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                match orientation {
                    AlignType::Left => {
                        write!(
                            result,
                            "{}{}{}{}{}",
                            line_preceding_blank, extracted, line_ending, leading, line_ending,
                        )?;
                    }
                    AlignType::Right => {
                        write!(
                            result,
                            "{}{}{}{}{}",
                            leading, line_ending, line_preceding_blank, extracted, line_ending
                        )?;
                    }
                    AlignType::Center => {
                        let mut leading = leading;
                        if !line_preceding_blank.is_empty() {
                            leading = leading.trim_start();
                        }
                        if extracted.ends_with("\r\n") {
                            extracted.pop();
                            extracted.pop();
                        } else if extracted.ends_with('\n') {
                            extracted.pop();
                        }

                        let extracted_spl = LSPA
                            .captures(&extracted)
                            .map(|s| s.get(0).unwrap().as_str())
                            .unwrap_or("");
                        let extracted_spf = FSPA
                            .captures(&extracted)
                            .map(|s| s.get(0).unwrap().as_str())
                            .unwrap_or("");

                        let extracted =
                            format!("{}{}{}", extracted_spf, extracted.trim(), extracted_spl);

                        let leading_spl = LSPA
                            .captures(leading)
                            .map(|s| s.get(0).unwrap().as_str())
                            .unwrap_or("");
                        let laeding_spf = FSPA
                            .captures(leading)
                            .map(|s| s.get(0).unwrap().as_str())
                            .unwrap_or("");

                        let leading = format!("{}{}{}", laeding_spf, leading.trim(), leading_spl);

                        write!(
                            result,
                            "{}{}{}{}{}",
                            line_preceding_blank, extracted, pattern, leading, line_ending
                        )?;
                    }
                }
            } else {
                write!(result, "{line}")?;
            }
        }

        Ok(Some(result))
    }

    /// Rotate inner
    ///
    /// # Note
    ///
    /// - Each pattern has to be unique
    ///
    /// # Usage
    ///
    /// $rotatei(start,end,sep,source)
    pub(crate) fn rotatei(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let start_pattern = args.get_text(0)?;
        let end_pattern = args.get_text(1)?;
        let separator = args.get_text(2)?;
        let source = args.get_text(3)?;

        let start_pattern_len = start_pattern.len();
        let end_pattern_len = end_pattern.len();

        // Leading blank spaces from the line itself.
        let first_split = source.find(start_pattern);
        if first_split.is_none() {
            return Err(RadError::InvalidArgument(format!(
                "Given source \"{}\" doesn't include start pattern",
                source,
            )));
        }

        let idx = first_split.unwrap();

        let (text_before_start, last) = (&source[0..idx], &source[idx + start_pattern_len..]);

        let last_split = if p.env.rotatei_order {
            last.find(end_pattern)
        } else {
            last.rfind(end_pattern)
        };
        if last_split.is_none() {
            return Err(RadError::InvalidArgument(format!(
                "Given source \"{}\" doesn't include end pattern",
                source,
            )));
        }

        let idx = last_split.unwrap();

        let (inner, leftover) = (&last[0..idx], &last[idx + end_pattern_len..]);

        // TODO TT? Backward inner split search? wow... really?
        let inner_split = inner.match_indices(separator).collect_vec();
        if inner_split.is_empty() {
            return Err(RadError::InvalidArgument(format!(
                "Given source \"{}\" doesn't include separator pattern",
                source,
            )));
        }
        let split_index = inner_split[inner_split.len() / 2].0;
        let (before, after) = (
            &inner[0..split_index],
            &inner[split_index + separator.len()..],
        );

        let result = format!(
            "{}{}{}{}{}{}{}",
            text_before_start, start_pattern, after, separator, before, end_pattern, leftover
        );

        Ok(Some(result))
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
    pub(crate) fn len(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(input.args.chars().count().to_string()))
    }

    /// Return a unicode length of the string
    ///
    /// # Usage
    ///
    /// $len(안녕하세요)
    /// $len(Hello)
    pub(crate) fn unicode_len(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(
            unicode_width::UnicodeWidthStr::width(input.args).to_string(),
        ))
    }

    /// Rename macro rule to other name
    ///
    /// # Usage
    ///
    /// $rename(name,target)
    pub(crate) fn rename_call(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let name = args.get_ctext(0)?;
        let new = args.get_ctext(1)?;

        if processor.contains_macro(name, MacroType::Any) {
            processor.rename_macro(name, new, MacroType::Any);
        } else {
            if processor.state.behaviour == ErrorBehaviour::Strict {
                return Err(RadError::UnsoundExecution(format!(
                    "Macro \"{}\" doesn't exist, therefore cannot be rename",
                    name,
                )));
            }

            processor.log_error(&format!(
                "Macro \"{}\" doesn't exist, therefore cannot rename",
                name,
            ))?;
        }

        Ok(None)
    }

    /// Pad texts with characters
    ///
    /// # Usage
    ///
    /// $pad(center,10,a,Content)
    pub(crate) fn pad_string(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let align_type = args.get_enum(0, AlignType::from_str)?;
        let width = args.get_uint(1)?;
        let text = args.get_text(3)?;
        let text_length = text.chars().count();
        if width < text_length {
            return Ok(Some(text.to_string()));
        }

        let filler: &str = args.get_ctext(2)?;
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
            }

            filler_char = ch.to_string();
        } else {
            return Err(RadError::InvalidArgument(
                "Filler should be a valid utf8 character".to_string(),
            ));
        }

        let space_count = width - text_length;

        let formatted = match align_type {
            AlignType::Left => format!("{0}{1}", text, &filler_char.repeat(space_count)),
            AlignType::Right => format!("{1}{0}", text, &filler_char.repeat(space_count)),
            AlignType::Center => {
                let right_sp = space_count / 2;
                let left_sp = space_count - right_sp;
                format!(
                    "{1}{0}{2}",
                    text,
                    &filler_char.repeat(left_sp),
                    &filler_char.repeat(right_sp)
                )
            }
        };

        Ok(Some(formatted))
    }

    /// Line up texts by separator but match to rearest
    ///
    /// # Usage
    ///
    /// $lineupr(%, contents to lineup)
    pub(crate) fn lineup_by_separator_match_rear(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        use std::fmt::Write;
        let args = ArgParser::new().args_with_len(input)?;

        let separator = args.get_text(0)?;
        let (c1, c2) = args.get_text(1)?.full_lines().tee();
        let mut max_length = 0usize;
        let mut result = String::new();

        let tab_width = p.env.rad_tab_width.unwrap_or(4);

        for line in c1 {
            let mut splitted = line.split(separator);
            let leading = splitted.next().unwrap();
            let width =
                UnicodeWidthStr::width(leading) + leading.matches('\t').count() * (tab_width - 1);
            if leading != line {
                max_length = max_length.max(width);
            }
        }
        for line in c2 {
            let splitted = line.split_once(separator);
            if splitted.is_some() {
                let (leading, following) = splitted.unwrap();
                let width = UnicodeWidthStr::width(leading)
                    + leading.matches('\t').count() * (tab_width - 1);

                // found matching line
                write!(
                    result,
                    "{}{}{}{}",
                    leading,
                    " ".repeat(max_length - width),
                    separator,
                    following,
                )?;
            } else {
                write!(result, "{}", line)?;
            }
        }
        Ok(Some(result))
    }

    /// Lineup texts by separator
    ///
    /// # Usage
    ///
    /// $lineup(%, contents to lineup)
    pub(crate) fn lineup_by_separator(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        use std::fmt::Write;
        let args = ArgParser::new().args_with_len(input)?;

        let separator = args.get_text(0)?;
        let (c1, c2) = args.get_text(1)?.full_lines().tee();
        let mut min_length = 0;
        let mut result = String::new();
        let mut put_after = "";

        let tab_width = p.env.rad_tab_width.unwrap_or(4);

        for line in c1 {
            let mut splitted = line.split(separator);
            let leading = splitted.next().unwrap();

            let width = if leading.trim().len() != leading.len() {
                UnicodeWidthStr::width(leading.trim())
                    + leading.trim().matches('\t').count() * (tab_width - 1)
            } else {
                UnicodeWidthStr::width(leading) + leading.matches('\t').count() * (tab_width - 1)
            };
            if leading != line {
                if put_after.is_empty() && width > min_length {
                    if leading.trim().len() != leading.len() {
                        put_after = " ";
                    } else {
                        put_after = "";
                    }
                }

                min_length = min_length.max(width);
            }
        }
        for line in c2 {
            let splitted = line.split_once(separator);
            if splitted.is_some() {
                let (leading, following) = splitted.unwrap();
                let width = UnicodeWidthStr::width(leading.trim())
                    + leading.trim().matches('\t').count() * (tab_width - 1);

                if width >= min_length {
                    // Bigger, trim it
                    // found matching line
                    write!(
                        result,
                        "{}{}{}{}",
                        leading.trim_end(),
                        put_after,
                        separator,
                        following
                    )?;
                } else {
                    // Smaller, increase it
                    // found matching line

                    write!(
                        result,
                        "{}{}{}{}{}",
                        leading.trim_end(),
                        " ".repeat(min_length - width),
                        put_after,
                        separator,
                        following,
                    )?;
                }
            } else {
                write!(result, "{}", line)?;
            }
        }
        Ok(Some(result))
    }

    /// Ailgn columns
    ///
    /// # Usage
    ///
    /// $alignc(c, contents to align)
    pub(crate) fn align_columns(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        use dcsv::VCont;
        let args = ArgParser::new().args_with_len(input)?;

        let align_type = args.get_enum(0, AlignType::from_str)?;

        let contents = args.get_ctext(1)?;
        let data = dcsv::Reader::new()
            .trim(true)
            .use_space_delimiter(true)
            .data_from_stream(contents.as_bytes())?;

        // TODO Check newline sanity
        Ok(Some(
            data.get_formatted_string(&p.state.newline, align_type.into()),
        ))
    }

    /// lineup texts multiple times
    ///
    /// # Usage
    ///
    /// $lineupm(rules, contents to lineup)
    pub(crate) fn lineup_by_rules(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let rules = args.get_ctext(0)?.chars().collect::<Vec<_>>();

        if rules.len() % 2 != 0 {
            return Err(RadError::InvalidArgument(format!(
                "lineupm needs specific syntax for rules but given \"{}\"",
                args.get_text(0)?
            )));
        }

        let mut contents = args
            .get_text(1)?
            .full_lines()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let tab_width = p.env.rad_tab_width.unwrap_or(4);

        let mut iter = rules.iter();
        while let (Some(count), Some(separator)) = (iter.next(), iter.next()) {
            let count = count.to_digit(10).ok_or_else(|| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a number",
                    count
                ))
            })?;
            lineup_step(&mut contents, *separator, count as usize, tab_width)?;
        }

        #[inline]
        fn lineup_step(
            contents: &mut [String],
            separator: char,
            count: usize,
            tab_width: usize,
        ) -> RadResult<()> {
            let mut max_length = 0usize;

            for line in contents.iter() {
                let splitted_index = line.chars().positions(|s| s == separator).nth(count - 1);
                if splitted_index.is_none() {
                    continue;
                }
                let leading = &line[0..splitted_index.unwrap()];
                let width = UnicodeWidthStr::width(leading)
                    + leading.matches('\t').count() * (tab_width - 1);
                if leading != line {
                    max_length = max_length.max(width);
                }
            }

            for line in contents.iter_mut() {
                let splitted_index = line.chars().positions(|s| s == separator).nth(count - 1);
                // found matching line
                if splitted_index.is_some() {
                    let (leading, following) = line.split_at(splitted_index.unwrap());
                    let width = UnicodeWidthStr::width(leading)
                        + leading.matches('\t').count() * (tab_width - 1);

                    *line = format!("{}{}{}", leading, " ".repeat(max_length - width), following,);
                }
            }
            Ok(())
        }

        let result = contents.join("");

        Ok(Some(result))
    }

    /// Align lines with given options
    ///
    /// # Usage
    ///
    /// TYPE : hierarchy, right, left, pr, pl
    /// $align(TYPE, contents to align)
    pub(crate) fn align(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let line_up_type = LineUpType::from_str(args.get_text(0)?)?;
        let c1 = args.get_text(1)?.full_lines();
        let mut standard_width = usize::MAX;
        let mut curr = 0;
        let mut result = String::new();

        #[inline]
        fn set_minimal_standard_width<'a, T>(
            iter: T,
            standard_indent_width: &mut usize,
        ) -> Vec<(usize, &'a str, usize)>
        where
            T: Iterator<Item = &'a str>,
        {
            iter.enumerate()
                .map(|(idx, line)| {
                    // TODO Fix this
                    let prefix = if let Some(leading) = LSPA.captures(line) {
                        let mut len = leading.get(0).unwrap().len();
                        if len != 0 && *standard_indent_width != 0 {
                            *standard_indent_width = *standard_indent_width.min(&mut len);
                        }
                        len
                    } else {
                        if !line.trim_start().is_empty() {
                            // Line starts from 0 index
                            *standard_indent_width = 0;
                        }
                        0
                    };
                    (idx, line, prefix)
                })
                .collect::<Vec<_>>()
        }

        #[inline]
        fn set_maximal_standard_width<'a, T>(
            iter: T,
            standard_indent_width: &mut usize,
        ) -> Vec<(usize, &'a str, usize)>
        where
            T: Iterator<Item = &'a str>,
        {
            *standard_indent_width = 0; // Start from 0 to set maximal value
            iter.enumerate()
                .map(|(idx, line)| {
                    let mut len = UnicodeWidthStr::width(line);
                    *standard_indent_width = *standard_indent_width.max(&mut len);
                    (idx, line, len)
                })
                .collect::<Vec<_>>()
        }

        #[inline]
        fn set_maximal_whitspace<'a, T>(
            iter: T,
            standard_indent_width: &mut usize,
        ) -> Vec<(usize, &'a str, usize)>
        where
            T: Iterator<Item = &'a str>,
        {
            *standard_indent_width = 0; // Start from 0 to set maximal value
            iter.enumerate()
                .map(|(idx, line)| {
                    let mut new_start = line.len() - line.trim_start().len();
                    *standard_indent_width = *standard_indent_width.max(&mut new_start);
                    (idx, line, 0)
                })
                .collect::<Vec<_>>()
        }

        // Set standard_width
        let c2 = match line_up_type {
            LineUpType::ParralelRight => {
                let ret = set_maximal_whitspace(c1, &mut standard_width);
                if standard_width == 0 {
                    return Ok(Some(args.get_text(1)?.to_string()));
                }
                ret
            }
            LineUpType::Right => {
                let ret = set_maximal_standard_width(c1, &mut standard_width);
                if standard_width == 0 {
                    return Ok(Some(args.get_text(1)?.to_string()));
                }
                ret
            }
            LineUpType::ParralelLeft | LineUpType::Hierarchy => {
                let ret = set_minimal_standard_width(c1, &mut standard_width);
                if standard_width == usize::MAX {
                    return Ok(Some(args.get_text(1)?.to_string()));
                }
                ret
            }
            _ => c1.enumerate().map(|(idx, line)| (idx, line, 0)).collect(),
        };

        let c3 = if let LineUpType::Hierarchy = line_up_type {
            if standard_width == 0 {
                standard_width = p.env.rad_tab_width.unwrap_or(4);
            }

            c2.into_iter()
                .sorted_by(|(_, _, a), (_, _, b)| a.cmp(b))
                .map(|(idx, line, prefix)| {
                    if prefix == 0 || line.trim_start().is_empty() {
                        return (idx, line, 0);
                    }
                    curr += 1;
                    (idx, line, curr)
                })
                .sorted_by(|(a, _, _), (b, _, _)| a.cmp(b))
                .collect::<Vec<_>>()
        } else {
            c2
        };

        // Reform lines
        // Target value is
        // 1. Offset for hierarchy type
        // 2. Prefix spaces for Left, ParralelLeft type
        // 3. Current length for right, ParralelRight type
        for (_, line, target_value) in c3 {
            match line_up_type {
                LineUpType::Hierarchy => {
                    let line_start = target_value * standard_width;
                    let line = " ".repeat(line_start) + line.trim_start();
                    result.push_str(&line);
                }
                LineUpType::Left => {
                    result.push_str(line.trim_start());
                }
                LineUpType::ParralelLeft => {
                    result.push_str(&" ".repeat(standard_width));
                    result.push_str(line.trim_start());
                }
                LineUpType::ParralelRight => {
                    result.push_str(&" ".repeat(standard_width));
                    result.push_str(line.trim_start());
                }
                LineUpType::Right => {
                    result.push_str(&" ".repeat(standard_width - target_value));
                    result.push_str(line);
                }
            };
        }
        Ok(Some(result))
    }

    /// Translate given char aray into corresponding char array
    ///
    /// # Usage
    ///
    /// $tr(abc,ABC,Source)
    /// TODO Check
    pub(crate) fn translate(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let source = args.get_text(2)?;
        let mut replaced = String::with_capacity(source.len());
        let target = args.get_text(0)?.chars().collect::<Vec<_>>();
        let destination = args.get_text(1)?.chars().collect::<Vec<_>>();

        if target.len() != destination.len() {
            return Err(RadError::InvalidArgument(format!("Tr's replacment should have same length of texts while given \"{:?}\" and \"{:?}\"", target, destination)));
        }

        let new_hash = rustc_hash::FxHashMap::from_iter(target.iter().zip(destination.iter()));

        for sh in source.chars() {
            if let Some(&&ch) = new_hash.get(&sh) {
                replaced.push(ch);
            } else {
                replaced.push(sh);
            }
        }

        Ok(Some(replaced))
    }

    /// Get a utf8 code based substring(indexed) from given source
    ///
    /// # Usage
    ///
    /// $rangeu(0,5,안녕하세요ㅎ)
    pub(crate) fn substring_utf8(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let source = args.get_text(2)?;

        let mut min: Option<isize> = None;
        let mut max: Option<isize> = None;

        let start = args.get_ctext(0)?;
        let end = args.get_ctext(1)?;

        if let Ok(num) = start.parse::<isize>() {
            check_neg(num, p.env.no_negative_index)?;
            min.replace(num);
        } else if start != "_" && !start.is_empty() {
            return Err(RadError::InvalidArgument(format!(
                "Rangeu's min value should be non zero positive integer but given \"{}\"",
                start
            )));
        }

        if let Ok(num) = end.parse::<isize>() {
            check_neg(num, p.env.no_negative_index)?;
            max.replace(num);
        } else if end != "_" && !end.is_empty() {
            return Err(RadError::InvalidArgument(format!(
                "Rangeu's max value should be singed integer or empty value but given \"{}\"",
                end
            )));
        }

        Ok(Some(Utils::utf_slice(source, min, max)?))
    }

    /// Get a substring(indexed) from given source
    ///
    /// # Usage
    ///
    /// $range(0,5,GivenString)
    pub(crate) fn substring(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let source = args.get_text(2)?;

        let mut min: Option<isize> = None;
        let mut max: Option<isize> = None;

        let start = args.get_ctext(0)?;
        let end = args.get_ctext(1)?;

        if let Ok(num) = start.parse::<isize>() {
            check_neg(num, p.env.no_negative_index)?;
            min.replace(num);
        } else if start != "_" && !start.is_empty() {
            return Err(RadError::InvalidArgument(format!(
                "Range's min value should be non zero positive integer but given \"{}\"",
                start
            )));
        }

        if let Ok(num) = end.parse::<isize>() {
            check_neg(num, p.env.no_negative_index)?;
            max.replace(num);
        } else if end != "_" && !end.is_empty() {
            return Err(RadError::InvalidArgument(format!(
                "Range's max value should be singed integer or empty value but given \"{}\"",
                end
            )));
        }

        Ok(Some(Utils::ascii_slice(source, min, max)?.to_string()))
    }

    /// Get a sublines(indexed) from given lines
    ///
    /// # Usage
    ///
    /// $rangel(0,5,GivenLines)
    pub(crate) fn range_lines(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let source = args.get_text(2)?;

        let mut min: Option<isize> = None;
        let mut max: Option<isize> = None;

        let start = args.get_ctext(0)?;
        let end = args.get_ctext(1)?;

        if let Ok(num) = start.parse::<isize>() {
            check_neg(num, p.env.no_negative_index)?;
            min.replace(num);
        } else if start != "_" && !start.is_empty() {
            return Err(RadError::InvalidArgument(format!(
                "rangel's min value should be non zero positive integer but given \"{}\"",
                start
            )));
        }

        if let Ok(num) = end.parse::<isize>() {
            check_neg(num, p.env.no_negative_index)?;
            max.replace(num);
        } else if end != "_" && !end.is_empty() {
            return Err(RadError::InvalidArgument(format!(
                "rangel's max value should be singed integer or empty value but given \"{}\"",
                end
            )));
        }

        Ok(Some(Utils::sub_lines(source, min, max)?.to_string()))
    }

    /// Get a sub_piecess(indexed) from given content
    ///
    /// # Usage
    ///
    /// $rangeby(0,5,Content)
    pub(crate) fn range_pieces(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let delimiter = args.get_text(0)?;
        let source = args.get_text(3)?;

        let mut min: Option<isize> = None;
        let mut max: Option<isize> = None;

        let start = args.get_ctext(1)?;
        let end = args.get_ctext(2)?;

        if let Ok(num) = start.parse::<isize>() {
            check_neg(num, p.env.no_negative_index)?;
            min.replace(num);
        } else if start != "_" && !start.is_empty() {
            return Err(RadError::InvalidArgument(format!(
                "rangeby's min value should be non zero positive integer but given \"{}\"",
                start
            )));
        }

        if let Ok(num) = end.parse::<isize>() {
            check_neg(num, p.env.no_negative_index)?;
            max.replace(num);
        } else if end != "_" && !end.is_empty() {
            return Err(RadError::InvalidArgument(format!(
                "rangeby's max value should be singed integer or empty value but given \"{}\"",
                end
            )));
        }

        Ok(Some(
            Utils::sub_pieces(source, delimiter, min, max)?.to_string(),
        ))
    }

    /// Get a substring(indexed) until a pattern
    ///
    /// # Usage
    ///
    /// $until(pattern,Content)
    pub(crate) fn get_slice_until(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let pattern = args.get_text(0)?;

        if pattern.is_empty() {
            return Err(RadError::InvalidArgument(
                "Empty value is not allowed in until".to_owned(),
            ));
        }
        let source = args.get_text(1)?;

        let index = source.find(pattern);
        if let Some(index) = index {
            Ok(Some(source[0..index].to_owned()))
        } else {
            Ok(Some(source.to_owned()))
        }
    }

    /// Get a substring(indexed) after a pattern
    ///
    /// # Usage
    ///
    /// $after(pattern,Content)
    pub(crate) fn get_slice_after(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let pattern = args.get_text(0)?;
        let offset = pattern.len();

        if pattern.is_empty() {
            return Err(RadError::InvalidArgument(
                "Empty value is not allowed in after".to_owned(),
            ));
        }
        let source = args.get_text(1)?;

        let index = source.find(pattern);
        if let Some(index) = index {
            Ok(Some(source[index + offset..].to_owned()))
        } else {
            Ok(Some(source.to_owned()))
        }
    }

    /// Save content to temporary file
    ///
    /// # Usage
    ///
    /// $tempout(Content)
    pub(crate) fn temp_out(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let content = args.get_text(0)?;
        if let Some(file) = p.get_temp_file() {
            file.write_all(content.as_bytes())?;
        } else {
            return Err(RadError::InvalidExecution(
                "You cannot use temp related macros in environment where fin/fout is not supported"
                    .to_owned(),
            ));
        }

        Ok(None)
    }

    /// Save content to a file
    ///
    /// # Usage
    ///
    /// $fileout(file_name,true,Content)
    pub(crate) fn file_out(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let file_name = args.get_path(0)?;
        let truncate = args.get_bool(1)?;
        let content = args.get_text(2)?;
        if truncate {
            // This doesn't use canonicalize, because fileout can write file to non-existent
            // file. Thus canonicalize can possibly yield error
            let path = std::env::current_dir()?.join(file_name);
            if path.exists() && !path.is_file() {
                return Err(RadError::InvalidExecution(format!(
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
                    return Err(RadError::InvalidExecution(format!("Failed to write \"{}\". Fileout without truncate option needs exsiting non-directory file",path.display())));
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
    }

    /// Get head of given text
    ///
    /// # Usage
    ///
    /// $head(2,Text To extract)
    pub(crate) fn head(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let count = args.get_uint(0)?;

        if count == 0 {
            return Ok(Some(String::new()));
        }
        let index = count.saturating_sub(1) as isize;

        let res = Utils::utf_slice(args.get_text(1)?, Some(0), Some(index))?;

        Ok(Some(res))
    }

    /// Get head of given text but for lines
    ///
    /// # Usage
    ///
    /// $headl(2,Text To extract)
    pub(crate) fn head_line(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let count = args.get_uint(0)?;

        if count == 0 {
            return Ok(Some(String::new()));
        }
        let index = count as isize - 1;

        Ok(Some(
            Utils::sub_lines(args.get_text(1)?, Some(0), Some(index))?.to_string(),
        ))
    }

    /// Get tail of given text
    ///
    /// # Usage
    ///
    /// $tail(2,Text To extract)
    pub(crate) fn tail(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let count_src = args.get_uint(0)?;
        if count_src == 0 {
            return Ok(Some(String::new()));
        }
        let min_count = -(count_src as isize - 1);

        let res = Utils::utf_slice(args.get_text(1)?, Some(min_count), None)?;

        Ok(Some(res))
    }

    /// Surround a text with given pair
    ///
    /// # Usage
    ///
    /// $surr(<p>,</p>,content)
    pub(crate) fn surround_with_pair(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let start = args.get_text(0)?;
        let end = args.get_text(1)?;
        let content = args.get_text(2)?;
        Ok(Some(format!("{}{}{}", start, content, end)))
    }

    /// Squeeze a line
    ///
    /// # Usage
    ///
    /// $squz(a b c d e)
    pub(crate) fn squeeze_line(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let mut content = args.get_text(0)?.to_string();
        let trailer = if content.ends_with('\n') {
            "\n"
        } else if content.ends_with("\r\n") {
            "\r\n"
        } else {
            ""
        };
        content.retain(|s| !s.is_whitespace());
        content.push_str(trailer);
        Ok(Some(content))
    }

    /// Get tail of given text but for lines
    ///
    /// # Usage
    ///
    /// $taill(2,Text To extract)
    pub(crate) fn tail_line(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let count = args.get_uint(0)?;

        if count == 0 {
            return Ok(Some(String::new()));
        }

        let min = if count == 0 {
            None
        } else {
            Some(-(count as isize - 1))
        };

        Ok(Some(
            Utils::sub_lines(args.get_text(1)?, min, None)?.to_string(),
        ))
    }

    /// Sort array
    ///
    /// # Usage
    ///
    /// $sort(asec,1,2,3,4,5)
    pub(crate) fn sort_array(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new()
            .split(SplitVariant::Always)
            .args_with_len(input)?;

        let order_type = args.get_enum(0, OrderType::from_str)?;

        let mut content = (1..args.len())
            .map(|i| args.get_text(i))
            .rev()
            .collect::<RadResult<Vec<_>>>()?;

        match order_type {
            OrderType::Ascending => content.sort_unstable(),
            OrderType::Descending => {
                content.sort_unstable();
                content.reverse()
            }
        }

        Ok(Some(content.join(",")))
    }

    /// Sort lines
    ///
    /// # Usage
    ///
    /// $sortl(asec,Content)
    pub(crate) fn sort_lines(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let order_type = args.get_enum(0, OrderType::from_str)?;
        let mut content = args.get_text(1)?.to_string();
        let mut line_ending = content.get_line_ending();
        let mut pop_last = false;
        if line_ending.is_empty() {
            content.push_str(&p.state.newline);
            line_ending = &p.state.newline;
            pop_last = true;
        }
        let mut content = content.full_lines().collect::<Vec<&str>>();
        if pop_last {
            content.pop();
        }

        match order_type {
            OrderType::Ascending => content.sort_unstable(),
            OrderType::Descending => {
                content.sort_unstable();
                content.reverse()
            }
        }

        let mut ret = content
            .iter()
            .map(|&s| {
                if s.get_line_ending().is_empty() {
                    Cow::Owned(s.to_string() + line_ending)
                } else {
                    Cow::Borrowed(s)
                }
            })
            .join("");

        // Retain original line ending
        for _ in 0..line_ending.len() {
            ret.pop();
        }

        Ok(Some(ret))
    }

    /// Sort lists ( chunks )
    ///
    /// # Usage
    ///
    /// $sortc(asec, ... chunk ... )
    pub(crate) fn sort_chunk(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let order_type = args.get_enum(0, OrderType::from_str)?;
        let mut content = args.get_text(1)?.to_string();
        let mut line_ending = content.get_line_ending();
        let mut skip_last = false;
        if line_ending.is_empty() {
            content.push_str(&p.state.newline);
            line_ending = &p.state.newline;
            skip_last = true;
        }
        // --- Chunk creation
        let mut clogged_chunk_list = vec![];
        let mut container = String::new();
        let mut iter = content.full_lines().peekable();
        while let Some(line) = iter.next() {
            // Skip last when newline was manually appended.
            if iter.peek().is_none() && skip_last {
                break;
            }
            // Has blank leading characters + has parent
            // -> Set as children
            if LSPA.captures(line).is_some() && !container.is_empty() {
                container.push_str(line);
            } else {
                // If not, it is parent object

                // End previous container
                if !container.is_empty() {
                    clogged_chunk_list.push(container);
                }
                // Start a new container
                container = line.to_string();
            }
        }
        if !container.is_empty() {
            clogged_chunk_list.push(container);
        }
        // ---
        match order_type {
            OrderType::Ascending => clogged_chunk_list.sort_unstable(),
            OrderType::Descending => {
                clogged_chunk_list.sort_unstable();
                clogged_chunk_list.reverse()
            }
        }

        let mut ret = clogged_chunk_list
            .iter_mut()
            .map(|s| {
                if s.get_line_ending().is_empty() {
                    s.push_str(line_ending);
                }
                s
            })
            .join("");

        // Retain original line ending
        for _ in 0..line_ending.len() {
            ret.pop();
        }

        Ok(Some(ret))
    }

    // [1 2 3]
    //  0 1 2
    //  -3-2-1

    /// Index array
    ///
    /// # Usage
    ///
    /// $index(1,1,2,3,4,5)
    pub(crate) fn index_array(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new()
            .split(SplitVariant::Always)
            .args_with_len(input)?;

        // TODO TT
        // POssibly don't allocate as vector if possible to improve performance
        let index = args.get_int(0)?;
        let values = (1..args.len())
            .map(|idx| args.get_text(idx))
            .collect::<RadResult<Vec<_>>>()?;

        let len = values.len();

        if index >= len as isize || index < -(len as isize) {
            return Err(RadError::InvalidArgument(format!(
                "Index out of range. Given index is \"{}\" but array length is \"{}\"",
                index, len
            )));
        }

        let final_index = if index < 0 {
            (len as isize + index) as usize
        } else {
            index.max(0) as usize
        };

        if len <= final_index {
            return Err(RadError::InvalidArgument(format!(
                "Index out of range. Given index is \"{}\" but array length is \"{}\"",
                index, len
            )));
        }

        Ok(Some(values[final_index].to_string()))
    }

    /// Index lines
    ///
    /// # Usage
    ///
    /// $indexl(1,1$nl()2$nl())
    pub(crate) fn index_lines(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let content = &mut args.get_text(1)?.full_lines();
        let index = args.get_int(0)?;

        let len = args.get_text(1)?.lines().count();

        if index >= len as isize || index < -(len as isize) {
            return Err(RadError::InvalidArgument(format!(
                "indexl out of range. Given index is \"{}\" but lines length is \"{}\"",
                index, len
            )));
        }

        let final_index = if index < 0 {
            (len as isize + index) as usize
        } else {
            index.max(0) as usize
        };

        if len <= final_index {
            return Err(RadError::InvalidArgument(format!(
                "Index out of range. Given index is \"{}\" but lines length is \"{}\"",
                index, len
            )));
        }

        // It is safe to unwrap because bound check was already done
        Ok(Some(content.nth(final_index).unwrap().to_owned()))
    }

    /// Strip content
    ///
    /// # Usage
    ///
    /// $strip()
    pub(crate) fn strip(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let count = args.get_uint(0)?;
        let content = args.get_text(1)?;

        if count == 0 {
            return Ok(Some(args.get_text(1)?.to_string()));
        }

        let char_count = content.chars().count();

        if count * 2 > char_count {
            return Err(RadError::InvalidArgument(
                "Cannot strip because given content's length is shorter".to_owned(),
            ));
        }

        // abcd
        // 2
        // 22

        Ok(Some(content[count..char_count - count].to_string()))
    }

    /// Strip front
    ///
    /// # Usage
    ///
    /// $stripf()
    pub(crate) fn stripf(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let count = args.get_uint(0)?;
        let content = args.get_text(1)?;

        if count == 0 {
            return Ok(Some(args.get_text(1)?.to_string()));
        }

        let char_count = content.chars().count();

        if count > char_count {
            return Err(RadError::InvalidArgument(
                "Cannot stripf because given content's length is shorter".to_owned(),
            ));
        }

        Ok(Some(content[count..].to_string()))
    }

    /// Strip front lines
    ///
    /// # Usage
    ///
    /// $stripfl()
    pub(crate) fn stripf_line(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let count = args.get_uint(0)?;
        let content = args.get_text(1)?;

        if count == 0 {
            return Ok(Some(args.get_text(1)?.to_string()));
        }

        let lines = content.full_lines().collect::<Vec<_>>();
        let line_count = lines.len();

        if count > line_count {
            return Err(RadError::InvalidArgument(
                "Cannot stripfl because given content's length is shorter".to_owned(),
            ));
        }

        let result = lines[count..].iter().fold(String::new(), |mut acc, a| {
            acc.push_str(a);
            acc
        });

        Ok(Some(result))
    }

    /// Strip rear lines
    ///
    /// # Usage
    ///
    /// $striprl()
    pub(crate) fn stripr_line(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let count = args.get_uint(0)?;
        let content = args.get_text(1)?;

        if count == 0 {
            return Ok(Some(args.get_text(1)?.to_string()));
        }

        let lines = content.full_lines().collect::<Vec<_>>();
        let line_count = lines.len();

        if count > line_count {
            return Err(RadError::InvalidArgument(
                "Cannot striprl because given content's length is shorter".to_owned(),
            ));
        }

        let result = lines[0..line_count - count]
            .iter()
            .fold(String::new(), |mut acc, a| {
                acc.push_str(a);
                acc
            });

        Ok(Some(result))
    }

    /// Strip rear
    ///
    /// # Usage
    ///
    /// $stripr()
    pub(crate) fn stripr(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let count = args.get_uint(0)?;
        let content = args.get_text(1)?;

        if count == 0 {
            return Ok(Some(args.get_text(1)?.to_string()));
        }

        let char_count = content.chars().count();

        if count > char_count {
            return Err(RadError::InvalidArgument(
                "Cannot stripr because given content's length is shorter".to_owned(),
            ));
        }

        // abcd
        // 2
        // 22

        Ok(Some(content[..char_count - count].to_string()))
    }

    /// Strip rear with pattern
    ///
    /// # Usage
    ///
    /// $striper()
    pub(crate) fn strip_expression_from_rear(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let expr = args.get_text(0)?;
        let content = args.get_text(1)?;
        let nl = p.state.newline.clone();
        let reg = p.try_get_or_create_regex(expr)?;

        let mut acc = String::new();
        // TODO CHeck lines method sanity
        for line in content.lines() {
            let last_item = reg.captures_iter(line).last();
            // Last item
            match last_item {
                Some(capped) => {
                    acc.push_str(&line[0..capped.get(0).unwrap().start()]);
                }
                None => {
                    acc.push_str(line);
                }
            }
            acc.push_str(&nl);
        }

        p.insert_regex(expr, reg.to_option())?;
        Ok(Some(acc))
    }

    /// Separate content
    ///
    /// # Usage
    ///
    /// $sep(1$nl()2$nl())
    pub(crate) fn separate(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let content = args.get_text(0)?;
        let mut formatted = String::with_capacity(content.len());
        let mut iter = content.full_lines().peekable();
        while let Some(line) = iter.next() {
            formatted.push_str(line);
            if !line.trim().is_empty() && !iter.peek().unwrap_or(&"0").trim().is_empty() {
                formatted.push_str(line.get_line_ending());
            }
        }
        Ok(Some(formatted))
    }

    /// Get range from array
    ///
    /// # Usage
    ///
    /// $rangea(1,2,1,2,3,4,5)
    pub(crate) fn range_array(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new()
            .split(SplitVariant::Always)
            .args_with_len(input)?;

        // THis strange type is a legacy of '_' syntax support
        // This is stayed as it is because it might support '_' later on.
        let mut min: Option<usize> = None;
        let mut max: Option<usize> = None;

        let start_src = args.get_text(0)?.trim();
        let end_src = args.get_text(1)?.trim();

        if let Ok(num) = start_src.parse::<usize>() {
            min.replace(num);
        } else if !start_src.is_empty() {
            return Err(RadError::InvalidArgument(format!("Rangea's min value should be non zero positive integer or empty value but given \"{}\"", start_src)));
        }

        if let Ok(num) = end_src.parse::<usize>() {
            max.replace(num);
        } else if !end_src.is_empty() {
            return Err(RadError::InvalidArgument(format!("Rangea's max value should be non zero positive integer or empty value but given \"{}\"", end_src)));
        }

        let content = (2..args.len())
            .map(|idx| {
                let value = args.get_text(idx)?;
                Ok(value)
            })
            .collect::<RadResult<Vec<_>>>()?;

        let min = min.unwrap();
        let max = max.unwrap();

        if content.len() <= min || content.len() <= max {
            return Err(RadError::InvalidArgument(format!(
                "Index out of range. Given indices are \"{}\" & \"{}\" but array length is \"{}\"",
                min,
                max,
                content.len()
            )));
        }

        Ok(Some(content[min..=max].join(",")))
    }

    /// Fold array
    ///
    /// # Usage
    ///
    /// $fold(1,2,3,4,5)
    pub(crate) fn fold(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new()
            .split(SplitVariant::Always)
            .args_with_len(input)?;

        let mut content = (0..args.len())
            .map(|idx| {
                let value = args.get_text(idx)?;
                Ok(value)
            })
            .collect::<RadResult<Vec<_>>>()?;

        let res = merge_container(&mut content, p.env, None);

        Ok(Some(res))
    }

    /// Fold lines
    ///
    /// This folds empty lines
    ///
    /// # Usage
    ///
    /// $foldl(1
    /// 1
    /// 2
    /// 3
    /// 4
    /// 5)
    pub(crate) fn fold_line(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let mut content = args.get_text(0)?.lines().collect::<Vec<_>>();
        let res = merge_container(&mut content, p.env, Some(&p.state.newline));

        Ok(Some(res))
    }

    /// Fold lines by regular expressions
    ///
    /// # Usage
    ///
    /// $folde(expr,1
    /// 2)
    pub(crate) fn fold_regular_expr(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let mut container = Vec::new();
        let mut folded = String::new();

        let nl = p.state.newline.clone();
        let values = (0..args.len())
            .map(|idx| {
                let value = args.get_text(idx)?;
                Ok(value)
            })
            .collect::<RadResult<Vec<_>>>()?;

        let env = p.env;
        let reg_start = p.try_get_or_create_regex(values[0])?;
        let reg_end = p.try_get_or_create_regex(values[1])?;

        for line in args.get_text(2)?.full_lines() {
            // Start new container
            if reg_start.find(line).is_some() && reg_end.find(line).is_none() {
                folded.push_str(&merge_container(&mut container, env, Some(&nl)));
                container.push(line);
            } else if reg_start.find(line).is_none() && reg_end.find(line).is_some() {
                container.push(line);
                folded.push_str(&merge_container(&mut container, env, Some(&nl)));
                // End regex doesn't add newline
                container.clear();
            } else if !container.is_empty() {
                container.push(line);
            } else {
                folded.push_str(line);
            }
        }
        folded.push_str(&merge_container(
            &mut container,
            p.env,
            Some(&p.state.newline),
        ));

        let reg_start = reg_start.to_option();
        let reg_end = reg_end.to_option();

        p.insert_regex(values[0], reg_start)?;
        p.insert_regex(values[1], reg_end)?;

        Ok(Some(folded))
    }

    /// Get os type
    ///
    /// # Usage
    ///
    /// $ostype()
    pub(crate) fn get_os_type(_: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        if cfg!(target_os = "windows") {
            Ok(Some("windows".to_owned()))
        } else {
            Ok(Some("unix".to_owned()))
        }
    }

    // TODO Move this to deterred macro
    /// Register expression
    ///
    /// # Usage
    ///
    /// $addexpr(name,EXPR)
    pub(crate) fn register_expression(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let name = args.get_text(0)?;
        let expr = args.get_text(1)?;

        p.state.regex_cache.register(name, expr)?;
        Ok(None)
    }

    /// Grep expressions
    ///
    /// # Usage
    ///
    /// $grep(expr,Content)
    pub(crate) fn grep_expr(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let expr = args.get_text(0)?;
        // TODO
        // Env : to separate it by comma
        let nl = p.state.newline.clone();
        let reg = p.try_get_or_create_regex(expr)?;
        let acc = reg
            .captures_iter(args.get_text(1)?)
            .fold(String::new(), |mut acc, x| {
                // TODO Make an env to make unmatched group an error
                let mut cap = x.iter().peekable();
                let total = cap.next(); // First capture group
                let captured = if cap.peek().is_some() {
                    cap.filter(|s| s.is_some())
                        .map(|s| s.unwrap().as_str())
                        .join(" ")
                } else {
                    total
                        .filter(|s| s.is_some())
                        .map(|s| s.unwrap().as_str())
                        .unwrap()
                        .to_string()
                };
                acc.push_str(&captured);
                acc.push_str(&nl);
                acc
            });
        p.insert_regex(expr, reg.to_option())?;
        Ok(acc.strip_suffix(&nl).map(|s| s.to_owned()))
    }

    /// Grep items from array
    ///
    /// # Usage
    ///
    /// $grepa(expr,Array)
    pub(crate) fn grep_array(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new()
            .split(SplitVariant::Always)
            .args_with_len(input)?;

        let expr = args.get_text(0)?;
        let reg = p.try_get_or_create_regex(expr)?;

        // It is mostly safe to unwrap because every input is text by default
        let mut grepped = (1..args.len())
            .map(|idx| args.get_text(idx).unwrap())
            .filter(|l| reg.is_match(l))
            .fold(String::new(), |mut acc, x| {
                acc.push_str(x);
                acc.push(',');
                acc
            });
        grepped.pop();
        p.insert_regex(expr, reg.to_option())?;
        Ok(Some(grepped))
    }

    /// Grepl
    ///
    /// # Usage
    ///
    /// $grepl(expr,Lines)
    pub(crate) fn grep_lines(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let expr = args.get_text(0)?;
        let reg = p.try_get_or_create_regex(expr)?;
        let content = args.get_text(1)?.full_lines();
        let grepped = content
            .filter(|l| reg.is_match(l))
            .fold(String::new(), |mut acc, l| {
                acc.push_str(l);
                acc
            });
        p.insert_regex(expr, reg.to_option())?;

        Ok(Some(grepped))
    }

    /// Grepf
    ///
    /// # Usage
    ///
    /// $grepf(EXPR,CONTENT)
    pub(crate) fn grep_file(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;
        let path = args.get_path(1)?;

        if path.exists() {
            let canonic = path.canonicalize()?;
            Utils::check_file_sanity(p, &canonic)?;
        } else {
            return Err(RadError::InvalidExecution(format!(
                "grepf requires a real file to read from but \"{}\" doesn't exist",
                path.display()
            )));
        };

        let expr = args.get_text(0)?;
        let reg = p.try_get_or_create_regex(expr)?;
        let file_stream = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file_stream);

        let mut vec = vec![];
        for line in Utils::full_lines(reader) {
            let line = line?;
            if reg.is_match(&line) {
                vec.push(line);
            }
        }

        p.insert_regex(expr, reg.to_option())?;
        Ok(Some(vec.join("")))
    }

    /// Condense
    ///
    /// # Usage
    ///
    /// $cond(a       b         c)
    pub(crate) fn condense(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        // TODO CHECK TO_string
        let content = args.get_text(0)?;
        Ok(Some(content.split_whitespace().join(" ")))
    }

    /// Condense
    ///
    /// # Usage
    ///
    /// $condl(a       b         c)
    pub(crate) fn condense_by_lines(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        use std::fmt::Write;
        let args = ArgParser::new().args_with_len(input)?;

        let content = args.get_text(0)?;
        let mut acc = String::new();
        let itr = content.full_lines().peekable();
        for line in itr {
            let line_ending = line.get_line_ending();
            write!(&mut acc, "{}", line.split_whitespace().join(" "))?;
            write!(&mut acc, "{}", line_ending)?;
        }
        Ok(Some(acc))
    }

    /// Count
    ///
    /// # Usage
    ///
    /// $count(1,2,3,4,5)
    pub(crate) fn count(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().allow_empty_input().args_with_len(input)?;
        Ok(Some(args.len().to_string()))
    }

    /// Count words
    ///
    /// # Usage
    ///
    /// $countw(1 2 3 4 5)
    pub(crate) fn count_word(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let array_count = args.get_text(0)?.split_whitespace().count();
        Ok(Some(array_count.to_string()))
    }

    /// Count lines
    ///
    /// # Usage
    ///
    /// $countl(CONTENT goes here)
    pub(crate) fn count_lines(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        if args.get_text(0)?.is_empty() {
            return Ok(Some("0".to_string()));
        }
        let line_count = Utils::count_sentences(args.get_text(0)?);
        Ok(Some(line_count.to_string()))
    }

    /// Relay all text into given target
    ///
    /// Every text including non macro calls are all sent to relay target
    ///
    /// # Usage
    ///
    /// $relay(type,argument)
    pub(crate) fn relay(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let target = args.get_ctext(0)?;

        p.log_warning(
            &format!("Relaying text content to \"{}\"", target),
            WarningType::Security,
        )?;

        if target.is_empty() {
            return Err(RadError::InvalidArgument(
                "relay requires second argument as macro name for macro relaying".to_owned(),
            ));
        }
        if !p.contains_macro(target, MacroType::Runtime) {
            let sim = p.get_similar_macro(target, true); // For relay only runtime
            return Err(RadError::NoSuchMacroName(target.to_string(), sim));
        }
        let relay_target = RelayTarget::Macro(target.to_string());

        p.state.relay.push(relay_target);
        Ok(None)
    }

    /// Relay all text into temporary file
    ///
    /// Every text including non macro calls are all sent to relay target
    ///
    /// # Usage
    ///
    /// $relayt(type,argument)
    pub(crate) fn relayt(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let target = args.get_ctext(0)?;

        p.log_warning(
            &format!("Relaying text content to \"{}\"", target),
            WarningType::Security,
        )?;

        let relay_target = RelayTarget::Temp;

        p.state.relay.push(relay_target);
        Ok(None)
    }

    /// Relay all text to file
    ///
    /// Every text including non macro calls are all sent to relay target
    ///
    /// # Usage
    ///
    /// $relayf(argument)
    pub(crate) fn relayf(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        use crate::common::FileTarget;
        let args = ArgParser::new().args_with_len(input)?;

        let target = args.get_path(0)?;

        p.log_warning(
            &format!("Relaying text content to \"{}\"", target.display()),
            WarningType::Security,
        )?;

        let file_target = FileTarget::from_path(target)?;

        let relay_target = RelayTarget::File(file_target);

        p.state.relay.push(relay_target);
        Ok(None)
    }

    // This function iterate through lines twice
    // 1. Regex and calculate nested level and corresponding identifier
    // 2. Regex again while replacing specific parts of string
    /// Rearrange
    ///
    /// # Usage
    ///
    /// $reo(
    /// 3.
    /// 2.
    /// 1.
    ///     4]
    ///     7]
    ///     8]
    /// )
    pub(crate) fn reorder(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let mut reo_hash = ReoHash::default();
        let mut blank_str: &str; // Container

        // TODO
        // Should I really collect it for indexing?
        // Can it be improved?
        let mut lines = args
            .get_text(0)?
            .full_lines()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        // TODO
        // Refactor iteration_cache into a separate struct
        let mut iteration_cache: Vec<(usize, usize)> = Vec::new();
        // Find list elements and save counts of each sorts
        for (ll, line) in lines.iter().enumerate() {
            if let Some(captured) = BLANKHASH_MATCH.captures(line) {
                blank_str = captured.get(1).map_or("", |m| m.as_str());
                let index_id = captured.get(2).map_or("", |m| m.as_str());
                let blank = reo_hash.try_insert(blank_str, index_id)?;
                iteration_cache.push((blank, ll));
            }
        }

        let mut blank_cache = 0;
        let mut index_cache: String = String::default();
        let mut counter = 0usize;
        let mut replaced;
        // Iterate lists and replace number according to proper order
        for (blank, ll) in iteration_cache {
            let line = &lines[ll];
            if let Some(captured) = REPLACER_MATCH.captures(line) {
                // REPLACER INGREdient
                // ---
                let leading_part = captured.get(1).map_or("", |m| m.as_str());
                let index = captured.get(3).map_or("", |m| m.as_str());
                let following_part = captured.get(4).map_or("", |m| m.as_str());
                // ---
                // Different index from prior line OR different indentation
                if index != index_cache || blank_cache != blank {
                    counter = reo_hash.get_current_count(blank, index);

                    // This means list items go up
                    if blank_cache > blank {
                        // Reset previous cache
                        reo_hash.update_counter(blank_cache, &index_cache, 1);
                    }
                } else {
                    counter += 1;
                }
                blank_cache = blank;
                index_cache = index.to_string();

                replaced = REPLACER_MATCH
                    .replace(
                        line,
                        format!("{}{}{}{}", leading_part, counter, index, following_part),
                    )
                    .to_string();
                reo_hash.update_counter(blank, index, counter + 1);
                lines[ll] = replaced;
            }
        }
        Ok(Some(lines.join("")))
    }

    /// Disable relaying
    ///
    /// # Usage
    ///
    /// $halt()
    pub(crate) fn halt_relay(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let halt_immediate = if let Ok(val) = args.get_bool(0) {
            val
        } else {
            false
        };

        if halt_immediate {
            // This remove last element from stack
            p.state.relay.pop();
        } else {
            p.state.escape_newline = true;
            p.insert_queue("$halt(true)");
        }
        Ok(None)
    }

    /// Set temporary file
    ///
    /// This forcefully merge paths
    ///
    /// # Usage
    ///
    /// $tempto(file_name)
    pub(crate) fn set_temp_target(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let path = &std::env::temp_dir().join(args.get_path(0)?);
        Utils::check_file_sanity(processor, path)?;
        processor.set_temp_file(path)?;
        Ok(None)
    }

    /// Get temporary path
    ///
    /// # Usage
    ///
    /// $temp()
    pub(crate) fn get_temp_path(
        _: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        Ok(Some(processor.state.temp_target.to_string()))
    }

    /// Get number
    ///
    /// # Usage
    ///
    /// $num(20%)
    pub(crate) fn get_number(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let src = args.get_ctext(0)?;
        let captured = NUM_MATCH.captures(src).ok_or_else(|| {
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
    }

    /// Capitalize text
    ///
    /// # Usage
    ///
    /// $upper(hello world)
    pub(crate) fn capitalize(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let src = args.get_text(0)?;
        Ok(Some(src.to_uppercase()))
    }

    /// Lower text
    ///
    /// # Usage
    ///
    /// $lower(hello world)
    pub(crate) fn lower(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let src = args.get_text(0)?;
        Ok(Some(src.to_lowercase()))
    }

    /// Comment
    ///
    /// # Usage
    ///
    /// $comment(any)
    pub(crate) fn require_comment(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let comment_type = args.get_enum(0, CommentType::from_str)?;

        if p.state.comment_type != comment_type {
            return Err(RadError::UnsoundExecution(format!(
                "Comment type, \"{:#?}\" is required but it is not",
                comment_type
            )));
        }
        Ok(None)
    }

    /// require
    ///
    /// # Usage
    ///
    /// $require(fout)
    pub(crate) fn require_permissions(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new()
            .split(SplitVariant::Always)
            .args_with_len(input)?;
        if args.is_empty() {
            p.log_warning(
                "Require macro used without any arguments.",
                WarningType::Sanity,
            )?;
        }
        for idx in 0..args.len() {
            let auth = args.get_enum(idx, AuthType::from_str)?;
            let state = p.state.auth_flags.get_state(&auth);
            if let AuthState::Restricted = state {
                return Err(RadError::UnsoundExecution(format!(
                    "Permission \"{}\" is required but is not provided.",
                    auth
                )));
            }
        }
        Ok(None)
    }

    /// Strict
    ///
    /// # Usage
    ///
    /// $strict(lenient)
    pub(crate) fn require_strict(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let vec = ArgParser::new().args_with_len(input)?;
        let mode = if let Ok(v) = vec.get_enum(0, ErrorBehaviour::from_str) {
            v
        } else {
            ErrorBehaviour::Strict
        };
        if p.state.behaviour != mode {
            return Err(RadError::UnsoundExecution(format!(
                "{:#?} mode is required but it is not provided",
                mode,
            )));
        }
        Ok(None)
    }

    /// Output
    ///
    /// # Usage
    ///
    /// $Output(fout)
    pub(crate) fn require_output(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;
        let output_type = args.get_enum(0, OutputType::from_str)?;
        match output_type {
            OutputType::Terminal => {
                if let WriteOption::Terminal = p.write_option {
                } else {
                    return Err(RadError::UnsoundExecution(
                        "Rad should write to a terminal and yet such flag was not satisfied."
                            .to_owned(),
                    ));
                }
            }
            OutputType::File => {
                if let WriteOption::File(_) = p.write_option {
                } else {
                    return Err(RadError::UnsoundExecution(
                        "Rad should write to a file and yet such flag was not satisfied."
                            .to_owned(),
                    ));
                }
            }
            OutputType::Discard => {
                if let WriteOption::Discard = p.write_option {
                } else {
                    return Err(RadError::UnsoundExecution(
                        "Rad should discard output and yet such flag was not satisfied.".to_owned(),
                    ));
                }
            }
        }
        Ok(None)
    }

    /// Log message
    ///
    /// # Usage
    ///
    /// $log(This is a problem)
    pub(crate) fn log_message(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        p.log_message(input.args)?;
        Ok(None)
    }

    /// Log error message
    ///
    /// # Usage
    ///
    /// $loge(This is a problem)
    pub(crate) fn log_error_message(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        p.print_error(input.args)?;
        Ok(None)
    }

    /// Print message
    ///
    /// # Usage
    ///
    /// $println(This is a problem)
    pub(crate) fn print_message(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        write!(std::io::stdout(), "{}{}", input.args, p.state.newline)?;
        Ok(None)
    }

    /// Get max value from array
    ///
    /// # Usage
    ///
    /// $max(1,2,3,4,5)
    pub(crate) fn get_max(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new()
            .split(SplitVariant::Always)
            .args_with_len(input)?;

        let max = (0..args.len())
            .map(|i| args.get_text(i).unwrap())
            .max()
            .unwrap();
        Ok(Some(max.to_string()))
    }

    /// Get min value from array
    ///
    /// # Usage
    ///
    /// $min(1,2,3,4,5)
    pub(crate) fn get_min(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new()
            .split(SplitVariant::Always)
            .args_with_len(input)?;

        let min = (0..args.len())
            .map(|i| args.get_text(i).unwrap())
            .min()
            .unwrap();
        Ok(Some(min.to_string()))
    }

    /// Increase source by value
    ///
    /// # Usage
    ///
    /// $inc(value)
    /// $inc(value,amount)
    #[cfg(feature = "evalexpr")]
    pub(crate) fn increase_number(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        if args.is_empty() {
            return Err(RadError::InvalidArgument(
                "inc requires a value to increment".to_owned(),
            ));
        }

        let number = args.get_ctext(0)?;
        let amount = if let Ok(amt) = args.get_uint(1) {
            amt
        } else {
            1
        };
        let ret = eval(&format!("{number} + {amount}"))?;
        Ok(Some(ret.to_string()))
    }

    /// Decrease source by value
    ///
    /// # Usage
    ///
    /// $dec(value)
    /// $dec(value,amount)
    #[cfg(feature = "evalexpr")]
    pub(crate) fn decrease_number(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        if args.is_empty() {
            return Err(RadError::InvalidArgument(
                "dec requires a value to decrement".to_owned(),
            ));
        }

        let number = args.get_ctext(0)?;
        let amount = if let Ok(amt) = args.get_uint(1) {
            amt
        } else {
            1
        };
        let ret = eval(&format!("{number} - {amount}"))?;
        Ok(Some(ret.to_string()))
    }

    /// Square
    ///
    /// # Usage
    ///
    /// $square(value)
    #[cfg(feature = "evalexpr")]
    pub(crate) fn square_number(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let number = args.get_text(0)?;
        let ret = eval(&format!(" {number} ^ 2"))?;
        Ok(Some(ret.to_string()))
    }

    /// Cube
    ///
    /// # Usage
    ///
    /// $cube(value)
    #[cfg(feature = "evalexpr")]
    pub(crate) fn cube_number(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let number = args.get_text(0)?;
        let ret = eval(&format!(" {number} ^ 3"))?;
        Ok(Some(ret.to_string()))
    }

    /// Power
    ///
    /// # Usage
    ///
    /// $pow(value,exponent)
    #[cfg(feature = "evalexpr")]
    pub(crate) fn power_number(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let number = args.get_text(0)?;
        let ex = args.get_text(1)?;
        let ret = eval(&format!(" {number} ^ {ex}"))?;
        Ok(Some(ret.to_string()))
    }

    /// square root
    ///
    /// # Usage
    ///
    /// $sqrt(value,exponent)
    #[cfg(feature = "evalexpr")]
    pub(crate) fn square_root(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let number = args.get_text(0)?;
        let ret = eval(&format!("math::sqrt({number})"))?;
        Ok(Some(ret.to_string()))
    }

    /// Round
    ///
    /// # Usage
    ///
    /// $round(value)
    pub(crate) fn round_number(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let number = &args.get_float(0)?;
        let ret = number.round() as isize;
        Ok(Some(ret.to_string()))
    }

    /// Get ceiling value
    ///
    /// # Usage
    ///
    /// $ceil(1.56)
    pub(crate) fn get_ceiling(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let number = args.get_float(0)?;
        Ok(Some(number.ceil().to_string()))
    }

    /// Get floor value
    ///
    /// # Usage
    ///
    /// $floor(1.23)
    pub(crate) fn get_floor(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let number = args.get_float(0)?;
        Ok(Some(number.floor().to_string()))
    }

    /// Precision
    ///
    /// # Usage
    ///
    /// $prec(1.56,2)
    pub(crate) fn prec(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let precision = args.get_uint(0)?;
        let number = args.get_float(1)?;
        let decimal_precision = 10.0f32.powi(precision as i32);
        let converted = f32::trunc(number * decimal_precision) / decimal_precision;
        let formatted = format!("{:.1$}", converted, precision);

        Ok(Some(formatted))
    }

    /// Reverse array
    ///
    /// # Usage
    ///
    /// $rev(1,2,3,4,5)
    pub(crate) fn reverse_array(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new()
            .split(SplitVariant::Always)
            .args_with_len(input)?;

        let rev = (0..args.len())
            .map(|i| args.get_text(i))
            .rev()
            .collect::<RadResult<Vec<_>>>()?
            .join(",");

        Ok(Some(rev))
    }

    /// Reverse lines
    ///
    /// # Usage
    ///
    /// $revl( ... )
    pub(crate) fn reverse_lines(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let text = args.get_text(0)?;

        let line_ending = text.get_line_ending();

        let mut tet = text
            .full_lines()
            .collect::<Vec<&str>>()
            .iter()
            .rev()
            .join("");

        if !tet.ends_with(line_ending) {
            tet.push_str(line_ending);
        }

        Ok(Some(tet))
    }

    /// Declare an empty macros
    ///
    /// # Usage
    ///
    /// $decl(n1,n2,n3)
    pub(crate) fn declare(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new()
            .split(SplitVariant::Always)
            .args_with_len(input)?;

        let runtime_rules = (0..args.len())
            .map(|idx| {
                let value = args.get_ctext(idx)?;
                Ok((value.to_string(), "", ""))
            })
            .collect::<RadResult<Vec<(String, &str, &str)>>>()?;

        // Check overriding. Warn or yield error
        for (name, _, _) in runtime_rules.iter() {
            if name.is_empty() {
                processor.log_warning(
                    "Declaring a macro with blank charcters is not valid",
                    WarningType::Sanity,
                )?;
            }
            if processor.contains_macro(name, MacroType::Any) {
                if processor.state.behaviour == ErrorBehaviour::Strict {
                    return Err(RadError::InvalidMacroDefinition(format!(
                        "Declaring a macro with a name already existing : \"{}\"",
                        name
                    )));
                }
                processor.log_warning(
                    &format!(
                        "Declaring a macro with a name already existing : \"{}\"",
                        name
                    ),
                    WarningType::Sanity,
                )?;
            }
        }

        // Add runtime rules
        processor.add_runtime_rules(&runtime_rules)?;
        Ok(None)
    }

    /// Dump a file
    ///
    /// # Usage
    ///
    /// $dump(macro,content)
    pub(crate) fn dump_file_content(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let file_name = args.get_path(0)?;

        if !file_name.is_file() {
            return Err(RadError::InvalidExecution(format!(
                "Dump requires a real file to dump but given file \"{}\" doesn't exist",
                file_name.display()
            )));
        }

        {
            std::fs::File::create(file_name)?;
        }

        Ok(None)
    }

    /// Document a macro
    ///
    /// # Usage
    ///
    /// $document(macro,content)
    pub(crate) fn document(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let macro_name = args.get_ctext(0)?;
        let content = args.get_text(1)?;

        // If operation failed
        if !processor.set_documentation(macro_name, content) {
            // Document can only be applied to
            // runtime macro
            let err = RadError::NoSuchMacroName(
                macro_name.to_string(),
                processor.get_similar_macro(macro_name, true),
            );

            if processor.state.behaviour == ErrorBehaviour::Strict {
                // Return error to processor
                return Err(err);
            }

            // Log error
            processor.log_error(&err.to_string())?;
        }

        Ok(None)
    }

    /// Declare a local macro
    ///
    /// Local macro gets deleted after macro execution
    ///
    /// # Usage
    ///
    /// $let(name,value)
    pub(crate) fn bind_to_local(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let name = args.get_ctext(0)?;
        let value = args.get_text(1)?;
        processor.add_new_local_macro(1, name, value);
        Ok(None)
    }

    /// Clear volatile macros
    pub(crate) fn clear(_: MacroInput, processor: &mut Processor) -> RadResult<Option<String>> {
        if processor.state.hygiene == Hygiene::None {
            processor.log_warning(
                "Currently hygiene mode is not set. Clear will do nothing.",
                WarningType::Sanity,
            )?;
        }
        processor.clear_volatile();
        Ok(None)
    }

    /// Enable/disable hygiene's macro mode
    ///
    /// # Usage
    ///
    /// $hygiene(true)
    /// $hygiene(false)
    pub(crate) fn toggle_hygiene(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        processor.toggle_hygiene(args.get_bool(0)?);
        Ok(None)
    }

    /// Pause every macro expansion
    ///
    /// Only other pause call is evaluated
    ///
    /// # Usage
    ///
    /// $pause(true)
    /// $pause(false)
    pub(crate) fn pause(input: MacroInput, processor: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        processor.state.paused = args.get_bool(0)?;
        Ok(None)
    }

    /// Define a static macro
    ///
    /// # Usage
    ///
    /// $static(name,value)
    pub(crate) fn define_static(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let name = args.get_ctext(0)?;
        let value = args.get_text(1)?;
        // Macro name already exists
        if processor.contains_macro(name, MacroType::Any) {
            // Strict mode prevents overriding
            // Return error
            if processor.state.behaviour == ErrorBehaviour::Strict {
                return Err(RadError::UnsoundExecution(format!(
                    "Creating a static macro with a name already existing : \"{}\"",
                    name
                )));
            }
            // Its warn-able anyway
            processor.log_warning(
                &format!(
                    "Creating a static macro with a name already existing : \"{}\"",
                    name
                ),
                WarningType::Sanity,
            )?;
        }
        processor.add_static_rules(&[(name, &value)])?;
        Ok(None)
    }

    /// Change a notation of a number
    ///
    /// # Usage
    ///
    /// $notat(23,binary)
    pub(crate) fn change_notation(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let num = args.get_int(0)?;
        let notation = args.get_ctext(1)?.to_lowercase();
        let format = match notation.as_str() {
            "bin" => format!("{:b}", num),
            "oct" => format!("{:o}", num),
            "hex" => format!("{:x}", num),
            _ => {
                return Err(RadError::InvalidArgument(format!(
                    "Unsupported notation format \"{}\"",
                    notation
                )))
            }
        };
        Ok(Some(format))
    }

    /// Replace value
    ///
    /// # Usage
    ///
    /// $repl(macro,value)
    pub(crate) fn replace(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let name = args.get_ctext(0)?;
        let target = args.get_text(1)?;
        if !processor.replace_macro(name, target) {
            return Err(RadError::NoSuchMacroName(
                name.to_owned(),
                processor.get_similar_macro(target, true), // Only runtime macro can be replaced.
            ));
        }
        Ok(None)
    }

    /// gt : is lvalue bigger than rvalue
    ///
    /// # Usage
    ///
    /// $gt(lvalue, rvalue)
    pub(crate) fn greater_than(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let lvalue = args.get_text(0)?;
        let rvalue = args.get_text(1)?;
        Ok(Some((lvalue > rvalue).to_string()))
    }

    /// gte : is lvalue bigger than or equal to rvalue
    ///
    /// # Usage
    ///
    /// $gte(lvalue, rvalue)
    pub(crate) fn greater_than_or_equal(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let lvalue = args.get_text(0)?;
        let rvalue = args.get_text(1)?;
        Ok(Some((lvalue >= rvalue).to_string()))
    }

    /// lt : is lvalue less than rvalue
    ///
    /// # Usage
    ///
    /// $lt(lvalue, rvalue)
    pub(crate) fn less_than(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let lvalue = args.get_text(0)?;
        let rvalue = args.get_text(1)?;
        Ok(Some((lvalue < rvalue).to_string()))
    }

    /// lte : is lvalue less than or equal to rvalue
    ///
    /// # Usage
    ///
    /// $lte(lvalue, rvalue)
    pub(crate) fn less_than_or_equal(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let lvalue = args.get_text(0)?;
        let rvalue = args.get_text(1)?;
        Ok(Some((lvalue <= rvalue).to_string()))
    }

    /// eq : are values equal
    ///
    /// # Usage
    ///
    /// $eq(lvalue, rvalue)
    pub(crate) fn are_values_equal(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let lvalue = args.get_text(0)?;
        let rvalue = args.get_text(1)?;
        Ok(Some(lvalue.eq(rvalue).to_string()))
    }

    /// isempty : Check if value is empty
    ///
    /// # Usage
    ///
    /// $isempty(value)
    pub(crate) fn is_empty(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let target = if input.attr.trim_input {
            input.args.trim()
        } else {
            input.args
        };
        Ok(Some(target.is_empty().to_string()))
    }

    /// iszero : Check if value is zero
    ///
    /// # Usage
    ///
    /// $iszero(value)
    pub(crate) fn is_zero(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let value = args.get_ctext(0)?;
        Ok(Some(value.eq("0").to_string()))
    }

    /// foldlc
    ///
    /// # Usage
    ///
    /// $foldlc(count,type)
    pub(crate) fn fold_lines_by_count(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        use std::fmt::Write;

        let count = args.get_uint(0)?;
        let mut formatted = String::new();
        let mut container = vec![];

        for (idx, line) in args.get_text(1)?.full_lines().enumerate() {
            container.push(line);
            if (idx + 1) % count == 0 {
                write!(
                    formatted,
                    "{}",
                    merge_container(&mut container, p.env, Some(&p.state.newline),)
                )?;
            }
        }
        if !container.is_empty() {
            write!(
                formatted,
                "{}",
                merge_container(&mut container, p.env, Some(&p.state.newline),)
            )?;
        }

        Ok(Some(formatted))
    }

    /// isolate
    ///
    /// # Usage
    ///
    /// $insulav(value)
    pub(crate) fn isolate_vertical(
        input: MacroInput,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        use std::fmt::Write;
        let args = ArgParser::new().args_with_len(input)?;

        let mut formatted = String::new();
        let mut only_blank = true;
        let mut first_contact = false;
        let mut nest_level = 1usize;
        for ch in args.get_text(0)?.chars() {
            let is_isolation = ISOLATION_CHARS.contains(&ch);
            if only_blank && !ch.is_whitespace() && !is_isolation {
                only_blank = false;
                first_contact = true;
            }
            if is_isolation {
                first_contact = false;
                if ISOLATION_CHARS_CLOSING.contains(&ch) {
                    nest_level -= 1;
                }
                if !only_blank {
                    write!(formatted, "{}", p.state.newline)?;
                }
                write!(
                    formatted,
                    "{2}{0}{1}",
                    ch,
                    p.state.newline,
                    " ".repeat((nest_level - 1) * 4)
                )?;
                if ISOLATION_CHARS_OPENING.contains(&ch) {
                    nest_level += 1;
                }

                only_blank = true;
            } else if !only_blank || !ch.is_whitespace() {
                // TODO Check first contact
                if first_contact {
                    write!(formatted, "{}", " ".repeat((nest_level - 1) * 4))?;
                    first_contact = false;
                }
                write!(formatted, "{ch}")?;
            }
        }

        Ok(Some(formatted))
    }

    /// isolate horizontal
    ///
    /// # Usage
    ///
    /// $insulah(value)
    pub(crate) fn isolate_horizontal(
        input: MacroInput,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let mut formatted = String::new();
        let mut iter = args.get_text(0)?.chars().peekable();
        let mut previous: char = '@';
        let mut put_after = false;
        while let Some(ch) = iter.next() {
            // --New-- code
            if previous.is_whitespace() && ch.is_whitespace() {
                previous = ch;
                continue;
            }

            let next_ch = iter.peek().unwrap_or(&' ');

            if ISOLATION_SINGLE_SPACE.contains(&ch)
                && !ISOLATION_SINGLE_SPACE.contains(next_ch)
                && !ISOLATION_SINGLE_SPACE.contains(&previous)
            {
                put_after = true;
            }

            if ISOLATION_CHARS_OPENING.contains(&ch) && !ISOLATION_CHARS.contains(next_ch) {
                put_after = true;
            }

            if ISOLATION_CHARS_CLOSING.contains(&ch) && !ISOLATION_CHARS.contains(&previous) {
                formatted.push(' ');
            }

            // Current is = put space before
            if ISOLATION_SURR_SPACE.contains(&ch) && !previous.is_whitespace() {
                formatted.push(' ');
                put_after = true;
            }

            formatted.push(ch);
            previous = ch;

            if put_after {
                if !next_ch.is_whitespace() {
                    formatted.push(' ');
                }
                put_after = false;
            }
        }

        Ok(Some(formatted))
    }

    /// istype : Qualify a value
    ///
    /// # Usage
    ///
    /// $istype(value,type)
    pub(crate) fn qualify_value(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let qtype = args.get_ctext(0)?;
        let value = args.get_ctext(1)?;
        let qualified = match qtype.to_lowercase().as_str() {
            "uint" => value.parse::<usize>().is_ok(),
            "int" => value.parse::<isize>().is_ok(),
            "float" => value.parse::<f64>().is_ok(),
            "bool" => value.is_arg_true().is_ok(),
            _ => {
                return Err(RadError::InvalidArgument(format!(
                    "Given type \"{}\" is not valid",
                    &qtype
                )));
            }
        };
        Ok(Some(qualified.to_string()))
    }

    /// Source static file
    ///
    /// Source file's format is mostly equivalent with env.
    /// $source(file_name.renv)
    pub(crate) fn source_static_file(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let path = args.get_path(0)?;
        if !path.exists() {
            return Err(RadError::InvalidExecution(format!(
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
                match processor.parse_chunk(0, MAIN_CALLER, body) {
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
                return Err(RadError::InvalidExecution(format!(
                    "Invalid line in source file, line \"{}\" \n = \"{}\"",
                    idx, line
                )));
            }
        }
        processor.set_sandbox(false);
        Ok(None)
    }

    /// Import a frozen file
    ///
    /// $import(file.r4f)
    pub(crate) fn import_frozen_file(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let path = args.get_path(0)?;
        if !path.exists() {
            return Err(RadError::InvalidExecution(format!(
                "Cannot import from non-existent file \"{}\"",
                path.display()
            )));
        }
        processor.import_frozen_file(path)?;

        Ok(None)
    }

    // TODO TT
    // Needs refactor
    // /// List directory files
    // ///
    // /// $listdir(path, is_abs, delimiter)
    // pub(crate) fn list_directory_files(
    //     input: MacroInput,
    //     processor: &mut Processor,
    // ) -> RadResult<Option<String>> {
    //     if !Utils::is_granted("listdir", AuthType::FIN, processor)? {
    //         return Ok(None);
    //     }
    //     let args = ArgParser::new().args_with_len(input)?;
    //     if args.is_empty() {
    //         return Err(RadError::InvalidArgument(
    //             "listdir at least requires an argument".to_owned(),
    //         ));
    //     }
    //
    //     let absolute = if let Some(val) = args.get(1) {
    //         match val.is_arg_true() {
    //             Ok(value) => value,
    //             Err(_) => {
    //                 return Err(RadError::InvalidArgument(format!(
    //                     "listdir's second argument should be a boolean value but given : \"{}\"",
    //                     args.get_text(0)?
    //                 )));
    //             }
    //         }
    //     } else {
    //         false
    //     };
    //
    //     let path;
    //     if let Some(val) = args.first() {
    //         path = if val.is_empty() {
    //             processor.get_current_dir()?
    //         } else {
    //             PathBuf::from(val.trim())
    //         };
    //         if !path.exists() {
    //             return Err(RadError::InvalidExecution(format!(
    //                 "Cannot list non-existent directory \"{}\"",
    //                 path.display()
    //             )));
    //         }
    //     } else {
    //         path = processor.get_current_dir()?
    //     };
    //
    //     let delim = if let Some(val) = args.get(2) {
    //         val
    //     } else {
    //         ","
    //     };
    //
    //     let mut vec = vec![];
    //     for entry in std::fs::read_dir(path)? {
    //         let entry = entry?;
    //         if absolute {
    //             vec.push(std::fs::canonicalize(entry.path().as_os_str())?);
    //         } else {
    //             vec.push(entry.file_name().into());
    //         }
    //     }
    //
    //     let result: Vec<_> = vec
    //         .iter()
    //         .map(|p| p.display().to_string())
    //         .collect::<Vec<_>>();
    //     Ok(Some(result.join(delim)))
    // }

    /// Paste unicode character in place
    ///
    /// $unicode(123)
    pub(crate) fn paste_unicode(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let unicode_character = args.get_ctext(0)?;
        let unicode_hex = u32::from_str_radix(unicode_character, 16).map_err(|_| {
            RadError::InvalidArgument(format!(
                "Could not convert given value \"{}\" into a u32 unicode value",
                unicode_character
            ))
        })?;
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
    }

    /// Pop value from macro
    ///
    /// THis never fails to pop
    ///
    /// $pop(macro_name)
    pub(crate) fn pop(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let level = input.level;
        let args = ArgParser::new().args_with_len(input)?;

        let name = args.get_ctext(0)?;
        p.modify_macro_body(name, "", level, |s: &str, _: &str| -> RadResult<String> {
            let last_ch_byte_index = s.char_indices().next_back().unwrap_or((0, '0')).0;
            Ok(s[0..last_ch_byte_index].to_string())
        })?;
        Ok(None)
    }

    /// append value to macro
    ///
    /// $append(macro_name, value)
    pub(crate) fn append(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let level = input.level;
        let args = ArgParser::new().args_with_len(input)?;

        let name = args.get_ctext(0)?;
        let value = args.get_text(1)?;
        p.modify_macro_body(
            name,
            value,
            level,
            |s: &str, args: &str| -> RadResult<String> {
                let mut ret = s.to_owned();
                ret.push_str(args);
                Ok(ret)
            },
        )?;
        Ok(None)
    }

    /// prepend value to macro
    ///
    /// $prepend(macro_name, value)
    pub(crate) fn prepend(input: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
        let level = input.level;
        let args = ArgParser::new().args_with_len(input)?;

        let name = args.get_ctext(0)?;
        let value = args.get_text(1)?;
        p.modify_macro_body(
            name,
            value,
            level,
            |s: &str, args: &str| -> RadResult<String> {
                let mut ret = s.to_owned();
                ret.insert_str(0, args);
                Ok(ret)
            },
        )?;
        Ok(None)
    }

    /// Get characters array
    ///
    /// $chars(abcde)
    pub(crate) fn chars_array(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let arg = args.get_ctext(0)?;
        let mut chars = arg.chars().fold(String::new(), |mut acc, ch| {
            acc.push(ch);
            acc.push(',');
            acc
        });
        chars.pop();
        Ok(Some(chars))
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
    pub(crate) fn hook_enable(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let hook_type = HookType::from_str(args.get_ctext(0)?)?;
        let index = args.get_ctext(1)?;
        processor.hook_map.switch_hook(hook_type, index, true)?;
        Ok(None)
    }

    /// Disable hook
    ///
    /// * Usage
    ///
    /// $hookoff(MacroType, macro_name)
    #[cfg(feature = "hook")]
    pub(crate) fn hook_disable(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let hook_type = HookType::from_str(args.get_ctext(0)?)?;
        let index = args.get_ctext(1)?;
        processor.hook_map.switch_hook(hook_type, index, false)?;
        Ok(None)
    }

    /// Wrap text
    ///
    /// * Usage
    ///
    /// $wrap(80, Content goes here)
    pub(crate) fn wrap(input: MacroInput, _: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let width = args.get_uint(0)?;
        let content = args.get_text(1)?;
        let result = textwrap::fill(content, width);
        Ok(Some(result))
    }

    /// Update storage
    ///
    /// # Usage
    ///
    /// $update(text)
    pub(crate) fn update_storage(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        // TODO
        // Improve by not allocating
        let args = ArgParser::new()
            .args_with_len(input)
            .into_iter()
            .enumerate()
            .map(|(i, s)| s.get_text(i).unwrap().to_string())
            .collect_vec();

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

    /// Extract storage
    ///
    /// # Usage
    ///
    /// $extract()
    pub(crate) fn extract_storage(
        _: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        // Execute update method for storage
        if let Some(storage) = processor.storage.as_mut() {
            match storage.extract(false) {
                Err(err) => Err(RadError::StorageError(format!("Update error : {}", err))),
                Ok(value) => {
                    if let Some(output) = value {
                        Ok(Some(output.into_printable()))
                    } else {
                        Ok(Some(String::new()))
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

    pub(crate) fn cindex_register(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        use cindex::ReaderOption;
        let args = ArgParser::new().args_with_len(input)?;

        let table_name = args.get_ctext(0)?;
        if processor.indexer.contains_table(table_name) {
            return Err(RadError::InvalidExecution(format!(
                "Cannot register exsiting table : \"{}\"",
                args.get_text(0)?
            )));
        }
        let mut option = ReaderOption::new();
        option.ignore_empty_row = true;
        processor.indexer.add_table_with_option(
            table_name,
            args.get_ctext(1)?.as_bytes(),
            option,
        )?;
        Ok(None)
    }

    /// Drop a table
    ///
    /// $dropcsv(table_name)
    #[cfg(feature = "cindex")]
    pub(crate) fn cindex_drop(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        processor.indexer.drop_table(args.get_ctext(0)?);
        Ok(None)
    }

    /// Execute query from indexer table
    ///
    /// $query(statment)
    #[cfg(feature = "cindex")]
    pub(crate) fn cindex_query(
        input: MacroInput,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_with_len(input)?;

        let mut value = String::new();
        processor
            .indexer
            .index_raw(args.get_ctext(0)?, OutOption::Value(&mut value))?;
        Ok(Some(value.trim().to_string()))
    }
}

// ---
// <MISC>
// Private structs for organizational purposes
// ---

/// Counter for total list items
#[derive(Default, Debug)]
struct ReoHash {
    index_hash: HashMap<String, ListCounterByLevel>,
}

impl ReoHash {
    pub fn update_counter(&mut self, blank: usize, index: &str, counter: usize) {
        *self
            .index_hash
            .get_mut(index)
            .unwrap()
            .counts
            .get_mut(&blank)
            .unwrap() = counter;
    }
    pub fn get_current_count(&self, blank: usize, index: &str) -> usize {
        *self
            .index_hash
            .get(index)
            .unwrap()
            .counts
            .get(&blank)
            .unwrap()
    }

    pub fn try_insert(&mut self, blank: &str, index: &str) -> RadResult<usize> {
        let blank: usize = BlankHash::from_str(blank)?.into();
        match self.index_hash.get_mut(index) {
            Some(hash) => {
                hash.counts.entry(blank).or_insert(1);
            }
            None => {
                // Create a new value
                let mut count_level = ListCounterByLevel::default();
                count_level.counts.insert(blank, 1);
                self.index_hash.insert(index.to_owned(), count_level);
            }
        }
        Ok(blank)
    }
}

/// This is sub-struct for rearrange macro
#[derive(Default, Debug, PartialEq, Eq, Hash)]
struct BlankHash {
    index: usize,
}

impl From<BlankHash> for usize {
    fn from(value: BlankHash) -> Self {
        value.index
    }
}

const SPACE_SIZE: usize = 1;
const TAB_SIZE: usize = 4;

impl FromStr for BlankHash {
    type Err = RadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut hash = BlankHash::default();
        for ch in s.chars() {
            if ch == '\t' {
                hash.index += TAB_SIZE;
            } else if ch == ' ' {
                hash.index += SPACE_SIZE;
            } else {
                return Err(RadError::InvalidConversion(format!(
                    "Could not create a BlankHash from string possibly due to \
            incorrect input : Source string \"{}\"",
                    s
                )));
            }
        }
        Ok(hash)
    }
}

#[derive(Debug, Default)]
struct ListCounterByLevel {
    // key means level and value means total count of list items
    counts: HashMap<usize, usize>,
}

#[derive(Debug, Default)]
struct InnerCursor {
    start_index: usize,
    end_index: usize,
    level: usize,
}

/// Merge container into a string
#[inline]
fn merge_container(
    container: &mut Vec<&str>,
    env: MacEnv,
    default_line_end: Option<&str>,
) -> String {
    let joiner = if env.fold_space { " " } else { "" };
    let mapper = if env.fold_trim { Some(str::trim) } else { None };
    let line_end: &str;
    // Reverse fold order
    if env.fold_reverse {
        if let Some(line_ending) = container.first() {
            line_end = line_ending.get_line_ending_always(default_line_end.unwrap_or(""));
        } else {
            // Empty vector
            return String::default();
        }
        std::mem::take(container)
            .iter()
            .rev()
            .map(|s| {
                let s = s.strip_newline();
                if let Some(mapper) = mapper {
                    mapper(s)
                } else {
                    s
                }
            })
            .join(joiner)
            + line_end
    } else {
        if let Some(line_ending) = container.last() {
            line_end = line_ending.get_line_ending_always(default_line_end.unwrap_or(""));
        } else {
            // Empty vector
            return String::default();
        }
        std::mem::take(container)
            .iter()
            .map(|s| {
                let s = s.strip_newline();
                if let Some(mapper) = mapper {
                    mapper(s)
                } else {
                    s
                }
            })
            .join(joiner)
            + line_end
    }
}

#[inline]
fn check_neg(num: isize, no_negative_index: bool) -> RadResult<()> {
    if no_negative_index && num.is_negative() {
        return Err(RadError::UnsoundExecution(
            "Negative index is an error because environment variable is set".to_string(),
        ));
    }
    Ok(())
}

// TEST

// ---
// </MISC>
// ---
