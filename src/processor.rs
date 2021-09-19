use std::io::{self, BufReader, Read, Write};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::{ Path , PathBuf};
use crate::basic::MacroType;
use crate::error::RadError;
use crate::logger::{Logger, LoggerLines};
#[cfg(feature = "debug")]
use crate::logger::DebugSwitch;
use crate::models::{MacroMap, MacroRule, RuleFile, UnbalancedChecker, WriteOption};
use crate::utils::Utils;
use crate::consts::*;
use crate::lexor::*;
use crate::arg_parser::{ArgParser, GreedyState};

// Methods of processor consists of multiple sections followed as
// <BUILDER>            -> Builder pattern related
// <PROCESS>            -> User functions related
// <DEBUG>              -> Debug related functions
// <PARSE>              -> Parse rleated functions
//     <LEX>            -> sub sectin of parse, this is technically not a lexing but it's named as
// <MISC>               -> Miscellaenous
//
// Find each section's start with <NAME> and find end of section with </NAME>
//
// e.g. <BUILDER> for builder section start and </BUILDER> for builder section end

/// Processor that parses(lexes) given input and print out to destined output
pub struct Processor{
    current_input : String, // This is either "stdin" or currently reading file's name
    map: MacroMap,
    define_parse: DefineParser,
    write_option: WriteOption,
    logger: Logger,
    checker: UnbalancedChecker,
    pub(crate) pipe_value: String,
    pub(crate) newline: String,
    pub(crate) paused: bool,
    pub(crate) redirect: bool,
    #[cfg(feature = "debug")]
    pub(crate) debug: bool,
    #[cfg(feature = "debug")]
    pub(crate) debug_log: bool,
    #[cfg(feature = "debug")]
    debug_switch : DebugSwitch,
    // This is a global line number storage for various deubbing usages
    #[cfg(feature = "debug")]
    line_number : usize,
    // This is a bit bloaty, but debugging needs functionality over efficiency
    #[cfg(feature = "debug")]
    pub(crate) line_caches: HashMap<usize, String>,
    sandbox: bool,
    purge: bool,
    strict: bool,
    always_greedy: bool,
    // Temp target needs to save both path and file
    // because file doesn't necessarily have path. 
    // Especially in unix, this is not so an unique case
    temp_target: (PathBuf,File), 
}

impl Processor {
    // ----------
    // Builder pattern methods
    // <BUILDER>
    /// Creates new processor with deafult options
    pub fn new() -> Self {
        Self::new_processor(true)
    }

    /// Creates new processor without default macros
    pub fn empty() -> Self {
        Self::new_processor(false)
    }

    /// Internal function to create Processor struct
    fn new_processor(use_basic: bool) -> Self {
        let temp_path= std::env::temp_dir().join("rad.txt");
        let temp_target = (temp_path.to_owned(),OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)
            .unwrap());

        let mut logger = Logger::new();
        logger.set_write_options(Some(WriteOption::Stdout));

        let map = if use_basic {
            MacroMap::new()
        } else {
            MacroMap::empty()
        };

        Self {
            current_input: String::from("stdin"),
            map,
            write_option: WriteOption::Stdout,
            define_parse: DefineParser::new(),
            logger,
            checker : UnbalancedChecker::new(),
            newline : LINE_ENDING.to_owned(),
            pipe_value: String::new(),
            paused: false,
            redirect: false,
            purge: false,
            strict: false,
            sandbox : false,
            #[cfg(feature = "debug")]
            debug: false,
            #[cfg(feature = "debug")]
            debug_log: false,
            #[cfg(feature = "debug")]
            debug_switch: DebugSwitch::NextLine,
            #[cfg(feature = "debug")]
            line_number: 1,
            #[cfg(feature = "debug")]
            line_caches: HashMap::new(),
            always_greedy: false,
            temp_target,
        }
    }

    /// Set write option to yield output to the file
    pub fn write_to_file(&mut self, target_file: Option<PathBuf>) -> Result<&mut Self, RadError> {
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
    pub fn error_to_file(&mut self, target_file: Option<PathBuf>) -> Result<&mut Self, RadError> {
        if let Some(target_file) = target_file {
            let target_file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(target_file)?;

            self.logger = Logger::new();
            self.logger.set_write_options(Some(WriteOption::File(target_file)));
        }
        Ok(self)
    }

    /// Use unix line ending instead of os default one
    pub fn unix_new_line(&mut self, use_unix_new_line: bool) -> &mut Self {
        if use_unix_new_line {
            self.newline = "\n".to_owned();
        }
        self
    }

    /// Set greedy option
    pub fn greedy(&mut self, greedy: bool) -> &mut Self {
        if greedy {
            self.always_greedy = true;
        }
        self
    }

    /// Set purge option
    pub fn purge(&mut self, purge: bool) -> &mut Self {
        if purge {
            self.purge = true;
            self.strict = false;
        }
        self
    }

    /// Set strict option
    pub fn strict(&mut self, strict: bool) -> &mut Self {
        if strict {
            self.strict = true;
            self.purge = false;
        }
        self
    }

    /// Set silent option
    pub fn silent(&mut self, silent: bool) -> &mut Self {
        if silent {
            self.logger = Logger::new();
            self.logger.set_write_options(None);
        }
        self
    }

    /// Add debug options
    #[cfg(feature = "debug")]
    pub fn debug(&mut self, debug: bool) -> Result<&mut Self, RadError> {
        if debug {
            self.debug = true;
        }
        Ok(self)
    }

    /// Add debug log options
    #[cfg(feature = "debug")]
    pub fn log(&mut self, log: bool) -> Result<&mut Self, RadError> {
        if log {
            self.debug_log = true;
        }
        Ok(self)
    }

    /// Add debug interactive options
    #[cfg(feature = "debug")]
    pub fn interactive(&mut self, interactive: bool) -> Result<&mut Self, RadError> {
        if interactive {
            self.logger.set_debug_interactive();
        }
        Ok(self)
    }

    /// Add custom rules
    pub fn custom_rules(&mut self, paths: Option<Vec<PathBuf>>) -> Result<&mut Self, RadError> {
        if let Some(paths) = paths {
            let mut rule_file = RuleFile::new(None);
            for p in paths.iter() {
                rule_file.melt(p)?;
            }
            self.map.custom.extend(rule_file.rules);
        }

        Ok(self)
    }

    /// Creates a unreferenced instance of processor
    pub fn build(&mut self) -> Self {
        std::mem::replace(self, Processor::new())
    }

    // </BUILDER>
    // End builder methods
    // ----------

    // ----------
    // Processing methods
    // <PROCESS>
    //
    /// Print the result of a processing
    #[allow(dead_code)]
    pub fn print_result(&mut self) -> Result<(), RadError> {
        self.logger.print_result()?;
        Ok(())
    }

    /// Freeze to single file
    pub fn freeze_to_file(&self, path: &Path) -> Result<(), RadError> {
        RuleFile::new(Some(self.map.custom.clone())).freeze(path)?;
        Ok(())
    }

    /// Add new basic rules
    pub fn add_basic_rules(&mut self, basic_rules:Vec<(&str,MacroType)>) {
        for (name, macro_ref) in basic_rules {
            self.map.basic.add_new_rule(name, macro_ref);
        }
    }

    /// Read from string
    pub fn from_string(&mut self, content: &str) -> Result<String, RadError> {
        // Set name as string
        self.set_input("String")?;

        let mut reader = content.as_bytes();
        self.from_buffer(&mut reader)
    }

    /// Read from standard input
    ///
    /// If debug mode is enabled this, doesn't read stdin line by line but by chunk because user
    /// input is also a standard input and processor cannot distinguish the two
    pub fn from_stdin(&mut self) -> Result<String, RadError> {
        let stdin = io::stdin();

        // Early return if debug
        // This read whole chunk of string 
        #[cfg(feature = "debug")]
        if self.debug {
            let mut input = String::new();
            stdin.lock().read_to_string(&mut input)?;
            // This is necessary to prevent unexpected output from being captured.
            return self.from_buffer(&mut input.as_bytes());
        }

        let mut reader = stdin.lock();
        self.from_buffer(&mut reader)
    }

    /// Process contents from a file
    pub fn from_file(&mut self, path :&Path) -> Result<String, RadError> {

        // Set file as name of given path
        self.set_file(path.to_str().unwrap())?;

        let file_stream = File::open(path)?;
        let mut reader = BufReader::new(file_stream);
        self.from_buffer(&mut reader)
    }

    /// Internal method for processing buffers line by line
    fn from_buffer(&mut self,buffer: &mut impl std::io::BufRead) -> Result<String, RadError> {
        // Sandboxed environment, backup
        let backup = if self.sandbox { Some(self.backup()) } else { None };
        let mut line_iter = Utils::full_lines(buffer).peekable();
        let mut lexor = Lexor::new();
        let mut frag = MacroFragment::new();
        let mut content = String::new();
        // Container is where sandboxed output is saved
        let mut container = if self.sandbox { Some(&mut content) } else { None };
        #[cfg(feature = "debug")]
        self.user_input_on_start()?;
        loop {
            #[cfg(feature = "debug")]
            if let Some(line) = line_iter.peek() {
                let line = line.as_ref().unwrap();
                // Update line cache
                self.line_caches.insert(self.line_number, line.lines().next().unwrap().to_owned());
                // Only if debug switch is nextline
                self.user_input_on_line(&frag)?;
            }
            let result = self.parse_line(&mut line_iter, &mut lexor ,&mut frag)?;
            match result {
                // This means either macro is not found at all
                // or previous macro fragment failed with invalid syntax
                ParseResult::Printable(remainder) => {
                    self.write_to(&remainder, &mut container)?;

                    // Test if this works
                    #[cfg(feature = "debug")]
                    self.line_caches.clear();

                    // Reset fragment
                    if &frag.whole_string != "" {
                        frag = MacroFragment::new();
                    }
                }
                ParseResult::FoundMacro(remainder) => {
                    self.write_to(&remainder, &mut container)?;
                }
                // This happens only when given macro involved text should not be printed
                ParseResult::NoPrint => { }
                // End of input, end loop
                ParseResult::EOI => break,
            }
            // Increaing number should be followed after evaluation
            // To ensure no panick occurs during user_input_on_line, which is caused by
            // out of index exception from getting current line_cache
            #[cfg(feature = "debug")]
            {
                // Increase absolute line number
                self.line_number = self.line_number + 1; 
            }
        } // Loop end

        // Recover
        if let Some(backup) = backup { self.recover(backup); self.sandbox = false; }

        Ok(content)
    }

    // End of process methods
    // </PROCESS>
    // ----------


    // ===========
    // Debug related methods
    // <DEBUG>
    
    #[cfg(feature = "debug")]
    fn is_local(&self, mut level: usize, name: &str) -> bool {
        while level > 0 {
            if self.map.local.contains_key(&Utils::local_name(level, &name)) {
                return true;
            }
            level = level - 1;
        }
        false
    }

    // This function can be used in non debug feature
    /// Process breakpoint
    fn break_point(&mut self, frag: &mut MacroFragment) -> Result<(), RadError> {
        if &frag.name == "BR" {
            #[cfg(feature = "debug")]
            if self.debug {
                if let DebugSwitch::NextBreakPoint(name) = &self.debug_switch {
                    // Name is empty or same with frag.args
                    if name == &frag.args || name == "" {
                        self.debug_switch = DebugSwitch::NextLine;
                    }
                }
                // Clear fragment
                frag.clear();
                return Ok(());
            } 

            self.logger.wlog("Breakpoint in non debug mode")?;
            frag.clear();
        }

        Ok(())
    }

    // Though this implementation is same with user_input_prompt
    // I thought modifying user_input_prompt isn't worth.
    #[cfg(feature = "debug")]
    fn user_input_on_start(&mut self) -> Result<(), RadError> {
        // Stop by lines if debug option is lines
        if self.debug {

            let mut log = "Default is next. Ctrl + c to exit.".to_owned();
            let mut prompt = self.current_input.as_str();
            let mut do_continue = true;
            while do_continue {
                // This technically strips newline feed regardless of platforms 
                // It is ok to simply convert to a single line because it is logically a single
                // line
                let input = self.debug_wait_input(&log, Some(prompt))?;
                // Strip newline
                let input = input.lines().next().unwrap();

                do_continue = self.parse_debug_command_and_continue(&input, None, &mut log)?;
                prompt = "output";
            }
        }
        Ok(())
    }

    #[cfg(feature = "debug")]
    /// Prompt user input until break condition has been met
    fn user_input_prompt(&mut self, frag: &MacroFragment, initial_prompt: &str) -> Result<(), RadError> {
        let mut do_continue = true;
        let mut log = match &self.debug_switch {
            &DebugSwitch::NextMacro | &DebugSwitch::StepMacro => {
                self.line_caches.get(&self.logger.get_abs_last_line()).unwrap().to_owned()
            }
            _ => {
                self.line_caches.get(&self.line_number).unwrap().to_owned()
            }
        };
        let mut prompt = initial_prompt;
        while do_continue {
            let input = self.debug_wait_input(
                &log,
                Some(prompt)
            )?;
            // Strip newline
            let input = input.lines().next().unwrap();

            do_continue = self.parse_debug_command_and_continue(&input, Some(frag),&mut log)?;
            prompt = "output";
        }

        Ok(())
    }

    #[cfg(feature = "debug")]
    /// Get user input on line 
    ///
    /// This method should be called before evaluation of a line
    fn user_input_on_line(&mut self,frag: &MacroFragment) -> Result<(), RadError> {
        // Stop by lines if debug option is lines
        if self.debug {
            // Only when debugswitch is nextline
            if let DebugSwitch::NextLine = self.debug_switch {
                // Continue;
            } else {
                return Ok(()); // Return early
            }
            self.user_input_prompt(frag, "line")?;
        }
        Ok(())
    }

    #[cfg(feature = "debug")]
    /// Get user input before macro execution
    fn user_input_before_macro(&mut self, frag: &MacroFragment) -> Result<(), RadError> {
        // Stop by lines if debug option is lines
        if self.debug {
            match &self.debug_switch {
                &DebugSwitch::UntilMacro => (),
                _ => return Ok(()),
            }
            self.user_input_prompt(frag, "until")?;
        }
        Ok(())
    }

    // This is possibly loopable
    #[cfg(feature = "debug")]
    /// Get user input after execution
    fn user_input_on_macro(&mut self, frag: &MacroFragment) -> Result<(), RadError> {
        // Stop by lines if debug option is lines
        if self.debug {
            match &self.debug_switch {
                &DebugSwitch::NextMacro | &DebugSwitch::StepMacro => (),
                _ => return Ok(()),
            }
            self.user_input_prompt(frag, "macro")?;
        }
        Ok(())
    }

    // This is possibly loopable
    #[cfg(feature = "debug")]
    /// Get user input on execution but nested macro can be 
    fn user_input_on_step(&mut self, frag: &MacroFragment) -> Result<(), RadError> {
        // Stop by lines if debug option is lines
        if self.debug {
            if let &DebugSwitch::StepMacro = &self.debug_switch {
                // Continue;
            } else {
                return Ok(()); // Return early
            }
            self.user_input_prompt(frag, "step")?;
        }
        Ok(())
    }

    #[cfg(feature = "debug")]
    /// Get user input and evaluates if loop should be breaked or not
    fn parse_debug_command_and_continue(&mut self, command_input: &str, frag: Option<&MacroFragment>, log: &mut String) -> Result<bool, RadError> {
        let command_input: Vec<&str> = command_input.split(' ').collect();
        let command = command_input[0];
        // Default is empty &str ""
        let command_args = if command_input.len() == 2 {command_input[1]} else { "" };

        match command.to_lowercase().as_str() {
            // Continues until next break point
            "cl" | "clear" => {
                Utils::clear_terminal()?;
                return Ok(true);
            }
            "c" | "continue" => {
                self.debug_switch = DebugSwitch::NextBreakPoint(command_args.to_owned());
            }
            // Continue to next line
            "n" | "next" | "" => {
                self.debug_switch = DebugSwitch::NextLine;
            }
            // Continue to next macro
            "m" | "macro" => {
                self.debug_switch = DebugSwitch::NextMacro;
            }
            // Continue to until next macro
            "u" | "until" => {
                self.debug_switch = DebugSwitch::UntilMacro;
            }
            // Setp into macro
            "s" | "step" => {
                self.debug_switch = DebugSwitch::StepMacro;
            }
            "h" | "help" => {
                *log = RDB_HELP.to_owned();
                return Ok(true);
            }
            // Print "variable"
            "p" | "print" => {
                if let Some(frag) = frag {
                    match command_args.to_lowercase().as_str() {
                        "name" | "n" => {
                           *log = frag.name.to_owned();
                        }
                        "line" | "l" => {
                            match &self.debug_switch{
                                DebugSwitch::StepMacro | DebugSwitch::NextMacro => {
                                    *log = self.logger.get_abs_last_line().to_string();
                                }
                                _ => {
                                    *log = self.line_number.to_string();
                                }
                            } 
                        }
                        "span" | "s" => {
                            let mut line_number = match &self.debug_switch {
                                &DebugSwitch::NextMacro | &DebugSwitch::StepMacro => {
                                    self.logger.get_abs_line()
                                }
                                _ => self.line_number
                            };

                            let mut sums = String::new();
                            while let Some(line) = self.line_caches.get(&line_number) {
                                let mut this_line = format!("{}{}",LINE_ENDING,line);
                                this_line.push_str(&sums);
                                sums = this_line;
                                line_number = line_number - 1;
                            }
                            *log = sums;
                        }
                        "text" | "t" => {
                            match &self.debug_switch{
                                DebugSwitch::StepMacro | DebugSwitch::NextMacro => {
                                    *log = self.line_caches.get(&self.logger.get_abs_last_line()).unwrap().to_owned();
                                }
                                _ => {
                                    *log = self.line_caches.get(&self.line_number).unwrap().to_owned();
                                }
                            } 
                        }
                        "arg" | "a" => {
                            *log = frag.args.to_owned();
                        }
                        _ => {
                            *log = format!("Invalid argument \"{}\"",&command_args);
                        } 
                    } // end inner match
                } // End if let
                else { // No fragment which means it is the start of file
                    return Ok(false);
                }

                // Get user input again
                return Ok(true); 

            } // End print match
            _ => {
                *log = format!("Invalid command : {} {}",command, &command_args);
                return Ok(true);
            },
        } // End Outer match

        // Unless specific cases,
        // Continue without any loop
        Ok(false)
    }

    #[cfg(feature = "debug")]
    pub fn debug_wait_input(&self, log: &str, prompt: Option<&str>) -> Result<String, RadError> {
        Ok(self.logger.dlog_command(log, prompt)?)
    }
    #[cfg(feature = "debug")]
    pub fn debug_print_log(&self,log : &str) -> Result<(), RadError> {
        self.logger.dlog_print(log)?;
        Ok(())
    }
    #[cfg(feature = "debug")]
    pub fn debug_print_command_result(&self,log : &str) -> Result<(), RadError> {
        self.logger.dlog_print(log)?;
        Ok(())
    }

    // </DEBUG>
    // End of debug methods
    // ----------

    // ----------
    // Parse related methods
    // <PARSE>
    /// Parse line is called only by the main loop thus, caller name is special name of @MAIN@
    fn parse_line(&mut self, lines :&mut impl std::iter::Iterator<Item = std::io::Result<String>>, lexor : &mut Lexor ,frag : &mut MacroFragment) -> Result<ParseResult, RadError> {
        self.logger.add_line_number();
        if let Some(line) = lines.next() {
            let line = line?;
            let remainder = self.parse(lexor, frag, &line, 0, MAIN_CALLER)?;

            // Clear local variable macros
            self.map.clear_local();

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

    /// Parse chunk args by separating it into lines which implements BufRead
    pub(crate) fn parse_chunk_args(&mut self, level: usize, _caller: &str, chunk: &str) -> Result<String, RadError> {
        let mut lexor = Lexor::new();
        let mut frag = MacroFragment::new();
        let mut result = String::new();
        let backup = self.logger.backup_lines();
        self.logger.set_chunk(true);
        for line in Utils::full_lines(chunk.as_bytes()) {
            let line = line?;

            // NOTE
            // Parse's final argument is some kind of legacy of previous logics
            // However it can detect self calling macros in some cases
            // parse_chunk_body needs this caller but, parse_chunk_args doesn't need because
            // this methods only parses arguments thus, infinite loop is unlikely to happen
            result.push_str(&self.parse(&mut lexor, &mut frag, &line, level, "")?);

            self.logger.add_line_number();
        }
        self.logger.set_chunk(false);
        self.logger.recover_lines(backup);
        return Ok(result);
    } // parse_chunk_lines end

    /// Parse chunk body without separating lines
    /// 
    /// In contrast to parse_chunk_lines, parse_chunk doesn't create lines iterator but parses the
    /// chunk as a single entity or line.
    fn parse_chunk_body(&mut self, level: usize, caller: &str, chunk: &str) -> Result<String, RadError> {
        let mut lexor = Lexor::new();
        let mut frag = MacroFragment::new();
        let backup = self.logger.backup_lines();

        // NOTE
        // Parse's final argument is some kind of legacy of previous logics
        // However it can detect self calling macros in some cases
        let result = self.parse(&mut lexor, &mut frag, &chunk, level, caller)?;
        self.logger.recover_lines(backup);
        return Ok(result);
    } // parse_chunk end

    /// Parse a given line
    fn parse(&mut self,lexor: &mut Lexor, frag: &mut MacroFragment, line: &str, level: usize, caller: &str) -> Result<String, RadError> {
        // Initiate values
        // Reset character number
        self.logger.reset_char_number();
        // Local values
        let mut remainder = String::new();

        // Reset lexor's escape_nl 
        lexor.escape_nl = false;
        for ch in line.chars() {
            self.logger.add_char_number();

            let lex_result = lexor.lex(ch)?;
            // Either add character to remainder or fragments
            match lex_result {
                LexResult::Discard => (),
                LexResult::Ignore => frag.whole_string.push(ch),
                // If given result is literal
                LexResult::Literal(cursor) => {
                    self.lex_branch_literal(ch, frag, &mut remainder, cursor);
                }
                LexResult::StartFrag => {
                    self.lex_branch_start_frag(ch, frag, &mut remainder, lexor)?;
                },
                LexResult::EmptyName => {
                    self.lex_branch_empty_name(ch, frag, &mut remainder, lexor);
                }
                LexResult::AddToRemainder => {
                    self.lex_branch_add_to_remainder(ch, &mut remainder)?;
                }
                LexResult::AddToFrag(cursor) => {
                    self.lex_branch_add_to_frag(ch, frag, cursor);
                }
                LexResult::EndFrag => {
                    self.lex_branch_end_frag(ch,frag,&mut remainder, lexor, level, caller)?;
                }
                // Remove fragment and set to remainder
                LexResult::ExitFrag => {
                    self.lex_branch_exit_frag(ch,frag,&mut remainder);
                }
            }
        } // End Character iteration
        Ok(remainder)
    }

    // Evaluate can be nested deeply
    // Disable caller for temporary
    /// Evaluate detected macro usage
    fn evaluate(&mut self,level: usize, caller: &str, name: &str, raw_args: &str, greedy: bool) -> Result<Option<String>, RadError> {
        let level = level + 1;
        // This parses and processes arguments
        // and macro should be evaluated after
        let args = self.parse_chunk_args(level, name, raw_args)?;

        #[cfg(feature = "debug")]
        if self.debug_log { 
            self.debug_print_log(
                &format!(
                    "Level = \"{}\"{}Name = \"{}\"{}Args = \"{}\"{}",
                    level,
                    LINE_ENDING,
                    name,
                    LINE_ENDING,
                    raw_args,
                    LINE_ENDING,
                )
            )?; 
        }

        // Possibly inifinite loop so warn user
        if caller == name {
            self.log_warning(&format!("Calling self, which is \"{}\", can possibly trigger infinite loop", name))?;
        }

        // Find local macro
        // The macro can be  the one defined in parent macro
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

    /// Invoke a custom rule and get a result
    fn invoke_rule(&mut self,level: usize ,name: &str, arg_values: &str, greedy: bool) -> Result<Option<String>, RadError> {
        // Get rule
        // Invoke is called only when key exists, thus unwrap is safe
        let rule = self.map.custom.get(name).unwrap().clone();
        let arg_types = &rule.args;
        let args: Vec<String>;
        // Set variable to local macros
        if let Some(content) = ArgParser::new().args_with_len(arg_values, arg_types.len(), greedy) {
            args = content;
        } else {
            // Necessary arg count is bigger than given arguments
            self.log_error(&format!("{}'s arguments are not sufficient. Given {}, but needs {}", name, ArgParser::new().args_to_vec(arg_values, ',', GreedyState::Never).len(), arg_types.len()))?;
            return Ok(None);
        }

        for (idx, arg_type) in arg_types.iter().enumerate() {
            //Set arg to be substitued
            self.map.new_local(level + 1, arg_type ,&args[idx]);
        }
        // Process the rule body
        let result = self.parse_chunk_body(level, &name, &rule.body)?;

        Ok(Some(result))
    }

    /// Add custom rule to macro map
    ///
    /// This doesn't clear fragment
    fn add_rule(&mut self, frag: &MacroFragment, remainder: &mut String) -> Result<(), RadError> {
        if let Some((name,args,body)) = self.define_parse.parse_define(&frag.args) {
            self.map.register(&name, &args, &body)?;
        } else {
            self.log_error(&format!(
                    "Failed to register a macro : \"{}\"", 
                    frag.args.split(',').collect::<Vec<&str>>()[0]
            ))?;
            remainder.push_str(&frag.whole_string);
        }
        Ok(())
    }

    /// Write text to either file or standard output according to processor's write option
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

    // ==========
    // <LEX>
    // Start of lex branch methods
    // These are parse's sub methods for eaiser reading
    fn lex_branch_literal(&mut self, ch: char,frag: &mut MacroFragment, remainder: &mut String, cursor: Cursor) {
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

    fn lex_branch_start_frag(&mut self, ch: char,frag: &mut MacroFragment, remainder: &mut String, lexor : &mut Lexor) -> Result<(), RadError> {
        #[cfg(feature = "debug")]
        self.user_input_before_macro(&frag)?;

        frag.whole_string.push(ch);

        // If paused and not pause, then reset lexor context
        if self.paused && frag.name != "pause" {
            lexor.reset();
            remainder.push_str(&frag.whole_string);
            frag.clear();
        }

        Ok(())
    }

    fn lex_branch_empty_name(&mut self, ch: char,frag: &mut MacroFragment, remainder: &mut String, lexor : &mut Lexor) {
        frag.whole_string.push(ch);
        // If paused, then reset lexor context
        self.logger.freeze_number(); 
        if self.paused {
            lexor.reset();
            remainder.push_str(&frag.whole_string);
            frag.clear();
        }

    }

    fn lex_branch_add_to_remainder(&mut self, ch: char,remainder: &mut String) -> Result<(), RadError> {
        if !self.checker.check(ch) {
            self.logger.freeze_number();
            self.log_warning("Unbalanced parenthesis detected.")?;
        }
        remainder.push(ch);

        Ok(())
    }

    fn lex_branch_add_to_frag(&mut self, ch: char,frag: &mut MacroFragment, cursor: Cursor) {
        match cursor{
            Cursor::Name => {
                if frag.name.len() == 0 {
                    self.logger.freeze_number();
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

    fn lex_branch_end_frag(&mut self, ch: char, frag: &mut MacroFragment, remainder: &mut String, lexor : &mut Lexor, level: usize, caller: &str) -> Result<(), RadError> {
        // Push character to whole string anyway
        frag.whole_string.push(ch);
        // define
        if frag.name == "define" {
            self.add_rule(frag, remainder)?;
            lexor.escape_nl = true;
            #[cfg(feature = "debug")]
            {
                // If debug switch target is next macro
                // Stop and wait for input
                // Only on main level macro
                // TODO This behaviour might change later
                if level == 0 { self.user_input_on_macro(&frag)?; }
                else {self.user_input_on_step(&frag)?;}

                // Clear line_caches
                if level == 0 {
                    self.line_caches.clear();
                }
            }
            frag.clear();
        } 
        else { // Invoke macro

            // Debug
            #[cfg(feature = "debug")]
            {
                // If debug switch target is break point
                // Set switch to next line.
                self.break_point(frag)?;
                // Break point is true , continue
                if frag.name.len() == 0 {
                    lexor.escape_nl = true;
                    return Ok(());
                }
            }

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

                // Debug
                // Debug command after macro evaluation
                // This goes to last line and print last line
                #[cfg(feature = "debug")]
                if !self.is_local(level + 1, &frag.name) {
                    // If debug switch target is next macro
                    // Stop and wait for input
                    // Only on main level macro
                    // TODO This behaviour might change later
                    if level == 0 {self.user_input_on_macro(&frag)?;}
                    else {self.user_input_on_step(&frag)?;}

                    // Clear line_caches
                    if level == 0 {
                        self.line_caches.clear();
                    }
                }

                // If content is none
                // Ignore new line after macro evaluation until any character
                if content.len() == 0 {
                    lexor.escape_nl = true;
                } else {
                    if frag.trimmed {
                        content = Utils::trim(&content)?;
                    }
                    if frag.yield_literal {
                        content = format!("\\*{}*\\", content);
                    }
                    // NOTE
                    // This should come later!!
                    if frag.pipe {
                        self.pipe_value = content;
                        lexor.escape_nl = true;
                    } else {
                        remainder.push_str(&content);
                    }
                }
            } else { // Failed to invoke
                // because macro doesn't exist

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
                } else {
                    // If purge mode
                    // set escape new line 
                    lexor.escape_nl = true;
                }
            }
            // Clear fragment regardless of success
            frag.clear()
        }

        Ok(())
    }

    fn lex_branch_exit_frag(&mut self,ch: char, frag: &mut MacroFragment, remainder: &mut String) {
        frag.whole_string.push(ch);
        remainder.push_str(&frag.whole_string);
        frag.clear();
    }

    // </LEX>
    // End of lex branch methods
    // ==========
    // </PARSE>
    // End of parse related methods
    // ----------

    // ----------
    // Start of miscellaenous methods
    // <MISC>
    /// Get mutable reference of macro map
    pub(crate) fn get_map(&mut self) -> &mut MacroMap {
        &mut self.map
    }

    /// Change temp file target
    pub(crate) fn set_temp_file(&mut self, path: &Path) {
        self.temp_target = (path.to_owned(),OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap());
    }

    pub(crate) fn set_sandbox(&mut self) {
        self.sandbox = true; 
    }

    /// Get temp file's path
    pub(crate) fn get_temp_path(&self) -> &Path {
        &self.temp_target.0
    }

    /// Get temp file's "file" struct
    pub(crate) fn get_temp_file(&self) -> &File {
        &self.temp_target.1
    }

    /// Backup information of current file before processing sandboxed input
    fn backup(&self) -> SandboxBackup {
        SandboxBackup { 
            current_input: self.current_input.clone(), 
            local_macro_map: self.map.local.clone(),
            logger_lines: self.logger.backup_lines(),
        }
    }

    /// Recover backup information into the processor
    fn recover(&mut self, backup: SandboxBackup) {
        // NOTE ::: Set file should come first becuase set_file override line number and character number
        self.logger.set_file(&backup.current_input);
        self.current_input = backup.current_input;
        self.map.local= backup.local_macro_map; 
        self.logger.recover_lines(backup.logger_lines);
    }

    /// Log error
    pub fn log_error(&mut self, log : &str) -> Result<(), RadError> {
        self.logger.elog(log)?;
        Ok(())
    }

    /// Log warning
    pub fn log_warning(&mut self, log : &str) -> Result<(), RadError> {
        self.logger.wlog(log)?;
        Ok(())
    }

    // This is not a backup but fresh set of file information
    /// Set current processing file information for the first time
    fn set_file(&mut self, file: &str) -> Result<(), RadError> {
        let path = &Path::new(file);
        if !path.exists() {
            Err(RadError::InvalidCommandOption(format!("File, \"{}\" doesn't exist, therefore cannot be read by r4d.", path.display())))
        } else {
            self.current_input = file.to_owned();
            self.logger.set_file(file);
            Ok(())
        }
    }

    fn set_input(&mut self, input: &str) -> Result<(), RadError> {
        self.current_input = input.to_owned();
        self.logger.set_file(input);
        Ok(())
    }

    /// Add custom rules without builder pattern
    #[allow(dead_code)]
    pub fn add_custom_rules(&mut self, rules: HashMap<String, MacroRule>) {
        self.map.custom.extend(rules.into_iter());
    }

    // End of miscellaenous methods
    // </MISC>
    // ----------
}

/// Struct for deinition parsing
struct DefineParser{
    arg_cursor :DefineCursor,
    name: String,
    args: String,
    body: String,
    bind: bool,
    container: String,
}

impl DefineParser {
    fn new() -> Self {
        Self {
            arg_cursor : DefineCursor::Name,
            name : String::new(),
            args : String::new(),
            body : String::new(),
            bind : false,
            container : String::new(),
        }
    }

    fn clear(&mut self) {
        self.arg_cursor = DefineCursor::Name;
        self.name.clear();
        self.args.clear();
        self.body.clear();
        self.bind = false;
        self.container.clear();
    }

    // NOTE This method expects valid form of macro invocation
    // Given value should be without outer prentheses
    // e.g. ) name,a1 a2=body text
    fn parse_define(&mut self, text: &str) -> Option<(String, String, String)> {
        self.clear(); // Start in fresh state
        let mut char_iter = text.chars().peekable();
        while let Some(ch) = char_iter.next() {
            match self.arg_cursor {
                DefineCursor::Name => {
                    if let ParseIgnore::Ignore = self.branch_name(ch) {continue;}
                    // If not valid name return None
                    if !self.is_valid_name(ch) { return None; }
                }
                DefineCursor::Args => {
                    if let ParseIgnore::Ignore = self.branch_args(ch) {continue;}
                    // If not valid name return None
                    if !self.is_valid_name(ch) { return None; }
                }
                // Add everything
                DefineCursor::Body => ()
            } 
            self.container.push(ch);
        }

        // This means pattern such as
        // $define(test,Test) 
        // -> This is not a valid pattern
        if self.args.len() == 0 && !self.bind {
            return None;
        }

        // End of body
        self.body.push_str(&self.container);

        Some((self.name.clone(), self.args.clone(), self.body.clone()))
    }

    /// Check if name complies with naming rule
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
    
    fn branch_name(&mut self, ch: char) -> ParseIgnore {
        // $define(variable=something)
        // Don't set argument but directly bind variable to body
        if ch == '=' {
            self.name.push_str(&self.container);
            self.container.clear();
            self.arg_cursor = DefineCursor::Body;
            self.bind = true;
            ParseIgnore::Ignore
        } 
        else if Utils::is_blank_char(ch) {
            // This means pattern like this
            // $define( name ) -> name is registered
            // $define( na me ) -> na is ignored and take me instead
            if self.name.len() != 0 {
                self.container.clear();
                ParseIgnore::None
            } else {
                // Ignore
                ParseIgnore::Ignore
            }
        } 
        // Comma go to args
        else if ch == ',' {
            self.name.push_str(&self.container);
            self.container.clear();
            self.arg_cursor = DefineCursor::Args;
            ParseIgnore::Ignore
        } else {
            ParseIgnore::None
        }
    }

    fn branch_args(&mut self, ch: char) -> ParseIgnore {
        // Blank space separates arguments 
        // TODO: Why check name's length? Is it necessary?
        if Utils::is_blank_char(ch) && self.name.len() != 0 {
            if self.container.len() != 0 {
                self.args.push_str(&self.container);
                self.args.push(' ');
                self.container.clear();
            }
            ParseIgnore::Ignore
        } 
        // Go to body
        else if ch == '=' {
            self.args.push_str(&self.container);
            self.container.clear();
            self.arg_cursor = DefineCursor::Body; 
            ParseIgnore::Ignore
        } 
        // Others
        else {
            ParseIgnore::None
        }
    }
}

enum DefineCursor {
    Name,
    Args,
    Body,
}

enum ParseIgnore {
    Ignore,
    None
}

/// Macro framgent that processor saves fragmented information of the mcaro invocation
#[derive(Debug)]
struct MacroFragment {
    pub whole_string: String,
    pub name: String,
    pub args: String,

    // Macroframgnet related options
    pub pipe: bool,
    pub greedy: bool,
    pub preceding: bool,
    pub yield_literal : bool,
    pub trimmed : bool,
}

impl MacroFragment {
    fn new() -> Self {
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

    fn clear(&mut self) {
        self.whole_string.clear();
        self.name.clear();
        self.args.clear();
        self.pipe = false; 
        self.greedy = false; 
        self.yield_literal = false;
        self.trimmed = false; 
    }

    fn is_empty(&self) -> bool {
        self.whole_string.len() == 0
    }
}

#[derive(Debug)]
enum ParseResult {
    FoundMacro(String),
    Printable(String),
    NoPrint,
    EOI,
}

/// Struct for backing current file and logging information
///
/// This is necessary because some macro processing should be executed in sandboxed environment.
/// e.g. when include macro is called, outer file's information is not helpful at all.
struct SandboxBackup {
    current_input: String,
    local_macro_map: HashMap<String,String>,
    logger_lines: LoggerLines,
}
