use std::io::{self, Write};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::{ Path , PathBuf};
use crate::error::{RadError, ErrorLogger, LoggerLines};
use crate::models::{MacroMap, MacroRule, RuleFile, UnbalancedChecker, WriteOption};
use crate::utils::Utils;
use crate::consts::*;
use crate::lexor::*;
use crate::arg_parser::ArgParser;

#[derive(Debug)]
pub struct MacroFragment {
    pub whole_string: String,
    pub name: String,
    pub args: String,
    pub pipe: bool,
    pub greedy: bool,
    pub preceding: bool,
    pub yield_literal : bool,
    pub trimmed : bool,
}

impl MacroFragment {
    pub fn new() -> Self {
        MacroFragment {
            whole_string : String::new(),
            name : String::new(),
            args : String::new(),
            pipe: false,
            greedy: false,
            preceding: false,
            yield_literal : false,
            trimmed: false,
        }
    }

    pub fn clear(&mut self) {
        self.whole_string.clear();
        self.name.clear();
        self.args.clear();
        self.pipe = false; 
        self.greedy = false; 
        self.yield_literal = false;
        self.trimmed= false; 
    }

    pub fn is_empty(&self) -> bool {
        self.whole_string.len() == 0
    }
}

pub enum ParseResult {
    FoundMacro(String),
    Printable(String),
    NoPrint,
    EOI,
}

pub struct SandboxBackup {
    current_input: String,
    local_macro_map: HashMap<String,String>,
    logger_lines: LoggerLines,
}

pub struct Processor{
    current_input : String,
    pub(crate) map: MacroMap,
    define_parse: DefineParser,
    write_option: WriteOption,
    error_logger: ErrorLogger,
    checker: UnbalancedChecker,
    pub(crate) pipe_value: String,
    pub(crate) newline: String,
    pub(crate) paused: bool,
    pub(crate) redirect: bool,
    purge: bool,
    strict: bool,
    always_greedy: bool,
    temp_target: (PathBuf,File),
}
// 1. Get string
// 2. Parse until macro invocation detected
// 3. Return remainder and macro fragments
// 4. Continue parsing with fragments

impl Processor {
    // ----------
    // Builder pattern methods
    /// Creates new processor with deafult options
    pub fn new_proc() -> Self {
        let temp_path= std::env::temp_dir().join("rad.txt");
        let temp_target = (temp_path.to_owned(),OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)
            .unwrap());

        Self {
            current_input: String::from("stdin"),
            map : MacroMap::new(),
            write_option: WriteOption::Stdout,
            define_parse: DefineParser::new(),
            error_logger: ErrorLogger::new(Some(WriteOption::Stdout)),
            checker : UnbalancedChecker::new(),
            newline : LINE_ENDING.to_owned(),
            pipe_value: String::new(),
            paused: false,
            redirect: false,
            purge: false,
            strict: false,
            always_greedy: false,
            temp_target,
        }
    }

    /// Set write option to yield output to the file
    pub fn write_to_file(mut self, target_file: Option<&Path>) -> Result<Self, RadError> {
        if let Some(target_file) = target_file {
            let target_file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(target_file)?;

            self.write_option = WriteOption::File(target_file);
        }
        Ok(self)
    }

    /// Yield error to the file
    pub fn error_to_file(mut self, target_file: Option<&Path>) -> Result<Self, RadError> {
        if let Some(target_file) = target_file {
            let target_file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(target_file)?;

            self.error_logger = ErrorLogger::new(Some(WriteOption::File(target_file)));
        }
        Ok(self)
    }

    /// Use unix line ending instead of os default one
    pub fn unix_new_line(mut self, use_unix_new_line: bool) -> Self {
        if use_unix_new_line {
            self.newline = "\n".to_owned();
        }
        self
    }

    pub fn greedy(mut self, greedy: bool) -> Self {
        if greedy {
            self.always_greedy = true;
        }
        self
    }

    pub fn purge(mut self, purge: bool) -> Self {
        if purge {
            self.purge = true;
            self.strict = false;
        }
        self
    }

    pub fn strict(mut self, strict: bool) -> Self {
        if strict {
            self.strict = true;
            self.purge = false;
        }
        self
    }

    pub fn silent(mut self, silent: bool) -> Self {
        if silent {
            self.error_logger = ErrorLogger::new(None);
        }
        self
    }

    pub fn custom_rules(mut self, paths: Option<Vec<&Path>>) -> Result<Self, RadError> {
        if let Some(paths) = paths {
            let mut rule_file = RuleFile::new(None);
            for p in paths.iter() {
                rule_file.melt(*p)?;
            }
            self.map.custom.extend(rule_file.rules);
        }

        Ok(self)
    }

    // =========
    // -->> Old constructor
    pub fn new(write_option: WriteOption, error_write_option : Option<WriteOption>, newline: String) -> Self {
        let temp_path= std::env::temp_dir().join("rad.txt");
        let temp_target = (temp_path.to_owned(),OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)
            .unwrap());

        Self {
            current_input: String::from("stdin"),
            map : MacroMap::new(),
            write_option,
            define_parse: DefineParser::new(),
            error_logger: ErrorLogger::new(error_write_option),
            checker : UnbalancedChecker::new(),
            newline,
            pipe_value: String::new(),
            paused: false,
            redirect: false,
            purge: false,
            strict: false,
            always_greedy: false,
            temp_target,
        }
    }
    pub fn print_result(&mut self) -> Result<(), RadError> {
        self.error_logger.print_result()?;
        Ok(())
    }
    pub fn set_greedy(&mut self) {
        self.always_greedy = true;
    }
    pub fn set_purge(&mut self) {
        self.purge = true;
    }
    pub fn set_strict(&mut self) {
        self.strict = true;
    }
    pub fn get_map(&self) -> &MacroMap {
        &self.map
    }

    pub fn set_temp_file(&mut self, path: &Path) {
        self.temp_target = (path.to_owned(),OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap());
    }

    pub fn get_temp_path(&self) -> &Path {
        &self.temp_target.0
    }

    pub fn get_temp_file(&self) -> &File {
        &self.temp_target.1
    }

    /// Backup information of current file before processing sandboxed input
    fn backup(&self) -> SandboxBackup {
        SandboxBackup { 
            current_input: self.current_input.clone(), 
            local_macro_map: self.map.local.clone(),
            logger_lines: self.error_logger.backup_lines(),
        }
    }

    fn recover(&mut self, backup: SandboxBackup) {
        // NOTE ::: Set file should come first becuase set_file override line number and character
        // number
        self.error_logger.set_file(&backup.current_input);
        self.current_input = backup.current_input;
        self.map.local= backup.local_macro_map; 
        self.error_logger.recover_lines(backup.logger_lines);
    }

    /// Read from standard input
    pub fn from_stdin(&mut self, sandbox: bool) -> Result<String, RadError> {

        // Sandboxed environment, backup
        let backup = if sandbox { Some(self.backup()) } else { None };

        let stdin = io::stdin();
        let mut line_iter = Utils::full_lines(stdin.lock());
        let mut lexor = Lexor::new();
        let mut invoke = MacroFragment::new();
        let mut content = String::new();
        let mut container = if sandbox { Some(&mut content) } else { None };
        loop {
            self.error_logger.add_line_number();
            let result = self.parse_line(&mut line_iter, &mut lexor ,&mut invoke)?;
            // Clear local variable macros
            self.map.clear_local();
            match result {
                // This means either macro is not found at all
                // or previous macro fragment failed with invalid syntax
                ParseResult::Printable(remainder) => {
                    self.write_to(&remainder, &mut container)?;
                    // Reset fragment
                    if &invoke.whole_string != "" {
                        invoke = MacroFragment::new();
                    }
                }
                ParseResult::FoundMacro(remainder) => {
                    self.write_to(&remainder, &mut container)?;
                }
                ParseResult::NoPrint => (), // Do nothing
                // End of input, end loop
                ParseResult::EOI => break,
            }
        } // Loop end

        // Recover
        if let Some(backup) = backup { self.recover(backup); }

        Ok(content)
    }

    pub fn from_file(&mut self, path :&Path, sandbox: bool) -> Result<String, RadError> {

        // Sandboxed environment, backup
        let backup = if sandbox { Some(self.backup()) } else { None };

        // Set file as name of given path
        self.set_file(path.to_str().unwrap())?;

        let file_stream = File::open(path)?;
        let reader = io::BufReader::new(file_stream);
        let mut line_iter = Utils::full_lines(reader);
        let mut lexor = Lexor::new();
        let mut invoke = MacroFragment::new();
        let mut content = String::new();
        let mut container = if sandbox { Some(&mut content) } else { None };
        loop {
            self.error_logger.add_line_number();
            let result = self.parse_line(&mut line_iter, &mut lexor ,&mut invoke)?;
            // Clear local variable macros
            self.map.clear_local();
            match result {
                // This means either macro is not found at all
                // or previous macro fragment failed with invalid syntax
                ParseResult::Printable(remainder) => {
                    self.write_to(&remainder, &mut container)?;
                    // Reset fragment
                    if &invoke.whole_string != "" {
                        invoke = MacroFragment::new();
                    }
                }
                ParseResult::FoundMacro(remainder) => {
                    self.write_to(&remainder, &mut container)?;
                }
                ParseResult::NoPrint => (), // Do nothing
                // End of input, end loop
                ParseResult::EOI => break,
            }
        } // Loop end

        // Recover
        if let Some(backup) = backup { self.recover(backup); }

        Ok(content)
    }
    
    /// Parse line is called only by the main loop thus, caller name is special name of @MAIN@
    fn parse_line(&mut self, lines :&mut impl std::iter::Iterator<Item = std::io::Result<String>>, lexor : &mut Lexor ,frag : &mut MacroFragment) -> Result<ParseResult, RadError> {
        if let Some(line) = lines.next() {
            let line = line?;
            let remainder = self.parse(lexor, frag, &line, 0, MAIN_CALLER)?;

            // Non macro string is included
            if remainder.len() != 0 {
                // Fragment is not empty
                if !frag.is_empty() {
                    Ok(ParseResult::FoundMacro(remainder))
                } 
                // Print everything
                else {
                    Ok(ParseResult::Printable(remainder))
                }
            } 
            // Nothing to print
            else {
                Ok(ParseResult::NoPrint)
            }
        } else {
            Ok(ParseResult::EOI)
        }
    } // parse_line end

    /// Parse chunk is called by non-main process, thus needs caller
    pub fn parse_chunk(&mut self, level: usize, caller: &str, chunk: &str) -> Result<String, RadError> {
        let mut lexor = Lexor::new();
        let mut frag = MacroFragment::new();
        let mut result = self.parse(&mut lexor, &mut frag, chunk, level, caller)?;
        if !frag.is_empty() {
            result.push_str(&frag.whole_string);
        }
        return Ok(result);
    } // parse_chunk end

    fn parse(&mut self,lexor: &mut Lexor, frag: &mut MacroFragment, line: &str, level: usize, caller: &str) -> Result<String, RadError> {
        self.error_logger.reset_char_number();
        // Local values
        let mut remainder = String::new();

        // Reset lexor's escape_nl 
        lexor.escape_nl = false;
        for ch in line.chars() {
            self.error_logger.add_char_number();
            let lex_result = lexor.lex(ch)?;
            // Either add character to remainder or fragments
            match lex_result {
                LexResult::Ignore => frag.whole_string.push(ch),
                // If given result is literal
                LexResult::Literal(cursor) => {
                    match cursor {
                        // Exit frag
                        // If literal is given on names
                        Cursor::Name => {
                            frag.whole_string.push(ch);
                            remainder.push_str(&frag.whole_string);
                            frag.clear();
                        }
                        // Simply push if none or arg
                        Cursor::None => { remainder.push(ch); }
                        Cursor::Arg => { 
                            frag.args.push(ch); 
                            frag.whole_string.push(ch);
                        }
                    }
                }
                LexResult::StartFrag => {
                    frag.whole_string.push(ch);

                    // If paused and not pause, then reset lexor context
                    if self.paused && frag.name != "pause" {
                        lexor.reset();
                        remainder.push_str(&frag.whole_string);
                        frag.clear();
                    }
                },
                LexResult::EmptyName => {
                    frag.whole_string.push(ch);
                    // If paused, then reset lexor context
                    self.error_logger.freeze_number(); 
                    if self.paused {
                        lexor.reset();
                        remainder.push_str(&frag.whole_string);
                        frag.clear();
                    }
                }
                LexResult::AddToRemainder => {
                    if !self.checker.check(ch) {
                        self.error_logger.freeze_number();
                        self.log_warning("Unbalanced parenthesis detected.")?;
                    }
                    remainder.push(ch);
                }
                LexResult::AddToFrag(cursor) => {
                    match cursor{
                        Cursor::Name => {
                            if frag.name.len() == 0 {
                                self.error_logger.freeze_number();
                            }
                            match ch {
                                '|' => frag.pipe = true,
                                '+' => frag.greedy = true,
                                '*' => frag.yield_literal = true,
                                '^' => frag.trimmed = true,
                                _ => frag.name.push(ch) 
                            }
                        },
                        Cursor::Arg => {
                            frag.args.push(ch)
                        },
                        _ => unreachable!(),
                    } 
                    frag.whole_string.push(ch);
                }
                // End of fragment
                // 1. Evaluate macro 
                // 1.5 -> If define, parse rule applied again.
                // 2. And append to remainder
                // 3. Reset fragment
                LexResult::EndFrag => {
                    frag.whole_string.push(ch);
                    // define
                    if frag.name == "define" {
                        self.add_define(frag, &mut remainder)?;
                        lexor.escape_nl = true;
                        frag.clear()
                    } 
                    // Invoke macro
                    else {
                        // Try to evaluate
                        let evaluation_result = self.evaluate(level, caller, &frag.name, &frag.args, frag.greedy || self.always_greedy);

                        // If panicked, this means unrecoverable error occured.
                        if let Err(error) = evaluation_result {
                            // this is equlvalent to conceptual if let not pattern
                            if let RadError::Panic = error{
                                // Do nothing
                                ();
                            } else {
                                self.log_error(&format!("{}", error))?;
                            }
                            return Err(RadError::Panic);
                        }
                        // else it is ok to proceed.
                        // thus it is safe to unwrap it
                        if let Some(mut content) = evaluation_result.unwrap() {
                            if frag.pipe {
                                self.pipe_value = content;
                                lexor.escape_nl = true;
                            }
                            // If content is none
                            // Ignore new line after macro evaluation until any character
                            else if content.len() == 0 {
                                lexor.escape_nl = true;
                            } else {
                                if frag.trimmed {
                                    content = Utils::trim(&content)?;
                                }
                                if frag.yield_literal {
                                    content = format!("\\*{}*\\", content);
                                }
                                remainder.push_str(&content);
                            }
                        } 
                        // Failed to invoke
                        // because macro doesn't exist
                        else {
                            // If strict mode is set, every error is panic error
                            if self.strict {
                                self.log_error(&format!("Failed to invoke a macro : \"{}\"", frag.name))?;
                                return Err(RadError::StrictPanic);
                            } 
                            // If purge mode is set, don't print anything 
                            // and don't print error
                            if !self.purge {
                                self.log_error(&format!("Failed to invoke a macro : \"{}\"", frag.name))?;
                                remainder.push_str(&frag.whole_string);
                            } 
                            else {
                                // If purge mode
                                // set escape new line 
                                lexor.escape_nl = true;
                            }
                        }
                        // Clear fragment regardless of success
                        frag.clear()
                    }
                }
                // Remove fragment and set to remainder
                LexResult::ExitFrag => {
                    frag.whole_string.push(ch);
                    remainder.push_str(&frag.whole_string);
                    frag.clear();
                }
            }
        } // End Character iteration
        Ok(remainder)
    }

    fn add_define(&mut self, frag: &mut MacroFragment, remainder: &mut String) -> Result<(), RadError> {
        if let Some((name,args,body)) = self.define_parse.parse_define(&frag.args) {
            self.map.register(&name, &args, &body)?;
        } else {
            self.log_error(&format!(
                    "Failed to register a macro : \"{}\"", frag.args.split(',').collect::<Vec<&str>>()[0]
            ))?;
            remainder.push_str(&frag.whole_string);
        }
        // Clear fragment regardless of success
        frag.clear();

        Ok(())
    }

    // Evaluate can be nested deeply
    // Disable caller for temporary
    fn evaluate(&mut self,level: usize, caller: &str, name: &str, args: &str, greedy: bool) -> Result<Option<String>, RadError> {
        let level = level + 1;
        // This parses and processes arguments
        // and macro should be evaluated after
        // TODO 
        // Make caller to name
        let args = self.parse_chunk(level, name, args)?; 

        // Find local macro
        // The macro can be be the macro defined in parent macro
        let mut temp_level = level;
        while temp_level > 0 {
            if caller == name {
                self.log_warning(&format!("Calling self, which is \"{}\", can possibly trigger infinite loop", name))?;
            }
            if let Some(local) = self.map.local.get(&Utils::local_name(temp_level, &name)) {
                return Ok(Some(local.to_owned()));
            } 
            temp_level = temp_level - 1;
        }
        // Find custom macro
        // custom macro comes before basic macro so that
        // user can override it
        if self.map.custom.contains_key(name) {
            if let Some(result) = self.invoke_rule(level, name, &args, greedy)? {
                return Ok(Some(result));
            } else {
                return Ok(None);
            }
        }
        // Find basic macro
        else if self.map.basic.contains(&name) {
            let final_result = self.map.basic.clone().call(name, &args, greedy, self)?;
            return Ok(Some(final_result));
        } 
        // No macros found to evaluate
        else { 
            return Ok(None);
        }
    }

    fn invoke_rule(&mut self,level: usize ,name: &str, arg_values: &str, greedy: bool) -> Result<Option<String>, RadError> {
        // Get rule
        // Invoke is called only when key exists, thus unwrap is safe
        let rule = self.map.custom.get(name).unwrap().clone();
        let arg_types = &rule.args;
        let args: Vec<String>;
        // Set variable to local macros
        if let Some(content) = ArgParser::args_with_len(arg_values, arg_types.len(), greedy) {
            args = content;
        } else {
            // Necessary arg count is bigger than given arguments
            self.log_error(&format!("{}'s arguments are not sufficient. Given {}, but needs {}", name, arg_values.len(), arg_types.len()))?;
            return Ok(None);
        }

        for (idx, arg_type) in arg_types.iter().enumerate() {
            //Set arg to be substitued
            self.map.new_local(level + 1, arg_type ,&args[idx]);
        }
        // parse the Chunk
        let result = self.parse_chunk(level, &name, &rule.body)?;

        Ok(Some(result))
    }

    fn write_to(&mut self, content: &str, container: &mut Option<&mut String>) -> Result<(), RadError> {
        // Don't try to write empty string, because it's a waste
        if content.len() == 0 { return Ok(()); }
        // Save to container
        if let Some(container) = container {
            container.push_str(content);
        } 
        // Write out to file or stdout
        else {
            if self.redirect {
                self.temp_target.1.write(content.as_bytes())?;
            } else {
                match &mut self.write_option {
                    WriteOption::File(f) => f.write_all(content.as_bytes())?,
                    WriteOption::Stdout => print!("{}", content),
                }
            }
        }

        Ok(())
    }

    pub fn log_error(&mut self, log : &str) -> Result<(), RadError> {
        self.error_logger.elog(log)?;
        Ok(())
    }

    pub fn log_warning(&mut self, log : &str) -> Result<(), RadError> {
        self.error_logger.wlog(log)?;
        Ok(())
    }

    /// This is not a backup but fresh set of file information
    fn set_file(&mut self, file: &str) -> Result<(), RadError> {
        let path = &Path::new(file);
        if !path.exists() {
            Err(RadError::InvalidCommandOption(format!("File, \"{}\" doesn't exist, therefore cannot be read by r4d.", path.display())))
        } else {
            self.current_input = file.to_owned();
            self.error_logger.set_file(file);
            Ok(())
        }
    }

    pub fn add_custom_rules(&mut self, rules: HashMap<String, MacroRule>) {
        self.map.custom.extend(rules.into_iter());
    }
}

pub(crate) struct DefineParser{
    arg_cursor :DefineCursor,
    name: String,
    args: String,
    body: String,
    dquote: bool,
    container: String,
}

impl DefineParser {
    pub fn new() -> Self {
        Self {
            arg_cursor : DefineCursor::Name,
            name : String::new(),
            args : String::new(),
            body : String::new(),
            dquote : false,
            container : String::new(),
        }
    }

    fn clear(&mut self) {
        self.arg_cursor = DefineCursor::Name;
        self.name.clear();
        self.args.clear();
        self.body.clear();
        self.dquote = false;
        self.container.clear();
    }

    // Static function
    // NOTE This method expects valid form of macro invocation
    // Given value should be without outer prentheses
    // e.g. ) name,a1 a2,body text
    pub fn parse_define(&mut self, text: &str) -> Option<(String, String, String)> {
        self.clear();
        let mut bind = false;
        let mut char_iter = text.chars().peekable();
        while let Some(ch) = char_iter.next() {
            match self.arg_cursor {
                DefineCursor::Name => {
                    // $define(variable=something)
                    // Don't set argument but directly bind variable to body
                    if ch == '=' {
                        self.name.push_str(&self.container);
                        self.container.clear();
                        self.arg_cursor = DefineCursor::Body;
                        bind = true;
                        continue;
                    } 
                    else if Utils::is_blank_char(ch) {
                        // This means pattern like this
                        // $define( name ) -> name is registered
                        // $define( na me ) -> na is ignored and take me instead
                        if self.name.len() != 0 {
                            self.container.clear();
                        } else {
                            // Ignore
                            continue;
                        }
                    } 
                    // Comma go to args
                    else if ch == ',' {
                        self.name.push_str(&self.container);
                        self.container.clear();
                        self.arg_cursor = DefineCursor::Args;
                        continue;
                    } 
                    else {
                        // If not valid name return None
                        if !self.is_valid_name(ch) { return None; }
                    }
                }
                DefineCursor::Args => {
                    // Blank space separates arguments 
                    if Utils::is_blank_char(ch) && self.name.len() != 0 {
                        self.args.push_str(&self.container);
                        self.args.push(' ');
                        self.container.clear();
                        continue;
                    } 
                    // Go to body
                    else if ch == '=' {
                        self.args.push_str(&self.container);
                        self.container.clear();
                        self.arg_cursor = DefineCursor::Body; 
                        continue;
                    } 
                    // Others
                    else {
                        // If not valid name return
                        if !self.is_valid_name(ch) { return None; }
                    }
                }
                // Add everything
                DefineCursor::Body => ()
            } 
            self.container.push(ch);
        }

        // This means pattern such as
        // $define(test,Test) 
        // -> This is not a valid pattern
        if self.args.len() == 0 && !bind {
            return None;
        }

        // End of body
        self.body.push_str(&self.container);

        Some((self.name.clone(), self.args.clone(), self.body.clone()))
    }

    fn is_valid_name(&mut self, ch : char) -> bool {
        if self.container.len() == 0 { // Start of string
            // Not alphabetic 
            // $define( 1name ) -> Not valid
            if !ch.is_alphabetic() {
                return false;
            }
        } else { // middle of string
            // Not alphanumeric and not underscore
            // $define( na*1me ) -> Not valid
            // $define( na_1me ) -> Valid
            if !ch.is_alphanumeric() && ch != '_' {
                return false;
            }
        }
        true
    }
    
    // Reserved for later refactoring
    //fn branch_name() {

    //}

    //fn branch_args() {

    //}

    //fn branch_body() {

    //}
}

enum DefineCursor {
    Name,
    Args,
    Body,
}
