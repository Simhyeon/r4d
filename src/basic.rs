use std::array::IntoIter;
use std::io::Write;
use std::fs::OpenOptions;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::process::Command;
use crate::error::RadError;
use crate::arg_parser::{ArgParser, GreedyState};
use crate::consts::MAIN_CALLER;
use regex::Regex;
use crate::utils::Utils;
use crate::processor::Processor;
#[cfg(feature = "csv")]
use crate::formatter::Formatter;
#[cfg(feature = "lipsum")]
use lipsum::lipsum;

// Args, greediness, processor
type MacroType = fn(&str, bool ,&mut Processor) -> Result<String, RadError>;

#[derive(Clone)]
pub struct BasicMacro {
    macros : HashMap<String, MacroType>,
}

impl BasicMacro {
    /// Creates new basic macro hashmap
    pub fn new() -> Self {
        // Create hashmap of functions
        let mut map = HashMap::from_iter(IntoIter::new([
            ("regex".to_owned(), BasicMacro::regex_sub as MacroType),
            ("trim".to_owned(), BasicMacro::trim as MacroType),
            ("chomp".to_owned(), BasicMacro::chomp as MacroType).to_owned(),
            ("comp".to_owned(), BasicMacro::compress as MacroType).to_owned(),
            ("include".to_owned(), BasicMacro::include as MacroType).to_owned(),
            ("repeat".to_owned(), BasicMacro::repeat as MacroType).to_owned(),
            ("syscmd".to_owned(), BasicMacro::syscmd as MacroType).to_owned(),
            ("if".to_owned(), BasicMacro::if_cond as MacroType).to_owned(),
            ("ifelse".to_owned(), BasicMacro::ifelse as MacroType).to_owned(),
            ("ifdef".to_owned(), BasicMacro::ifdef as MacroType).to_owned(),
            ("foreach".to_owned(), BasicMacro::foreach as MacroType).to_owned(),
            ("forloop".to_owned(), BasicMacro::forloop as MacroType).to_owned(),
            ("undef".to_owned(), BasicMacro::undefine_call as MacroType).to_owned(),
            ("rename".to_owned(), BasicMacro::rename_call as MacroType).to_owned(),
            ("append".to_owned(), BasicMacro::append as MacroType).to_owned(),
            ("len".to_owned(), BasicMacro::len as MacroType).to_owned(),
            ("tr".to_owned(), BasicMacro::translate as MacroType).to_owned(),
            ("sub".to_owned(), BasicMacro::substring as MacroType).to_owned(),
            ("pause".to_owned(), BasicMacro::pause as MacroType).to_owned(),
            ("tempto".to_owned(), BasicMacro::set_temp_target as MacroType).to_owned(),
            ("tempout".to_owned(), BasicMacro::temp_out as MacroType).to_owned(),
            ("tempin".to_owned(), BasicMacro::temp_include as MacroType).to_owned(),
            ("redir".to_owned(), BasicMacro::temp_redirect as MacroType).to_owned(),
            ("fileout".to_owned(), BasicMacro::file_out as MacroType).to_owned(),
            ("pipe".to_owned(), BasicMacro::pipe as MacroType).to_owned(),
            ("bind".to_owned(), BasicMacro::bind_to_local as MacroType).to_owned(),
            ("env".to_owned(), BasicMacro::get_env as MacroType).to_owned(),
            ("path".to_owned(), BasicMacro::merge_path as MacroType).to_owned(),
            ("nl".to_owned(), BasicMacro::newline as MacroType).to_owned(),
            ("-".to_owned(), BasicMacro::get_pipe as MacroType).to_owned(),
        ]));
        
        // Optional macros
        #[cfg(feature = "csv")]
        {
            map.insert("from".to_owned(), BasicMacro::from_data as MacroType);
            map.insert("table".to_owned(), BasicMacro::table as MacroType);
        }
        #[cfg(feature = "chrono")]
        {
            map.insert("time".to_owned(), BasicMacro::time as MacroType);
            map.insert("date".to_owned(), BasicMacro::date as MacroType);
        }
        #[cfg(feature = "lipsum")]
        map.insert("lipsum".to_owned(), BasicMacro::placeholder as MacroType);
        #[cfg(feature = "evalexpr")]
        map.insert("eval".to_owned(), BasicMacro::eval as MacroType);

        // Return struct
        Self { macros : map }
    }

    /// Check if a given macro exists
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the macro to find
    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// Call a macro
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the macro to call
    /// * `args` - Argument sto supply to macro
    /// * `greedy` - Whether macro should interpret arguments greedily
    /// * `processor` - Processor instance to execute macro
    pub fn call(&mut self, name : &str, args: &str,greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        // TODO
        // Check if this code is necessary
        if let Some(func) = self.macros.get(name) {
            // Print out macro call result
            let result = func(args, greedy, processor)?;
            Ok(result)
        } else {
            Ok(String::new())
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
    fn time(_: &str, _: bool, _ : &mut Processor) -> Result<String, RadError> {
        Ok(format!("{}", chrono::offset::Local::now().format("%H:%M:%S")))
    }

    /// Print out current date
    ///
    /// # Usage
    ///
    /// $date()
    #[cfg(feature = "chrono")]
    fn date(_: &str, _: bool, _ : &mut Processor) -> Result<String, RadError> {
        Ok(format!("{}", chrono::offset::Local::now().format("%Y-%m-%d")))
    }

    /// Substitute the given source with following match expressions
    ///
    /// # Usage
    ///
    /// $regex(source_text,regex_match,substitution)
    fn regex_sub(args: &str, greedy: bool, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, greedy) {
            let source= &args[0];
            let match_expr= &args[1];
            let substitution= &args[2];

            // This is regex expression without any preceding and trailing commands
            let reg = Regex::new(&format!(r"{}", match_expr))?;
            let result = reg.replace_all(source, substitution); // This is a cow, moo~
            Ok(result.to_string())
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
    fn eval(args: &str, greedy: bool,_: &mut Processor ) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let formula = &args[0];
            let result = evalexpr::eval(formula)?;
            // TODO
            // Enable floating points length (or something similar)
            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Eval requires an argument".to_owned()))
        }
    }

    /// Trim preceding and trailing whitespaces (' ', '\n', '\t', '\r')
    ///
    /// # Usage
    ///
    /// $trim(expression)
    fn trim(args: &str, greedy: bool, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            Utils::trim(&args[0])
        } else {
            Err(RadError::InvalidArgument("Trim requires an argument".to_owned()))
        }
    }

    /// Removes duplicate newlines whithin given input
    ///
    /// # Usage
    ///
    /// $chomp(expression)
    fn chomp(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let source = &args[0];
            let reg = Regex::new(&format!(r"{0}\s*{0}", &processor.newline))?;
            let result = reg.replace_all(source, &format!("{0}{0}", &processor.newline));

            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Chomp requires an argument".to_owned()))
        }
    }

    /// Both apply trim and chomp to given expression
    ///
    /// # Usage
    ///
    /// $comp(Expression)
    fn compress(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let source = &args[0];
            // Chomp and then compress
            let result = Utils::trim(&BasicMacro::chomp(source,greedy, processor)?)?;

            Ok(result.to_string())
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
    fn placeholder(args: &str, greedy: bool,_: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let word_count = &args[0];
            if let Ok(count) = Utils::trim(word_count)?.parse::<usize>() {
                Ok(lipsum(count))
            } else {
                Err(RadError::InvalidArgument("Lipsum needs a number bigger or equal to 0 (unsigned integer)".to_owned()))
            }
        } else {
            Err(RadError::InvalidArgument("Placeholder requires an argument".to_owned()))
        }
    }

    /// Paste given file's content
    ///
    /// Every macros within the file is also expanded
    ///
    /// # Usage
    ///
    /// $include(path)
    fn include(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let raw = Utils::trim(&args[0])?;
            let file_path = std::path::Path::new(&raw);
            if file_path.exists() { 
                processor.set_sandbox();
                let result = processor.from_file(file_path)?;
                Ok(result)
            } else {
                let formatted = format!("File path : \"{}\" doesn't exist", file_path.display());
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
    fn repeat(args: &str, greedy: bool,_: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let repeat_count;
            if let Ok(count) = Utils::trim(&args[0])?.parse::<usize>() {
                repeat_count = count;
            } else {
                return Err(RadError::InvalidArgument("Repeat needs a number bigger or equal to 0 (unsigned integer)".to_owned()));
            }
            let repeat_object = &args[1];
            let mut repeated = String::new();
            for _ in 0..repeat_count {
                repeated.push_str(&repeat_object);
            }
            Ok(repeated)
        } else {
            Err(RadError::InvalidArgument("Repeat requires two arguments".to_owned()))
        }
    }

    /// Call system command
    ///
    /// This calls via 'CMD \C' in windows platform while unix call is operated without any mediation.
    ///
    /// # Usage
    ///
    /// $syscmd(system command -a arguments)
    fn syscmd(args: &str, greedy: bool,_: &mut Processor) -> Result<String, RadError> {
        if let Some(args_content) = ArgParser::new().args_with_len(args, 1, greedy) {
            let source = &args_content[0];
            let arg_vec = ArgParser::new().args_to_vec(&source, ' ', GreedyState::None);

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

            Ok(String::from_utf8(output)?)
        } else {
            Err(RadError::InvalidArgument("Syscmd requires an argument".to_owned()))
        }
    }

    /// Print content according to given condition
    /// 
    /// # Usage 
    ///
    /// $if(evaluation, ifstate)
    fn if_cond(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let boolean = &args[0];
            let if_state = &args[1];

            // Given condition is true
            let trimmed_cond = Utils::trim(boolean)?;
            if let Ok(cond) = trimmed_cond.parse::<bool>() {
                if cond { 
                    let result = processor.parse_chunk_args(0, &MAIN_CALLER.to_owned(), if_state)?;
                    return Ok(result); 
                }
            } else if let Ok(number) = trimmed_cond.parse::<i32>() {
                if number != 0 { 
                    let result = processor.parse_chunk_args(0, &MAIN_CALLER.to_owned(), if_state)?;
                    return Ok(result); 
                }
            } else {
                return Err(RadError::InvalidArgument("If requires either true/false or zero/nonzero integer.".to_owned()))
            }

            Ok(String::new())
        } else {
            Err(RadError::InvalidArgument("if requires two arguments".to_owned()))
        }
    }

    /// Print content according to given condition
    /// 
    /// # Usage 
    ///
    /// $ifelse(evaluation, ifstate, elsestate)
    fn ifelse(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, greedy) {
            let boolean = &args[0];
            let if_state = &args[1];

            // Given condition is true
            let trimmed_cond = Utils::trim(boolean)?;
            if let Ok(cond) = trimmed_cond.parse::<bool>() {
                if cond { 
                    let result = processor.parse_chunk_args(0, &MAIN_CALLER.to_owned(), if_state)?;
                    return Ok(result); 
                }
            } else if let Ok(number) = trimmed_cond.parse::<i32>() {
                if number != 0 { 
                    let result = processor.parse_chunk_args(0, &MAIN_CALLER.to_owned(), if_state)?;
                    return Ok(result); 
                }
            } else {
                return Err(RadError::InvalidArgument("Ifelse requires either true/false or zero/nonzero integer.".to_owned()))
            }

            // Else state
            let else_state = &args[2];
            let result = processor.parse_chunk_args(0, &MAIN_CALLER.to_owned(), else_state)?;
            return Ok(result);
        } else {
            Err(RadError::InvalidArgument("ifelse requires three argument".to_owned()))
        }
    }

    /// Check if macro is defined or not
    ///
    /// This return 'true' or 'false'
    ///
    /// # Usage
    ///
    /// $ifdef(macro_name) 
    fn ifdef(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let name = &Utils::trim(&args[0])?;
            let map = processor.get_map();

            // Return true or false by the definition
            if map.basic.contains(name) || map.custom.contains_key(name) {
                Ok("true".to_owned())
            } else {
                Ok("false".to_owned())
            }
        } else {
            Err(RadError::InvalidArgument("Ifdef requires an argument".to_owned()))
        }
    }

    /// Undefine a macro 
    ///
    /// 'Define' cannot be undefined because it is not actually a macro 
    ///
    /// # Usage
    ///
    /// $undef(macro_name)
    fn undefine_call(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let name = &Utils::trim(&args[0])?;

            processor.get_map().undefine(name);
            Ok("".to_owned())
        } else {
            Err(RadError::InvalidArgument("Undefine requires an argument".to_owned()))
        }
    }

    /// Loop around given values and substitute iterators  with the value
    ///
    /// # Usage 
    ///
    /// $foreach(\*a,b,c*\,$:)
    fn foreach(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let mut sums = String::new();
            let target = &args[1]; // evaluate on loop
            let loopable = &args[0];

            for value in loopable.split(',') {
                let result = processor.parse_chunk_args(0, &MAIN_CALLER.to_owned(), &target.replace("$:", value))?;
                sums.push_str(&result);
            }
            Ok(sums)
        } else {
            Err(RadError::InvalidArgument("Foreach requires two argument".to_owned()))
        }
    }

    /// For loop around given min, max value and finally substitue iterators with value
    ///
    /// # Usage
    ///
    /// $forloop(1,5,$:)
    fn forloop(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, greedy) {
            let mut sums = String::new();
            let expression = &args[2]; // evaluate on loop

            let min: usize; 
            let max: usize; 
            if let Ok(num) = Utils::trim(&args[0])?.parse::<usize>() {
                min = num;
            } else { return Err(RadError::InvalidArgument("Forloop's min value should be non zero positive integer".to_owned())); }
            if let Ok(num) = Utils::trim(&args[1])?.parse::<usize>() {
                max = num
            } else { return Err(RadError::InvalidArgument("Forloop's min value should be non zero positive integer".to_owned())); }
            
            for value in min..=max {
                let result = processor.parse_chunk_args(0, &MAIN_CALLER.to_owned(), &expression.replace("$:", &value.to_string()))?;
                sums.push_str(&result);
            }

            Ok(sums)
        } else {
            Err(RadError::InvalidArgument("Forloop requires two argument".to_owned()))
        }
    }

    /// Create multiple macro executions from given csv value
    ///
    /// # Usage
    ///
    /// $from(\*1,2,3
    /// 4,5,6*\, macro_name)
    #[cfg(feature = "csv")]
    fn from_data(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let macro_data = &args[0];
            let macro_name = &Utils::trim(&args[1])?;

            let result = Formatter::csv_to_macros(macro_name, macro_data, &processor.newline)?;
            // This is necessary
            let result = processor.parse_chunk_args(0, &MAIN_CALLER.to_owned(), &result)?;
            Ok(result)
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
    fn table(args: &str, greedy: bool, p: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let table_format = &args[0]; // Either gfm, wikitex, latex, none
            let csv_content = &args[1];
            let result = Formatter::csv_to_table(table_format, csv_content, &p.newline)?;
            Ok(result)
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
    fn pipe(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            processor.pipe_value = args[0].to_owned();
        }
        Ok(String::new())
    }

    /// Bind a local macro
    ///
    /// Bound macro gets deleted after macro execution
    ///
    /// # Usage
    ///
    /// $bind(name,value)
    fn bind_to_local(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let name = &args[0];
            let value = &args[1];
            processor.get_map().new_local(1, name, value);
            if processor.get_map().custom.contains_key(name) {
                processor.log_warning(&format!("Creating a binding with a name already existing : \"{}\"", name))?;
            }
        }
        Ok(String::new())
    }

    /// Get environment variable with given name
    ///
    /// # Usage
    ///
    /// $env(SHELL)
    fn get_env(args: &str, _: bool, _: &mut Processor) -> Result<String, RadError> {
        let out = std::env::var(args)?;
        Ok(out)
    }

    /// Merge two path into a single path
    ///
    /// This creates platform agonistic path which can be consumed by other macros.
    ///
    /// # Usage
    ///
    /// $path($env(HOME),document)
    fn merge_path(args: &str, greedy: bool, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let target = Utils::trim(&args[0])?;
            let added = Utils::trim(&args[1])?;

            let out = format!("{}",&std::path::Path::new(&target).join(&added).display());
            Ok(out)
        } else {
            Err(RadError::InvalidArgument("Path macro needs two arguments".to_owned()))
        }
    }

    /// Yield newline according to platform or user option
    ///
    /// # Usage
    ///
    /// $nl()
    fn newline(_: &str, _: bool, p: &mut Processor) -> Result<String, RadError> {
        Ok(p.newline.to_owned())
    }

    /// Pop pipe value
    ///
    /// # Usage
    ///
    /// $-()
    fn get_pipe(_: &str, _: bool, processor: &mut Processor) -> Result<String, RadError> {
        let out = processor.pipe_value.clone();
        processor.pipe_value.clear();
        Ok(out)
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
    fn len(args: &str, _: bool, _: &mut Processor) -> Result<String, RadError> {
        Ok(args.chars().count().to_string())
    }

    /// Rename macro rule to other name
    ///
    /// # Usage
    ///
    /// $rename(name,target)
    fn rename_call(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let target = &args[0];
            let new = &args[1];
            processor.get_map().rename(target, new);

            Ok(String::new())
        } else {
            Err(RadError::InvalidArgument("Rename requires two arguments".to_owned()))
        }
    }

    /// Append content to a macro
    ///
    /// # Usage
    ///
    /// $append(macro_name,Content)
    fn append(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let name = &args[0];
            let target = &args[1];
            processor.get_map().append(name, target);

            Ok(String::new())
        } else {
            Err(RadError::InvalidArgument("Append requires two arguments".to_owned()))
        }
    }

    /// Translate given char aray into corresponding char array
    ///
    /// # Usage
    ///
    /// $tr(Source,abc,ABC)
    fn translate(args: &str, greedy: bool, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, greedy) {
            let mut source = args[0].clone();
            let target = &args[1].chars().collect::<Vec<char>>();
            let destination = &args[2].chars().collect::<Vec<char>>();

            if target.len() != destination.len() {
                return Err(RadError::InvalidArgument("Tr's replacment should have same length of texts".to_owned()));
            }

            for i in 0..target.len() {
                source = source.replace(target[i], &destination[i].to_string());
            }

            Ok(source)
        } else {
            Err(RadError::InvalidArgument("Tr requires two arguments".to_owned()))
        }
    }

    /// Get a substring(indexed) from given source
    ///
    /// # Usage
    ///
    /// $sub(0,5,GivenString)
    fn substring(args: &str, greedy: bool, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, greedy) {
            let source = &args[2];

            let mut min: Option<usize> = None;
            let mut max: Option<usize> = None;

            let start = Utils::trim(&args[0])?;
            let end = Utils::trim(&args[1])?;

            if let Ok(num) = start.parse::<usize>() {
                min.replace(num);
            } else { 
                if start.len() != 0 {
                    return Err(RadError::InvalidArgument("Sub's min value should be non zero positive integer or empty value".to_owned())); 
                }
            }

            if let Ok(num) = end.parse::<usize>() {
                max.replace(num);
            } else { 
                if end.len() != 0 {
                    return Err(RadError::InvalidArgument("Sub's max value should be non zero positive integer or empty value".to_owned())); 
                }
            }

            Ok(Utils::utf8_substring(source, min, max))

        } else {
            Err(RadError::InvalidArgument("Sub requires some arguments".to_owned()))
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
    fn pause(args: &str, greedy: bool, processor : &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let arg = &args[0];
            if let Ok(value) =Utils::is_arg_true(arg) {
                if value {
                    processor.paused = true;
                } else {
                    processor.paused = false;
                }
                Ok(String::new())
            } 
            // Failed to evaluate
            else {
                Err(RadError::InvalidArgument("Pause requires either true/false or zero/nonzero integer.".to_owned()))
            }
        } else {
            Err(RadError::InvalidArgument("Pause requires an argument".to_owned()))
        }
    }

    /// Save content to temporary file
    ///
    /// # Usage
    ///
    /// $tempout(true,Content)
    fn temp_out(args: &str, greedy: bool, p: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            let content = &args[0];
            p.get_temp_file().write_all(content.as_bytes())?;
            Ok(String::new())
        } else {
            Err(RadError::InvalidArgument("Tempout requires an argument".to_owned()))
        }
    }

    /// Save content to a file
    ///
    /// # Usage
    ///
    /// $fileout(true,file_name,Content)
    fn file_out(args: &str, greedy: bool, _: &mut Processor) -> Result<String, RadError> {
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
                        target_file = OpenOptions::new()
                            .append(true)
                            .open(file)
                            .unwrap();
                }
                target_file.write_all(content.as_bytes())?;
                Ok(String::new())
            } else {
                Err(RadError::InvalidArgument("Temp requires either true/false or zero/nonzero integer.".to_owned()))
            }
        } else {
            Err(RadError::InvalidArgument("Temp requires an argument".to_owned()))
        }
    }

    /// Include but for temporary file
    ///
    /// # Usage
    ///
    /// $tempin()
    fn temp_include(_: &str, _: bool, processor: &mut Processor) -> Result<String, RadError> {
        let file = processor.get_temp_path().to_owned();
        processor.set_sandbox();
        Ok(processor.from_file(&file)?)
    }

    /// Redirect all text into temporary file
    ///
    /// Every text including non macro calls are all sent to current temporary files.
    ///
    /// # Usage
    ///
    /// $redir(true) 
    /// $redir(false) 
    fn temp_redirect(args: &str, _: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, false) {
            let toggle = if let Ok(toggle) =Utils::is_arg_true(&args[0]) { 
                toggle
            } else {
                return Err(RadError::InvalidArgument("Redir's agument should be valid boolean value".to_owned()));
            };
            processor.redirect = toggle;
            Ok(String::new())
        } else {
            Err(RadError::InvalidArgument("Redir requires an argument".to_owned()))
        }
    }

    /// Set temporary file
    ///
    /// # Usage
    ///
    /// $tempto(file_name)
    fn set_temp_target(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, greedy) {
            processor.set_temp_file(&PathBuf::from(std::env::temp_dir()).join(&args[0]));
            Ok(String::new())
        } else {
            Err(RadError::InvalidArgument("Temp requires an argument".to_owned()))
        }
    }
}
