use std::io::{self, Write};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use crate::error::{RadError, ErrorLogger};
use crate::models::{MacroMap, WriteOption, UnbalancedChecker, MacroRule};
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
pub struct Processor{
    pub map: MacroMap,
    define_parse: DefineParser,
    write_option: WriteOption,
    error_logger: ErrorLogger,
    checker: UnbalancedChecker,
    line_number: u64,
    ch_number: u64,
    pub pipe_value: String,
    pub newline: String,
    pub paused: bool,
    purge: bool,
    always_greedy: bool,
    pub temp_target: PathBuf,
}
// 1. Get string
// 2. Parse until macro invocation detected
// 3. Return remainder and macro fragments
// 4. Continue parsing with fragments

impl Processor {
    pub fn new(write_option: WriteOption, error_write_option : Option<WriteOption>, newline: String) -> Self {
        Self {
            map : MacroMap::new(),
            write_option,
            define_parse: DefineParser::new(),
            error_logger: ErrorLogger::new(error_write_option),
            checker : UnbalancedChecker::new(),
            line_number :0,
            ch_number:0,
            newline,
            pipe_value: String::new(),
            paused: false,
            purge: false,
            always_greedy: false,
            temp_target: PathBuf::from(std::env::temp_dir()).join("rad.txt"),
        }
    }
    pub fn set_greedy(&mut self) {
        self.always_greedy = true;
    }
    pub fn set_purge(&mut self) {
        self.purge = true;
    }
    pub fn get_map(&self) -> &MacroMap {
        &self.map
    }
    pub fn from_stdin(&mut self, get_result: bool) -> Result<String, RadError> {
        let stdin = io::stdin();
        let mut line_iter = Utils::full_lines(stdin.lock());
        let mut lexor = Lexor::new();
        let mut invoke = MacroFragment::new();
        let mut content = String::new();
        let mut container = if get_result { Some(&mut content) } else { None };
        loop {
            self.line_number = self.line_number + 1;
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

        Ok(content)
    }

    pub fn from_file(&mut self, path :&Path, get_result: bool) -> Result<String, RadError> {
        let file_stream = File::open(path)?;
        let reader = io::BufReader::new(file_stream);
        let mut line_iter = Utils::full_lines(reader);
        let mut lexor = Lexor::new();
        let mut invoke = MacroFragment::new();
        let mut content = String::new();
        let mut container = if get_result { Some(&mut content) } else { None };
        loop {
            self.line_number = self.line_number + 1;
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
        self.ch_number = 0;
        // Local values
        let mut remainder = String::new();

        // Reset lexor's escape_nl 
        lexor.escape_nl = false;
        for ch in line.chars() {
            self.ch_number = self.ch_number + 1;
            let lex_result = lexor.lex(ch)?;
            // Either add character to remainder or fragments
            match lex_result {
                LexResult::Ignore => frag.whole_string.push(ch),
                // If given result is literal
                LexResult::Literal(cursor) => {
                    match cursor {
                        // Exit frag
                        // If literal is given on names
                        Cursor::Name | Cursor::NameToArg => {
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
                    self.error_logger
                        .set_number(
                            self.line_number, 
                            self.ch_number
                        );
                    // If paused, then reset lexor context
                    if self.paused {
                        lexor.reset();
                        remainder.push_str(&frag.whole_string);
                        frag.clear();
                    }
                }
                LexResult::AddToRemainder => {
                    if !self.checker.check(ch) {
                        self.error_logger.set_number(self.line_number, self.ch_number);
                        self.log_warning("Unbalanced parenthesis detected.")?;
                    }
                    remainder.push(ch);
                }
                LexResult::AddToFrag(cursor) => {
                    match cursor{
                        Cursor::Name => {
                            if frag.name.len() == 0 {
                                self.error_logger
                                    .set_number(
                                        self.line_number, 
                                        self.ch_number
                                    );
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
                            // If purge mode is set, don't print anything 
                            // and don't print error
                            if !self.purge {
                                self.log_error(&format!("Failed to invoke a macro : \"{}\"", frag.name))?;
                                remainder.push_str(&frag.whole_string);
                            } else {
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
    fn evaluate(&mut self,level: usize, _caller: &str, name: &str, args: &str, greedy: bool) -> Result<Option<String>, RadError> {
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
        // Save to container
        if let Some(container) = container {
            container.push_str(content);
        } 
        // Write out to file or stdout
        else {
            match &mut self.write_option {
                WriteOption::File(f) => f.write_all(content.as_bytes())?,
                WriteOption::Stdout => print!("{}", content),
            }
        }

        Ok(())
    }

    fn log_error(&mut self, log : &str) -> Result<(), RadError> {
        self.error_logger.elog(log)?;
        Ok(())
    }

    fn log_warning(&mut self, log : &str) -> Result<(), RadError> {
        self.error_logger.wlog(log)?;
        Ok(())
    }

    pub fn set_file(&mut self, file: &str) {
        self.error_logger.set_file(file);
        self.line_number = 0;
        self.ch_number = 0;
    }

    pub fn add_custom_rules(&mut self, rules: HashMap<String, MacroRule>) {
        self.map.custom.extend(rules.into_iter());
    }
}

pub struct DefineParser{
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
                    // This means pattern like this
                    // $define( name ) -> name is registered
                    // $define( na me ) -> only "na" is registered
                    else if Utils::is_blank_char(ch) && self.name.len() != 0 {
                        self.name.push_str(&self.container);
                        self.container.clear();
                        self.arg_cursor = DefineCursor::Args;
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
