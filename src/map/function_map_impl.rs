use super::function_map::FunctionMacroMap;

use crate::auth::{AuthState, AuthType};
use crate::common::{
    AlignType, ErrorBehaviour, FlowControl, MacroType, ProcessInput, RadResult, RelayTarget,
};
use crate::consts::{
    LOREM, LOREM_SOURCE, LOREM_WIDTH, MACRO_SPECIAL_LIPSUM, MAIN_CALLER, PATH_SEPARATOR,
};
use crate::error::RadError;
use crate::formatter::Formatter;
#[cfg(feature = "hook")]
use crate::hookmap::HookType;
use crate::logger::WarningType;
use crate::utils::{Utils, NUM_MATCH};
use crate::{stake, trim, CommentType, WriteOption};
use crate::{ArgParser, SplitVariant};
use crate::{Hygiene, Processor};
#[cfg(feature = "cindex")]
use cindex::OutOption;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use unicode_width::UnicodeWidthStr;

static ISOLATION_SINGLE: [char; 1] = [','];
static ISOLATION_CHARS: [char; 6] = ['(', ')', '[', ']', '{', '}'];
static ISOLATION_CHARS_OPENING: [char; 3] = ['(', '[', '{'];
static ISOLATION_CHARS_CLOSING: [char; 3] = [')', ']', '}'];

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
/// Path separator match
static PATH_MATCH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(\\|/)"#).expect("Failed to create path separator matches"));

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
    pub(crate) fn time(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn hms(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn date(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn regex_sub(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let match_expr = &args[0];
            let substitution = &args[1];
            let source = &args[2];

            if match_expr.is_empty() {
                return Err(RadError::InvalidArgument(
                    "Regex expression cannot be an empty string".to_string(),
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
    pub(crate) fn print_current_input(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        match &p.state.current_input {
            ProcessInput::Stdin => Ok(Some("Stdin".to_string())),
            ProcessInput::File(path) => {
                let args = ArgParser::new().args_to_vec(args, ',', SplitVariant::Always);
                if !args.is_empty() && !trim!(&args[0]).is_empty() {
                    let print_absolute = Utils::is_arg_true(trim!(&args[0]))?;
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
    pub(crate) fn get_file_time(
        args: &str,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("ftime", AuthType::FIN, processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let file = trim!(&args[0]);
            let path = Path::new(file);
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
    pub(crate) fn find_occurence(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let match_expr = &args[0];
            let source = &args[1];

            if match_expr.is_empty() {
                return Err(RadError::InvalidArgument(
                    "Regex expression cannot be an empty string".to_string(),
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
    pub(crate) fn find_multiple_occurence(
        args: &str,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let match_expr = &args[0];
            let source = &args[1];

            if match_expr.is_empty() {
                return Err(RadError::InvalidArgument(
                    "Regex expression cannot be an empty string".to_string(),
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
    pub(crate) fn eval(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    /// $evalk(expression)
    pub(crate) fn eval_keep(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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

    /// Evaluate given expression but force floating point
    ///
    /// This returns true, false or evaluated number
    ///
    /// # Usage
    ///
    /// $eval(expression)
    pub(crate) fn evalf(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let formula = args[0]
                .split_whitespace()
                .map(|item| {
                    let mut new_str = item.to_string();
                    if item.parse::<isize>().is_ok() {
                        new_str.push_str(".0");
                    }
                    new_str
                })
                .collect::<Vec<_>>()
                .join(" ");
            let result = evalexpr::eval(&formula)?;
            Ok(Some(result.to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "Eval requires an argument".to_owned(),
            ))
        }
    }

    /// Evaluate given expression but keep original expression and force float expression
    ///
    /// This returns true, false or evaluated number
    ///
    /// # Usage
    ///
    /// $evalkf(expression)
    pub(crate) fn eval_keep_as_float(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            // This is the processed raw formula
            let formula = args[0]
                .split_whitespace()
                .map(|item| {
                    let mut new_str = item.to_string();
                    if item.parse::<isize>().is_ok() {
                        new_str.push_str(".0");
                    }
                    new_str
                })
                .collect::<Vec<_>>()
                .join(" ");
            let result = format!("{}= {}", formula, evalexpr::eval(&formula)?);
            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "Evalkf requires an argument".to_owned(),
            ))
        }
    }

    /// Pipe in replace evaluation
    ///
    /// # Usage
    ///
    /// $pie(expression)
    pub(crate) fn pipe_ire(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(mut args) = ArgParser::new().args_with_len(args, 1) {
            let mut formula = std::mem::take(&mut args[0]);
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
        } else {
            Err(RadError::InvalidArgument(
                "pie requires an argument".to_owned(),
            ))
        }
    }

    /// Macro in replace evaluation
    ///
    /// # Usage
    ///
    /// $mie(expression)
    pub(crate) fn macro_ire(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(mut args) = ArgParser::new().args_with_len(args, 2) {
            let macro_name = std::mem::take(&mut args[0]);
            let mut formula = std::mem::take(&mut args[1]);
            let body = p.get_runtime_macro_body(&macro_name)?;
            let replaced = if formula.contains('m') {
                formula.replace('p', body)
            } else {
                formula.insert_str(0, body);
                formula
            };
            let result = evalexpr::eval(&replaced)?;
            p.replace_macro(&macro_name, &result.to_string());
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "mie requires two arguments".to_owned(),
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
    pub(crate) fn not(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn trim(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            Ok(Some(trim!(&args[0]).to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "Trim requires an argument".to_owned(),
            ))
        }
    }

    /// Trim preceding whitespaces (' ', '\n', '\t', '\r')
    ///
    /// # Usage
    ///
    /// $trimf(expression)
    pub(crate) fn trimf(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            Ok(Some(args[0].trim_start().to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "Trimf requires an argument".to_owned(),
            ))
        }
    }

    /// Trim trailing whitespaces (' ', '\n', '\t', '\r')
    ///
    /// # Usage
    ///
    /// $trimr(expression)
    pub(crate) fn trimr(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let content = &args[0];
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
        } else {
            Err(RadError::InvalidArgument(
                "Trimr requires an argument".to_owned(),
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
    pub(crate) fn indent_lines(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let indenter = &args[0];
            let mut lines = String::new();
            let mut iter = args[1].lines().peekable();
            while let Some(line) = iter.next() {
                if !line.is_empty() {
                    lines.push_str(indenter);
                    lines.push_str(line);
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
    pub(crate) fn triml(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let mut lines = String::new();
            let mut iter = args[0].lines().peekable();
            while let Some(line) = iter.next() {
                lines.push_str(trim!(line));
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
    pub(crate) fn trimla(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let option = trim!(&args[0]);
            let source = &args[1];
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
    pub(crate) fn chomp(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let source = &args[0];
            let chomp_result =
                &*TWO_NL_MATCH.replace_all(source, &processor.state.newline.repeat(2));

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
    pub(crate) fn compress(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn lipsum_words(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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

    /// Creates placeholder with given amount of word counts but for repeated purposes.
    ///
    /// # Usage
    ///
    /// $lipsumr(Number)
    pub(crate) fn lipsum_repeat(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let word_count = &args[0];
            let mut current_index = match p.get_runtime_macro_body(MACRO_SPECIAL_LIPSUM) {
                Ok(value) => value.parse::<usize>().unwrap(),
                Err(_) => {
                    p.add_static_rules(&[(MACRO_SPECIAL_LIPSUM, "0")])?;
                    0usize
                }
            };

            if let Ok(count) = trim!(word_count).parse::<usize>() {
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
            } else {
                Err(RadError::InvalidArgument(format!("Lipsumr needs a number bigger or equal to 0 (unsigned integer) but given \"{}\"", word_count)))
            }
        } else {
            Err(RadError::InvalidArgument(
                "Lipsum requires an argument".to_owned(),
            ))
        }
    }

    /// Repeat given expression about given amount times
    ///
    /// # Usage
    ///
    /// $repeat(count,text)
    pub(crate) fn repeat(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn syscmd(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
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
                Command::new(arg_vec[0])
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
    pub(crate) fn undefine_call(
        args: &str,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let name = trim!(&args[0]);

            if processor.contains_macro(name, MacroType::Any) {
                processor.undefine_macro(name, MacroType::Any);
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
    pub(crate) fn define_type(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(None)
    }

    /// Squash
    ///
    /// # Usage
    ///
    /// $squash(/,a/b/c)
    pub(crate) fn squash(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn split(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let sep = &args[0];
            let text = &args[1];

            let mut result = text
                .split_terminator(sep)
                .fold(String::new(), |mut acc, v| {
                    acc.push_str(v);
                    acc.push(',');
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

    /// Split by whitespaces and cut
    ///
    /// # Usage
    ///
    /// $scut(0,a/b/c)
    pub(crate) fn split_whitespace_and_cut(
        args: &str,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let split = &mut args[1].split_whitespace();
            let len = split.clone().count();

            let index = trim!(&args[0]).parse::<isize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "scut requires an index to be a integer type but got \"{}\"",
                    &args[0]
                ))
            })?;

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
        } else {
            Err(RadError::InvalidArgument(
                "scut requires two arguments".to_owned(),
            ))
        }
    }

    /// Split and cut
    ///
    /// # Usage
    ///
    /// $cut(/,a/b/c)
    pub(crate) fn split_and_cut(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let sep = &args[0];
            let split = &mut args[2].split_terminator(sep);
            let len = split.clone().count();

            let index = trim!(&args[1]).parse::<isize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "cut requires an index to be a integer type but got \"{}\"",
                    &args[0]
                ))
            })?;

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
        } else {
            Err(RadError::InvalidArgument(
                "cut requires three arguments".to_owned(),
            ))
        }
    }

    /// Split whitespaces
    ///
    /// # Usage
    ///
    /// $ssplit(a/b/c)
    pub(crate) fn space_split(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let text = trim!(&args[0]);

            let mut result = text.split_whitespace().fold(String::new(), |mut acc, v| {
                acc.push_str(v);
                acc.push(',');
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

    /// Assert trimmed
    ///
    /// # Usage
    ///
    /// $assertt( abc ,abc)
    pub(crate) fn assert_trimmed(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            if trim!(args[0]) == trim!(args[1]) {
                p.track_assertion(true)?;
                Ok(None)
            } else {
                p.track_assertion(false)?;
                Err(RadError::AssertFail)
            }
        } else {
            Err(RadError::InvalidArgument(
                "Assertt requires two arguments".to_owned(),
            ))
        }
    }

    /// Assert
    ///
    /// # Usage
    ///
    /// $assert(abc,abc)
    pub(crate) fn assert(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn assert_ne(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn change_counter(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_to_vec(args, ',', SplitVariant::Always);
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
        if !p.contains_macro(counter_name, MacroType::Runtime) {
            p.add_static_rules(&[(&counter_name, "0")])?;
        }
        let body = p
            .get_runtime_macro_body(counter_name)?
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
    pub(crate) fn join(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let sep = &args[0];
            let text = &args[1];
            let join = text.split(',').fold(String::new(), |mut acc, s| {
                acc.push_str(s);
                acc.push_str(sep);
                acc
            });
            Ok(join.strip_suffix(sep).map(|s| s.to_owned()))
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
    /// $joinl(" ",a\nb\nc\n)
    pub(crate) fn join_lines(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let sep = &args[0];
            let text = &args[1];
            let join = text.lines().fold(String::new(), |mut acc, s| {
                acc.push_str(s);
                acc.push_str(sep);
                acc
            });
            Ok(join.strip_suffix(sep).map(|s| s.to_owned()))
        } else {
            Err(RadError::InvalidArgument(
                "joinl requires two arguments".to_owned(),
            ))
        }
    }

    /// Join lines
    ///
    /// # Usage
    ///
    /// $joinl(" ",text)
    pub(crate) fn joinl(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let sep = &args[0];
            let text = &args[1];
            let joined = text.lines().fold(String::new(), |mut acc, l| {
                acc.push_str(l);
                acc.push_str(sep);
                acc
            });
            Ok(joined.strip_suffix(sep).map(|s| s.to_owned()))
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
    pub(crate) fn table(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn pipe(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn pipe_to(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            processor
                .state
                .add_pipe(Some(trim!(&args[0])), args[1].to_owned());
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
    pub(crate) fn get_env(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("env", AuthType::ENV, p)? {
            return Ok(None);
        }
        if let Ok(out) = std::env::var(trim!(args)) {
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
    pub(crate) fn set_env(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("envset", AuthType::ENV, p)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let name = trim!(&args[0]);
            let value = &args[1];

            if p.state.behaviour == ErrorBehaviour::Strict && std::env::var(name).is_ok() {
                return Err(RadError::InvalidArgument(format!(
                    "You cannot override environment variable in strict mode. Failed to set \"{}\"",
                    name
                )));
            }

            std::env::set_var(name, value);
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Envset requires two arguments".to_owned(),
            ))
        }
    }

    /// Trigger panic
    pub(crate) fn manual_panic(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        p.state.behaviour = ErrorBehaviour::Interrupt;
        Err(RadError::ManualPanic(args.to_string()))
    }

    /// Escape processing
    pub(crate) fn escape(_: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        processor.state.flow_control = FlowControl::Escape;
        Ok(None)
    }

    /// Exit processing
    pub(crate) fn exit(_: &str, processor: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn merge_path(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        let vec = ArgParser::new().args_to_vec(args, ',', SplitVariant::Always);

        let out = vec
            .iter()
            .map(|s| trim!(PATH_MATCH.replace_all(s, PATH_SEPARATOR)).to_string())
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
    pub(crate) fn print_tab(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        let count = if !args.is_empty() {
            trim!(args)
                .parse::<usize>()
                .map_err(|_| RadError::InvalidArgument("tab requires number".to_string()))?
        } else {
            1
        };

        Ok(Some("\t".repeat(count)))
    }

    /// Print a literal percent
    ///
    /// # Usage
    ///
    /// $percent()
    pub(crate) fn print_percent(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some('%'.to_string()))
    }

    /// Print a literal comma
    ///
    /// # Usage
    ///
    /// $comma()
    pub(crate) fn print_comma(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(','.to_string()))
    }

    /// Yield spaces
    ///
    /// # Usage
    ///
    /// $space()
    pub(crate) fn space(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        let count = if !args.is_empty() {
            trim!(args)
                .parse::<usize>()
                .map_err(|_| RadError::InvalidArgument("space requires number".to_string()))?
        } else {
            1
        };

        Ok(Some(" ".repeat(count)))
    }

    /// Path separator
    ///
    /// # Usage
    ///
    /// $PS()
    pub(crate) fn path_separator(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(PATH_SEPARATOR.to_string()))
    }

    /// Print nothing
    pub(crate) fn print_empty(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(None)
    }

    /// Yield newline according to platform or user option
    ///
    /// # Usage
    ///
    /// $nl()
    pub(crate) fn newline(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn deny_newline(_: &str, p: &mut Processor) -> RadResult<Option<String>> {
        p.state.deny_newline = true;
        Ok(None)
    }

    /// escape new line
    ///
    /// # Usage
    ///
    /// $enl()
    pub(crate) fn escape_newline(_: &str, p: &mut Processor) -> RadResult<Option<String>> {
        p.state.escape_newline = true;
        Ok(None)
    }

    /// Get name from given path
    ///
    /// # Usage
    ///
    /// $name(path/file.exe)
    pub(crate) fn get_name(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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

    /// Check if file exists
    ///
    /// # Usage
    ///
    /// $exist(../canonic_path.txt)
    pub(crate) fn file_exists(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("exist", AuthType::FIN, p)? {
            return Ok(None);
        }

        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let boolean = Path::new(trim!(&args[0])).exists();
            Ok(Some(boolean.to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "Exist requires an argument".to_owned(),
            ))
        }
    }

    /// Get absolute path from given path
    ///
    /// # Usage
    ///
    /// $abs(../canonic_path.txt)
    pub(crate) fn absolute_path(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("abs", AuthType::FIN, p)? {
            return Ok(None);
        }

        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let path = std::fs::canonicalize(p.get_current_dir()?.join(trim!(&args[0])))?
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
    pub(crate) fn get_parent(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn get_pipe(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        let pipe = if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let name = trim!(&args[0]);
            if name.is_empty() {
                let out = processor.state.get_pipe("-", false);

                if out.is_none() {
                    processor.log_warning("Empty pipe", WarningType::Sanity)?;
                }

                out
            } else if let Some(pipe) = processor.state.get_pipe(&args[0], false) {
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
            let out = processor.state.get_pipe("-", false).unwrap_or_default();
            Some(out)
        };
        Ok(pipe)
    }

    /// Print left parenthesis
    ///
    /// # Usage
    ///
    /// $lp()
    pub(crate) fn left_parenthesis(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some("(".to_string()))
    }

    /// Print right parenthesis
    ///
    /// # Usage
    ///
    /// $rp()
    pub(crate) fn right_parenthesis(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(")".to_string()))
    }

    /// Rotate lines which is separated by pattern
    ///
    /// # Usage
    ///
    /// $rotatel(//,left,Content)
    pub(crate) fn rotatel(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        use std::fmt::Write;
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let pattern = &args[0];
            let orientation = AlignType::from_str(trim!(&args[1]))?;
            let source = &args[2];

            let mut result = String::new();
            let mut extracted = String::new();
            // Leading blank spaces from the line itself.
            let mut line_preceding_blank;
            for line in Utils::full_lines(source.as_bytes()) {
                let line = line?;
                if let Some((leading, following)) = line.split_once(pattern) {
                    // Don't "rotate" for pattern starting line
                    if leading.trim().is_empty() {
                        write!(result, "{line}{}", p.state.newline)?;
                        continue;
                    }

                    extracted.clear();
                    let leader_pattern = if orientation == AlignType::Center {
                        ""
                    } else {
                        pattern
                    };
                    write!(extracted, "{leader_pattern}{following}")?;
                    line_preceding_blank = LSPA
                        .find(leading)
                        .map(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();
                    match orientation {
                        AlignType::Left => {
                            write!(
                                result,
                                "{}{}{}{}",
                                line_preceding_blank, extracted, leading, p.state.newline,
                            )?;
                        }
                        AlignType::Right => {
                            write!(
                                result,
                                "{}{}{}{}",
                                leading, p.state.newline, line_preceding_blank, extracted
                            )?;
                        }
                        AlignType::Center => {
                            let mut leading = leading;
                            if !line_preceding_blank.is_empty() {
                                leading = leading.trim_start();
                            }
                            let line_end = if extracted.ends_with("\r\n") {
                                extracted.pop();
                                extracted.pop();
                                "\r\n"
                            } else if extracted.ends_with('\n') {
                                extracted.pop();
                                "\n"
                            } else {
                                ""
                            };

                            let extracted_spl = LSPA
                                .captures(&extracted)
                                .map(|s| s.get(0).unwrap().as_str())
                                .unwrap_or("");
                            let extracted_spf = FSPA
                                .captures(&extracted)
                                .map(|s| s.get(0).unwrap().as_str())
                                .unwrap_or("");

                            let extracted =
                                format!("{}{}{}", extracted_spf, trim!(extracted), extracted_spl);

                            let leading_spl = LSPA
                                .captures(leading)
                                .map(|s| s.get(0).unwrap().as_str())
                                .unwrap_or("");
                            let laeding_spf = FSPA
                                .captures(leading)
                                .map(|s| s.get(0).unwrap().as_str())
                                .unwrap_or("");

                            let leading =
                                format!("{}{}{}", laeding_spf, trim!(leading), leading_spl);

                            write!(
                                result,
                                "{}{}{}{}{}",
                                line_preceding_blank, extracted, pattern, leading, line_end
                            )?;
                        }
                    }
                } else {
                    write!(result, "{line}")?;
                }
            }

            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "Rotatel requires three arguments".to_owned(),
            ))
        }
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
    pub(crate) fn len(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(args.chars().count().to_string()))
    }

    /// Return a unicode length of the string
    ///
    /// # Usage
    ///
    /// $len(안녕하세요)
    /// $len(Hello)
    pub(crate) fn unicode_len(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(
            unicode_width::UnicodeWidthStr::width(args).to_string(),
        ))
    }

    /// Rename macro rule to other name
    ///
    /// # Usage
    ///
    /// $rename(name,target)
    pub(crate) fn rename_call(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
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
    /// $pad(center,10,a,Content)
    pub(crate) fn pad_string(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 4) {
            let align_type = AlignType::from_str(trim!(&args[0]))?;
            let width = trim!(&args[1]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "pad requires positive integer number as width but got \"{}\"",
                    &args[1]
                ))
            })?;
            let filler: &str = trim!(&args[2]);
            let text = &args[3];
            let text_length = text.chars().count();
            if width < text_length {
                return Err(RadError::InvalidArgument(
                    "Width should be bigger than source texts".to_string(),
                ));
            }

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
        } else {
            Err(RadError::InvalidArgument(
                "pad requires four arguments".to_owned(),
            ))
        }
    }

    /// Ailgn texts by separator
    ///
    /// # Usage
    ///
    /// $align(%, contents to align)
    pub(crate) fn align_by_separator(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        use std::fmt::Write;
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let separator = args[0].to_string();
            let contents = args[1].lines();
            let mut max_length = 0usize;
            let mut result = String::new();
            let nl = &p.state.newline;

            let tab_width: usize = std::env::var("RAD_TAB_WIDTH")
                .unwrap_or("4".to_string())
                .parse::<usize>()
                .map_err(|_| {
                    RadError::InvalidCommandOption("RAD_TAB_WIDTH is not a valid value".to_string())
                })?;

            for line in contents.clone() {
                let mut splitted = line.split(&separator);
                let leading = splitted.next().unwrap();
                let width = UnicodeWidthStr::width(leading)
                    + leading.matches('\t').count() * (tab_width - 1);
                if leading != line {
                    max_length = max_length.max(width);
                }
            }
            for line in contents {
                let splitted = line.split_once(&separator);
                if splitted.is_some() {
                    let (leading, following) = splitted.unwrap();
                    let width = UnicodeWidthStr::width(leading)
                        + leading.matches('\t').count() * (tab_width - 1);

                    // found matching line
                    write!(
                        result,
                        "{}{}{}{}{}",
                        leading,
                        " ".repeat(max_length - width),
                        separator,
                        following,
                        nl
                    )?;
                } else {
                    write!(result, "{}{}", line, nl)?;
                }
            }
            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "Align requires two arguments".to_owned(),
            ))
        }
    }

    /// Ailgn columns
    ///
    /// # Usage
    ///
    /// $alignc(%, contents to align)
    pub(crate) fn align_columns(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let contents = &args[0];
            let data = dcsv::Reader::new()
                .trim(true)
                .use_space_delimiter(true)
                .data_from_stream(contents.as_bytes())?;

            Ok(Some(data.pretty_table(&p.state.newline)))
        } else {
            Err(RadError::InvalidArgument(
                "Alignc requires arguments".to_owned(),
            ))
        }
    }

    /// Ailgn texts by rules
    ///
    /// # Usage
    ///
    /// $alignby(rules, contents to align)
    pub(crate) fn align_by_rules(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        use itertools::Itertools;
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let rules = trim!(args[0]).chars().collect::<Vec<_>>();

            if rules.len() % 2 != 0 {
                return Err(RadError::InvalidCommandOption(
                    "alignby needs specific syntax for rules".to_string(),
                ));
            }

            let mut contents = args[1].lines().map(|s| s.to_owned()).collect::<Vec<_>>();

            let tab_width: usize = std::env::var("RAD_TAB_WIDTH")
                .unwrap_or("4".to_string())
                .parse::<usize>()
                .map_err(|_| {
                    RadError::InvalidCommandOption("RAD_TAB_WIDTH is not a valid value".to_string())
                })?;

            let mut iter = rules.iter();
            while let (Some(count), Some(separator)) = (iter.next(), iter.next()) {
                let count = count.to_digit(10).ok_or_else(|| {
                    RadError::InvalidArgument(format!(
                        "Could not convert given value \"{}\" into a number",
                        args[0]
                    ))
                })?;
                align_step(&mut contents, *separator, count as usize, tab_width)?;
            }

            #[inline]
            fn align_step(
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

                        *line =
                            format!("{}{}{}", leading, " ".repeat(max_length - width), following,);
                    }
                }
                Ok(())
            }

            let result = contents.join(&p.state.newline);

            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "Alignby requires two arguments".to_owned(),
            ))
        }
    }

    /// Apart texts by separator
    ///
    /// # Usage
    ///
    /// $apart(%, contents %to% align)
    pub(crate) fn apart_by_separator(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let separator = args[0].to_string();
            let contents = &args[1];
            let result = contents
                .split_inclusive(&separator)
                .fold(String::new(), |mut acc, a| {
                    acc.push_str(trim!(a));
                    acc.push_str(&p.state.newline);
                    acc
                });
            Ok(Some(
                result
                    .strip_suffix(&p.state.newline)
                    .map(|s| s.to_string())
                    .unwrap_or(result),
            ))
        } else {
            Err(RadError::InvalidArgument(
                "Apart requires two arguments".to_owned(),
            ))
        }
    }

    /// Translate given char aray into corresponding char array
    ///
    /// # Usage
    ///
    /// $tr(abc,ABC,Source)
    pub(crate) fn translate(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3) {
            let mut source = args[2].clone();
            let target = args[0].chars();
            let destination = args[1].chars();

            if target.clone().count() != destination.clone().count() {
                return Err(RadError::InvalidArgument(format!("Tr's replacment should have same length of texts while given \"{:?}\" and \"{:?}\"", target, destination)));
            }

            let iter = target.zip(destination);

            for (t, d) in iter {
                source = source.replace(t, d.to_string().as_str());
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
    pub(crate) fn substring(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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

    /// Get a substring(indexed) until a pattern
    ///
    /// # Usage
    ///
    /// $until(pattern,Content)
    pub(crate) fn get_slice_until(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let pattern = &args[0];

            if pattern.is_empty() {
                return Err(RadError::InvalidArgument(
                    "Empty value is not allowed in until".to_owned(),
                ));
            }
            let source = &args[1];

            let index = source.find(pattern);
            if let Some(index) = index {
                Ok(Some(source[0..index].to_owned()))
            } else {
                Ok(Some(source.to_owned()))
            }
        } else {
            Err(RadError::InvalidArgument(
                "until requires two arguments".to_owned(),
            ))
        }
    }

    /// Get a substring(indexed) after a pattern
    ///
    /// # Usage
    ///
    /// $after(pattern,Content)
    pub(crate) fn get_slice_after(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let pattern = &args[0];
            let offset = pattern.len();

            if pattern.is_empty() {
                return Err(RadError::InvalidArgument(
                    "Empty value is not allowed in after".to_owned(),
                ));
            }
            let source = &args[1];

            let index = source.find(pattern);
            if let Some(index) = index {
                Ok(Some(source[index + offset..].to_owned()))
            } else {
                Ok(Some(source.to_owned()))
            }
        } else {
            Err(RadError::InvalidArgument(
                "after requires two arguments".to_owned(),
            ))
        }
    }

    /// Save content to temporary file
    ///
    /// # Usage
    ///
    /// $tempout(Content)
    pub(crate) fn temp_out(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn file_out(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
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
                let path = std::env::current_dir()?.join(file_name);
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
    pub(crate) fn head(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn head_line(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn tail(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    /// $surr(<p>,</p>,content)
    pub(crate) fn surround_with_pair(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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

    /// Squeeze a line
    ///
    /// # Usage
    ///
    /// $squz(a b c d e)
    pub(crate) fn squeeze_line(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(mut args) = ArgParser::new().args_with_len(args, 1) {
            let mut content = stake!(args[0]);
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
        } else {
            Err(RadError::InvalidArgument(
                "Squeeze requires an argument".to_owned(),
            ))
        }
    }

    /// Get tail of given text but for lines
    ///
    /// # Usage
    ///
    /// $taill(2,Text To extract)
    pub(crate) fn tail_line(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn sort_array(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn sort_lines(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
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

    /// Sort lists ( chunks )
    ///
    /// # Usage
    ///
    /// $sortc(asec, ... chunk ... )
    pub(crate) fn sort_chunk(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(mut args) = ArgParser::new().args_with_len(args, 2) {
            let order_type = trim!(stake!(args[0])).to_string();
            let mut content = stake!(args[1]);
            let line_end = if content.ends_with("\r\n") {
                content.pop();
                content.pop();
                "\r\n"
            } else if content.ends_with('\n') {
                content.pop();
                "\n"
            } else {
                content.push_str(&p.state.newline);
                ""
            };
            // --- Chunk creation
            let mut clogged_chunk_list = vec![];
            let mut container = String::new();
            for line in Utils::full_lines(content.as_bytes()) {
                let line = line?;
                // Has empty leading + has parent
                if LSPA.captures(&line).is_some() && !container.is_empty() {
                    container.push_str(&line);
                } else {
                    clogged_chunk_list.push(container);
                    container = line;
                }
            }
            clogged_chunk_list.push(container);
            // ---
            match order_type.to_lowercase().as_str() {
                "asec" => clogged_chunk_list.sort_unstable(),
                "desc" => {
                    clogged_chunk_list.sort_unstable();
                    clogged_chunk_list.reverse()
                }
                _ => {
                    return Err(RadError::InvalidArgument(format!(
                        "Sortc requires either asec or desc but given \"{}\"",
                        order_type
                    )))
                }
            }

            Ok(Some(clogged_chunk_list.join("") + line_end))
        } else {
            Err(RadError::InvalidArgument(
                "sortc requires two arguments".to_owned(),
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
    pub(crate) fn index_array(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            // Don't allocate as vector if possible to improve performance
            let content = &mut args[1].split(',');
            let index = trim!(&args[0]).parse::<isize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "index requires to be an integer but got \"{}\"",
                    &args[0]
                ))
            })?;

            let len = args[1].split(',').count();

            if index >= len as isize || index < -(len as isize) {
                return Err(RadError::InvalidArgument(format!(
                    "Index out of range. Given index is \"{}\" but array length is \"{}\"",
                    index,
                    content.count()
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
            // Safe to unwrap because bound check was already done
            Ok(Some(content.nth(final_index).unwrap().to_owned()))
        } else {
            Err(RadError::InvalidArgument(
                "index requires two arguments".to_owned(),
            ))
        }
    }

    /// Index lines
    ///
    /// # Usage
    ///
    /// $indexl(1,1$nl()2$nl())
    pub(crate) fn index_lines(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let content = &mut args[1].lines();
            let index = trim!(&args[0]).parse::<isize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "indexl requires to be an integer but got \"{}\"",
                    &args[0]
                ))
            })?;

            let len = args[1].lines().count();

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
        } else {
            Err(RadError::InvalidArgument(
                "indexl requires two arguments".to_owned(),
            ))
        }
    }

    /// Strip content
    ///
    /// # Usage
    ///
    /// $strip()
    pub(crate) fn strip(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(mut args) = ArgParser::new().args_with_len(args, 2) {
            let count = trim!(&args[0]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a number",
                    args[0]
                ))
            })?;
            let content = &args[1];

            if count == 0 {
                return Ok(Some(std::mem::take(&mut args[1])));
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
        } else {
            Err(RadError::InvalidArgument(
                "strip requires two arguments".to_owned(),
            ))
        }
    }

    /// Strip front
    ///
    /// # Usage
    ///
    /// $stripf()
    pub(crate) fn stripf(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(mut args) = ArgParser::new().args_with_len(args, 2) {
            let count = trim!(&args[0]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a number",
                    args[0]
                ))
            })?;
            let content = &args[1];

            if count == 0 {
                return Ok(Some(std::mem::take(&mut args[1])));
            }

            let char_count = content.chars().count();

            if count > char_count {
                return Err(RadError::InvalidArgument(
                    "Cannot stripf because given content's length is shorter".to_owned(),
                ));
            }

            Ok(Some(content[count..].to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "stripf requires two arguments".to_owned(),
            ))
        }
    }

    /// Strip front lines
    ///
    /// # Usage
    ///
    /// $stripfl()
    pub(crate) fn stripf_line(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(mut args) = ArgParser::new().args_with_len(args, 2) {
            let count = trim!(&args[0]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a number",
                    args[0]
                ))
            })?;
            let content = &args[1];

            if count == 0 {
                return Ok(Some(std::mem::take(&mut args[1])));
            }

            let lines = content.lines().collect::<Vec<_>>();
            let line_count = lines.len();

            if count > line_count {
                return Err(RadError::InvalidArgument(
                    "Cannot stripfl because given content's length is shorter".to_owned(),
                ));
            }

            let result = lines[count..].iter().fold(String::new(), |mut acc, a| {
                acc.push_str(a);
                acc.push_str(&p.state.newline);
                acc
            });

            Ok(Some(
                result
                    .strip_suffix(&p.state.newline)
                    .map(|s| s.to_string())
                    .unwrap_or(result),
            ))
        } else {
            Err(RadError::InvalidArgument(
                "stripfl requires two arguments".to_owned(),
            ))
        }
    }

    /// Strip rear lines
    ///
    /// # Usage
    ///
    /// $striprl()
    pub(crate) fn stripr_line(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(mut args) = ArgParser::new().args_with_len(args, 2) {
            let count = trim!(&args[0]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a number",
                    args[0]
                ))
            })?;
            let content = &args[1];

            if count == 0 {
                return Ok(Some(std::mem::take(&mut args[1])));
            }

            let lines = content.lines().collect::<Vec<_>>();
            let line_count = lines.len();

            if count > line_count {
                return Err(RadError::InvalidArgument(
                    "Cannot striprl because given content's length is shorter".to_owned(),
                ));
            }

            let result =
                lines[0..line_count - count - 1]
                    .iter()
                    .fold(String::new(), |mut acc, a| {
                        acc.push_str(a);
                        acc.push_str(&p.state.newline);
                        acc
                    });

            Ok(Some(
                result
                    .strip_suffix(&p.state.newline)
                    .map(|s| s.to_string())
                    .unwrap_or(result),
            ))
        } else {
            Err(RadError::InvalidArgument(
                "striprl requires two arguments".to_owned(),
            ))
        }
    }

    /// Strip rear
    ///
    /// # Usage
    ///
    /// $stripr()
    pub(crate) fn stripr(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(mut args) = ArgParser::new().args_with_len(args, 2) {
            let count = trim!(&args[0]).parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert given value \"{}\" into a number",
                    args[0]
                ))
            })?;
            let content = &args[1];

            if count == 0 {
                return Ok(Some(std::mem::take(&mut args[1])));
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
        } else {
            Err(RadError::InvalidArgument(
                "stripr requires two arguments".to_owned(),
            ))
        }
    }

    /// Strip rear with pattern
    ///
    /// # Usage
    ///
    /// $striper()
    pub(crate) fn strip_expression_from_rear(
        args: &str,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let expr = &args[0];
            let content = &args[1];
            let nl = p.state.newline.clone();
            let reg = p.try_get_or_insert_regex(expr)?;

            let mut acc = String::new();
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

            Ok(Some(acc))
        } else {
            Err(RadError::InvalidArgument(
                "stripr requires two arguments".to_owned(),
            ))
        }
    }

    /// Separate content
    ///
    /// # Usage
    ///
    /// $sep(1$nl()2$nl())
    pub(crate) fn separate(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let content = &args[0];
            let mut separated = vec![];
            let mut iter = content.lines().peekable();
            while let Some(line) = iter.next() {
                separated.push(line);
                if !line.is_empty() && !iter.peek().unwrap_or(&"0").is_empty() {
                    separated.push("");
                }
            }
            Ok(Some(separated.join(&p.state.newline)))
        } else {
            Err(RadError::InvalidArgument(
                "sep requires an argument".to_owned(),
            ))
        }
    }

    /// Get a sliced array
    ///
    /// # Usage
    ///
    /// $slice(1,2,1,2,3,4,5)
    pub(crate) fn slice(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn fold(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let content = args[0].split(',').fold(String::new(), |mut acc, a| {
                acc.push_str(a);
                acc
            });
            Ok(Some(content))
        } else {
            Err(RadError::InvalidArgument(
                "fold requires an argument".to_owned(),
            ))
        }
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
    pub(crate) fn fold_line(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let content = args[0].lines().fold(String::new(), |mut acc, a| {
                acc.push_str(a);
                acc
            });

            Ok(Some(content))
        } else {
            Err(RadError::InvalidArgument(
                "foldl requires an argument".to_owned(),
            ))
        }
    }

    /// Fold lines by separator
    ///
    /// This folds empty lines
    ///
    /// # Usage
    ///
    /// $foldby( ,1
    /// 1
    /// 2
    /// 3
    /// 4
    /// 5)
    pub(crate) fn fold_by(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let sep = &args[0];
            let content = args[1].lines().fold(String::new(), |mut acc, a| {
                acc.push_str(a);
                acc.push_str(sep);
                acc
            });

            Ok(Some(content))
        } else {
            Err(RadError::InvalidArgument(
                "foldby requires two arguments".to_owned(),
            ))
        }
    }

    /// Fold lines as trimmed
    ///
    /// # Usage
    ///
    /// $foldt(1
    /// 1
    /// 2
    /// 3
    /// 4
    /// 5)
    pub(crate) fn foldt(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let content = args[0].lines().fold(String::new(), |mut acc, a| {
                acc.push_str(trim!(a));
                acc
            });

            Ok(Some(content))
        } else {
            Err(RadError::InvalidArgument(
                "foldt requires an argument".to_owned(),
            ))
        }
    }

    /// Fold lines by regular expressions
    ///
    /// # Usage
    ///
    /// $foldreg(expr,1
    /// 2)
    pub(crate) fn fold_regular_expr(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let mut container = String::new();
            let mut folded = String::new();
            let nl = p.state.newline.to_owned();
            let reg = p.try_get_or_insert_regex(&args[0])?;
            for line in Utils::full_lines(args[1].as_bytes()) {
                let line = line?;
                // Start new container
                if reg.find(&line).is_some() {
                    folded.push_str(std::mem::take(&mut container.as_str()));
                    if !container.is_empty() {
                        folded.push_str(&nl);
                    }
                    container.clear();
                    container.push_str(line.trim_end());
                } else if !container.is_empty() {
                    container.push_str(trim!(line));
                } else {
                    folded.push_str(&line);
                }
            }
            folded.push_str(&container);

            Ok(Some(folded))
        } else {
            Err(RadError::InvalidArgument(
                "foldreg requires two argument".to_owned(),
            ))
        }
    }

    /// Get os type
    ///
    /// # Usage
    ///
    /// $ostype()
    pub(crate) fn get_os_type(_: &str, _: &mut Processor) -> RadResult<Option<String>> {
        #[cfg(windows)]
        return Ok(Some("windows".to_owned()));
        #[cfg(not(windows))]
        return Ok(Some("unix".to_owned()));
    }

    /// Register expressino
    ///
    /// # Usage
    ///
    /// $regexpr(name,EXPR)
    pub(crate) fn register_expression(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
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

    /// Capture expressions
    ///
    /// # Usage
    ///
    /// $capture(expr,Array)
    pub(crate) fn capture(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let expr = &args[0];
            let nl = p.state.newline.clone();
            let reg = p.try_get_or_insert_regex(expr)?;
            let acc = reg
                .captures_iter(&args[1])
                .fold(String::new(), |mut acc, x| {
                    acc.push_str(x.get(0).map_or("", |s| s.as_str()));
                    acc.push_str(&nl);
                    acc
                });
            Ok(acc.strip_suffix(&nl).map(|s| s.to_owned()))
        } else {
            Err(RadError::InvalidArgument(
                "capture requires two arguments".to_owned(),
            ))
        }
    }

    /// Grep items from array
    ///
    /// # Usage
    ///
    /// $grep(expr,Array)
    pub(crate) fn grep_array(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let expr = &args[0];
            let reg = p.try_get_or_insert_regex(expr)?;
            let mut grepped =
                args[1]
                    .split(',')
                    .filter(|l| reg.is_match(l))
                    .fold(String::new(), |mut acc, x| {
                        acc.push_str(x);
                        acc.push(',');
                        acc
                    });
            grepped.pop();
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
    /// $grepl(expr,Lines)
    pub(crate) fn grep_lines(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let expr = &args[0];
            let nl = p.state.newline.clone();
            let reg = p.try_get_or_insert_regex(expr)?;
            let content = args[1].lines();
            let grepped = content
                .filter(|l| reg.is_match(l))
                .fold(String::new(), |mut acc, l| {
                    acc.push_str(l);
                    acc.push_str(&nl);
                    acc
                });
            Ok(grepped.strip_suffix(&nl).map(|s| s.to_owned()))
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
    pub(crate) fn grep_file(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("grepf", AuthType::FIN, p)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let file = trim!(&args[1]);
            let path = Path::new(file);

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

    /// Condense
    ///
    /// # Usage
    ///
    /// $cond(a       b         c)
    pub(crate) fn condense(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        use itertools::Itertools;
        if let Some(mut args) = ArgParser::new().args_with_len(args, 1) {
            let content = std::mem::take(&mut args[0]);
            Ok(Some(content.split_whitespace().join(" ")))
        } else {
            Err(RadError::InvalidArgument(
                "cond requires an argument".to_owned(),
            ))
        }
    }

    /// Condense
    ///
    /// # Usage
    ///
    /// $cond(a       b         c)
    pub(crate) fn condense_by_lines(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        use itertools::Itertools;
        use std::fmt::Write;
        if let Some(mut args) = ArgParser::new().args_with_len(args, 1) {
            let content = std::mem::take(&mut args[0]);
            let mut acc = String::new();
            let mut itr = content.lines().peekable();
            while let Some(line) = itr.next() {
                write!(&mut acc, "{}", line.split_whitespace().join(" "),)?;
                if itr.peek().is_some() {
                    write!(&mut acc, "{}", &p.state.newline)?;
                }
            }
            Ok(Some(acc))
        } else {
            Err(RadError::InvalidArgument(
                "condl requires an argument".to_owned(),
            ))
        }
    }

    /// Count
    ///
    /// # Usage
    ///
    /// $count(1,2,3,4,5)
    pub(crate) fn count(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            if trim!(&args[0]).is_empty() {
                return Ok(Some("0".to_string()));
            }
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
    pub(crate) fn count_word(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn count_lines(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            if args[0].is_empty() {
                return Ok(Some("0".to_string()));
            }
            let line_count = args[0].split('\n').count();
            Ok(Some(line_count.to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "countl requires an argument".to_owned(),
            ))
        }
    }

    /// Relay all text into given target
    ///
    /// Every text including non macro calls are all sent to relay target
    ///
    /// # Usage
    ///
    /// $relay(type,argument)
    pub(crate) fn relay(args_src: &str, p: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_to_vec(args_src, ',', SplitVariant::Always);
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
        let relay_type = match raw_type {
            "temp" => {
                if !Utils::is_granted("relay", AuthType::FOUT, p)? {
                    return Ok(None);
                }
                RelayTarget::Temp
            }
            "file" => {
                use crate::common::FileTarget;
                if !Utils::is_granted("relay", AuthType::FOUT, p)? {
                    return Ok(None);
                }
                if args.len() == 1 {
                    return Err(RadError::InvalidArgument(
                        "relay requires second argument as file name for file relaying".to_owned(),
                    ));
                }
                let file_target = FileTarget::from_path(Path::new(&target))?;
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
                    let sim = p.get_similar_macro(&target, true);
                    let adder = if let Some(mac) = sim {
                        format!(" Did you mean {mac}?")
                    } else {
                        String::default()
                    };
                    return Err(RadError::InvalidMacroReference(format!(
                        "Cannot relay to non-exsitent macro or non-runtime macro \"{}\".{adder}",
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

    // This function iterate through lines twice
    // 1. Regex and calculate nested level and corresponding identifier
    // 2. Regex again while replacing specific parts of string
    /// Rearrange
    ///
    /// # Usage
    ///
    /// $rer(
    /// 3.
    /// 2.
    /// 1.
    ///     4]
    ///     7]
    ///     8]
    /// )
    pub(crate) fn rearrange(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let mut rer_hash = RerHash::default();
            let mut blank_str: &str; // Container

            // TODO
            // Should I really collect it for indexing?
            // Can it be improved?
            let mut lines = args[0]
                .lines()
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            // TODO
            // Refactor iteration_cache into a separate struct
            let mut iteration_cache: Vec<(usize, usize)> = Vec::new();
            // Find list elements and save counts of each sorts
            for (ll, line) in lines.iter().enumerate() {
                if let Some(captured) = BLANKHASH_MATCH.captures(line) {
                    blank_str = captured.get(1).map_or("", |m| m.as_str());
                    let index_id = captured.get(2).map_or("", |m| m.as_str());
                    let blank = rer_hash.try_insert(blank_str, index_id)?;
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
                        counter = rer_hash.get_current_count(blank, index);

                        // This means list items go up
                        if blank_cache > blank {
                            // Reset previous cache
                            rer_hash.update_counter(blank_cache, &index_cache, 1);
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
                    rer_hash.update_counter(blank, index, counter + 1);
                    lines[ll] = replaced;
                }
            }
            Ok(Some(lines.join(&p.state.newline)))
        } else {
            Err(RadError::InvalidArgument(
                "rer requires an argument".to_owned(),
            ))
        }
    }

    /// Disable relaying
    ///
    /// # Usage
    ///
    /// $hold()
    pub(crate) fn halt_relay(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let halt_immediate = if args[0].is_empty() {
                false
            } else {
                Utils::is_arg_true(trim!(&args[0]))?
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
    pub(crate) fn set_temp_target(
        args: &str,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("tempto", AuthType::FOUT, processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let path = &std::env::temp_dir().join(trim!(&args[0]));
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
    pub(crate) fn get_temp_path(_: &str, processor: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn get_number(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn capitalize(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let src = trim!(&args[0]);
            Ok(Some(src.to_uppercase()))
        } else {
            Err(RadError::InvalidArgument(
                "upper requires an argument".to_owned(),
            ))
        }
    }

    /// Lower text
    ///
    /// # Usage
    ///
    /// $lower(hello world)
    pub(crate) fn lower(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let src = trim!(&args[0]);
            Ok(Some(src.to_lowercase()))
        } else {
            Err(RadError::InvalidArgument(
                "lower requires an argument".to_owned(),
            ))
        }
    }

    /// Comment
    ///
    /// # Usage
    ///
    /// $comment(any)
    pub(crate) fn require_comment(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(mut args) = ArgParser::new().args_with_len(args, 1) {
            let comment_src = std::mem::take(&mut args[0]);
            let comment_type = CommentType::from_str(trim!(&comment_src));
            if comment_type.is_err() {
                return Err(RadError::InvalidArgument(format!(
                    "Comment requires valid comment type but given \"{}\"",
                    comment_src
                )));
            }

            let comment_type = comment_type?;

            if p.state.comment_type != comment_type {
                return Err(RadError::UnsoundExecution(format!(
                    "Comment type, \"{:#?}\" is required but it is not",
                    comment_type
                )));
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "comment requires an argument".to_owned(),
            ))
        }
    }

    /// require
    ///
    /// # Usage
    ///
    /// $require(fout)
    pub(crate) fn require_permissions(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        let vec = ArgParser::new().args_to_vec(args, ',', SplitVariant::Always);
        if vec.is_empty() {
            p.log_warning(
                "Require macro used without any arguments.",
                WarningType::Sanity,
            )?;
        }
        for auth in vec {
            let auth_type = AuthType::from(&auth).ok_or_else(|| {
                RadError::InvalidArgument(format!(
                    "Require needs valid permission but given \"{}\"",
                    auth
                ))
            })?;
            let state = p.state.auth_flags.get_state(&auth_type);
            if let AuthState::Restricted = state {
                return Err(RadError::UnsoundExecution(format!(
                    "Permission \"{}\" is required but is not.",
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
    pub(crate) fn require_strict(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        let vec = ArgParser::new().args_to_vec(args, ',', SplitVariant::Always);
        let mode = &vec[0];
        let trimmed_mode = trim!(mode);
        match trimmed_mode.to_lowercase().as_str() {
            "lenient" => {
                if p.state.behaviour != ErrorBehaviour::Lenient {
                    return Err(RadError::UnsoundExecution(
                        "Lenient mode is required but it is not".to_owned(),
                    ));
                }
            }
            "purge" => {
                if p.state.behaviour != ErrorBehaviour::Purge {
                    return Err(RadError::UnsoundExecution(
                        "Purge mode is required but it is not".to_owned(),
                    ));
                }
            }
            "" => {
                if p.state.behaviour != ErrorBehaviour::Strict {
                    return Err(RadError::UnsoundExecution(
                        "Strict mode is required but it is not".to_owned(),
                    ));
                }
            }
            _ => {
                return Err(RadError::InvalidArgument(format!(
                    "Received invalid strict mode which is \"{}\"",
                    trimmed_mode
                )));
            }
        }
        Ok(None)
    }

    /// Output
    ///
    /// # Usage
    ///
    /// $Output(fout)
    pub(crate) fn require_output(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        match trim!(args).to_lowercase().as_str() {
            "terminal" => {
                if let WriteOption::Terminal = p.write_option {
                } else {
                    return Err(RadError::UnsoundExecution(
                        "Rad should write to a terminal and yet such flag was not satisfied."
                            .to_owned(),
                    ));
                }
            }
            "file" => {
                if let WriteOption::File(_) = p.write_option {
                } else {
                    return Err(RadError::UnsoundExecution(
                        "Rad should write to a file and yet such flag was not satisfied."
                            .to_owned(),
                    ));
                }
            }
            "discard" => {
                if let WriteOption::Discard = p.write_option {
                } else {
                    return Err(RadError::UnsoundExecution(
                        "Rad should discard output and yet such flag was not satisfied.".to_owned(),
                    ));
                }
            }
            _ => {
                p.log_warning(
                    "No output type was given for the macro",
                    WarningType::Sanity,
                )?;
            }
        }
        Ok(None)
    }

    /// Log message
    ///
    /// # Usage
    ///
    /// $log(This is a problem)
    pub(crate) fn log_message(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        p.log_message(args)?;
        Ok(None)
    }

    /// Log error message
    ///
    /// # Usage
    ///
    /// $loge(This is a problem)
    pub(crate) fn log_error_message(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        p.print_error(args)?;
        Ok(None)
    }

    /// Print message
    ///
    /// # Usage
    ///
    /// $print(This is a problem)
    pub(crate) fn print_message(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        write!(std::io::stdout(), "{}{}", args, p.state.newline)?;
        Ok(None)
    }

    /// Get max value from array
    ///
    /// # Usage
    ///
    /// $max(1,2,3,4,5)
    pub(crate) fn get_max(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
                "max requires an argument".to_owned(),
            ))
        }
    }

    /// Get min value from array
    ///
    /// # Usage
    ///
    /// $min(1,2,3,4,5)
    pub(crate) fn get_min(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
                "min requires an argument".to_owned(),
            ))
        }
    }

    /// Get ceiling value
    ///
    /// # Usage
    ///
    /// $ceiling(1.56)
    pub(crate) fn get_ceiling(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn get_floor(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn prec(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn reverse_array(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if args.is_empty() {
            Err(RadError::InvalidArgument(
                "rev requires an argument".to_owned(),
            ))
        } else {
            let reversed = args.rsplit(',').fold(String::new(), |mut acc, a| {
                acc.push_str(a);
                acc.push(',');
                acc
            });
            Ok(Some(reversed))
        }
    }

    /// Declare an empty macros
    ///
    /// # Usage
    ///
    /// $declare(n1,n2,n3)
    pub(crate) fn declare(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        let names = ArgParser::new().args_to_vec(args, ',', SplitVariant::Always);
        let runtime_rules = names
            .iter()
            .map(|name| (trim!(name).to_string(), "", ""))
            .collect::<Vec<(String, &str, &str)>>();

        // Check overriding. Warn or yield error
        for (name, _, _) in runtime_rules.iter() {
            if processor.contains_macro(name, MacroType::Any) {
                if processor.state.behaviour == ErrorBehaviour::Strict {
                    return Err(RadError::InvalidMacroDefinition(format!(
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

    /// Dump a file
    ///
    /// # Usage
    ///
    /// $dump(macro,content)
    pub(crate) fn dump_file_content(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("dump", AuthType::FOUT, p)? {
            return Ok(None);
        }

        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let name = trim!(&args[0]);
            let file_name = Path::new(name);

            if !file_name.is_file() {
                return Err(RadError::InvalidArgument(format!(
                    "Dump requires a real file to dump but given file \"{}\" doesn't exist",
                    file_name.display()
                )));
            }

            {
                std::fs::File::create(file_name)?;
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Dump requires an file".to_owned(),
            ))
        }
    }

    /// Document a macro
    ///
    /// # Usage
    ///
    /// $document(macro,content)
    pub(crate) fn document(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let macro_name = trim!(&args[0]);
            let content = &args[1];

            // If operation failed
            if !processor.set_documentation(&macro_name, content)
                && processor.state.behaviour == ErrorBehaviour::Strict
            {
                let err = RadError::NoSuchMacroName(
                    macro_name.to_string(),
                    processor.get_similar_macro(macro_name, true),
                );
                processor.log_error(&err.to_string())?;
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
    pub(crate) fn bind_to_local(
        args: &str,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
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
    pub(crate) fn bind_to_local_raw(
        args: &str,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
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
    pub(crate) fn clear(_: &str, processor: &mut Processor) -> RadResult<Option<String>> {
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
        args: &str,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
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
    pub(crate) fn pause(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn define_static(
        args: &str,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let name = trim!(&args[0]);
            let value = trim!(&args[1]);
            // Macro name already exists
            if processor.contains_macro(&name, MacroType::Any) {
                // Strict mode prevents overriding
                // Return error
                if processor.state.behaviour == ErrorBehaviour::Strict {
                    return Err(RadError::InvalidMacroDefinition(format!(
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
    pub(crate) fn define_static_raw(
        args: &str,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let name = trim!(&args[0]);
            let value = &args[1];
            // Macro name already exists
            if processor.contains_macro(&name, MacroType::Any) {
                // Strict mode prevents overriding
                // Return error
                if processor.state.behaviour == ErrorBehaviour::Strict {
                    return Err(RadError::InvalidMacroDefinition(format!(
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
    pub(crate) fn change_notation(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn replace(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
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

    /// gt : is lvalue bigger than rvalue
    ///
    /// # Usage
    ///
    /// $gt(lvalue, rvalue)
    pub(crate) fn greater_than(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let lvalue = &args[0];
            let rvalue = &args[1];
            Ok(Some((lvalue > rvalue).to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "gt requires two arguments".to_owned(),
            ))
        }
    }

    /// gte : is lvalue bigger than or equal to rvalue
    ///
    /// # Usage
    ///
    /// $gte(lvalue, rvalue)
    pub(crate) fn greater_than_or_equal(
        args: &str,
        _: &mut Processor,
    ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let lvalue = &args[0];
            let rvalue = &args[1];
            Ok(Some((lvalue >= rvalue).to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "gte requires two arguments".to_owned(),
            ))
        }
    }

    /// lt : is lvalue less than rvalue
    ///
    /// # Usage
    ///
    /// $lt(lvalue, rvalue)
    pub(crate) fn less_than(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let lvalue = &args[0];
            let rvalue = &args[1];
            Ok(Some((lvalue < rvalue).to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "lt requires two arguments".to_owned(),
            ))
        }
    }

    /// lte : is lvalue less than or equal to rvalue
    ///
    /// # Usage
    ///
    /// $lte(lvalue, rvalue)
    pub(crate) fn less_than_or_equal(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let lvalue = &args[0];
            let rvalue = &args[1];
            Ok(Some((lvalue <= rvalue).to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "lte requires two arguments".to_owned(),
            ))
        }
    }

    /// eq : are values equal
    ///
    /// # Usage
    ///
    /// $eq(lvalue, rvalue)
    pub(crate) fn are_values_equal(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn is_empty(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn is_zero(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let value = trim!(&args[0]);
            Ok(Some(value.eq("0").to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "iszero requires an argument".to_owned(),
            ))
        }
    }

    /// foldlc
    ///
    /// # Usage
    ///
    /// $foldlc(count,type)
    pub(crate) fn fold_lines_by_count(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            use std::fmt::Write;

            let count = args[0].parse::<usize>().map_err(|_| {
                RadError::InvalidArgument(format!(
                    "Could not convert a given value \"{}\" into a unsigned integer",
                    args[0]
                ))
            })?;
            let mut formatted = String::new();

            for (idx, line) in args[1].lines().enumerate() {
                write!(formatted, "{}", line)?;
                if idx % count == 0 {
                    write!(formatted, "{}", p.state.newline)?;
                }
            }

            Ok(Some(formatted))
        } else {
            Err(RadError::InvalidArgument(
                "foldlc requires two arguments".to_owned(),
            ))
        }
    }

    /// isolate
    ///
    /// # Usage
    ///
    /// $insulav(value,type)
    pub(crate) fn isolate_vertical(args: &str, p: &mut Processor) -> RadResult<Option<String>> {
        use std::fmt::Write;
        let mut formatted = String::new();
        let mut only_blank = true;
        let mut first_contact = false;
        let mut nest_level = 1usize;
        for ch in args.chars() {
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
    /// $insulah(value,type)
    pub(crate) fn isolate_horizontal(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        let mut formatted = String::new();
        let mut iter = args.chars().peekable();
        let mut previous: char = '@';
        while let Some(ch) = iter.next() {
            if !previous.is_whitespace() && ISOLATION_CHARS_CLOSING.contains(&ch) {
                formatted.push(' ');
            }
            previous = ch;
            formatted.push(ch);
            if let Some(next_ch) = iter.peek() {
                if !next_ch.is_whitespace()
                    && (ISOLATION_CHARS.contains(&ch) || ISOLATION_SINGLE.contains(&ch))
                {
                    formatted.push(' ');
                    previous = ' ';
                }
            }
        }

        Ok(Some(formatted))
    }

    /// istype : Qualify a value
    ///
    /// # Usage
    ///
    /// $istype(value,type)
    pub(crate) fn qualify_value(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let qtype = trim!(&args[0]);
            let value = trim!(&args[1]);
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
    pub(crate) fn source_static_file(
        args: &str,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("source", AuthType::FIN, processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let path = &trim!(&args[0]);
            let path = Path::new(path);
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
    pub(crate) fn import_frozen_file(
        args: &str,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("import", AuthType::FIN, processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let path = &trim!(&args[0]);
            let path = Path::new(path);
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
    pub(crate) fn list_directory_files(
        args: &str,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("listdir", AuthType::FIN, processor)? {
            return Ok(None);
        }
        let args = ArgParser::new().args_to_vec(args, ',', SplitVariant::Always);
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
                PathBuf::from(trim!(val))
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
    pub(crate) fn paste_unicode(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn chars_array(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let arg = trim!(&args[0]);
            let mut chars = arg.chars().fold(String::new(), |mut acc, ch| {
                acc.push(ch);
                acc.push(',');
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
    pub(crate) fn hook_enable(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let hook_type = HookType::from_str(trim!(&args[0]))?;
            let index = trim!(&args[1]);
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
    pub(crate) fn hook_disable(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let hook_type = HookType::from_str(trim!(&args[0]))?;
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
    pub(crate) fn wrap(args: &str, _: &mut Processor) -> RadResult<Option<String>> {
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

    /// Update storage
    ///
    /// # Usage
    ///
    /// $update(text)
    pub(crate) fn update_storage(
        args: &str,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_to_vec(args, ',', SplitVariant::Always);

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
    pub(crate) fn extract_storage(_: &str, processor: &mut Processor) -> RadResult<Option<String>> {
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
    pub(crate) fn cindex_register(
        args: &str,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        use cindex::ReaderOption;

        if let Some(args) = ArgParser::new().args_with_len(args, 2) {
            let table_name = trim!(&args[0]);
            if processor.indexer.contains_table(table_name) {
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
    pub(crate) fn cindex_drop(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            processor.indexer.drop_table(trim!(&args[0]));
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
    pub(crate) fn cindex_query(args: &str, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1) {
            let mut value = String::new();
            processor
                .indexer
                .index_raw(trim!(&args[0]), OutOption::Value(&mut value))?;
            Ok(Some(trim!(&value).to_string()))
        } else {
            Err(RadError::InvalidArgument(
                "query requires an argument".to_owned(),
            ))
        }
    }
}

// ---
// Private structs for organizational purposes
// ---

/// Counter for total list items
#[derive(Default, Debug)]
struct RerHash {
    index_hash: HashMap<String, ListCounterByLevel>,
}

impl RerHash {
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
                return Err(RadError::InvalidCommandOption(format!(
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
