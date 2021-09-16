use std::io::Write;
use crate::models::WriteOption;
use crate::consts::LINE_ENDING;
use colored::*;
use crate::error::RadError;

/// Logger that controls whether to print error, warning or not and where to yield the error and warnings.
pub(crate) struct Logger {
    line_number: usize,
    char_number: usize,
    last_line_number: usize,
    last_char_number: usize,
    current_file: String,
    write_option: Option<WriteOption>,
    debug_options: Option<Vec<DebugOption>>,
    error_count: usize,
    warning_count: usize,
}
/// Struct specifically exists for error loggers backup information
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
            debug_options: None,
            error_count:0,
            warning_count:0,
        }
    }

    pub fn set_write_options(&mut self, write_option: Option<WriteOption>) {
        self.write_option = write_option; 
    }

    pub fn set_debug_options(&mut self, debug_options: Option<Vec<DebugOption>>) {
        self.debug_options = debug_options; 
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

    /// Set file's logging information
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
        self.last_line_number = self.line_number;
        self.last_char_number = self.char_number;
    }

    // TODO
    // Check if this is necessary
    #[allow(dead_code)]
    pub fn elog_panic(&mut self, log: &str, error: RadError) -> Result<(), RadError> {
        self.elog(log)?;

        Err(error)
    }

    /// Log error
    pub fn elog(&mut self, log : &str) -> Result<(), RadError> {
        self.error_count = self.error_count + 1; 
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(format!("error : {} -> {}:{}:{}{}",log,self.current_file, self.line_number, self.char_number,LINE_ENDING).as_bytes())?;
                }
                WriteOption::Stdout => {
                    eprint!(
                        "{}: {} {} --> {}:{}:{}{}",
                        "error".red(),
                        log,
                        LINE_ENDING,
                        self.current_file,
                        self.last_line_number,
                        self.last_char_number,
                        LINE_ENDING);
                }
            }
        } else {
            // Silent option
            // Do nothing
        }

        Ok(())
    }

    /// Log warning
    pub fn wlog(&mut self, log : &str) -> Result<(), RadError> {
        self.warning_count = self.warning_count + 1; 
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(format!("warning : {} -> {}:{}:{}{}",log,self.current_file, self.line_number, self.char_number,LINE_ENDING).as_bytes())?;
                }
                WriteOption::Stdout => {
                    eprintln!(
                        "{}: {} {} --> {}:{}:{}",
                        "warning".yellow(),
                        log,
                        LINE_ENDING,
                        self.current_file,
                        self.last_line_number,
                        self.last_char_number);
                }
            }
        } else {
            // Silent option
            // Do nothing
        }

        Ok(())
    }

    pub fn print_result(&mut self) -> Result<(), RadError> {
        if let Some(option) = &mut self.write_option {
            // No warning or error
            if self.error_count == 0 && self.warning_count == 0 {
                return Ok(())
            }
    
            // There is either error or warning
            let error_result = format!("{}: found {} errors","error".red(), self.error_count);
            let warning_result = format!("{}: found {} warnings","warning".yellow(), self.warning_count);
            match option {
                WriteOption::File(file) => {
                    if self.error_count > 0 {file.write_all(error_result.as_bytes())?;}
                    if self.warning_count > 0 {file.write_all(warning_result.as_bytes())?;}
                }
                WriteOption::Stdout => {
                    if self.error_count > 0 { eprintln!("{}",error_result);}
                    if self.warning_count > 0 {eprintln!("{}",warning_result);}
                }
            }
        } else {
            // Silent option
            // Do nothing
        }

        Ok(())
    }

    // ==========
    // Debug related methods
    
    /// Log debug information
    #[cfg(feature = "debug")]
    pub fn dlog_wait(&self, debug_option: DebugOption) -> Result<(), RadError> {
        // Don't do anything
        if !self.has_debug_option(debug_option) {
            return Ok(());
        }

        use crossterm::{ExecutableCommand, terminal::ClearType};

        let mut input = String::new();
        print!("{} ", format!("{}:{}", self.line_number,"input").green());
        // Flush because print! is not "printed" yet
        std::io::stdout().flush()?;
        // Get user input
        std::io::stdin().read_line(&mut input)?;
        // Clear user input line
        std::io::stdout()
            .execute(crossterm::terminal::Clear(ClearType::CurrentLine))?
            .execute(crossterm::cursor::MoveUp(1))?
            .execute(crossterm::terminal::Clear(ClearType::CurrentLine))?;
        
        Ok(())
    }

    /// Log debug information
    #[cfg(feature = "debug")]
    pub fn dlog_print(&self, debug_option: DebugOption, log : &str) -> Result<(), RadError> {
        // Don't do anything
        if !self.has_debug_option(debug_option) {
            return Ok(());
        }

        print!("{} ->> {}", format!("{}:{}", self.line_number,"log").green(), log);
        
        Ok(())
    }

    #[cfg(feature = "debug")]
    fn has_debug_option(&self, debug_option: DebugOption) -> bool {
        if let Some(options) = &self.debug_options {
            for opt in options {
                if *opt == debug_option {
                    return true;
                }
            }
            false
        } else {
            false
        }
    }
} 

#[derive(PartialEq)]
pub enum DebugOption {
    Log,
    Lines,
    Break,
    None,
}

impl DebugOption {
    pub fn new(raw : &str) -> Self {
        match raw.to_lowercase().as_str() {
            "lines" => {
                Self::Lines
            }
            "break" => {
                Self::Break
            }
            "log" => {
                Self::Log
            }
            _ => {
                Self::None
            }
        }
    }
}
