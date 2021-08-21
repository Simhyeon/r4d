use std::array::IntoIter;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::process::Command;
use crate::error::RadError;
use crate::consts::MAIN_CALLER;
use regex::Regex;
use crate::utils::Utils;
use crate::processor::Processor;
use crate::formatter::Formatter;
use lipsum::lipsum;
use lazy_static::lazy_static;

type MacroType = fn(&str, &mut Processor) -> Result<String, RadError>;

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
            ("rsub".to_owned(), BasicMacro::regex_sub as MacroType),
            ("rdel".to_owned(), BasicMacro::regex_del as MacroType),
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
            ("-".to_owned(), BasicMacro::get_pipe as MacroType).to_owned(),
        ]));
        // Return struct
        Self { macros : map }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    pub fn call(&self, name : &str, args: &str, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(func) = self.macros.get(name) {
            // Print out macro call result
            let result = func(args, processor)?;
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

    // ==========
    // Basic Macros
    // ==========
    fn time(_: &str, _ : &mut Processor) -> Result<String, RadError> {
        Ok(format!("{}", chrono::offset::Local::now().format("%H:%M:%S")))
    }

    fn date(_: &str, _ : &mut Processor) -> Result<String, RadError> {
        Ok(format!("{}", chrono::offset::Local::now().format("%Y-%m-%d")))
    }

    fn regex_sub(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        let args = &processor.parse_chunk(
            1000, 
            &MAIN_CALLER.to_owned(), 
            args
        )?;

        if let Some(args) = Utils::args_with_len(args, 3) {
            let source= &args[0];
            let target= &args[1];
            let object= &args[2];

            // This is regex expression without any preceding and trailing commands
            let reg = Regex::new(&format!(r"{}", target))?;
            let result = reg.replace_all(source, object); // This is a cow, moo~
            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Regex sub requires three arguments"))
        }
    }

    fn regex_del(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        let args = &processor.parse_chunk(
            1000, 
            &MAIN_CALLER.to_owned(), 
            args
        )?;

        if let Some(args) = Utils::args_with_len(args, 2) {
            let source = &args[0];
            let target = &args[1];

            // This is regex expression without any preceding and trailing commands
            let reg = Regex::new(&format!(r"{}", target))?;
            let result = reg.replace_all(source, ""); // This is a cow, moo~, btw this replaces all match as empty character which technically deletes matches
            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Regex del requires two arguments"))
        }
    }

    fn eval(args: &str, processor: &mut Processor ) -> Result<String, RadError> {
        let args = &processor.parse_chunk(
            1000, 
            &MAIN_CALLER.to_owned(), 
            args
        )?;

        if let Some(args) = Utils::args_with_len(args, 1) {
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
    fn trim(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        let args = &processor.parse_chunk(
            1000, 
            &MAIN_CALLER.to_owned(), 
            args
        )?;

        Utils::trim(args)
    }

    // Remove duplicate newlines
    fn chomp(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        let args = &processor.parse_chunk(
            1000, 
            &MAIN_CALLER.to_owned(), 
            args
        )?;

        if let Some(args) = Utils::args_with_len(args, 1) {
            let source = &args[0];
            let reg = Regex::new(&format!(r"{0}\s*{0}", &processor.newline))?;
            let result = reg.replace_all(source, &format!("{0}{0}", &processor.newline));

            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Chomp requires an argument"))
        }
    }

    fn compress(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        let args = &processor.parse_chunk(
            1000, 
            &MAIN_CALLER.to_owned(), 
            args
        )?;

        if let Some(args) = Utils::args_with_len(args, 1) {
            let source = &args[0];
            // Chomp and then compress
            let result = Utils::trim(&BasicMacro::chomp(source, processor)?)?;

            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Compress requires an argument"))
        }
    }

    fn placeholder(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        let args = &processor.parse_chunk(
            1000, 
            &MAIN_CALLER.to_owned(), 
            args
        )?;

        if let Some(args) = Utils::args_with_len(args, 1) {
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

    fn include(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        let args = &processor.parse_chunk(
            1000, 
            &MAIN_CALLER.to_owned(), 
            args
        )?;

        if let Some(args) = Utils::args_with_len(args, 1) {
            let file_path = std::path::Path::new(&args[0]);
            Ok(processor.from_file(file_path, true)?)
        } else {
            Err(RadError::InvalidArgument("Include requires an argument"))
        }
    }

    fn repeat(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        let args = &processor.parse_chunk(
            1000, 
            &MAIN_CALLER.to_owned(), 
            args
        )?;

        if let Some(args) = Utils::args_with_len(args, 2) {
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

    // $syscmd(echo 'this is printed')
    fn syscmd(args: &str, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args_content) = Utils::args_with_len(args, 1) {
            let source = &args_content[0];
            let arg_vec = Utils::args_to_vec(&source, ' ', ('\'', '\''));

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
    // $ifelse(evaluation, ifstate, elsestate)
    fn ifelse(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 2) {
            let boolean = &args[0];
            let if_state = &processor.parse_chunk(
                1000, 
                &MAIN_CALLER.to_owned(), 
                &args[1]
            )?;

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
                let else_state = &processor.parse_chunk(
                    1000, 
                    &MAIN_CALLER.to_owned(), 
                    &args[2]
                )?;
                return Ok(else_state.to_owned());
            }

            Ok(String::new())
        } else {
            Err(RadError::InvalidArgument("ifelse requires an argument"))
        }
    }

    // This is composite basic macro
    // Which means this macro acts differently by the context(Processor state)
    // $ifdef(macro_name) -> return string true or false
    fn ifdef(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        let args = &processor.parse_chunk(
            1000, 
            &MAIN_CALLER.to_owned(), 
            args
        )?;

        if let Some(args) = Utils::args_with_len(args, 1) {
            let name = &args[0];
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

    fn undefine_call(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        let args = &processor.parse_chunk(
            1000, 
            &MAIN_CALLER.to_owned(), 
            args
        )?;

        if let Some(args) = Utils::args_with_len(args, 1) {
            let name = &args[0];

            processor.map.undefine(name);
            Ok("".to_owned())
        } else {
            Err(RadError::InvalidArgument("Undefine requires an argument"))
        }
    }

    // $foreach()
    // $foreach("a,b,c",$:)
    fn foreach(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 2) {
            let mut sums = String::new();
            let target = args[1].to_owned(); // evaluate on loop
            let loopable = &processor.parse_chunk(
                1000, 
                &MAIN_CALLER.to_owned(), 
                &args[0]
            )?;

            let processed = processor.parse_chunk(0, &MAIN_CALLER.to_owned(),&target)?;

            for value in loopable.split(',') {
                sums.push_str(&ITER.replace_all(&processed, value));
            }
            Ok(sums)
        } else {
            Err(RadError::InvalidArgument("Foreach requires two argument"))
        }
    }

    // $forloop("1,5",$:)
    fn forloop(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 2) {
            let mut sums = String::new();
            let target = args[1].to_owned(); // evaluate on loop

            let loopable = &processor.parse_chunk(
                1000, 
                &MAIN_CALLER.to_owned(), 
                &args[0]
            )?;
            let loopable = loopable.split(',').collect::<Vec<&str>>();

            if loopable.len() != 2 {
                RadError::InvalidArgument("Forloop's second argument should be quoted min,max value e.g \"2,5\"");
            }
            let min: usize; 
            let max: usize; 
            if let Ok(num) = Utils::trim(loopable[0])?.parse::<usize>() {
                min = num;
            } else { return Err(RadError::InvalidArgument("Forloop's min value should be non zero positive integer")); }
            if let Ok(num) = Utils::trim(loopable[1])?.parse::<usize>() {
                max = num
            } else { return Err(RadError::InvalidArgument("Forloop's min value should be non zero positive integer")); }

            let processed = processor.parse_chunk(0, &MAIN_CALLER.to_owned(), &target)?;

            for value in min..=max {
                sums.push_str(&ITER.replace_all(&processed, &value.to_string()));
            }

            Ok(sums)
        } else {
            Err(RadError::InvalidArgument("Forloop requires two argument"))
        }
    }

    // $from("1,2,3\n4,5,6", macro_name)
    fn from_data(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 2) {
            let macro_data = &args[0];
            let macro_name = &Utils::trim(&args[1])?;

            let result = Formatter::csv_to_macros(macro_name, macro_data, &processor.newline)?;
            let result = processor.parse_chunk(0, &MAIN_CALLER.to_owned(), &result)?;
            Ok(result)
        } else {
            Err(RadError::InvalidArgument("From requires two arguments"))
        }
    }

    fn table(args: &str, p: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 2) {
            let table_format = &args[0]; // Either gfm, wikitex, latex, none
            let csv_content = &args[1];
            let result = Formatter::csv_to_table(table_format, csv_content, &p.newline)?;
            Ok(result)
        } else {
            Err(RadError::InvalidArgument("Table requires two arguments"))
        }
    }

    fn get_pipe(_: &str, processor: &mut Processor) -> Result<String, RadError> {
        let out = processor.pipe_value.clone();
        processor.pipe_value.clear();
        Ok(out)
    }

    /// Return a length of the string
    /// This is O(n) operation
    /// String.len() function returns byte length not "Character" length
    /// therefore, chars().count() is used
    fn len(args: &str, _: &mut Processor) -> Result<String, RadError> {
        Ok(args.chars().count().to_string())
    }

    fn rename_call(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 2) {
            let target = &args[0];
            let new = &args[1];
            processor.map.rename(target, new);

            Ok(String::new())
        } else {
            Err(RadError::InvalidArgument("Rename requires two arguments"))
        }
    }

    fn append(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 2) {
            let name = &args[0];
            let target = &args[1];
            processor.map.append(name, target);

            Ok(String::new())
        } else {
            Err(RadError::InvalidArgument("Append requires two arguments"))
        }
    }
    fn translate(args: &str, _: &mut Processor) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 3) {
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

    fn substring(args: &str, _: &mut Processor) -> Result<String, RadError> {Ok(String::new())}
    fn print(args: &str, _: &mut Processor) -> Result<String, RadError> {Ok(String::new())}
    fn toggle(args: &str, _: &mut Processor) -> Result<String, RadError> {Ok(String::new())}
    fn temp_file(args: &str, _: &mut Processor) -> Result<String, RadError> {Ok(String::new())}
}
