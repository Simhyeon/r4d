use std::io::{Write, BufRead};
#[cfg(feature = "debug")]
use crossterm::{ExecutableCommand, terminal::ClearType};
use crate::models::WriteOption;
use crate::consts::*;
use colored::*;
use crate::error::RadError;

/// Logger that controls whether to print error, warning or not and where to yield the error and warnings.
pub(crate) struct Logger {
    pub(crate) line_number: usize,
    pub(crate) char_number: usize,
    // This is a varaible that calibrates char position when parent caller is same line with child
    // caller find <OFFSET> for detailed explanation
    pub(crate) char_offset_number: usize,
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
/// Struct specifically exists for error loggers backup information
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
            char_offset_number: 0,
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
        if self.chunked > 0 {
            self.last_line_number = self.line_number + self.last_line_number;
            //self.last_char_number = self.char_number + self.last_char_number;
        } else {
            // LINK <OFFSET>
            // $if($test()) -> In this case,
            // $if's last char is 2 while theortically $test()'s last char should be 6
            // Freeze number is called while a cursor is positioned in (, which index is 4
            // and inner argument parse starts from $ which index is 5
            // Thus offset should be char - last char + 2 
            // Constant is 2 not 1 because subtraction gets a value of "distance - 1"
            // Other 1 is for un-calculated "(" character
            self.char_offset_number = self.char_number - self.last_char_number + 2;
            self.last_line_number = self.line_number;
            self.last_char_number = self.char_number;
        }
        //self.print();
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
            let last_char = if self.chunked > 0 && self.line_number == 0 {
                self.last_char_number + self.char_offset_number
            } else {
                self.last_char_number
            };

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
                        "error".red(),
                        log,
                        LINE_ENDING,
                        self.current_file,
                        self.last_line_number,
                        last_char,
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
            let last_char = if self.chunked > 0 && self.line_number == 0 {
                self.last_char_number + self.char_offset_number
            } else {
                self.last_char_number
            };

            match option {
                WriteOption::File(file) => {
                    file.write_all(format!("warning : {} -> {}:{}:{}{}",log,self.current_file, self.last_line_number, last_char,LINE_ENDING).as_bytes())?;
                }
                WriteOption::Stdout => {
                    eprintln!(
                        "{}: {} {} --> {}:{}:{}",
                        "warning".yellow(),
                        log,
                        LINE_ENDING,
                        self.current_file,
                        self.last_line_number,
                        last_char);
                }
            }
        } else {
            // Silent option
            // Do nothing
        }

        Ok(())
    }
    
    #[cfg(feature = "debug")]
    pub fn get_abs_last_line(&self) -> usize {
        self.last_line_number
    }

    #[cfg(feature = "debug")]
    pub fn get_abs_line(&self) -> usize {
        if self.chunked > 0{
            self.last_line_number + self.line_number - 1
        } else {
            self.line_number
        }
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
    pub fn dlog_command(&self, log : &str, prompt: Option<&str>) -> Result<String, RadError> {
        // Disable line wrap
        if self.debug_interactive {
            std::io::stdout()
                .execute(crossterm::terminal::DisableLineWrap)?;
        }

        let mut input = String::new();
        let prompt = if let Some(content) = prompt { content } else { "" };
        println!("{} : {}",format!("({})", &prompt.green()).green(), log);
        print!(">> ");

        // Restore wrapping
        if self.debug_interactive {
            std::io::stdout()
                .execute(crossterm::terminal::EnableLineWrap)?;
        }
        // Flush because print! is not "printed" yet
        std::io::stdout().flush()?;

        use crate::utils::Utils;
        // Get user input
        let stdin = std::io::stdin();
        stdin.lock().read_line(&mut input)?;
        if self.debug_interactive {
            // Clear user input line
            // Preceding 1 is for "(input)" prompt
            self.remove_terminal_lines(1 + Utils::count_newlines(log))?;
        }

        Ok(input)
    }

    /// Log debug information
    #[cfg(feature = "debug")]
    pub fn dlog_print(&self, log : &str) -> Result<(), RadError> {
        print!("{} :{}{}", format!("{}:{}", self.last_line_number,"log").green(),LINE_ENDING,log);
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
} 

#[cfg(feature = "debug")]
pub enum DebugSwitch {
    UntilMacro,
    NextLine,
    NextMacro,
    StepMacro,
    NextBreakPoint(String),
}
