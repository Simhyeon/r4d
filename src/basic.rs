//! # Basic module
//!
//! Basic module includes struct and methods related to basic macros which are technically function
//! pointers.

use std::array::IntoIter;
use std::io::Write;
use std::fs::OpenOptions;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{PathBuf,Path};
use std::process::Command;
use crate::error::RadError;
use crate::arg_parser::{ArgParser, GreedyState};
use regex::Regex;
use crate::utils::Utils;
use crate::processor::Processor;
use crate::auth::AuthType;
#[cfg(feature = "csv")]
use crate::formatter::Formatter;
#[cfg(feature = "lipsum")]
use lipsum::lipsum;

/// Type signature of basic macros
///
/// This is in order of args, greediness, processor's mutable reference
///
/// # Example
///
/// ```rust
/// fn demo(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
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
pub type MacroType = fn(&str, bool ,&mut Processor) -> Result<Option<String>, RadError>;

#[derive(Clone)]
pub struct BasicMacro {
    macros : HashMap<String, MacroType>,
}

impl BasicMacro {
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
            ("-".to_owned(),       BasicMacro::get_pipe         as MacroType),
            ("append".to_owned(),  BasicMacro::append           as MacroType),
            ("bind".to_owned(),    BasicMacro::bind_to_local    as MacroType),
            ("chomp".to_owned(),   BasicMacro::chomp            as MacroType),
            ("comp".to_owned(),    BasicMacro::compress         as MacroType),
            ("env".to_owned(),     BasicMacro::get_env          as MacroType),
            ("fileout".to_owned(), BasicMacro::file_out         as MacroType),
            ("global".to_owned(),  BasicMacro::global           as MacroType),
            ("include".to_owned(), BasicMacro::include          as MacroType),
            ("len".to_owned(),     BasicMacro::len              as MacroType),
            ("name".to_owned(),    BasicMacro::get_name         as MacroType),
            ("not".to_owned(),     BasicMacro::not              as MacroType),
            ("nl".to_owned(),      BasicMacro::newline          as MacroType),
            ("parent".to_owned(),  BasicMacro::get_parent       as MacroType),
            ("path".to_owned(),    BasicMacro::merge_path       as MacroType),
            ("paths".to_owned(),   BasicMacro::merge_path_vec   as MacroType),
            ("pipe".to_owned(),    BasicMacro::pipe             as MacroType),
            ("read".to_owned(),    BasicMacro::read             as MacroType),
            ("redir".to_owned(),   BasicMacro::temp_redirect    as MacroType),
            ("regex".to_owned(),   BasicMacro::regex_sub        as MacroType),
            ("rename".to_owned(),  BasicMacro::rename_call      as MacroType),
            ("repeat".to_owned(),  BasicMacro::repeat           as MacroType),
            ("repl".to_owned(),    BasicMacro::replace          as MacroType),
            ("sub".to_owned(),     BasicMacro::substring        as MacroType),
            ("syscmd".to_owned(),  BasicMacro::syscmd           as MacroType),
            ("tempin".to_owned(),  BasicMacro::temp_include     as MacroType),
            ("tempout".to_owned(), BasicMacro::temp_out         as MacroType),
            ("tempto".to_owned(),  BasicMacro::set_temp_target  as MacroType),
            ("tr".to_owned(),      BasicMacro::translate        as MacroType),
            ("trim".to_owned(),    BasicMacro::trim             as MacroType),
            ("undef".to_owned(),   BasicMacro::undefine_call    as MacroType),
        ]));
        
        // Optional macros
        #[cfg(feature = "csv")]
        {
            map.insert("from".to_owned(), BasicMacro::from_data as MacroType);
            map.insert("table".to_owned(), BasicMacro::table    as MacroType);
        }
        #[cfg(feature = "chrono")]
        {
            map.insert("time".to_owned(), BasicMacro::time      as MacroType);
            map.insert("date".to_owned(), BasicMacro::date      as MacroType);
        }
        #[cfg(feature = "lipsum")]
        map.insert("lipsum".to_owned(), BasicMacro::placeholder as MacroType);
        #[cfg(feature = "evalexpr")]
        map.insert("eval".to_owned(), BasicMacro::eval          as MacroType);

        // Return struct
        Self { macros : map }
    }

    /// Add new basic rule
    pub fn add_new_rule(&mut self, name: &str, macro_ref: MacroType) {
        self.macros.insert(name.to_owned(), macro_ref);
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
    pub fn get(&self, name: &str) -> Option<&MacroType> {
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
    fn time(_: &str, _: bool, _ : &mut Processor) -> Result<Option<String>, RadError> {
        Ok(Some(format!("{}", chrono::offset::Local::now().format("%H:%M:%S"))))
    }

    /// Print out current date
    ///
    /// # Usage
    ///
    /// $date()
    #[cfg(feature = "chrono")]
    fn date(_: &str, _: bool, _ : &mut Processor) -> Result<Option<String>, RadError> {
        Ok(Some(format!("{}", chrono::offset::Local::now().format("%Y-%m-%d"))))
    }

    /// Substitute the given source with following match expressions
    ///
    /// # Usage
    ///
    /// $regex(source_text,regex_match,substitution)
    fn regex_sub(args: &str, greedy: bool, _: &mut Processor) -> Result<Option<String>, RadError> {
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
    fn eval(args: &str, greedy: bool,_: &mut Processor ) -> Result<Option<String>, RadError> {
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
    fn not(args: &str, greedy: bool,_: &mut Processor ) -> Result<Option<String>, RadError> {
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
    fn trim(args: &str, greedy: bool, _: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            Ok(Some(Utils::trim(&args[0])))
        } else {
            Err(RadError::InvalidArgument("Trim requires an argument".to_owned()))
        }
    }

    /// Removes duplicate newlines whithin given input
    ///
    /// # Usage
    ///
    /// $chomp(expression)
    fn chomp(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let source = &args[0];
            let reg = Regex::new(&format!(r"{0}\s*{0}", &processor.newline))?;
            let result = reg.replace_all(source, &format!("{0}{0}", &processor.newline));

            Ok(Some(result.to_string()))
        } else {
            Err(RadError::InvalidArgument("Chomp requires an argument".to_owned()))
        }
    }

    /// Both apply trim and chomp to given expression
    ///
    /// # Usage
    ///
    /// $comp(Expression)
    fn compress(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let source = &args[0];
            // Chomp and then compress
            let result = Utils::trim(&BasicMacro::chomp(source,greedy, processor)?.unwrap());

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
    fn placeholder(args: &str, greedy: bool,_: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let word_count = &args[0];
            if let Ok(count) = Utils::trim(word_count).parse::<usize>() {
                Ok(Some(lipsum(count)))
            } else {
                Err(RadError::InvalidArgument(format!("Lipsum needs a number bigger or equal to 0 (unsigned integer) but given \"{}\"", word_count)))
            }
        } else {
            Err(RadError::InvalidArgument("Placeholder requires an argument".to_owned()))
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
    fn include(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
        if !Utils::is_granted("include", AuthType::FIN,processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let raw = Utils::trim(&args[0]);
            let mut file_path = PathBuf::from(&raw);

            // if current input is not stdin and file path is relative
            // Create new file path that starts from current file path
            if processor.current_input != "stdin" && file_path.is_relative() {
                // It is ok get parent because any path that has a length can return parent
                file_path = PathBuf::from(&processor.current_input).parent().unwrap().join(file_path);
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
    fn read(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
        if !Utils::is_granted("read", AuthType::FIN,processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let raw = Utils::trim(&args[0]);
            let mut file_path = PathBuf::from(&raw);

            // if current input is not stdin and file path is relative
            // Create new file path that starts from current file path
            if processor.current_input != "stdin" && file_path.is_relative() {
                // It is ok get parent because any path that has a length can return parent
                file_path = PathBuf::from(&processor.current_input).parent().unwrap().join(file_path);
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
    fn repeat(args: &str, greedy: bool,_: &mut Processor) -> Result<Option<String>, RadError> {
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
    fn syscmd(args: &str, _: bool,p: &mut Processor) -> Result<Option<String>, RadError> {
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
    fn undefine_call(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
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
    fn from_data(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let macro_name = Utils::trim(&args[0]);
            // Trimming data might be very costly operation
            // Plus, it is already trimmed by csv crate.
            let macro_data = &args[1];

            let result = Formatter::csv_to_macros(&macro_name, macro_data, &processor.newline)?;
            // This is necessary
            let result = processor.parse_chunk_args(0, "", &result)?;
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
    fn table(args: &str, greedy: bool, p: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let table_format = &args[0]; // Either gfm, wikitex, latex, none
            let csv_content = &args[1];
            let result = Formatter::csv_to_table(table_format, csv_content, &p.newline)?;
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
    fn pipe(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            processor.pipe_value = args[0].to_owned();
        }
        Ok(None)
    }

    /// Bind a global macro
    ///
    /// # Usage
    ///
    /// $global(name,value)
    fn global(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let name = &args[0];
            let value = &args[1];
            if processor.get_map().contains(name) {
                processor.log_warning(&format!("Creating a global with a name already existing : \"{}\"", name))?;
            }
            processor.add_custom_rules(vec![(name,"",value)]);
        }
        Ok(None)
    }

    /// Bind a local macro
    ///
    /// Bound macro gets deleted after macro execution
    ///
    /// # Usage
    ///
    /// $bind(name,value)
    fn bind_to_local(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let name = &args[0];
            let value = &args[1];
            processor.get_map().new_local(1, name, value);
            if processor.get_map().contains(name) {
                processor.log_warning(&format!("Creating a binding with a name already existing : \"{}\"", name))?;
            }
        }
        Ok(None)
    }

    /// Get environment variable with given name
    ///
    /// # Usage
    ///
    /// $env(SHELL)
    fn get_env(args: &str, _: bool, p : &mut Processor) -> Result<Option<String>, RadError> {
        if !Utils::is_granted("env", AuthType::ENV,p)? {
            return Ok(None);
        }
        if let Ok(out) = std::env::var(args) {
            Ok(Some(out))
        } else { 
            if p.strict {
                p.log_warning(&format!("Env : \"{}\" is not defined.", args))?;
            }
            Ok(None) 
        }
    }

    /// Merge two path into a single path
    ///
    /// This creates platform agonistic path which can be consumed by other macros.
    ///
    /// # Usage
    ///
    /// $path($env(HOME),document)
    fn merge_path(args: &str, greedy: bool, _: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let target = Utils::trim(&args[0]);
            let added = Utils::trim(&args[1]);

            let out = format!("{}",&std::path::Path::new(&target).join(&added).display());
            Ok(Some(out))
        } else {
            Err(RadError::InvalidArgument("Path macro needs two arguments".to_owned()))
        }
    }

    /// Merge multiple paths into a single path
    ///
    /// This creates platform agonistic path which can be consumed by other macros.
    ///
    /// # Usage
    ///
    /// $paths($env(HOME) document test.docx)
    fn merge_path_vec(args: &str, _: bool, _: &mut Processor) -> Result<Option<String>, RadError> {
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
    fn newline(_: &str, _: bool, p: &mut Processor) -> Result<Option<String>, RadError> {
        Ok(Some(p.newline.to_owned()))
    }

    /// Get name from given path
    ///
    /// # Usage
    ///
    /// $name(path/file.exe)
    fn get_name(args: &str, _: bool, _: &mut Processor) -> Result<Option<String>, RadError> {
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

    /// Get parent from given path
    ///
    /// # Usage
    ///
    /// $parent(path/file.exe)
    fn get_parent(args: &str, _: bool, _: &mut Processor) -> Result<Option<String>, RadError> {
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
    fn get_pipe(_: &str, _: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
        let out = processor.pipe_value.clone();
        Ok(Some(out))
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
    fn len(args: &str, _: bool, _: &mut Processor) -> Result<Option<String>, RadError> {
        Ok(Some(args.chars().count().to_string()))
    }

    /// Rename macro rule to other name
    ///
    /// Define and BR can't be renamed.
    ///
    /// # Usage
    ///
    /// $rename(name,target)
    fn rename_call(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
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
    fn append(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
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
    fn translate(args: &str, greedy: bool, _: &mut Processor) -> Result<Option<String>, RadError> {
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
    fn substring(args: &str, greedy: bool, _: &mut Processor) -> Result<Option<String>, RadError> {
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
    fn temp_out(args: &str, greedy: bool, p: &mut Processor) -> Result<Option<String>, RadError> {
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
    fn file_out(args: &str, greedy: bool, p: &mut Processor) -> Result<Option<String>, RadError> {
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

    /// Include but for temporary file
    ///
    /// This reads file's content into memory. Use read macro if streamed write is needed.
    ///
    /// # Usage
    ///
    /// $tempin()
    fn temp_include(_: &str, _: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
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
    fn temp_redirect(args: &str, _: bool, p: &mut Processor) -> Result<Option<String>, RadError> {
        if !Utils::is_granted("redir", AuthType::FOUT,p)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 1, false) {
            let toggle = if let Ok(toggle) =Utils::is_arg_true(&args[0]) { 
                toggle
            } else {
                return Err(RadError::InvalidArgument(format!("Redir's agument should be valid boolean value but given \"{}\"", &args[0])));
            };
            p.redirect = toggle;
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
    fn set_temp_target(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
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

    /// Replace value
    ///
    /// # Usage
    ///
    /// $repl(macro,value)
    fn replace(args: &str, greedy: bool, processor: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let name = args[0].as_str();
            let target = args[1].as_str();
            if !processor.get_map().replace(name, target) {
                return Err(RadError::InvalidArgument(format!("{} doesn't exist, thus cannot replace it's content", name)))
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("Replace requires two arguments".to_owned()))
        }
    }
}
