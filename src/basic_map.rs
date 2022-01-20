//! # Basic module
//!
//! Basic module includes struct and methods related to basic macros which are technically function
//! pointers.

use crate::consts::ESR;
use std::array::IntoIter;
use std::io::Write;
use std::fs::OpenOptions;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{PathBuf,Path};
use std::process::Command;
use crate::error::RadError;
use crate::models::{RadResult, FlowControl};
use crate::arg_parser::{ArgParser, GreedyState};
use regex::Regex;
use crate::utils::Utils;
use crate::processor::Processor;
use crate::auth::AuthType;
#[cfg(feature = "hook")]
use crate::hookmap::HookType;
#[cfg(feature = "csv")]
use crate::formatter::Formatter;
#[cfg(feature = "lipsum")]
use lipsum::lipsum;
use lazy_static::lazy_static;
#[cfg(feature = "cindex")]
use cindex::OutOption;

lazy_static! {
    static ref CLRF_MATCH: Regex = Regex::new(r#"\r\n"#).unwrap();
    static ref CHOMP_MATCH : Regex = Regex::new(r#"\n\s*\n"#).expect("Failed to crate chomp regex");
}

/// Type signature of basic macros
///
/// This is in order of args, greediness, processor's mutable reference
///
/// # Example
///
/// ```rust
/// fn demo(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
///     let mut medium = String::new();
///     // Some logics go here
///     if this_macro_prints_something {
///         Ok(Some(medium))
///     } else {
///         // If return "None", then single newline will be removed
///         Ok(None)
///     }
/// }
///
/// // ... While building a processor ...
/// processor.add_basic_rules(vec![("test", test as MacroType)]);
/// ```
pub type MacroType = fn(&str, bool ,&mut Processor) -> RadResult<Option<String>>;

#[derive(Clone)]
pub(crate) struct BasicMacroMap {
    pub(crate) macros : HashMap<String, BMacroSign>,
}

impl BasicMacroMap {
    /// Creates empty map
    pub fn empty() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    /// Creates new basic macro hashmap
    ///
    /// Optional macros are included only when a feature is enabled
    pub fn new() -> Self {
        // Create hashmap of functions
        #[allow(unused_mut)]
        let mut map = HashMap::from_iter(IntoIter::new([
            ("-".to_owned(),       BMacroSign::new("-",       ESR,Self::get_pipe)),
            ("abs".to_owned(),     BMacroSign::new("abs",     ["a_path"],Self::absolute_path)),
            ("append".to_owned(),  BMacroSign::new("append",  ["a_macro_name","a_content"],Self::append)),
            ("arr".to_owned(),     BMacroSign::new("arr",     ["a_values"],Self::array)),
            ("assert".to_owned(),  BMacroSign::new("assert",  ["a_lvalue","a_rvalue"],Self::assert)),
            ("nassert".to_owned(), BMacroSign::new("nassert", ["a_lvalue","a_rvalue"],Self::assert_ne)),
            ("chomp".to_owned(),   BMacroSign::new("chomp",   ["a_content"],Self::chomp)),
            ("comp".to_owned(),    BMacroSign::new("comp",    ["a_content"],Self::compress)),
            ("dnl".to_owned(),     BMacroSign::new("dnl",     ESR,Self::deny_newline)),
            ("env".to_owned(),     BMacroSign::new("env",     ["a_env_name"],Self::get_env)),
            ("envset".to_owned(),  BMacroSign::new("envset",  ["a_env_name","a_env_value"],Self::set_env)),
            ("escape".to_owned(),  BMacroSign::new("escape",  ESR,Self::escape)),
            ("exit".to_owned(),    BMacroSign::new("exit",    ESR,Self::exit)),
            ("fileout".to_owned(), BMacroSign::new("fileout", ["a_truncate?","a_filename","a_content"],Self::file_out)),
            ("head".to_owned(),    BMacroSign::new("head",    ["a_count","a_content"],Self::head)),
            ("headl".to_owned(),   BMacroSign::new("headl",   ["a_count","a_content"],Self::head_line)),
            ("include".to_owned(), BMacroSign::new("include", ["a_filename"],Self::include)),
            ("len".to_owned(),     BMacroSign::new("len",     ["a_string"],Self::len)),
            ("name".to_owned(),    BMacroSign::new("name",    ["a_path"],Self::get_name)),
            ("not".to_owned(),     BMacroSign::new("not",     ["a_boolean"],Self::not)),
            ("nl".to_owned(),      BMacroSign::new("nl",      ESR,Self::newline)),
            ("parent".to_owned(),  BMacroSign::new("parent",  ["a_path"],Self::get_parent)),
            ("panic".to_owned(),   BMacroSign::new("panic",   ["a_msg"],Self::manual_panic)),
            ("path".to_owned(),    BMacroSign::new("path",    ["a_paths"],Self::merge_path)),
            ("pipe".to_owned(),    BMacroSign::new("pipe",    ["a_value"],Self::pipe)),
            ("pipeto".to_owned(),  BMacroSign::new("pipe",    ["a_pipe_name","a_value"],Self::pipe_to)),
            ("read".to_owned(),    BMacroSign::new("read",    ["a_filename"],Self::read)),
            ("redir".to_owned(),   BMacroSign::new("redir",   ["a_redirect?"],Self::temp_redirect)),
            ("regex".to_owned(),   BMacroSign::new("regex",   ["a_source","a_match","a_substitution"],Self::regex_sub)),
            ("rename".to_owned(),  BMacroSign::new("rename",  ["a_macro_name","a_new_name"],Self::rename_call)),
            ("repeat".to_owned(),  BMacroSign::new("repeat",  ["a_count","a_source"],Self::repeat)),
            ("strip".to_owned(),   BMacroSign::new("tail",    ["a_count","a_direction","a_content"],Self::strip)),
            ("stripl".to_owned(),  BMacroSign::new("taill",   ["a_count","a_direction","a_content"],Self::strip_line)),
            ("sub".to_owned(),     BMacroSign::new("sub",     ["a_start_index","a_end_index","a_source"],Self::substring)),
            ("syscmd".to_owned(),  BMacroSign::new("syscmd",  ["a_command"],Self::syscmd)),
            ("tail".to_owned(),    BMacroSign::new("tail",    ["a_count","a_content"],Self::tail)),
            ("taill".to_owned(),   BMacroSign::new("taill",   ["a_count","a_content"],Self::tail_line)),
            ("tempin".to_owned(),  BMacroSign::new("tempin",  ["a_tempin"],Self::temp_include)),
            ("tempout".to_owned(), BMacroSign::new("tempout", ["a_tempout"],Self::temp_out)),
            ("tempto".to_owned(),  BMacroSign::new("tempto",  ["a_filename"],Self::set_temp_target)),
            ("tr".to_owned(),      BMacroSign::new("tr",      ["a_source","a_matches","a_substitutions"],Self::translate)),
            ("trim".to_owned(),    BMacroSign::new("trim",    ["a_content"],Self::trim)),
            ("triml".to_owned(),   BMacroSign::new("triml",   ["a_content"],Self::triml)),
            ("undef".to_owned(),   BMacroSign::new("undef",   ["a_macro_name"],Self::undefine_call)),
            // THis is simply a placeholder
            ("define".to_owned(),  BMacroSign::new("define",  ESR,Self::define_type)),
        ]));
        
        // Optional macros
        #[cfg(feature = "csv")]
        {
            map.insert("from".to_owned(),    BMacroSign::new("from", ["a_macro_name","a_csv_value"],Self::from_data));
            map.insert("table".to_owned(),   BMacroSign::new("table",["a_table_form","a_csv_value"],Self::table));
        }
        #[cfg(feature = "cindex")]
        {
            map.insert("regcsv".to_owned(),  BMacroSign::new("regcsv", ["a_table_name","a_table"], Self::cindex_register));
            map.insert("query".to_owned(),   BMacroSign::new("query",  ["a_query"], Self::cindex_query));
        }

        #[cfg(feature = "chrono")]
        {
            map.insert("time".to_owned(),    BMacroSign::new("time",ESR,Self::time));
            map.insert("date".to_owned(),    BMacroSign::new("date",ESR,Self::date));
        }
        #[cfg(feature = "lipsum")]
        map.insert("lipsum".to_owned(),      BMacroSign::new("lipsum",["a_word_count"],Self::lipsum_words));
        #[cfg(feature = "evalexpr")]
        map.insert("eval".to_owned(),        BMacroSign::new("eval",  ["a_expression"],Self::eval));
        #[cfg(feature = "textwrap")]
        map.insert("wrap".to_owned(),        BMacroSign::new("wrap",  ["a_width","a_content"],Self::wrap));

        #[cfg(feature = "hook")]
        {
            map.insert("hookon".to_owned(),  BMacroSign::new("hookon", ["a_macro_type","a_target_name"],Self::hook_enable));
            map.insert("hookoff".to_owned(), BMacroSign::new("hookoff",["a_macro_type","a_target_name"],Self::hook_disable));
        }

        #[cfg(feature = "storage")]
        {
            map.insert("update".to_owned(),      BMacroSign::new("update",  ["a_text"],Self::update_storage));
            map.insert("extract".to_owned(),     BMacroSign::new("extract", ESR,Self::extract_storage));
        }

        // Return struct
        Self { macros : map }
    }

    /// Add new basic rule
    pub fn add_new_rule(&mut self, name: &str, macro_ref: MacroType) {
        let signature = BMacroSign::new(name,["unknown"],macro_ref);
        self.macros.insert(name.to_owned(), signature);
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
    pub fn get_func(&self, name: &str) -> Option<&MacroType> {
        if let Some(sig) = self.macros.get(name) {
            Some(&sig.logic)
        } else {
            None
        }
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
    pub fn rename(&mut self, name: &str, target: &str) {
        let func = self.macros.remove(name).unwrap();
        self.macros.insert(target.to_owned(), func);
    }

    // ==========
    // Basic Macros
    // ==========
    /// Print out current time
    ///
    /// # Usage
    ///
    /// $time()
    #[cfg(feature = "chrono")]
    fn time(_: &str, _: bool, _ : &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(format!("{}", chrono::offset::Local::now().format("%H:%M:%S"))))
    }

    /// Print out current date
    ///
    /// # Usage
    ///
    /// $date()
    #[cfg(feature = "chrono")]
    fn date(_: &str, _: bool, _ : &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(format!("{}", chrono::offset::Local::now().format("%Y-%m-%d"))))
    }

    /// Substitute the given source with following match expressions
    ///
    /// # Usage
    ///
    /// $regex(source_text,regex_match,substitution)
    fn regex_sub(args: &str, greedy: bool, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, greedy) {
            let source= &args[0];
            let match_expr= &args[1];
            let substitution= &args[2];

            // This is regex expression without any preceding and trailing commands
            let reg = Regex::new(&format!(r"{}", match_expr))?;
            let result = reg.replace_all(source, substitution); // This is a cow, moo~
            Ok(Some(result.to_string()))
        } else {
            Err(RadError::InvalidArgument("Regex sub requires three arguments".to_owned()))
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
    fn eval(args: &str, greedy: bool,_: &mut Processor ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let formula = &args[0];
            let result = evalexpr::eval(formula)?;
            // TODO Enable floating points length (or something similar)
            Ok(Some(result.to_string()))
        } else {
            Err(RadError::InvalidArgument("Eval requires an argument".to_owned()))
        }
    }

    /// Negate given value
    ///
    /// This returns true, false or evaluated number
    ///
    /// # Usage
    ///
    /// $not(expression)
    fn not(args: &str, greedy: bool,_: &mut Processor ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let args = &args[0];
            if let Ok(value) = Utils::is_arg_true(args) {
                Ok(Some((!value).to_string()))
            } else {
                Err(RadError::InvalidArgument(format!("Not requires either true/false or zero/nonzero integer but given \"{}\"", args)))
            }
        } else {
            Err(RadError::InvalidArgument("Not requires an argument".to_owned()))
        }
    }

    /// Trim preceding and trailing whitespaces (' ', '\n', '\t', '\r')
    ///
    /// # Usage
    ///
    /// $trim(expression)
    fn trim(args: &str, greedy: bool, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            Ok(Some(Utils::trim(&args[0])))
        } else {
            Err(RadError::InvalidArgument("Trim requires an argument".to_owned()))
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
    fn triml(args: &str, greedy: bool, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let mut lines = String::new();
            let mut iter = args[0].lines().peekable();
            while let Some(line) = iter.next() {
                lines.push_str(&Utils::trim(line));
                // Append newline because String.lines() method cuts off all newlines
                if let Some(_) = iter.peek() {
                    lines.push_str(&p.state.newline);
                }
            }
            Ok(Some(lines))
        } else {
            Err(RadError::InvalidArgument("Trim requires an argument".to_owned()))
        }
    }

    /// Removes duplicate newlines whithin given input
    ///
    /// # Usage
    ///
    /// $chomp(expression)
    fn chomp(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let source = &args[0];
            // First convert all '\r\n' into '\n' and reformat it into current newline characters
            let lf_converted = &*CLRF_MATCH.replace_all(source, "\n");
            let chomp_result = &*CHOMP_MATCH.replace_all(lf_converted, format!("{0}{0}",&processor.state.newline));

            Ok(Some(chomp_result.to_string()))
        } else {
            Err(RadError::InvalidArgument("Chomp requires an argument".to_owned()))
        }
    }

    /// Both apply trim and chomp to given expression
    ///
    /// # Usage
    ///
    /// $comp(Expression)
    fn compress(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let source = &args[0];
            // Chomp and then compress
            let result = Utils::trim(&BasicMacroMap::chomp(source,greedy, processor)?.unwrap());

            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument("Compress requires an argument".to_owned()))
        }
    }

    /// Creates placeholder with given amount of word counts
    ///
    /// # Usage
    ///
    /// $lipsum(Number)
    #[cfg(feature = "lipsum")]
    fn lipsum_words(args: &str, greedy: bool,_: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let word_count = &args[0];
            if let Ok(count) = Utils::trim(word_count).parse::<usize>() {
                Ok(Some(lipsum(count)))
            } else {
                Err(RadError::InvalidArgument(format!("Lipsum needs a number bigger or equal to 0 (unsigned integer) but given \"{}\"", word_count)))
            }
        } else {
            Err(RadError::InvalidArgument("Lipsum requires an argument".to_owned()))
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
    fn include(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("include", AuthType::FIN,processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let raw = Utils::trim(&args[0]);
            let mut file_path = PathBuf::from(&raw);

            // if current input is not stdin and file path is relative
            // Create new file path that starts from current file path
            if processor.state.current_input != "stdin" && file_path.is_relative() {
                // It is ok get parent because any path that has a length can return parent
                file_path = PathBuf::from(&processor.state.current_input).parent().unwrap().join(file_path);
            }

            if file_path.is_file() { 
                processor.set_sandbox();
                let chunk = processor.from_file_as_chunk(file_path)?;
                Ok(chunk)
            } else {
                let formatted = format!("File path : \"{}\" doesn't exist or not a file", file_path.display());
                Err(RadError::InvalidArgument(formatted))
            }
        } else {
            Err(RadError::InvalidArgument("Include requires an argument".to_owned()))
        }
    }

    /// Paste given file's content as bufstream
    ///
    /// Every macros within the file is also expanded
    ///
    /// Read include given file's content as form of bufstream and doesn't 
    /// save to memory. Therefore cannot be used with macro definition.
    ///
    /// # Usage
    ///
    /// $read(path)
    fn read(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("read", AuthType::FIN,processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let raw = Utils::trim(&args[0]);
            let mut file_path = PathBuf::from(&raw);

            // if current input is not stdin and file path is relative
            // Create new file path that starts from current file path
            if processor.state.current_input != "stdin" && file_path.is_relative() {
                // It is ok get parent because any path that has a length can return parent
                file_path = PathBuf::from(&processor.state.current_input).parent().unwrap().join(file_path);
            }

            if file_path.is_file() { 
                processor.set_sandbox();
                processor.from_file(file_path)?;
                Ok(None)
            } else {
                let formatted = format!("File path : \"{}\" doesn't exist or not a file", file_path.display());
                Err(RadError::InvalidArgument(formatted))
            }
        } else {
            Err(RadError::InvalidArgument("Include requires an argument".to_owned()))
        }
    }

    /// Repeat given expression about given amount times
    ///
    /// # Usage
    ///
    /// $repeat(count,text)
    fn repeat(args: &str, greedy: bool,_: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let repeat_count;
            if let Ok(count) = Utils::trim(&args[0]).parse::<usize>() {
                repeat_count = count;
            } else {
                return Err(RadError::InvalidArgument(format!("Repeat needs a number bigger or equal to 0 (unsigned integer) but given \"{}\"", &args[0])));
            }
            let repeat_object = &args[1];
            let mut repeated = String::new();
            for _ in 0..repeat_count {
                repeated.push_str(&repeat_object);
            }
            Ok(Some(repeated))
        } else {
            Err(RadError::InvalidArgument("Repeat requires two arguments".to_owned()))
        }
    }

    /// Call system command
    ///
    /// This calls via 'CMD \C' in windows platform while unix call is operated without any mediation.
    ///
    /// Syscmd is always greedy
    ///
    /// # Usage
    ///
    /// $syscmd(system command -a arguments)
    fn syscmd(args: &str, _: bool,p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("syscmd", AuthType::CMD,p)? {
            return Ok(None);
        }
        if let Some(args_content) = ArgParser::new().args_with_len(args, 1, true) {
            let source = &args_content[0];
            let arg_vec = source.split(' ').collect::<Vec<&str>>();

            let output = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .arg("/C")
                    .args(arg_vec)
                    .output()
                    .expect("failed to execute process")
                    .stdout
            } else {
                let sys_args = if arg_vec.len() > 1 { &arg_vec[1..] } else { &[] };
                Command::new(&arg_vec[0])
                    .args(sys_args)
                    .output()
                    .expect("failed to execute process")
                    .stdout
            };

            Ok(Some(String::from_utf8(output)?))
        } else {
            Err(RadError::InvalidArgument("Syscmd requires an argument".to_owned()))
        }
    }

    /// Undefine a macro 
    ///
    /// 'Define' and 'BR' cannot be undefined because it is not actually a macro 
    ///
    /// # Usage
    ///
    /// $undef(macro_name)
    fn undefine_call(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let name = Utils::trim(&args[0]);

            let map = processor.get_map();
            if map.contains(&name) { 
                map.undefine(&name);
            } else {
                processor.log_error(&format!("Macro \"{}\" doesn't exist, therefore cannot undefine", name))?;
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("Undefine requires an argument".to_owned()))
        }
    }

    /// Placeholder for define
    fn define_type(_: &str, _: bool, _: &mut Processor) -> RadResult<Option<String>> { Ok(None) }

    /// Array
    ///
    /// # Usage
    ///
    /// $arr(1 2 3)
    fn array(args: &str, _: bool, _: &mut Processor) -> RadResult<Option<String>> {
        let parsed = ArgParser::new().args_to_vec(args, ',', GreedyState::Never);
        if parsed.len() == 0 {
            Err(RadError::InvalidArgument("Array requires an argument".to_owned()))
        } else {
            let separater = if parsed.len() >= 2 {
                &parsed[1] // Use given separater
            } else { " " }; // Use whitespace as default
            let mut vec = parsed[0].split(separater).collect::<Vec<&str>>();

            // Also given filter argument, then filter with regex expression
            if parsed.len() == 3 {
                let reg = Regex::new(&parsed[2])?;
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
    fn assert(args: &str, greedy: bool, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            if args[0] == args[1] {
                p.track_assertion(true)?;
                Ok(None)
            } else {
                p.track_assertion(false)?;
                Err(RadError::AssertFail)
            }
        } else {
            Err(RadError::InvalidArgument("Assert requires two arguments".to_owned()))
        }
    }

    /// Assert not equal
    ///
    /// # Usage
    ///
    /// $nassert(abc,abc)
    fn assert_ne(args: &str, greedy: bool, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            if args[0] != args[1] {
                p.track_assertion(true)?;
                Ok(None)
            } else {
                p.track_assertion(false)?;
                Err(RadError::AssertFail)
            }
        } else {
            Err(RadError::InvalidArgument("Assert_ne requires two arguments".to_owned()))
        }
    }

    /// Create multiple macro executions from given csv value
    ///
    /// # Usage
    ///
    /// $from(macro_name,\*1,2,3
    /// 4,5,6*\)
    ///
    /// $from+(macro_name,
    /// 1,2,3
    /// 4,5,6
    /// )
    #[cfg(feature = "csv")]
    fn from_data(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let macro_name = Utils::trim(&args[0]);
            // Trimming data might be very costly operation
            // Plus, it is already trimmed by csv crate.
            let macro_data = &args[1];

            let result = Formatter::csv_to_macros(&macro_name, macro_data, &processor.state.newline)?;

            // TODO 
            // This behaviour might can be improved
            // Disable debugging for nested macro expansion
            #[cfg(feature = "debug")]
            let original = processor.is_debug();

            // Now this might look strange, 
            // "Why not just enclose two lines with curly brackets?"
            // The answer is such appraoch somehow doesn't work.
            // Compiler cannot deduce the variable original and will yield error on
            // processor.debug(original)
            #[cfg(feature = "debug")]
            processor.set_debug(false);

            // Parse macros
            let result = processor.parse_chunk_args(0, "", &result)?;

            // Set custom prompt log to indicate user thatn from macro doesn't support
            // debugging inside macro expansion
            #[cfg(feature = "debug")]
            {
                use crate::debugger::DebugSwitch;
                processor.set_debug(original);
                match processor.get_debug_switch() {
                    DebugSwitch::StepMacro | DebugSwitch::NextMacro => {
                        processor.set_prompt_log("\"From macro\"")
                    }
                    _ => ()
                }
            }

            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument("From requires two arguments".to_owned()))
        }
    }

    /// Create a table with given format and csv input
    ///
    /// Available formats are 'github', 'wikitext' and 'html'
    ///
    /// # Usage
    ///
    /// $table(github,"1,2,3
    /// 4,5,6")
    #[cfg(feature = "csv")]
    fn table(args: &str, greedy: bool, p: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let table_format = &args[0]; // Either gfm, wikitex, latex, none
            let csv_content = &args[1];
            let result = Formatter::csv_to_table(table_format, csv_content, &p.state.newline)?;
            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument("Table requires two arguments".to_owned()))
        }
    }

    /// Put value into a temporary stack called pipe
    ///
    /// Piped value can be popped with macro '-'
    ///
    /// # Usage
    ///
    /// $pipe(Value)
    fn pipe(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
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
    fn pipe_to(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            processor.state.add_pipe(Some(&args[0]), args[1].to_owned());
        } else {
            return Err(RadError::InvalidArgument("pipeto requires two arguments".to_owned()));
        }
        Ok(None)
    }

    /// Get environment variable with given name
    ///
    /// # Usage
    ///
    /// $env(SHELL)
    fn get_env(args: &str, _: bool, p : &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("env", AuthType::ENV,p)? {
            return Ok(None);
        }
        if let Ok(out) = std::env::var(args) {
            Ok(Some(out))
        } else { 
            if p.state.strict {
                p.log_warning(&format!("Env : \"{}\" is not defined.", args))?;
            }
            Ok(None) 
        }
    }

    /// Set environment variable with given name
    ///
    /// # Usage
    ///
    /// $envset(SHELL,value)
    fn set_env(args: &str, greedy: bool, p : &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("envset", AuthType::ENV,p)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let name = &args[0];
            let value = &args[1];

            if p.state.strict && std::env::var(name).is_ok() {
                return Err(RadError::InvalidArgument(format!("You cannot override environment variable in strict mode. Failed to set \"{}\"", name)));
            }

            std::env::set_var(name, value);
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("Envset requires two arguments".to_owned()))
        }
    }

    /// Trigger panic
    fn manual_panic(args: &str, _: bool, _: &mut Processor) -> RadResult<Option<String>> {
        Err(RadError::ManualPanic(args.to_string()))
    }

    /// Escape processing
    fn escape(_: &str, _: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        processor.state.flow_control= FlowControl::Escape;
        Ok(None)
    }

    /// Exit processing
    fn exit(_: &str, _: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        processor.state.flow_control= FlowControl::Exit;
        Ok(None)
    }

    /// Merge multiple paths into a single path
    ///
    /// This creates platform agonistic path which can be consumed by other macros.
    ///
    /// # Usage
    ///
    /// $path($env(HOME),document,test.docx)
    fn merge_path(args: &str, _: bool, _: &mut Processor) -> RadResult<Option<String>> {
        let vec = ArgParser::new().args_to_vec(args, ',', GreedyState::Never);

        let out = vec
            .iter()
            .map(|s| Utils::trim(s))
            .collect::<PathBuf>();

        if let Some(value) = out.to_str() {
            Ok(Some(value.to_owned()))
        } else {
            Err(RadError::InvalidArgument(format!("Invalid path : {}", out.display())))
        }
    }

    /// Yield newline according to platform or user option
    ///
    /// # Usage
    ///
    /// $nl()
    fn newline(_: &str, _: bool, p: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(p.state.newline.to_owned()))
    }
    
    /// deny new line
    ///
    /// # Usage
    ///
    /// $dnl()
    fn deny_newline(_: &str, _: bool, p: &mut Processor) -> RadResult<Option<String>> {
        p.state.deny_newline = true;
        Ok(None)
    }

    /// Get name from given path
    ///
    /// # Usage
    ///
    /// $name(path/file.exe)
    fn get_name(args: &str, _: bool, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, false) {

            let path = Path::new(&args[0]);

            if let Some(name) = path.file_name() {
                if let Some(value) = name.to_str() {
                    return Ok(Some(value.to_owned()));
                }
            } 
            Err(RadError::InvalidArgument(format!("Invalid path : {}", path.display())))
        } else {
            Err(RadError::InvalidArgument("name requires an argument".to_owned()))
        }
    }

    /// Get absolute path from given path
    ///
    /// # Usage
    ///
    /// $abs(../canonic_path.txt)
    fn absolute_path(args: &str, _: bool, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("abs", AuthType::FIN, p)? {
            return Ok(None);
        }

        if let Some(args) = ArgParser::new().args_with_len(args, 1, false) {
            let path = Path::new(&args[0]);
            let canonic = std::fs::canonicalize(path)?.to_str().unwrap().to_owned();
            Ok(Some(canonic))
        } else {
            Err(RadError::InvalidArgument("Abs requires an argument".to_owned()))
        }
    }

    /// Get parent from given path
    ///
    /// # Usage
    ///
    /// $parent(path/file.exe)
    fn get_parent(args: &str, _: bool, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, false) {

            let path = Path::new(&args[0]);

            if let Some(name) = path.parent() {
                if let Some(value) = name.to_str() {
                    return Ok(Some(value.to_owned()));
                }
            } 
            Err(RadError::InvalidArgument(format!("Invalid path : {}", path.display())))
        } else {
            Err(RadError::InvalidArgument("parent requires an argument".to_owned()))
        }
    }

    /// Get pipe value
    ///
    /// # Usage
    ///
    /// $-()
    /// $-(p1)
    fn get_pipe(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        let pipe = if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let name = Utils::trim(&args[0]);
            if name.is_empty() {
                let out = processor.state.get_pipe("-").unwrap_or(String::new()).clone();
                Some(out)
            } else {
                if let Some(pipe) = processor.state.get_pipe(&args[0]) {
                    Some(pipe.clone())
                } else {
                    None
                }
            }
        } else {
            // "-" Always exsit, thus safe to unwrap
            let out = processor.state.get_pipe("-").unwrap_or(String::new()).clone();
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
    fn len(args: &str, _: bool, _: &mut Processor) -> RadResult<Option<String>> {
        Ok(Some(args.chars().count().to_string()))
    }

    /// Rename macro rule to other name
    ///
    /// Define and BR can't be renamed.
    ///
    /// # Usage
    ///
    /// $rename(name,target)
    fn rename_call(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let target = &args[0];
            let new = &args[1];

            let map = processor.get_map();
            if map.contains(target) { 
                processor.get_map().rename(target, new);
            } else {
                processor.log_error(&format!("Macro \"{}\" doesn't exist, therefore cannot rename", target))?;
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument("Rename requires two arguments".to_owned()))
        }
    }

    /// Append content to a macro
    ///
    /// Only custom macros can be appended.
    ///
    /// # Usage
    ///
    /// $append(macro_name,Content)
    fn append(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let name = &args[0];
            let target = &args[1];
            let map = processor.get_map();
            if map.custom.contains_key(name) {
                map.append(name, target);
            } else {
                processor.log_error(&format!("Macro \"{}\" doesn't exist", name))?;
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument("Append requires two arguments".to_owned()))
        }
    }

    /// Translate given char aray into corresponding char array
    ///
    /// # Usage
    ///
    /// $tr(Source,abc,ABC)
    fn translate(args: &str, greedy: bool, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, greedy) {
            let mut source = args[0].clone();
            let target = &args[1].chars().collect::<Vec<char>>();
            let destination = &args[2].chars().collect::<Vec<char>>();

            if target.len() != destination.len() {
                return Err(RadError::InvalidArgument(format!("Tr's replacment should have same length of texts while given \"{:?}\" and \"{:?}\"", target, destination)));
            }

            for i in 0..target.len() {
                source = source.replace(target[i], &destination[i].to_string());
            }

            Ok(Some(source))
        } else {
            Err(RadError::InvalidArgument("Tr requires three arguments".to_owned()))
        }
    }

    /// Get a substring(indexed) from given source
    ///
    /// # Usage
    ///
    /// $sub(0,5,GivenString)
    fn substring(args: &str, greedy: bool, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, greedy) {
            let source = &args[2];

            let mut min: Option<usize> = None;
            let mut max: Option<usize> = None;

            let start = Utils::trim(&args[0]);
            let end = Utils::trim(&args[1]);

            if let Ok(num) = start.parse::<usize>() {
                min.replace(num);
            } else { 
                if start.len() != 0 {
                    return Err(RadError::InvalidArgument(format!("Sub's min value should be non zero positive integer or empty value but given \"{}\"", start))); 
                }
            }

            if let Ok(num) = end.parse::<usize>() {
                max.replace(num);
            } else { 
                if end.len() != 0 {
                    return Err(RadError::InvalidArgument(format!("Sub's max value should be non zero positive integer or empty value but given \"{}\"", end))); 
                }
            }

            Ok(Some(Utils::utf8_substring(source, min, max)))

        } else {
            Err(RadError::InvalidArgument("Sub requires three arguments".to_owned()))
        }
    }

    /// Save content to temporary file
    ///
    /// # Usage
    ///
    /// $tempout(Content)
    fn temp_out(args: &str, greedy: bool, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("tempout", AuthType::FOUT,p)? {
            return Ok(None);
        }

        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let content = &args[0];
            p.get_temp_file().write_all(content.as_bytes())?;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("Tempout requires an argument".to_owned()))
        }
    }

    /// Save content to a file
    ///
    /// # Usage
    ///
    /// $fileout(true,file_name,Content)
    fn file_out(args: &str, greedy: bool, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("fileout", AuthType::FOUT,p)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 3, greedy) {
            let truncate = &args[0];
            let file_name = &args[1];
            let content = &args[2];
            if let Ok(truncate) = Utils::is_arg_true(truncate) {
                let file = std::env::current_dir()?.join(file_name);
                let mut target_file; 
                if truncate {
                    target_file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(file)
                        .unwrap();
                    } else {

                        if !file.is_file() {
                            return Err(RadError::InvalidArgument(format!("Failed to read \"{}\". Fileout without truncate option needs exsiting file",file.display())));
                        }

                        target_file = OpenOptions::new()
                            .append(true)
                            .open(file)
                            .unwrap();
                }
                target_file.write_all(content.as_bytes())?;
                Ok(None)
            } else {
                Err(RadError::InvalidArgument(format!("Fileout requires either true/false or zero/nonzero integer but given \"{}\"", truncate)))
            }
        } else {
            Err(RadError::InvalidArgument("Fileout requires three argument".to_owned()))
        }
    }

    /// Get head of given text
    ///
    /// # Usage
    ///
    /// $head(2,Text To extract)
    fn head(args: &str, greedy: bool, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let count = &args[0].parse::<usize>().map_err(|_| RadError::InvalidArgument(format!("Head requires positive integer number but got \"{}\"", &args[0])))?;
            let content = &args[1];
            let length = *count.min(&content.len());

            Ok(Some(content[0..length].to_string()))
        } else {
            Err(RadError::InvalidArgument("head requires two argument".to_owned()))
        }
    }

    /// Get head of given text but for lines
    ///
    /// # Usage
    ///
    /// $headl(2,Text To extract)
    fn head_line(args: &str, greedy: bool, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let count = &args[0].parse::<usize>().map_err(|_| RadError::InvalidArgument(format!("Head requires positive integer number but got \"{}\"", &args[0])))?;
            let lines = Utils::full_lines(args[1].as_bytes())
                .map(|line| line.unwrap())
                .collect::<Vec<String>>();
            let length = *count.min(&lines.len());

            Ok(Some(lines[0..length].concat()))
        } else {
            Err(RadError::InvalidArgument("head requires two argument".to_owned()))
        }
    }

    /// Get tail of given text
    ///
    /// # Usage
    ///
    /// $tail(2,Text To extract)
    fn tail(args: &str, greedy: bool, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let count = &args[0].parse::<usize>().map_err(|_| RadError::InvalidArgument(format!("Head requires positive integer number but got \"{}\"", &args[0])))?;
            let content = &args[1];
            let length = *count.min(&content.len());

            Ok(Some(content[content.len()-length..content.len()].to_string()))
        } else {
            Err(RadError::InvalidArgument("tail requires two argument".to_owned()))
        }
    }

    /// Get tail of given text but for lines
    ///
    /// # Usage
    ///
    /// $taill(2,Text To extract)
    fn tail_line(args: &str, greedy: bool, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let count = &args[0].parse::<usize>().map_err(|_| RadError::InvalidArgument(format!("Head requires positive integer number but got \"{}\"", &args[0])))?;
            let lines = Utils::full_lines(args[1].as_bytes())
                .map(|line| line.unwrap())
                .collect::<Vec<String>>();
            let length = *count.min(&lines.len());

            Ok(Some(lines[lines.len()-length..lines.len()].concat()))
        } else {
            Err(RadError::InvalidArgument("taill requires two argument".to_owned()))
        }
    }

    /// Strip from given text
    ///
    /// # Usage
    ///
    /// $strip(2,head,Text To extract)
    /// $strip(2,tail,Text To extract)
    fn strip(args: &str, greedy: bool, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, greedy) {
            let count = &args[0].parse::<usize>().map_err(|_| RadError::InvalidArgument(format!("Head requires positive integer number but got \"{}\"", &args[0])))?;
            let variant = &args[1];
            let content = &args[2];
            let length = *count.min(&content.len());

            match variant.to_lowercase().as_str() {
                "head" => Ok(Some(content[length..].to_string())),
                "tail" => Ok(Some(content[..content.len() - length].to_string())),
                _ => return Err(RadError::InvalidArgument(format!("Strip reqruies either head or tail but given \"{}\"", variant))),
            }

        } else {
            Err(RadError::InvalidArgument("strip requires three argument".to_owned()))
        }
    }

    /// Strip lines from given text
    ///
    /// # Usage
    ///
    /// $stripl(2,Text To extract)
    fn strip_line(args: &str, greedy: bool, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, greedy) {
            let count = &args[0].parse::<usize>().map_err(|_| RadError::InvalidArgument(format!("Head requires positive integer number but got \"{}\"", &args[0])))?;
            let variant = &args[1];
            let lines = Utils::full_lines(args[2].as_bytes())
                .map(|line| line.unwrap())
                .collect::<Vec<String>>();
            let length = *count.min(&lines.len());

            match variant.to_lowercase().as_str() {
                "head" => Ok(Some(lines[length..].concat())),
                "tail" => Ok(Some(lines[..lines.len() - length].concat())),
                _ => return Err(RadError::InvalidArgument(format!("Stripl reqruies either head or tail but given \"{}\"", variant))),
            }
        } else {
            Err(RadError::InvalidArgument("head requires two argument".to_owned()))
        }
    }

    /// Include but for temporary file
    ///
    /// This reads file's content into memory. Use read macro if streamed write is needed.
    ///
    /// # Usage
    ///
    /// $tempin()
    fn temp_include(_: &str, _: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("tempin", AuthType::FIN,processor)? {
            return Ok(None);
        }
        let file = processor.get_temp_path().to_owned();
        processor.set_sandbox();
        let chunk = processor.from_file_as_chunk(&file)?;
        Ok(chunk)
    }

    /// Redirect all text into temporary file
    ///
    /// Every text including non macro calls are all sent to current temporary files.
    ///
    /// # Usage
    ///
    /// $redir(true) 
    /// $redir(false) 
    fn temp_redirect(args: &str, _: bool, p: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("redir", AuthType::FOUT,p)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1, false) {
            let toggle = if let Ok(toggle) = Utils::is_arg_true(&args[0]) { 
                toggle
            } else {
                return Err(RadError::InvalidArgument(format!("Redir's agument should be valid boolean value but given \"{}\"", &args[0])));
            };
            p.state.redirect = toggle;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("Redir requires an argument".to_owned()))
        }
    }

    /// Set temporary file
    ///
    /// # Usage
    ///
    /// $tempto(file_name)
    fn set_temp_target(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("tempto", AuthType::FOUT,processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            processor.set_temp_file(&PathBuf::from(std::env::temp_dir()).join(&args[0]));
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("Temp requires an argument".to_owned()))
        }
    }

    /// Enable hook
    ///
    /// * Usage
    ///
    /// $hookon(MacroType, macro_name)
    #[cfg(feature = "hook")]
    fn hook_enable(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let hook_type = HookType::from_str(&args[0])?;
            let index = &args[1] ;
            processor.hook_map.switch_hook(hook_type, index, true)?;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("hookon requires two arguments".to_owned()))
        }
    }

    /// Disable hook
    ///
    /// * Usage
    ///
    /// $hookoff(MacroType, macro_name)
    #[cfg(feature = "hook")]
    fn hook_disable(args: &str, greedy: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let hook_type = HookType::from_str(&args[0])?;
            let index = &args[1] ;
            processor.hook_map.switch_hook(hook_type, index, false)?;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("hookoff requires two arguments".to_owned()))
        }
    }

    /// Wrap text
    ///
    /// * Usage
    ///
    /// $wrap(80, Content goes here)
    #[cfg(feature = "textwrap")]
    fn wrap(args: &str, _: bool, _: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, true) {
            let width = Utils::trim(&args[0]).parse::<usize>()?;
            let content = &args[1];
            let result = textwrap::fill(content, width);
            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument("Wrap requires two arguments".to_owned()))
        }
    }

    #[cfg(feature = "storage")]
    fn update_storage(args: &str, _: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        let args = ArgParser::new().args_to_vec(args, ',', GreedyState::Never);

        // Execute update method for storage
        if let Some(storage) = processor.storage.as_mut() {
            if let Err(err) = storage.update(&args) {
                return Err(RadError::StorageError(format!("Update error : {}", err)));
            }
        } else { 
            processor.log_warning("Empty storage, update didn't triggerd")?;
        }
        Ok(None)
    }

    #[cfg(feature = "storage")]
    fn extract_storage(_: &str, _: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        // Execute update method for storage
        if let Some(storage) = processor.storage.as_mut() {
            match storage.extract(false) {
                Err(err) => Err(RadError::StorageError(format!("Update error : {}", err))),
                Ok(value) => {
                    if let Some(output) = value {
                        Ok(Some(output.into_printable()))
                    } else { Ok(None) }
                },
            }
        } else { Err(RadError::StorageError(String::from("Empty storage"))) }
    }

    #[cfg(feature = "cindex")]
    fn cindex_register(args: &str, _: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, true) {
            processor.indexer.add_table_fast(&args[0], args[1].as_bytes())?;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("regcsv requires two arguments".to_owned()))
        }
    }

    #[cfg(feature = "cindex")]
    fn cindex_query(args: &str, _: bool, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, true) {
            let mut value = String::new();
            processor.indexer.index_raw(&Utils::trim(&args[0]), OutOption::Value(&mut value))?;
            Ok(Some(Utils::trim(&value)))
        } else {
            Err(RadError::InvalidArgument("query requires an argument".to_owned()))
        }
    }
}

// TODO
// Curently implementation declard logic and signatrue separately.
// Is this ideal?
// Or the whole process should be automated?
// Though I dought the possibility of automation because each logic is so relaxed and hardly follow
// any concrete rules
/// Basic Macro signature
#[derive(Clone)]
pub(crate) struct BMacroSign {
    name: String,
    args: Vec<String>,
    pub logic: MacroType,
}

impl BMacroSign {
    pub fn new(name: &str, args: impl IntoIterator<Item = impl AsRef<str>>, logic: MacroType) -> Self {
        let args = args.into_iter().map(|s| s.as_ref().to_owned()).collect::<Vec<String>>();
        Self {
            name : name.to_owned(),
            args,
            logic,
        }
    }
}

impl std::fmt::Display for BMacroSign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut inner = self.args.iter().fold(String::new(),|acc, arg| acc + &arg + ",");
        // This removes last "," character
        inner.pop();
        write!(f,"${}({})", self.name, inner)
    }
}

#[cfg(feature = "signature")]
impl From<&BMacroSign> for crate::sigmap::MacroSignature {
    fn from(bm: &BMacroSign) -> Self {
        Self {
            variant: crate::sigmap::MacroVariant::Basic,
            name: bm.name.to_owned(),
            args: bm.args.to_owned(),
            expr: bm.to_string(),
        }
    }
}
