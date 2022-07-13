use crate::consts::*;
use crate::logger::Logger;
use crate::models::{DiffOption, MacroFragment, RadResult};
use crate::utils::Utils;
use crossterm::{terminal::ClearType, ExecutableCommand};
use similar::ChangeTag;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Write};
use std::path::Path;

/// Debugger
pub(crate) struct Debugger {
    pub(crate) debug: bool,
    pub(crate) log: bool,
    pub(crate) debug_switch: DebugSwitch,
    pub(crate) line_number: usize,
    // This is a global line number storage for various debugging usages
    // This is a bit bloaty, but debugging needs functionality over efficiency
    pub(crate) line_caches: HashMap<usize, String>,
    pub(crate) do_yield_diff: bool,
    pub(crate) diff_only_change: bool,
    pub(crate) diff_original: Option<File>,
    pub(crate) diff_processed: Option<File>,
    pub(crate) interactive: bool,
    prompt_log: Option<String>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            debug: false,
            log: false,
            debug_switch: DebugSwitch::NextLine,
            line_number: 1,
            line_caches: HashMap::new(),
            do_yield_diff: false,
            diff_only_change: false,
            diff_original: None,
            diff_processed: None,
            interactive: false,
            prompt_log: None,
        }
    }

    /// Enable diff logic
    ///
    /// WIth diff enabled, diff information will be saved to two separate files
    pub fn enable_diff(&mut self, diff_option: DiffOption) -> RadResult<()> {
        // DiffOption specific operation
        match diff_option {
            DiffOption::None => return Ok(()), // No diff, return
            DiffOption::Change => self.diff_only_change = true,
            _ => (),
        }
        self.do_yield_diff = true;
        self.diff_original = Some(
            OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .truncate(true)
                .open(Path::new(DIFF_SOURCE_FILE))?,
        );
        self.diff_processed = Some(
            OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .truncate(true)
                .open(Path::new(DIFF_OUT_FILE))?,
        );

        Ok(())
    }

    /// Enable interactive mode
    ///
    /// This clears terminal if possible
    pub fn set_interactive(&mut self) {
        self.interactive = true;
    }

    /// Get debug command
    ///
    /// Get user input and parsed the given command
    pub fn get_command(&self, log: &str, prompt: Option<&str>) -> RadResult<String> {
        // Disable line wrap
        if self.interactive {
            std::io::stdout().execute(crossterm::terminal::DisableLineWrap)?;
        }

        let mut input = String::new();
        let prompt = if let Some(content) = prompt {
            content
        } else {
            ""
        };
        // TODO
        // This utilizes eprint variant which can yield not handled error
        // but it is generally ok because debugger is mostly self contained process
        eprintln!(
            "{} : {}",
            Utils::green(&format!("({})", &prompt), false),
            log
        );
        eprint!(">> ");

        // Restore wrapping
        if self.interactive {
            std::io::stdout().execute(crossterm::terminal::EnableLineWrap)?;
        }
        // Flush because eprint! is not "printed" yet
        std::io::stdout().flush()?;

        // Get user input
        let stdin = std::io::stdin();
        stdin.lock().read_line(&mut input)?;
        if self.interactive {
            // Clear user input line
            // Preceding 1 is for "(input)" prompt
            self.remove_terminal_lines(1 + Utils::count_sentences(log))?;
        }

        Ok(input)
    }

    /// Remove terminal lines by given count
    fn remove_terminal_lines(&self, count: usize) -> RadResult<()> {
        // Clear current line
        std::io::stdout().execute(crossterm::terminal::Clear(ClearType::CurrentLine))?;

        // Range is max exclusive thus min should start from 0
        // e.g. 0..1 only tries once with index 0
        for _ in 0..count {
            std::io::stdout()
                .execute(crossterm::cursor::MoveUp(1))?
                .execute(crossterm::terminal::Clear(ClearType::CurrentLine))?;
        }

        Ok(())
    }

    /// Print differences of original and processed
    pub fn yield_diff(&self, logger: &mut Logger) -> RadResult<()> {
        if !self.do_yield_diff {
            return Ok(());
        }

        let source = std::fs::read_to_string(Path::new(DIFF_SOURCE_FILE))?;
        let processed = std::fs::read_to_string(Path::new(DIFF_OUT_FILE))?;
        let result = similar::TextDiff::from_lines(&source, &processed);

        let mut log: String;
        // Color function reference
        let mut colorfunc: Option<ColorDisplayFunc>;

        // Print header
        logger.elog_no_line(format!("{0}DIFF : {0}", LINE_ENDING))?;

        // Print changes with color support
        for change in result.iter_all_changes() {
            colorfunc = None;
            match change.tag() {
                ChangeTag::Delete => {
                    log = format!("- {}", change);
                    colorfunc.replace(Utils::red);
                }
                ChangeTag::Insert => {
                    log = format!("+ {}", change);
                    colorfunc.replace(Utils::green);
                }
                ChangeTag::Equal => {
                    if !self.diff_only_change {
                        log = format!("  {}", change);
                    } else {
                        // Log should be assigned at least once
                        // because later codes expectes log to be exsitent.
                        // It is ok to create empty owned string,
                        // compiler is smart enough not to allocate
                        // any memory for empty string.
                        log = "".to_owned();
                    }
                }
            }
            if let Some(func) = colorfunc {
                // Bind display to log
                let log = if logger.is_logging_to_file() {
                    // No color
                    Box::new(log)
                } else {
                    func(&log, false)
                };
                logger.elog_no_line(&log)?;
            } else {
                logger.elog_no_line(&log)?;
            }
        }

        std::fs::remove_file(DIFF_SOURCE_FILE)?;
        std::fs::remove_file(DIFF_OUT_FILE)?;
        Ok(())
    }

    /// Process breakpoint
    pub(crate) fn break_point(&mut self, frag: &mut MacroFragment) -> RadResult<()> {
        if &frag.name == "BR" {
            if self.debug {
                if let DebugSwitch::NextBreakPoint(name) = &self.debug_switch {
                    // Name is empty or same with frag.args
                    if name == &frag.args || name.is_empty() {
                        self.debug_switch = DebugSwitch::NextLine;
                    }
                }
                // Clear fragment
                frag.clear();
                return Ok(());
            }
            frag.clear();
        }

        Ok(())
    }

    /// Print debug information log
    pub(crate) fn print_log(
        &mut self,
        macro_name: &str,
        raw_args: &str,
        frag: &MacroFragment,
        logger: &mut Logger,
    ) -> RadResult<()> {
        if !self.log {
            return Ok(());
        }
        let attributes = self.print_macro_attr(frag);
        logger.dlog_print(&format!(
            r#"Name    = "{}"{}Attr    ={}{}Args    = "{}"{}---{}"#,
            macro_name, LINE_ENDING, LINE_ENDING, attributes, raw_args, LINE_ENDING, LINE_ENDING
        ))?;
        Ok(())
    }

    /// Format macro framgent attributes
    fn print_macro_attr(&self, frag: &MacroFragment) -> String {
        format!(
            r#"Greedy  : {}{}Pipe    : {}{}Literal : {}{}Trimmed : {}{}"#,
            frag.greedy,
            LINE_ENDING,
            frag.pipe,
            LINE_ENDING,
            frag.yield_literal,
            LINE_ENDING,
            frag.trimmed,
            LINE_ENDING
        )
    }

    /// Get user input command before processing starts
    pub(crate) fn user_input_on_start(
        &mut self,
        current_input: &str,
        logger_line: usize,
    ) -> RadResult<()> {
        // Stop by lines if debug option is lines
        if self.debug {
            let mut log = if let Some(pl) = self.prompt_log.take() {
                pl
            } else {
                "Default is next. Ctrl + c to exit.".to_owned()
            };

            self.command_loop(&mut log, current_input, None, logger_line)?;
        }
        Ok(())
    }

    /// Prompt user input until break condition has been met
    fn user_input_prompt(
        &mut self,
        frag: &MacroFragment,
        initial_prompt: &str,
        logger_line: usize,
    ) -> RadResult<()> {
        // Respect custom prompt log if it exists
        let mut log = if let Some(pl) = self.prompt_log.take() {
            pl
        } else {
            match &self.debug_switch {
                &DebugSwitch::NextMacro | &DebugSwitch::StepMacro => {
                    self.line_caches.get(&logger_line).unwrap().to_owned()
                }
                _ => self.line_caches.get(&self.line_number).unwrap().to_owned(),
            }
        };

        self.command_loop(&mut log, initial_prompt, Some(frag), logger_line)?;
        Ok(())
    }

    /// Continuously get user input until break situation
    fn command_loop(
        &mut self,
        log: &mut String,
        prompt: &str,
        frag: Option<&MacroFragment>,
        logger_line: usize,
    ) -> RadResult<()> {
        let mut do_continue = true;
        while do_continue {
            // This technically strips newline feed regardless of platforms
            // It is ok to simply convert to a single line because it is logically a single
            let input = self.debug_wait_input(log, Some(prompt))?;
            // Strip newline
            let input = input.lines().next().unwrap();

            do_continue = self.parse_debug_command_and_continue(input, frag, log, logger_line)?;
        }

        Ok(())
    }

    /// Get user input on line
    ///
    /// This method should be called before evaluation of a line
    pub fn user_input_on_line(
        &mut self,
        frag: &MacroFragment,
        logger_line: usize,
    ) -> RadResult<()> {
        // Stop by lines if debug option is lines
        if self.debug {
            // Only when debugswitch is nextline
            if DebugSwitch::NextLine == self.debug_switch {
                self.user_input_prompt(frag, "line", logger_line)?;
            }
        }
        Ok(())
    }

    /// Get user input before macro execution
    pub fn user_input_before_macro(
        &mut self,
        frag: &MacroFragment,
        logger_line: usize,
    ) -> RadResult<()> {
        // Stop by lines if debug option is lines
        if self.debug && self.debug_switch == DebugSwitch::UntilMacro {
            self.user_input_prompt(frag, &(String::from("until:") + &frag.name), logger_line)?;
        }
        Ok(())
    }

    /// Get user input after execution
    pub fn user_input_on_macro(
        &mut self,
        frag: &MacroFragment,
        logger_line: usize,
    ) -> RadResult<()> {
        // Stop by lines if debug option is lines
        if self.debug
            && (self.debug_switch == DebugSwitch::NextMacro
                || self.debug_switch == DebugSwitch::StepMacro)
        {
            self.user_input_prompt(frag, &frag.name, logger_line)?;
        }
        Ok(())
    }

    /// Get user input on execution but also nested macro can be targeted
    pub fn user_input_on_step(
        &mut self,
        frag: &MacroFragment,
        logger_line: usize,
    ) -> RadResult<()> {
        // Stop by lines if debug option is lines
        if self.debug && DebugSwitch::StepMacro == self.debug_switch {
            self.user_input_prompt(frag, &frag.name, logger_line)?;
        }
        Ok(())
    }

    /// Get user input and evaluates whether loop of input prompt should be breaked or not
    pub fn parse_debug_command_and_continue(
        &mut self,
        command_input: &str,
        frag: Option<&MacroFragment>,
        log: &mut String,
        logger_line: usize,
    ) -> RadResult<bool> {
        let command_input: Vec<&str> = command_input.split(' ').collect();
        let command = command_input[0];
        // Default is empty &str ""
        let command_args = if command_input.len() == 2 {
            command_input[1]
        } else {
            ""
        };

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
                    self.check_command_print(log, command_args, frag, logger_line);
                } else {
                    // No fragment which means it is the start of file
                    // or maybe in static line
                    return Ok(false);
                }
                // Get user input again
                return Ok(true);
            }
            // Invalid command
            _ => {
                *log = format!("Invalid command : {} {}", command, &command_args);
                return Ok(true);
            }
        } // End match

        // Unless specific cases,
        // Continue without any loop
        Ok(false)
    }

    /// Check command print's content
    fn check_command_print(
        &self,
        log: &mut String,
        command_args: &str,
        frag: &MacroFragment,
        logger_line: usize,
    ) {
        match command_args.to_lowercase().as_str() {
            // Only name
            "name" | "n" => {
                *log = frag.name.to_owned();
            }
            // Current line number
            "line" | "l" => match &self.debug_switch {
                DebugSwitch::StepMacro | DebugSwitch::NextMacro => {
                    *log = logger_line.to_string();
                }
                _ => {
                    *log = self.line_number.to_string();
                }
            },
            // Span of codes,macro chunk
            "span" | "s" => {
                let mut line_number = match &self.debug_switch {
                    &DebugSwitch::NextMacro => logger_line,
                    _ => self.line_number,
                };

                let mut sums = String::new();
                // This puts lines in invert order
                while let Some(line) = self.line_caches.get(&line_number) {
                    let mut this_line = format!("{}{}", LINE_ENDING, line);
                    this_line.push_str(&sums);
                    sums = this_line;
                    line_number -= 1;
                }

                // Put prompt log "Span" on top
                let mut this_line = "\"Span\"".to_string();
                this_line.push_str(&sums);
                sums = this_line;

                *log = sums;
            }
            // Current line text
            "text" | "t" => match &self.debug_switch {
                DebugSwitch::StepMacro | DebugSwitch::NextMacro => {
                    *log = self.line_caches.get(&logger_line).unwrap().to_owned();
                }
                _ => {
                    *log = self.line_caches.get(&self.line_number).unwrap().to_owned();
                }
            },
            // Current argument
            "arg" | "a" => {
                *log = frag.processed_args.to_owned();
            }
            // Original argument
            "raw" | "r" => {
                *log = frag.args.to_owned();
            }
            // Invalid argument
            _ => {
                *log = format!("Invalid argument \"{}\"", &command_args);
            }
        } // End match
    }

    /// Bridge function that waits user's stdin input
    pub fn debug_wait_input(&self, log: &str, prompt: Option<&str>) -> RadResult<String> {
        self.get_command(log, prompt)
    }

    pub fn inc_line_number(&mut self) {
        self.line_number += 1;
    }

    pub fn add_line_cache(&mut self, line: &str) {
        self.line_caches
            .insert(self.line_number, line.lines().next().unwrap().to_owned());
    }

    pub fn clear_line_cache(&mut self) {
        self.line_caches.clear();
    }

    // Save original content to a file for diff check
    pub fn write_diff_original(&mut self, content: &str) -> RadResult<()> {
        if self.do_yield_diff {
            self.diff_original
                .as_ref()
                .unwrap()
                .write_all(content.as_bytes())?;
        }
        Ok(())
    }

    // Save processed content to a file for diff check
    pub fn write_diff_processed(&mut self, content: &str) -> RadResult<()> {
        if self.do_yield_diff {
            self.diff_processed
                .as_ref()
                .unwrap()
                .write_all(content.as_bytes())?;
        }
        Ok(())
    }

    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt_log.replace(prompt.to_owned());
    }
}

/// Debug switch(state) that indicates what debugging behaviours are intended for next branch
#[derive(PartialEq)]
pub enum DebugSwitch {
    UntilMacro,
    NextLine,
    NextMacro,
    StepMacro,
    NextBreakPoint(String),
}
