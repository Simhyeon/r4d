use std::array::IntoIter;
use std::env::temp_dir;
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
use crate::formatter::Formatter;
use lipsum::lipsum;
use lazy_static::lazy_static;

// Args, greediness, processor
type MacroType = fn(&str, bool ,&mut Processor) -> Result<String, RadError>;

lazy_static!{
   pub static ref ITER: Regex = Regex::new(r"\$:").unwrap();
}

#[derive(Clone)]
pub struct BasicMacro {
    macros : HashMap<String, MacroType>,
}

impl BasicMacro {
    pub fn new() -> Self {
        // Create hashmap of functions
        let map = HashMap::from_iter(IntoIter::new([
            ("regex".to_owned(), BasicMacro::regex_sub as MacroType),
            ("eval".to_owned(), BasicMacro::eval as MacroType),
            ("trim".to_owned(), BasicMacro::trim as MacroType),
            ("chomp".to_owned(), BasicMacro::chomp as MacroType).to_owned(),
            ("comp".to_owned(), BasicMacro::compress as MacroType).to_owned(),
            ("lipsum".to_owned(), BasicMacro::placeholder as MacroType).to_owned(),
            ("time".to_owned(), BasicMacro::time as MacroType).to_owned(),
            ("date".to_owned(), BasicMacro::date as MacroType).to_owned(),
            ("include".to_owned(), BasicMacro::include as MacroType).to_owned(),
            ("repeat".to_owned(), BasicMacro::repeat as MacroType).to_owned(),
            ("syscmd".to_owned(), BasicMacro::syscmd as MacroType).to_owned(),
            ("ifelse".to_owned(), BasicMacro::ifelse as MacroType).to_owned(),
            ("ifdef".to_owned(), BasicMacro::ifdef as MacroType).to_owned(),
            ("foreach".to_owned(), BasicMacro::foreach as MacroType).to_owned(),
            ("forloop".to_owned(), BasicMacro::forloop as MacroType).to_owned(),
            ("undef".to_owned(), BasicMacro::undefine_call as MacroType).to_owned(),
            ("rename".to_owned(), BasicMacro::rename_call as MacroType).to_owned(),
            ("append".to_owned(), BasicMacro::append as MacroType).to_owned(),
            ("from".to_owned(), BasicMacro::from_data as MacroType).to_owned(),
            ("table".to_owned(), BasicMacro::table as MacroType).to_owned(),
            ("len".to_owned(), BasicMacro::len as MacroType).to_owned(),
            ("tr".to_owned(), BasicMacro::translate as MacroType).to_owned(),
            ("sub".to_owned(), BasicMacro::substring as MacroType).to_owned(),
            ("pause".to_owned(), BasicMacro::pause as MacroType).to_owned(),
            ("tempto".to_owned(), BasicMacro::set_temp_target as MacroType).to_owned(),
            ("tempout".to_owned(), BasicMacro::temp as MacroType).to_owned(),
            ("tempin".to_owned(), BasicMacro::temp_include as MacroType).to_owned(),
            ("fileout".to_owned(), BasicMacro::file_out as MacroType).to_owned(),
            ("pipe".to_owned(), BasicMacro::pipe as MacroType).to_owned(),
            ("env".to_owned(), BasicMacro::get_env as MacroType).to_owned(),
            ("path".to_owned(), BasicMacro::merge_path as MacroType).to_owned(),
            ("-".to_owned(), BasicMacro::get_pipe as MacroType).to_owned(),
        ]));
        // Return struct
        Self { macros : map }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    pub fn call(&mut self, name : &str, args: &str,greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        let args = self.parse_inner(processor, args)?;
        if let Some(func) = self.macros.get(name) {
            // Print out macro call result
            let result = func(&args, greedy, processor)?;
            Ok(result)
        } else {
            Ok(String::new())
        }
    }

    pub fn undefine(&mut self, name: &str) {
        self.macros.remove(name);
    }

    pub fn rename(&mut self, name: &str, target: &str) {
        let func = self.macros.remove(name).unwrap();
        self.macros.insert(target.to_owned(), func);
    }

    fn parse_inner(&mut self,processor : &mut Processor, target: &str) -> Result<String, RadError> {
        processor.parse_chunk(0, &MAIN_CALLER.to_owned(), target)
    }

    // ==========
    // Basic Macros
    // ==========
    /// $time()
    fn time(_: &str, _: bool, _ : &mut Processor) -> Result<String, RadError> {
        Ok(format!("{}", chrono::offset::Local::now().format("%H:%M:%S")))
    }

    /// $date()
    fn date(_: &str, _: bool, _ : &mut Processor) -> Result<String, RadError> {
        Ok(format!("{}", chrono::offset::Local::now().format("%Y-%m-%d")))
    }

    /// $regex(source_text,regex_match,substitution)
    fn regex_sub(args: &str, greedy: bool, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 3, greedy) {
            let source= &args[0];
            let match_expr= &args[1];
            let substitution= &args[2];

            // This is regex expression without any preceding and trailing commands
            let reg = Regex::new(&format!(r"{}", match_expr))?;
            let result = reg.replace_all(source, substitution); // This is a cow, moo~
            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Regex sub requires three arguments"))
        }
    }

    /// $eval(expression)
    /// This returns true, false or evaluated number
    fn eval(args: &str, greedy: bool,_: &mut Processor ) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 1, greedy) {
            let formula = &args[0];
            let result = evalexpr::eval(formula)?;
            // TODO
            // Enable floating points length (or something similar)
            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Eval requires an argument"))
        }
    }

    // Trim preceding and trailing whitespaces
    /// $trim(text)
    fn trim(args: &str, greedy: bool, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 1, greedy) {
            Utils::trim(&args[0])
        } else {
            Err(RadError::InvalidArgument("Trim requires an argument"))
        }
    }

    // Remove duplicate newlines
    /// $chomp(test)
    fn chomp(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 1, greedy) {
            let source = &args[0];
            let reg = Regex::new(&format!(r"{0}\s*{0}", &processor.newline))?;
            let result = reg.replace_all(source, &format!("{0}{0}", &processor.newline));

            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Chomp requires an argument"))
        }
    }

    /// $comp(text)
    fn compress(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 1, greedy) {
            let source = &args[0];
            // Chomp and then compress
            let result = Utils::trim(&BasicMacro::chomp(source,greedy, processor)?)?;

            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Compress requires an argument"))
        }
    }

    /// $lipsum(Number: usize)
    fn placeholder(args: &str, greedy: bool,_: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 1, greedy) {
            let word_count = &args[0];
            if let Ok(count) = Utils::trim(word_count)?.parse::<usize>() {
                Ok(lipsum(count))
            } else {
                Err(RadError::InvalidArgument("Lipsum needs a number bigger or equal to 0 (unsigned integer)"))
            }
        } else {
            Err(RadError::InvalidArgument("Placeholder requires an argument"))
        }
    }

    /// $include(path)
    fn include(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 1, greedy) {
            let raw = Utils::trim(&args[0])?;
            let file_path = std::path::Path::new(&raw);
            Ok(processor.from_file(file_path, true)?)
        } else {
            Err(RadError::InvalidArgument("Include requires an argument"))
        }
    }

    /// $repeat(count: usize,text)
    fn repeat(args: &str, greedy: bool,_: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 2, greedy) {
            let repeat_count;
            if let Ok(count) = Utils::trim(&args[0])?.parse::<usize>() {
                repeat_count = count;
            } else {
                return Err(RadError::InvalidArgument("Repeat needs a number bigger or equal to 0 (unsigned integer)"));
            }
            let repeat_object = &args[1];
            let mut repeated = String::new();
            for _ in 0..repeat_count {
                repeated.push_str(&repeat_object);
            }
            Ok(repeated)
        } else {
            Err(RadError::InvalidArgument("Repeat requires two arguments"))
        }
    }

    /// $syscmd(system command -a arguments)
    fn syscmd(args: &str, greedy: bool,_: &mut Processor) -> Result<String, RadError> {
        if let Some(args_content) = ArgParser::args_with_len(args, 1, greedy) {
            let source = &args_content[0];
            let arg_vec = ArgParser::args_to_vec(&source, ' ', GreedyState::None);

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
            Err(RadError::InvalidArgument("Syscmd requires an argument"))
        }
    }

    // Special macro
    // Argument is expanded after vectorization
    /// $ifelse(evaluation, ifstate, elsestate)
    fn ifelse(args: &str, greedy: bool, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 2, greedy) {
            let boolean = &args[0];
            let if_state = &args[1];

            // Given condition is true
            let trimmed_cond = Utils::trim(boolean)?;
            if let Ok(cond) = trimmed_cond.parse::<bool>() {
                if cond { return Ok(if_state.to_owned()); }
            } else if let Ok(number) = trimmed_cond.parse::<i32>() {
                if number != 0 { return Ok(if_state.to_owned()); }
            } else {
                return Err(RadError::InvalidArgument("Ifelse requires either true/false or zero/nonzero integer."))
            }
            // if else statement exsits
            if args.len() >= 3 {
                let else_state = &args[2];
                return Ok(else_state.to_owned());
            }

            Ok(String::new())
        } else {
            Err(RadError::InvalidArgument("ifelse requires an argument"))
        }
    }

    // This is composite basic macro
    // Which means this macro acts differently by the context(Processor state)
    /// $ifdef(macro_name) 
    /// This return string true or false
    fn ifdef(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 1, greedy) {
            let name = &Utils::trim(&args[0])?;
            let map = processor.get_map();

            // Return true or false by the definition
            if map.basic.contains(name) || map.custom.contains_key(name) {
                Ok("true".to_owned())
            } else {
                Ok("false".to_owned())
            }
        } else {
            Err(RadError::InvalidArgument("Ifdef requires an argument"))
        }
    }

    /// $undef(macro_name)
    fn undefine_call(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 1, greedy) {
            let name = &Utils::trim(&args[0])?;

            processor.map.undefine(name);
            Ok("".to_owned())
        } else {
            Err(RadError::InvalidArgument("Undefine requires an argument"))
        }
    }

    // $foreach()
    // $foreach(\*a,b,c*\,$:)
    fn foreach(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 2, greedy) {
            let mut sums = String::new();
            let target = &args[1]; // evaluate on loop
            let loopable = &args[0];

            for value in loopable.split(',') {
                let result = processor.parse_chunk(0, &MAIN_CALLER.to_owned(), &ITER.replace_all(target, value))?;
                sums.push_str(&result);
            }
            Ok(sums)
        } else {
            Err(RadError::InvalidArgument("Foreach requires two argument"))
        }
    }

    // $forloop(1,5,$:)
    fn forloop(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 3, greedy) {
            let mut sums = String::new();
            let expression = &args[2]; // evaluate on loop

            let min: usize; 
            let max: usize; 
            if let Ok(num) = Utils::trim(&args[0])?.parse::<usize>() {
                min = num;
            } else { return Err(RadError::InvalidArgument("Forloop's min value should be non zero positive integer")); }
            if let Ok(num) = Utils::trim(&args[1])?.parse::<usize>() {
                max = num
            } else { return Err(RadError::InvalidArgument("Forloop's min value should be non zero positive integer")); }

            for value in min..=max {
                let result = processor.parse_chunk(0, &MAIN_CALLER.to_owned(), &ITER.replace_all(expression, &value.to_string()))?;
                sums.push_str(&result);
            }

            Ok(sums)
        } else {
            Err(RadError::InvalidArgument("Forloop requires two argument"))
        }
    }

    // $from(\*1,2,3\n4,5,6*\, macro_name)
    fn from_data(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 2, greedy) {
            let macro_data = &args[0];
            let macro_name = &Utils::trim(&args[1])?;

            let result = Formatter::csv_to_macros(macro_name, macro_data, &processor.newline)?;
            // This is necessary
            let result = processor.parse_chunk(0, &MAIN_CALLER.to_owned(), &result)?;
            Ok(result)
        } else {
            Err(RadError::InvalidArgument("From requires two arguments"))
        }
    }

    /// $table(github,"1,2,3\n4,5,6")
    fn table(args: &str, greedy: bool, p: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 2, greedy) {
            let table_format = &args[0]; // Either gfm, wikitex, latex, none
            let csv_content = &args[1];
            let result = Formatter::csv_to_table(table_format, csv_content, &p.newline)?;
            Ok(result)
        } else {
            Err(RadError::InvalidArgument("Table requires two arguments"))
        }
    }

    /// $pipe(Value)
    fn pipe(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 1, greedy) {
            processor.pipe_value = args[0].to_owned();
        }
        Ok(String::new())
    }

    fn get_env(args: &str, _: bool, _: &mut Processor) -> Result<String, RadError> {
        let out = std::env::var(args)?;
        Ok(out)
    }

    fn merge_path(args: &str, greedy: bool, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 2, greedy) {
            let target = Utils::trim(&args[0])?;
            let added = Utils::trim(&args[1])?;

            let out = format!("{}",&std::path::Path::new(&target).join(&added).display());
            Ok(out)
        } else {
            Err(RadError::InvalidArgument("Path macro needs two arguments"))
        }
    }

    /// $-()
    fn get_pipe(_: &str, _: bool, processor: &mut Processor) -> Result<String, RadError> {
        let out = processor.pipe_value.clone();
        processor.pipe_value.clear();
        Ok(out)
    }

    /// Return a length of the string
    /// This is O(n) operation
    /// String.len() function returns byte length not "Character" length
    /// therefore, chars().count() is used
    fn len(args: &str, _: bool, _: &mut Processor) -> Result<String, RadError> {
        Ok(args.chars().count().to_string())
    }

    fn rename_call(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 2, greedy) {
            let target = &args[0];
            let new = &args[1];
            processor.map.rename(target, new);

            Ok(String::new())
        } else {
            Err(RadError::InvalidArgument("Rename requires two arguments"))
        }
    }

    fn append(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 2, greedy) {
            let name = &args[0];
            let target = &args[1];
            processor.map.append(name, target);

            Ok(String::new())
        } else {
            Err(RadError::InvalidArgument("Append requires two arguments"))
        }
    }

    fn translate(args: &str, greedy: bool, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 3, greedy) {
            let mut source = args[0].clone();
            let target = &args[1].chars().collect::<Vec<char>>();
            let destination = &args[2].chars().collect::<Vec<char>>();

            if target.len() != destination.len() {
                return Err(RadError::InvalidArgument("Tr's replacment should have same length of texts"));
            }

            for i in 0..target.len() {
                source = source.replace(target[i], &destination[i].to_string());
            }

            Ok(source)
        } else {
            Err(RadError::InvalidArgument("Tr requires two arguments"))
        }
    }

    // $sub(0,5,GivenString)
    fn substring(args: &str, greedy: bool, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 2, greedy) {
            let source = &args[2];

            let mut min: Option<usize> = None;
            let mut max: Option<usize> = None;

            let start = Utils::trim(&args[0])?;
            let end = Utils::trim(&args[1])?;

            if let Ok(num) = start.parse::<usize>() {
                min.replace(num);
            } else { 
                if start.len() != 0 {
                    return Err(RadError::InvalidArgument("Sub's min value should be non zero positive integer or empty value")); 
                }
            }

            if let Ok(num) = end.parse::<usize>() {
                max.replace(num);
            } else { 
                if end.len() != 0 {
                    return Err(RadError::InvalidArgument("Sub's max value should be non zero positive integer or empty value")); 
                }
            }

            Ok(Utils::utf8_substring(source, min, max))

        } else {
            Err(RadError::InvalidArgument("Sub requires some arguments"))
        }
    }
    fn pause(args: &str, greedy: bool, processor : &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 1, greedy) {
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
                Err(RadError::InvalidArgument("Pause requires either true/false or zero/nonzero integer."))
            }
        } else {
            Err(RadError::InvalidArgument("Pause requires an argument"))
        }
    }

    fn temp(args: &str, greedy: bool, p: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 2, greedy) {
            let truncate = &args[0];
            let content = &args[1];
            if let Ok(truncate) = Utils::is_arg_true(truncate) {
                let file = temp_dir().join(&p.temp_target);
                let mut temp_file; 
                if truncate {
                    temp_file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(file)
                        .unwrap();
                } else {
                    temp_file = OpenOptions::new()
                        .append(true)
                        .open(file)
                        .unwrap();
                }
                temp_file.write_all(content.as_bytes())?;
                Ok(String::new())
            } else {
                Err(RadError::InvalidArgument("Temp requires either true/false or zero/nonzero integer."))
            }
        } else {
            Err(RadError::InvalidArgument("Temp requires an argument"))
        }
    }

    fn file_out(args: &str, greedy: bool, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 3, greedy) {
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
                Err(RadError::InvalidArgument("Temp requires either true/false or zero/nonzero integer."))
            }
        } else {
            Err(RadError::InvalidArgument("Temp requires an argument"))
        }
    }

    fn temp_include(_: &str, _: bool, processor: &mut Processor) -> Result<String, RadError> {
        let file_path = temp_dir().join(&processor.temp_target);
        Ok(processor.from_file(&file_path, true)?)
    }

    fn set_temp_target(args: &str, greedy: bool, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = ArgParser::args_with_len(args, 1, greedy) {
            processor.temp_target = PathBuf::from(std::env::temp_dir()).join(&args[0]);
            Ok(String::new())
        } else {
            Err(RadError::InvalidArgument("Temp requires an argument"))
        }
    }
}
