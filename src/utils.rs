//! Utility struct, methods for various operations

use crate::auth::{AuthState, AuthType};
use crate::common::{ProcessInput, RadResult};
use crate::error::RadError;
use crate::logger::WarningType;
use crate::{NewArgParser, Processor, WriteOption};
use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::io::BufRead;
use std::path::Path;

use crate::common::RelayTarget;

// Thanks stack overflow! SRC : https://stackoverflow.com/questions/12643009/regular-expression-for-floating-point-numbers
/// Number matches
pub static NUM_MATCH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"[+-]?([\d]*[.])?\d+"#).expect("Failed to create number regex"));

// ----------
// MACRO RULES
// ----------

/// string std::mem::take
#[macro_export]
macro_rules! stake {
    ($e:expr) => {
        std::mem::take(&mut $e)
    };
}

// Include function macro manaul
#[macro_export]
macro_rules! man_fun {
    ($fname:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/manual_src/fun/",
            $fname
        ))
        .to_string() // assumes Linux ('/')!
    };
}

// Include deterred macro manaul
#[macro_export]
macro_rules! man_det {
    ($fname:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/manual_src/det/",
            $fname
        ))
        .to_string() // assumes Linux ('/')!
    };
}
// ----------

#[cfg(feature = "color")]
use colored::*;

/// Utility relates struct
pub(crate) struct Utils;

impl Utils {
    pub fn get_split_arguments_or_error<'a>(
        macro_name: &'a str,
        arguments: &'a &str,
        length: usize,
        parser: Option<&mut NewArgParser>,
    ) -> RadResult<Vec<Cow<'a, str>>> {
        if let Some(args) = parser
            .unwrap_or(&mut NewArgParser::new())
            .args_with_len(arguments, length)
        {
            Ok(args)
        } else {
            let error_message = match length {
                0 => format!("{} does not require any arguments", macro_name),
                1 => format!("{} requires an argument", macro_name),
                _ => format!("{} requires {} arguments", macro_name, length),
            };
            Err(RadError::InvalidArgument(error_message))
        }
    }

    /// Split string by whitespaces but respect spaces within commas and strip commas
    ///
    /// # Example
    ///
    /// "a b c"           -> [a,b,c]
    /// "a b ' ' d"       -> [a,b, ,d]
    /// "a b c' ' d"      -> [a,b,c ,d]
    /// "a b ' c f g ' d" -> ["a","b"," c f g ","d",]
    pub fn get_whitespace_split_retain_quote_rule<'a>(args: &'a str) -> Vec<Cow<'a, str>> {
        let mut split: Vec<Cow<'a, str>> = vec![];
        let args = args.trim();
        if args.is_empty() {
            return split;
        }
        let mut index = 0usize;
        let mut quoted = false;
        let mut strip_quote = false;
        let mut prev = '0';
        for (idx, ch) in args.chars().enumerate() {
            // Decide whether split needs or not
            if ch == ' ' {
                if quoted {
                    // Ignore space
                    // continue;
                } else {
                    // Not quoted

                    // if previous was also space
                    // don't split and continue
                    if prev == ' ' {
                    } else {
                        // Previous was concrete character
                        if strip_quote {
                            split.push(args[index..idx].trim_start().replace('\'', "").into());
                            strip_quote = false;
                        } else {
                            split.push(args[index..idx].trim_start().into());
                        }
                    }
                    index = idx; // Update index
                }
            }
            if ch == '\'' {
                quoted = !quoted;
                strip_quote = true;
            }

            // GLobal process
            prev = ch;
        }

        // Collect last
        if strip_quote {
            split.push(args[index..].trim_start().replace('\'', "").into());
        } else {
            split.push(args[index..].trim_start().into());
        }
        split
    }

    /// Split macro name and arguments from given source
    pub(crate) fn get_name_n_arguments(
        src: &str,
        with_trailing_comma: bool,
    ) -> RadResult<(&str, String)> {
        let macro_src_joined = src.split_whitespace().collect::<Vec<_>>();
        let macro_name = macro_src_joined.first().ok_or(RadError::InvalidArgument(
            "Macro name cannot be empty".to_string(),
        ))?;
        let mut macro_arguments = macro_src_joined[1..].join(",");
        if with_trailing_comma && !macro_arguments.is_empty() {
            macro_arguments.push(',');
        }
        Ok((macro_name, macro_arguments))
    }

    /// Generic levenshtein distance function
    ///
    /// SOURCE : https://en.wikibooks.org/wiki/Algorithm_Implementation/Strings/Levenshtein_distance#Rust
    pub(crate) fn levenshtein<T1: AsRef<[u8]>, T2: AsRef<[u8]>>(s1: T1, s2: T2) -> usize {
        let v1 = s1.as_ref();
        let v2 = s2.as_ref();

        // Early exit if one of the strings is empty
        let v1len = v1.len();
        let v2len = v2.len();
        if v1len == 0 {
            return v2len;
        }
        if v2len == 0 {
            return v1len;
        }

        #[inline]
        fn min3<T: Ord>(v1: T, v2: T, v3: T) -> T {
            std::cmp::min(v1, std::cmp::min(v2, v3))
        }

        #[inline]
        fn delta(x: u8, y: u8) -> usize {
            if x == y {
                0
            } else {
                1
            }
        }

        let mut column: Vec<usize> = (0..v1len + 1).collect();
        for x in 1..v2len + 1 {
            column[0] = x;
            let mut lastdiag = x - 1;

            for y in 1..v1len + 1 {
                let olddiag = column[y];
                column[y] = min3(
                    column[y] + 1,
                    column[y - 1] + 1,
                    lastdiag + delta(v1[y - 1], v2[x - 1]),
                );
                lastdiag = olddiag;
            }
        }

        column[v1len]
    }

    /// Create a local name from level and name
    pub(crate) fn local_name(level: usize, name: &str) -> String {
        format!("{}.{}", level, name)
    }

    // Shamelessly copied from
    // https://stackoverflow.com/questions/64517785/read-full-lines-from-stdin-including-n-until-end-of-file
    /// Read full lines of bufread iterator which doesn't chop new lines
    pub fn full_lines(mut input: impl BufRead) -> impl Iterator<Item = std::io::Result<String>> {
        std::iter::from_fn(move || {
            let mut vec = String::new();
            match input.read_line(&mut vec) {
                Ok(0) => None,
                Ok(_) => Some(Ok(vec)),
                Err(e) => Some(Err(e)),
            }
        })
    }

    /// Check if a character is a blank chracter
    pub(crate) fn is_blank_char(ch: char) -> bool {
        ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r'
    }

    /// Check if a character is true
    ///
    /// In this contenxt, true and non zero number is 'true' while false and zero number is false
    pub(crate) fn is_arg_true(arg: &str) -> RadResult<bool> {
        let arg = arg.trim();
        if let Ok(value) = arg.parse::<usize>() {
            if value == 0 {
                return Ok(false);
            } else {
                return Ok(true);
            }
        } else if arg.to_lowercase() == "true" {
            return Ok(true);
        } else if arg.to_lowercase() == "false" {
            return Ok(false);
        }

        Err(RadError::InvalidArgument(
            "Neither true nor false".to_owned(),
        ))
    }

    pub(crate) fn ascii_slice(source: &str, min: usize, max: isize) -> RadResult<&str> {
        unimplemented!();
        Ok("")
    }

    /// Get a substring of utf8 encoded text.
    pub(crate) fn utf8_slice(source: &str, min: usize, max: isize) -> RadResult<Cow<'_, str>> {
        let mut min_byte_index: Option<usize> = None;
        let mut max_byte_index: Option<usize> = None;
        // This doesn't allocate
        if max >= 0 {
            for (char_index, (byte_index, _)) in source.char_indices().enumerate() {
                if char_index == min {
                    min_byte_index.replace(byte_index);
                }

                if char_index == max as usize {
                    max_byte_index.replace(byte_index);
                }

                if min_byte_index.is_some() && max_byte_index.is_some() {
                    break;
                }
            }
        } else {
            // This does allocate, sadly
            // and return early if possible
            let chars = source.char_indices().collect::<Vec<_>>();
            if let Some(min) = chars.get(min) {
                min_byte_index.replace(min.0);
            }

            // THis is preferred way because usize is bigger than isize
            // Thus chars.len() should not be casted as isize
            // rather subtracter should be casted as usize
            if let Some(max) = chars.get(chars.len() - max.unsigned_abs()) {
                max_byte_index.replace(max.0);
            }
        }

        match (min_byte_index, max_byte_index) {
            (Some(a), Some(b)) => Ok(source[a..=b].into()),
            _ => Err(RadError::InvalidArgument(
                "Given slice index is not in the boundary of string length".to_string(),
            )),
        }
    }

    /// Get a sub lines from text
    ///
    /// None means end of slice
    #[allow(clippy::needless_late_init)]
    #[allow(clippy::iter_skip_zero)]
    pub(crate) fn sub_lines(
        source: &str,
        min_src: Option<isize>,
        max_src: Option<isize>,
    ) -> RadResult<&str> {
        // Critical exception
        // which is...
        //
        // empty source :)
        if source.is_empty() {
            return Err(RadError::InvalidArgument(
                "Given source is empty which cannot be sliced".to_string(),
            ));
        }

        let raw_min: isize;
        let raw_max: isize;
        let mut min_index: usize = 0; // Converted value corresponds to real index
        let mut max_index: usize = 0; // Converted value corresponds to real index
        let mut min_byte_index: Option<usize> = None; // Byte index from str
        let mut max_byte_index: Option<usize> = None; // Byte index from str

        // ABOUT: Trailing newline
        //
        // Shell doesn't have trailing newline
        // while text editor has trailing newline
        // therefore following situations should be considered

        // ----------
        // >>>>>>>>>>
        // <NEWLINE LOGICS>
        // ----------
        let (ending_newline, strip_count) = if source.ends_with("\r\n") {
            (true, 2)
        } else if source.ends_with('\n') {
            (true, 1)
        } else {
            (false, 0)
        };

        let final_index = source.len() - 1;
        let chain_src = [(final_index, "\n")]; // Attach additional newline

        let chained = if ending_newline {
            chain_src.into_iter().rev().skip(1).rev() // Remove it
        } else {
            // This is required because of type coercion
            chain_src.into_iter().rev().skip(0).rev() // Retain it
        };

        // Final iterator that is used globally
        let mut lines_indices = source
            .match_indices('\n')
            .chain(chained)
            .enumerate()
            .peekable();

        // ----------
        // <<<<<<<<<<
        // </NEWLINE LOGICS>
        // ----------

        // --------------------
        // ABOUT : Early return cases
        //
        // [1] _,_
        // [2] 0,0
        // [3] 0,_

        // ----------
        // >>>>>>>>>>
        // <SPECIAL SYNTAX CHECK>
        // ----------

        if min_src.is_none() {
            if max_src.is_none() {
                // -> Both min and max is '_'
                //
                // This is valid and
                // return final line early

                let lines_indices_collected = lines_indices.collect::<Vec<_>>();

                // No line separator
                if !ending_newline && lines_indices_collected.len() == 1 {
                    return Ok(source);
                }

                // Slice until end of string
                let sliced = if lines_indices_collected.len() <= 1 {
                    source
                } else {
                    let (_, (idx, _)) = lines_indices_collected
                        .get(lines_indices_collected.len() - 2)
                        .unwrap();
                    &source[idx + 1..]
                };

                if ending_newline {
                    return Ok(&sliced[..sliced.len() - strip_count]);
                } else {
                    return Ok(sliced);
                }
            } else {
                // Min = '_'
                // Max = positive number or negative number ( Real number )
                // This is not valid
                return Err(RadError::InvalidArgument(
                "Given slice index is not in the boundary of lines length. '_' min index cannot be used with real max index number".to_string(),
            ));
            }
        }

        // ----------
        // </SPECIAL SYNTAX CHECK>
        // <<<<<<<<<<
        // ----------

        // ----------
        // <ZERO CHECK>
        // <<<<<<<<<<
        // ----------
        // Min and max is both 0
        // Simply return first line without processing
        let min_temp = min_src.unwrap_or(10);
        if min_temp == max_src.unwrap_or(10) && min_temp == 0 {
            return Ok(match source.split_once('\n') {
                None => source,
                Some((prefix, _)) => prefix,
            });
        }
        // ----------
        // </ZERO CHECK>
        // <<<<<<<<<<
        // ----------

        // ----------
        // Early set values and possibly early return
        // ----------
        if let Some(0) = min_src {
            min_byte_index.replace(0);
        }
        if max_src.is_none() {
            max_byte_index.replace(final_index);
        }

        if min_byte_index.is_some() && max_byte_index.is_some() {
            return Ok(source);
        }

        // ----------
        // : END OF EARLY RETURN CASES
        // ----------

        let lines_indices_collected: Vec<(usize, (usize, &str))>;

        // MIN MAX
        // _   _   -> Early return
        // _   +   -> Error
        // _   -   -> Error
        // From 3x3 cases all none('_') cases were already addressed
        raw_min = min_src.expect("Logical error and should not happen");

        // ----------
        // [NORMALIZE] min and max index ( Usize )
        // Optinoally collect iterator to valid allocation

        // Max byte index is real number
        // -> Not '_'
        // Only find min index
        if max_byte_index.is_some() {
            max_index = usize::MAX; // This value is not used rather it's a placeholder
            if raw_min >= 0 {
                min_index = raw_min as usize;
            } else {
                lines_indices_collected = lines_indices.clone().collect();
                min_index = lines_indices_collected.len() - raw_min.unsigned_abs() - 1;
            }
        } else {
            raw_max = max_src.expect("Logical error and should not happen");
            if raw_min < 0 || raw_max < 0 {
                lines_indices_collected = lines_indices.clone().collect();

                // ----------
                // Empty iteratoable list -> Error
                // ----------
                if !ending_newline && lines_indices_collected.len() == 1 {
                    return Err(RadError::InvalidArgument(
                        "Cannot slice a string without any newlines with non 0 integer".to_string(),
                    ));
                }

                if raw_min < 0 {
                    min_index = lines_indices_collected.len() - raw_min.unsigned_abs() - 1;
                }
                if raw_max < 0 {
                    max_index = lines_indices_collected.len() - raw_max.unsigned_abs() - 1;
                }
            }
            // Max is not real number
            if raw_min >= 0 {
                min_index = raw_min as usize;
            }

            if raw_max >= 0 {
                max_index = raw_max as usize;
            }
        }
        // ----------
        // </NORMALIZE>
        // ----------

        // Global iteration
        while let Some((line_index, (byte_index, _))) = lines_indices.next() {
            // Min byte is known +
            // final newline is not existent +
            // max value is same with lines length
            // = slice until last
            if lines_indices.peek().is_none()
                && !ending_newline
                && max_byte_index.is_none()
                && line_index == max_index.saturating_sub(2)
            {
                if let Some(min) = min_byte_index {
                    let sliced = &source[min..];
                    if ending_newline {
                        return Ok(&sliced[..sliced.len() - strip_count]);
                    } else {
                        return Ok(sliced);
                    }
                }
            }

            if min_byte_index.is_none() && line_index == min_index.saturating_sub(1) {
                min_byte_index.replace(byte_index + 1);
            }

            if max_byte_index.is_none() && max_index == line_index {
                max_byte_index.replace(byte_index);
            }

            if min_byte_index.is_some() && max_byte_index.is_some() {
                break;
            }
        }

        // TODO REmove this
        //
        // log
        // eprintln!("RAW : MIN : {:#?} MAX : {:#?}", raw_min, raw_max);
        // eprintln!("INDEX : MIN : {:#?} MAX : {:#?}", min_index, max_index);
        // eprintln!(
        //     "BINDEX : MIN : {:#?} MAX : {:#?}",
        //     min_byte_index, max_byte_index
        // );
        //
        // eprintln!("LEN : {}", source.len());

        // TODO
        // Back up lines were here

        match (min_byte_index, max_byte_index) {
            (Some(a), Some(b)) => {
                if a > b {
                    return Err(RadError::InvalidArgument(
                        "Given slice index is not in the boundary of lines length".to_string(),
                    ));
                }

                let sliced = &source[a..=b];
                // TODO This is ugly
                if let Some(sliced) = sliced.strip_suffix("\r\n") {
                    Ok(sliced)
                } else if let Some(sliced) = sliced.strip_suffix('\n') {
                    Ok(sliced)
                } else {
                    Ok(sliced)
                }
            }
            _ => Err(RadError::InvalidArgument(
                "Given slice index is not in the boundary of lines length".to_string(),
            )),
        }
    }

    /// Print text as green if possible
    #[allow(unused_variables)]
    pub fn green(string: &str, to_file: bool) -> Box<dyn std::fmt::Display> {
        if cfg!(feature = "color") {
            #[cfg(feature = "color")]
            if !to_file {
                return Box::new(string.green());
            }
        }
        Box::new(string.to_owned())
    }

    /// Print text as red if possible
    #[allow(unused_variables)]
    pub fn red(string: &str, to_file: bool) -> Box<dyn std::fmt::Display> {
        if cfg!(feature = "color") {
            #[cfg(feature = "color")]
            if !to_file {
                return Box::new(string.red());
            }
        }
        Box::new(string.to_owned())
    }

    /// Print text as yellow if possible
    #[allow(unused_variables)]
    pub fn yellow(string: &str, to_file: bool) -> Box<dyn std::fmt::Display> {
        if cfg!(feature = "color") {
            #[cfg(feature = "color")]
            if !to_file {
                return Box::new(string.yellow());
            }
        }
        Box::new(string.to_owned())
    }

    // Copied from
    // https://llogiq.github.io/2016/09/24/newline.html
    // Actually the source talks about how to make following function faster
    // yet I don't want to use simd because r4d's logic is currently very synchronous
    // and making it a asynchornous would take much more effort and time
    // NOTE : Trailing single is necessary because this only checks newline chracter
    // thus line without trailing newline doesn't count as 1
    /// Count new lines
    #[allow(dead_code)]
    pub(crate) fn count_sentences(s: &str) -> usize {
        s.as_bytes().iter().filter(|&&c| c == b'\n').count() + 1
    }

    #[cfg(feature = "debug")]
    /// Clear terminal cells
    pub fn clear_terminal() -> RadResult<()> {
        use crossterm::{terminal::ClearType, ExecutableCommand};

        std::io::stdout()
            .execute(crossterm::terminal::Clear(ClearType::All))?
            .execute(crossterm::cursor::MoveTo(0, 0))?;

        Ok(())
    }

    /// Check if path is really in file system or not
    pub fn is_real_path(path: &std::path::Path) -> RadResult<()> {
        if !path.exists() {
            return Err(RadError::InvalidFile(path.display().to_string()));
        }
        Ok(())
    }

    /// Pop only a single newline from a source
    pub fn pop_newline(s: &mut String) {
        if s.ends_with('\n') {
            s.pop();
            if s.ends_with('\r') {
                s.pop();
            }
        }
    }

    /// Check file authority
    pub(crate) fn is_granted(
        name: &str,
        auth_type: AuthType,
        processor: &mut Processor,
    ) -> RadResult<bool> {
        match processor.get_auth_state(&auth_type) {
            AuthState::Restricted => Err(RadError::PermissionDenied(name.to_owned(), auth_type)),
            AuthState::Warn => {
                processor.log_warning(
                    &format!(
                        "\"{}\" was called with \"{:?}\" permission",
                        name, auth_type
                    ),
                    WarningType::Security,
                )?;
                Ok(true)
            }
            AuthState::Open => Ok(true),
        }
    }

    /// Execute a subprocess with given arguments
    #[cfg(feature = "basic")]
    pub(crate) fn subprocess(args: &[&str]) -> RadResult<()> {
        use std::io::Write;
        use std::process::Stdio;
        #[cfg(target_os = "windows")]
        let process = std::process::Command::new("cmd")
            .arg("/C")
            .args(&args[0..])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|_| {
                RadError::InvalidArgument(format!("Failed to execute command : \"{:?}\"", &args[0]))
            })?;

        #[cfg(not(target_os = "windows"))]
        let process = std::process::Command::new("sh")
            .arg("-c")
            .arg(&args[0..].join(" "))
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|_| {
                RadError::InvalidArgument(format!("Failed to execute command : \"{:?}\"", &args[0]))
            })?;

        let output = process.wait_with_output()?;
        let out_content = String::from_utf8_lossy(&output.stdout);
        let err_content = String::from_utf8_lossy(&output.stderr);

        if out_content.len() != 0 {
            write!(std::io::stdout(), "{}", &out_content)?;
        }
        if err_content.len() != 0 {
            write!(std::io::stderr(), "{}", &err_content)?;
        }
        Ok(())
    }

    /// This checks if a file is safely modifiable
    ///
    /// File operation can be nested and somtimes logically implausible. Such as referencing self,
    /// or referencing parent file is would cause infinite loop
    ///
    /// Alos, opening processor's out option and err option would be impossible from the start,
    /// while creating a hard to read error message.
    ///
    /// Unallowed files are followed
    ///
    /// - Current input
    /// - Input that is saved in stack
    /// - File that is being relayed to
    /// - Processor's out option
    /// - Processor's err option
    ///
    /// # Argument
    ///
    /// - processor : Processor to get multiple files from
    /// - canoic    : Real absolute path to evaluate ( If not this possibly panicks )
    pub(crate) fn check_file_sanity(processor: &Processor, canonic: &Path) -> RadResult<()> {
        // Rule 1
        // You cannot include self
        if let ProcessInput::File(path) = &processor.state.current_input {
            if path.canonicalize()? == canonic {
                return Err(RadError::UnallowedMacroExecution(format!(
                    "Processing self is not allowed : \"{}\"",
                    &path.display()
                )));
            }
        }

        // Rule 2
        // Field is in input stack
        // This unwraps is mostly ok ( I guess )
        if processor.state.input_stack.contains(canonic) {
            return Err(RadError::UnallowedMacroExecution(format!(
                "Processing self is not allowed : \"{}\"",
                &canonic
                    .file_name()
                    .unwrap_or_else(|| OsStr::new("input_file"))
                    .to_string_lossy()
            )));
        }

        // Rule 3
        // You cannot include file that is being relayed
        if let Some(RelayTarget::File(file)) = &processor.state.relay.last() {
            if file.path() == canonic {
                return Err(RadError::UnallowedMacroExecution(format!(
                    "Processing relay target while relaying to the file is not allowed : \"{}\"",
                    &file.name().display()
                )));
            }
        }

        // Rule 4
        // You cannot include processor's out file
        if let WriteOption::File(target) = &processor.write_option {
            if target.path() == canonic {
                return Err(RadError::UnallowedMacroExecution(format!(
                    "Cannot process an out file : \"{}\"",
                    &target.name().display()
                )));
            }
        }

        // Rule 5
        // You cannot include processor's error file
        if let Some(WriteOption::File(target)) = &processor.get_logger_write_option() {
            if target.path() == canonic {
                return Err(RadError::UnallowedMacroExecution(format!(
                    "Cannot process an error file : \"{}\"",
                    &target.name().display()
                )));
            }
        }
        Ok(())
    }
}
