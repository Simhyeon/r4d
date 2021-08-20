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

type MacroType = fn(&str, &mut Processor) -> Result<String, RadError>;

#[derive(Clone)]
pub struct BasicMacro<'a> {
    macros : HashMap<&'a str, MacroType>,
}

impl<'a> BasicMacro<'a> {
    pub fn new() -> Self {
        // Create hashmap of functions
        let map = HashMap::from_iter(IntoIter::new([
            ("rsub", BasicMacro::regex_sub as MacroType),
            ("rdel", BasicMacro::regex_del as MacroType),
            ("eval", BasicMacro::eval as MacroType),
            ("trim", BasicMacro::trim as MacroType),
            ("chomp", BasicMacro::chomp as MacroType),
            ("comp", BasicMacro::compress as MacroType),
            ("lipsum", BasicMacro::placeholder as MacroType),
            ("time", BasicMacro::time as MacroType),
            ("date", BasicMacro::date as MacroType),
            ("include", BasicMacro::include as MacroType),
            ("repeat", BasicMacro::repeat as MacroType),
            ("syscmd", BasicMacro::syscmd as MacroType),
            ("ifelse", BasicMacro::ifelse as MacroType),
            ("ifdef", BasicMacro::ifdef as MacroType),
            ("foreach", BasicMacro::foreach as MacroType),
            ("forloop", BasicMacro::forloop as MacroType),
            ("undef", BasicMacro::undef as MacroType),
            ("from", BasicMacro::from_data as MacroType),
            ("table", BasicMacro::table as MacroType),
        ]));
        // Return struct
        Self {  macros : map}
    }

    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    pub fn unset(&mut self, name: &str) {
        self.macros.remove(name);
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
            Err(RadError::InvalidArgument("Regex del requires an argument"))
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
    fn syscmd(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        let args = &processor.parse_chunk(
            1000, 
            &MAIN_CALLER.to_owned(), 
            args
        )?;

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
                Command::new("sh")
                    .arg("-c")
                    .args(arg_vec)
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
                return Err(RadError::InvalidArgument("Ifelse requires true/fals or zero/nonzero."))
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
            Err(RadError::InvalidArgument("Syscmd requires an argument"))
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
            Err(RadError::InvalidArgument("Syscmd requires an argument"))
        }
    }

    fn undef(args: &str, processor: &mut Processor) -> Result<String, RadError> {
        let args = &processor.parse_chunk(
            1000, 
            &MAIN_CALLER.to_owned(), 
            args
        )?;

        if let Some(args) = Utils::args_with_len(args, 1) {
            let name = &args[0];

            // Return true or false by the definition
            if processor.map.basic.contains(name) {
                processor.map.basic.unset(name);
            }
            if processor.map.custom.contains_key(name) {
                processor.map.custom.remove(name);
            }
            Ok("".to_owned())
        } else {
            Err(RadError::InvalidArgument("Syscmd requires an argument"))
        }
    }

    // $foreach()
    // $foreach($testo($_()),"a,b,c")
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
                sums.push_str(&processed.replace("$_", value));
            }
            Ok(sums)
        } else {
            Err(RadError::InvalidArgument("Foreach requires two argument"))
        }
    }

    // $forloop("1,5",$testo($_))
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
                sums.push_str(&processed.replace("$_", &value.to_string()));
            }

            Ok(sums)
        } else {
            Err(RadError::InvalidArgument("Foreach requires two argument"))
        }
    }

    // $from("1,2,3\n4,5,6", $_)
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

    fn len(args: &str, _: &mut Processor) -> Result<String, RadError> {

        Ok(String::new())
    }
    fn substring(args: &str, _: &mut Processor) -> Result<String, RadError> {Ok(String::new())}
    fn translate(args: &str, _: &mut Processor) -> Result<String, RadError> {Ok(String::new())}
    fn rename(args: &str, _: &mut Processor) -> Result<String, RadError> {Ok(String::new())}
    fn append(args: &str, _: &mut Processor) -> Result<String, RadError> {Ok(String::new())}
    fn print(args: &str, _: &mut Processor) -> Result<String, RadError> {Ok(String::new())}
    fn toggle(args: &str, _: &mut Processor) -> Result<String, RadError> {Ok(String::new())}
    fn temp_file(args: &str, _: &mut Processor) -> Result<String, RadError> {Ok(String::new())}
}
