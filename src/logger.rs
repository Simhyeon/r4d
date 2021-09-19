//! # Logger
//!
//! Logger handles all kinds of logging logics. Such log can be warning, error or debug logs.

#[cfg(feature = "debug")]
use std::io::BufRead;
use std::io::Write;
#[cfg(feature = "debug")]
use crossterm::{ExecutableCommand, terminal::ClearType};
use crate::models::WriteOption;
use crate::consts::*;
use crate::utils::Utils;
use crate::error::RadError;

/// Logger that controls logging
pub(crate) struct Logger {
    pub(crate) line_number: usize,
    pub(crate) char_number: usize,
    pub(crate) last_line_number: usize,
    pub(crate) last_char_number: usize,
    current_file: String,
    write_option: Option<WriteOption>,
    error_count: usize,
    warning_count: usize,
    chunked: usize,
    #[cfg(feature = "debug")]
    debug_interactive: bool,
}
/// Struct specifically exists to backup information of logger
#[derive(Debug)]
pub(crate) struct LoggerLines {
    line_number: usize,
    char_number: usize,
    last_line_number: usize,
    last_char_number: usize,
}

impl Logger{
    pub fn new() -> Self {
        Self {
            line_number: 0,
            char_number: 0,
            last_line_number: 0,
            last_char_number: 0,
            current_file: String::from("stdin"),
            write_option: None,
            error_count:0,
            warning_count:0,
            chunked : 0,
            #[cfg(feature = "debug")]
            debug_interactive: false,
        }
    }

    /// Enables "chunk" mode whtin logger
    ///
    /// If chunk mode is enabled line_number doesn't mean real line number, rather it means how
    /// much lines has passed since last_line_number.
    pub fn set_chunk(&mut self, switch: bool) {
        if switch {
            self.chunked = self.chunked + 1;
            self.line_number = 0;
            self.char_number = 0;
        } else {
            self.chunked = self.chunked - 1;
        }
    }

    #[cfg(feature = "debug")]
    pub fn set_debug_interactive(&mut self) {
        self.debug_interactive = true;
    }

    pub fn set_write_options(&mut self, write_option: Option<WriteOption>) {
        self.write_option = write_option; 
    }

    /// Backup current line information into a struct
    pub fn backup_lines(&self) -> LoggerLines {
        LoggerLines { line_number: self.line_number, char_number: self.char_number, last_line_number: self.last_line_number, last_char_number: self.last_char_number }
    }

    /// Recover backuped line information from a struct
    pub fn recover_lines(&mut self, logger_lines: LoggerLines) {
        self.line_number =          logger_lines.line_number;
        self.char_number =          logger_lines.char_number;
        self.last_line_number =     logger_lines.last_line_number;
        self.last_char_number =     logger_lines.last_char_number;
    }

    /// Set file's logging information and reset state
    pub fn set_file(&mut self, file: &str) {
        self.current_file = file.to_owned();
        self.line_number = 0;
        self.char_number = 0;
        self.last_line_number = 0;
        self.last_char_number = 0;
    }

    /// Increase line number
    pub fn add_line_number(&mut self) {
        self.line_number = self.line_number + 1;
        self.char_number = 0;
    }
    /// Increase char number
    pub fn add_char_number(&mut self) {
        self.char_number = self.char_number + 1;
    }
    /// Reset char number
    pub fn reset_char_number(&mut self) {
        self.char_number = 0;
    }

    /// Freeze line and char number for logging
    pub fn freeze_number(&mut self) {
        if self.chunked > 0 {
            self.last_line_number = self.line_number + self.last_line_number;
            // In the same line
            if self.line_number != 0 {
                self.last_char_number = self.char_number;
            }
        } else {
            self.last_line_number = self.line_number;
            self.last_char_number = self.char_number;
        }
    }

    // Debug method for development not rdb debugger
    #[allow(dead_code)]
    pub(crate) fn deb(&self) {
        println!("LAST : {}", self.last_line_number);
        println!("LINE : {}", self.line_number);
    }

    /// Try getting last character
    ///
    /// This will have trailing ```->``` if caller macro and callee macro is in same line
    fn try_get_last_char(&self) -> String {
        if self.chunked > 0 && self.line_number == 0 {
            format!("{}->",self.last_char_number)
        }  else {
            self.last_char_number.to_string()
        }
    }

    /// Log error
    pub fn elog(&mut self, log : &str) -> Result<(), RadError> {
        self.error_count = self.error_count + 1; 
        let last_char = self.try_get_last_char();
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(
                        format!(
                            "error : {} -> {}:{}:{}{}",
                            log,
                            self.current_file,
                            self.last_line_number,
                            last_char,
                            LINE_ENDING
                        ).as_bytes()
                    )?;
                }
                WriteOption::Stdout => {
                    eprint!(
                        "{}: {} {} --> {}:{}:{}{}",
                        Utils::red("error"),
                        log,
                        LINE_ENDING,
                        self.current_file,
                        self.last_line_number,
                        last_char,
                        LINE_ENDING
                    );
                }
                WriteOption::Discard => ()
            } // Match end
        }
        Ok(())
    }

    /// Log warning
    pub fn wlog(&mut self, log : &str) -> Result<(), RadError> {
        self.warning_count = self.warning_count + 1; 
        let last_char = self.try_get_last_char();
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(
                        format!(
                            "warning : {} -> {}:{}:{}{}",
                            log,
                            self.current_file,
                            self.last_line_number,
                            last_char,
                            LINE_ENDING
                        ).as_bytes()
                    )?;
                }
                WriteOption::Stdout => {
                    eprintln!(
                        "{}: {} {} --> {}:{}:{}",
                        Utils::yellow("warning"),
                        log,
                        LINE_ENDING,
                        self.current_file,
                        last_char,
                        self.last_char_number
                    );
                }
                WriteOption::Discard => ()
            } // match end
        } 

        Ok(())
    }

    /// Print result of logging of warnings and errors
    pub fn print_result(&mut self) -> Result<(), RadError> {
        if let Some(option) = &mut self.write_option {
            // No warning or error
            if self.error_count == 0 && self.warning_count == 0 {
                return Ok(())
            }
    
            // There is either error or warning
            let error_result = format!("{}: found {} errors",Utils::red("error"), self.error_count);
            let warning_result = format!("{}: found {} warnings",Utils::yellow("warning"), self.warning_count);
            match option {
                WriteOption::File(file) => {
                    if self.error_count > 0 {file.write_all(error_result.as_bytes())?;}
                    if self.warning_count > 0 {file.write_all(warning_result.as_bytes())?;}
                }
                WriteOption::Stdout => {
                    if self.error_count > 0 { eprintln!("{}",error_result);}
                    if self.warning_count > 0 {eprintln!("{}",warning_result);}
                }
                WriteOption::Discard => ()
            }
        } else {
            // Silent option
            // Do nothing
        }

        Ok(())
    }

    // ----------
    // Debug related methods
    // <DEBUG>
    
    // TODO
    // Now this looks worthless?
    #[cfg(feature = "debug")]
    /// Get absolute last line position
    pub fn get_abs_last_line(&self) -> usize {
        self.last_line_number
    }

    #[cfg(feature = "debug")]
    /// Get absolute line position
    pub fn get_abs_line(&self) -> usize {
        if self.chunked > 0{
            self.last_line_number + self.line_number - 1
        } else {
            self.line_number
        }
    }

    /// Log debug information
    #[cfg(feature = "debug")]
    pub fn dlog_command(&self, log : &str, prompt: Option<&str>) -> Result<String, RadError> {
        // Disable line wrap
        if self.debug_interactive {
            std::io::stdout()
                .execute(crossterm::terminal::DisableLineWrap)?;
        }

        let mut input = String::new();
        let prompt = if let Some(content) = prompt { content } else { "" };
        println!("{} : {}",Utils::green(&format!("({})", &prompt)), log);
        print!(">> ");

        // Restore wrapping
        if self.debug_interactive {
            std::io::stdout()
                .execute(crossterm::terminal::EnableLineWrap)?;
        }
        // Flush because print! is not "printed" yet
        std::io::stdout().flush()?;

        // Get user input
        let stdin = std::io::stdin();
        stdin.lock().read_line(&mut input)?;
        if self.debug_interactive {
            // Clear user input line
            // Preceding 1 is for "(input)" prompt
            self.remove_terminal_lines(1 + Utils::count_sentences(log))?;
        }

        Ok(input)
    }

    /// Log debug information
    #[cfg(feature = "debug")]
    pub fn dlog_print(&self, log : &str) -> Result<(), RadError> {
        print!("{} :{}{}", Utils::green(&format!("{}:{}", self.last_line_number,"log")),LINE_ENDING,log);
        Ok(())
    }

    #[cfg(feature = "debug")]
    fn remove_terminal_lines(&self, count: usize) -> Result<(), RadError> {

        std::io::stdout()
            .execute(crossterm::terminal::Clear(ClearType::CurrentLine))?;

        // Range is max exclusive thus min should start from 0
        // e.g. 0..1 only tries once with index 0
        for _ in 0..count {
            std::io::stdout()
                .execute(crossterm::cursor::MoveUp(1))?
                .execute(crossterm::terminal::Clear(ClearType::CurrentLine))?;
        }

        Ok(())
    }

    // End of debug related methods
    // </DEBUG>
    // ----------
} 

/// Debug switch(state) that indicates what debugging behaviours are intended for next branch
#[cfg(feature = "debug")]
pub enum DebugSwitch {
    UntilMacro,
    NextLine,
    NextMacro,
    StepMacro,
    NextBreakPoint(String),
}
