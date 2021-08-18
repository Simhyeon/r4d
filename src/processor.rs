use std::io::{self, Lines, BufRead, Write};
use std::fs::File;
use std::path::Path;
use crate::error::RadError;
use crate::models::MacroMap;
use crate::utils::Utils;
use crate::consts::*;
use crate::lexor::*;
use crate::consts::LINE_ENDING;

pub struct MacroFragment {
    pub whole_string: String,
    pub name: String,
    pub args: String,
}

impl MacroFragment {
    pub fn new() -> Self {
        MacroFragment {  
            whole_string : String::new(),
            name : String::new(),
            args : String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.whole_string.clear();
        self.name.clear();
        self.args.clear();
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

pub enum WriteOption {
    File(std::fs::File),
    Stdout,
}

pub struct Processor<'a> {
    pub map: MacroMap<'a>,
    write_option: WriteOption,
}
// 1. Get string
// 2. Parse until macro invocation detected
// 3. Return remainder and macro fragments
// 4. Continue parsing with fragments

impl<'a> Processor<'a> {
    pub fn new(write_option: WriteOption) -> Self {
        Self {
            map : MacroMap::new(),
            write_option,
        }
    }
    pub fn get_map(&self) -> &MacroMap {
        &self.map
    }
    pub fn from_stdin(&mut self, get_result: bool) -> Result<String, RadError> {
        let stdin = io::stdin();
        let mut line_iter = stdin.lock().lines();
        let mut lexor = Lexor::new();
        let mut invoke = MacroFragment::new();
        let mut content = String::new();
        let mut container = if get_result { Some(&mut content) } else { None };
        loop {
            let result = self.parse_line(&mut line_iter, &mut lexor ,&mut invoke)?;
            match result {
                // This means either macro is not found at all
                // or previous macro fragment failed with invalid syntax
                ParseResult::Printable(mut remainder) => {
                    remainder.push_str(LINE_ENDING);
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
        let mut line_iter = reader.lines();
        let mut lexor = Lexor::new();
        let mut invoke = MacroFragment::new();
        let mut content = String::new();
        let mut container = if get_result { Some(&mut content) } else { None };
        loop {
            let result = self.parse_line(&mut line_iter, &mut lexor ,&mut invoke)?;
            match result {
                // This means either macro is not found at all
                // or previous macro fragment failed with invalid syntax
                ParseResult::Printable(mut remainder) => {
                    remainder.push_str(LINE_ENDING);
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
    
    // TODO
    /// Parse line is called only by the main loop thus, caller name is special name of @MAIN
    fn parse_line(&mut self, lines :&mut Lines<impl BufRead>, lexor : &mut Lexor ,frag : &mut MacroFragment) -> Result<ParseResult, RadError> {
        if let Some(line) = lines.next() {
            // Rip off a result into a string
            let line = line?;
            // Local values
            let mut remainder = String::new();
            let level = 0; // 0 is main level

            for ch in line.chars() {
                let lex_result = lexor.lex(ch)?;
                // TODO
                // Either add character to remainder or fragments
                match lex_result {
                    LexResult::Ignore => frag.whole_string.push(ch),
                    LexResult::AddToRemainder => {
                        remainder.push(ch);
                    }
                    LexResult::AddToFrag(cursor) => {
                        match cursor{
                            Cursor::Name => frag.name.push(ch),
                            Cursor::Arg => frag.args.push(ch),
                            _ => unreachable!(),
                        } 
                        frag.whole_string.push(ch);
                    }
                    // TODO
                    // End of fragment
                    // 1. Evaluate macro 
                    // 1.5 -> If define, parse rule applied again.
                    // 2. And append to remainder
                    // 3. Reset fragment
                    LexResult::EndFrag => {
                        frag.whole_string.push(ch);
                        if frag.name == "define" {
                            // Failed to register macro
                            if let Some((name,args,body)) = Self::parse_define(&frag.args) {
                                self.map.register(&name, &args, &body)?;
                            } else {
                                eprintln!("Failed to register macro");
                                remainder.push_str(&frag.whole_string);
                            }
                            // Clear fragment regardless of success
                            frag.clear();
                        } else {
                            // Invoke
                            if let Some(content) = self.evaluate(level, &MAIN_CALLER.to_owned(), &frag.name, &frag.args)? {
                                remainder.push_str(&content);
                                frag.clear();

                                // Clear local variable macros
                                self.map.clear_local();
                            } 
                            // Failed to invoke
                            // because macro doesn't exist
                            else {
                                remainder.push_str(&frag.whole_string);
                                frag.clear()
                            }
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

            // Post-parse process
            
            // Add new line to fragment
            if !frag.is_empty() {
                match lexor.cursor {
                    Cursor::Name => frag.name.push_str(LINE_ENDING),
                    Cursor::Arg => frag.args.push_str(LINE_ENDING),
                    // Name to arg is ignored
                    _ => (),
                }
            }

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
    pub fn parse_chunk(&mut self, level: usize, caller: &String, chunk: &str) -> Result<String, RadError> {
        let mut lexor = Lexor::new();
        let mut frag = MacroFragment::new();
        let mut remainder = String::new();

        // Cap the chunk to resesrve new lines
        let chunk = Utils::cap(chunk);

        let mut lines = chunk.lines().peekable();
        let mut count = 0;
        while let Some(line) = lines.next() {
            count = count + 1;
            // Rip off a result into a string
            // Local values
            for ch in line.chars() {
                let lex_result = lexor.lex(ch)?;
                // TODO
                // Either add character to remainder or fragments
                match lex_result {
                    LexResult::Ignore => frag.whole_string.push(ch),
                    LexResult::AddToRemainder => {
                        remainder.push(ch);
                    }
                    LexResult::AddToFrag(cursor) => {
                        match cursor{
                            Cursor::Name => frag.name.push(ch),
                            Cursor::Arg => frag.args.push(ch),
                            _ => unreachable!(),
                        } 
                        frag.whole_string.push(ch);
                    }
                    // TODO
                    // End of fragment
                    // 1. Evaluate macro 
                    // 1.5 -> If define, parse rule applied again.
                    // 2. And append to remainder
                    // 3. Reset fragment
                    LexResult::EndFrag => {
                        frag.whole_string.push(ch);
                        if frag.name == "define" {
                            // Failed to register macro
                            if let Some((name,args,body)) = Self::parse_define(&frag.args) {
                                self.map.register(&name, &args, &body)?;
                            } else {
                                remainder.push_str(&frag.whole_string);
                            }
                            // Clear fragment regardless of success
                            frag.clear();
                        } else {
                            // Invoke
                            if let Some(content) = self.evaluate(level + 1, caller, &frag.name, &frag.args)? {
                                remainder.push_str(&content);
                                frag.clear();
                            }
                        }
                    }
                    // Remove fragment and set to remainder
                    LexResult::ExitFrag => {
                        frag.whole_string.push(ch);
                        remainder.push_str(&frag.whole_string);
                        frag.clear();
                    }
                }
            }
            // Add new line to remainder because this operation is line based
            if let Some(_) = lines.peek() {
                remainder.push_str(LINE_ENDING);
            }
        }  // End while
        // Uncap LINE_CAP
        Utils::uncap(&mut remainder);
        return Ok(remainder)
    } // parse_line end

    // Static function
    // NOTE This method expects valid form of macro invocation
    pub fn parse_define(text: &str) -> Option<(String, String, String)> {
        let mut arg_cursor = "name";
        let mut name = String::new();
        let mut args = String::new();
        let mut body = String::new();

        let mut container = String::new();

        for ch in text.chars() {
            if ch == ',' {
                match arg_cursor {
                    "name" => {
                        name.push_str(&container);
                        container.clear();
                        arg_cursor = "args"; 
                        continue;
                    },
                    "args" => {
                        args.push_str(&container);
                        container.clear();
                        arg_cursor = "body"; 
                        continue;
                    },
                    "body" => {
                        body.push_str(&container);
                        container.clear();
                        return Some((name, args, body));
                    }
                    _ => unreachable!()
                }
            }
            if ch == ' ' || ch == '\n' || ch == '\r' || ch == '\t' {
                // This means pattern like this
                // $define( name ) -> name is registered
                // $define( na me ) -> only "na" is registered
                if arg_cursor == "name" && name.len() != 0 {
                    name.push_str(&container);
                    container.clear();
                    arg_cursor = "args";
                    continue;
                } 
                // This emans pattern like this
                // $define(name, arg1 ,)
                //                   |
                //                  -This part makes argument empty
                //                  and starts argument evaluation as new state
                else if arg_cursor == "args" && container.len() != 0 {
                    args.push_str(&container);
                    args.push(' ');
                    container.clear();
                    continue;
                }
            } else {
                // Body can be any text but, 
                // name and arg should follow rules
                if arg_cursor != "body" {
                    if container.len() == 0 { // Start of string
                        // Not alphabetic 
                        // $define( 1name ) -> Not valid
                        if !ch.is_alphabetic() {
                            return None;
                        }
                    } else { // middle of string
                        // Not alphanumeric and not underscore
                        // $define( na*1me ) -> Not valid
                        // $define( na_1me ) -> Valid
                        if !ch.is_alphanumeric() && ch != '_' {
                            return None;
                        }
                    }
                }
            }

            container.push(ch);
        }

        // Natural end of body
        body.push_str(&container);

        Some((name, args, body))
    }

    // TODO
    // This method's logic should be similar to that of from_stdin
    // Evaluate can be nested deeply
    // TODO Add local macro map to be used for custom macro expansion
    fn evaluate(&mut self,level: usize, caller: &String, name: &String, args: &String) -> Result<Option<String>, RadError> {

        // This parses and processes arguments
        // and macro should be evaluated after
        let args = self.parse_chunk(level, caller, args)?; 

        // Ok, this is devastatingly hard to read 
        // Find Local macro first
        if let Some(local) = self.map.local.get(&Utils::local_name(level, &caller, &name)) {
            return Ok(Some(local.to_owned()))
        } 
        // Find custom macro
        // custom macro comes before basic macro so that
        // user can override it
        else if self.map.custom.contains_key(name) {
            if let Some(result) = self.invoke_rule(level, name, &args)? {
                return Ok(Some(result));
            } else {
                return Ok(None);
            }
        }
        // Find basic macro
        else if self.map.basic.contains(&name) {
            let final_result = self.map.basic.clone().call(name, &args, self)?;
            return Ok(Some(final_result));
        } 
        // No macros found..? // possibly be unreachable
        else { 
            println!("{}", Utils::local_name(level, caller, name));
            println!("No macro found");
            return Ok(None);
        }
    }

    // TODO
    // Inconsistency in arg_values array is seriously bad
    // Pick one space separated string, or vector
    fn invoke_rule(&mut self,level: usize ,name: &String, arg_values: &String) -> Result<Option<String>, RadError> {
        // Get rule
        // Invoke is called only when key exists, thus unwrap is safe
        let rule = self.map.custom.get(name).unwrap().clone();
        let arg_types = &rule.args;
        // Set variable to local macros
        let arg_values = Utils::args_to_vec(arg_values, ',', ('"','"'));

        // Necessary arg count is bigger than given arguments
        if arg_types.len() > arg_values.len() {
            eprintln!("Arg types : {:?}\nArg values : {:?}", arg_types, arg_values);
            eprintln!("{}'s arguments are not sufficient", name);
            return Ok(None);
        }

        for (idx, arg_type) in arg_types.iter().enumerate() {
            //Set arg to be substitued
            self.map.new_local(level + 1, name, arg_type ,&arg_values[idx]);
        }
        // PARSE Chunk
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

}
