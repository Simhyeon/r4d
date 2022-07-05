//! # Logger
//!
//! Logger handles all kinds of logging logics. Such log can be warning, error or debug logs.

use crate::models::{ProcessInput, RadResult, WriteOption};
use crate::utils::Utils;
use crate::{consts::*, RadError};
use std::fmt::Write;
use std::io::Write as _;

/// Struct specifically exists to backup information of logger
#[derive(Debug)]
pub(crate) struct LoggerLines {
    line_number: usize,
    char_number: usize,
    last_line_number: usize,
    last_char_number: usize,
}

/// Logger that controls logging
pub(crate) struct Logger<'logger> {
    suppresion_type: WarningType,
    line_number: usize,
    char_number: usize,
    last_line_number: usize,
    last_char_number: usize,
    current_input: ProcessInput,
    write_option: Option<WriteOption<'logger>>,
    error_count: usize,
    warning_count: usize,
    assert_success: usize,
    assert_fail: usize,
    chunked: usize,
    pub(crate) assert: bool,
}

impl<'logger> Logger<'logger> {
    pub fn new() -> Self {
        Self {
            suppresion_type: WarningType::None,
            line_number: 0,
            char_number: 0,
            last_line_number: 0,
            last_char_number: 0,
            current_input: ProcessInput::Stdin,
            write_option: None,
            error_count: 0,
            warning_count: 0,
            assert_success: 0,
            assert_fail: 0,
            chunked: 0,
            assert: false,
        }
    }

    pub fn assert(&mut self) {
        self.assert = true;
    }

    pub fn suppress_warning(&mut self, warning_type: WarningType) {
        self.suppresion_type = warning_type;
    }

    /// Enables "chunk" mode whtin logger
    ///
    /// If chunk mode is enabled line_number doesn't mean real line number,
    /// rather it means how much lines has passed since last_line_number.
    pub fn set_chunk(&mut self, switch: bool) {
        if switch {
            self.chunked += 1;
            self.line_number = 0;
            self.char_number = 0;
        } else {
            self.chunked -= 1;
        }
    }

    pub fn set_write_option(&mut self, write_option: Option<WriteOption<'logger>>) {
        self.write_option = write_option;
    }

    /// Backup current line information into a struct
    pub fn backup_lines(&self) -> LoggerLines {
        LoggerLines {
            line_number: self.line_number,
            char_number: self.char_number,
            last_line_number: self.last_line_number,
            last_char_number: self.last_char_number,
        }
    }

    /// Recover backuped line information from a struct
    pub fn recover_lines(&mut self, logger_lines: LoggerLines) {
        self.line_number = logger_lines.line_number;
        self.char_number = logger_lines.char_number;
        self.last_line_number = logger_lines.last_line_number;
        self.last_char_number = logger_lines.last_char_number;
    }

    /// Set file's logging information and reset state
    pub fn set_input(&mut self, input: &ProcessInput) {
        self.current_input = input.clone();
        self.line_number = 0;
        self.char_number = 0;
        self.last_line_number = 0;
        self.last_char_number = 0;
    }

    /// Increase line number
    pub fn add_line_number(&mut self) {
        self.line_number += 1;
        self.char_number = 0;
    }
    /// Increase char number
    pub fn add_char_number(&mut self) {
        self.char_number += 1;
    }

    pub fn reset_everything(&mut self) {
        self.line_number = 0;
        self.char_number = 0;
        self.last_line_number = 0;
        self.last_char_number = 0;
    }

    /// Reset char number
    pub fn reset_char_number(&mut self) {
        self.char_number = 0;
    }

    /// Freeze line and char number for logging
    pub fn freeze_number(&mut self) {
        if self.chunked > 0 {
            self.last_line_number += self.line_number;
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
        eprintln!("LAST : {}", self.last_line_number);
        eprintln!("LINE : {}", self.line_number);
    }

    /// Try getting last character
    ///
    /// This will have trailing ```->``` if caller macro and callee macro is in same line
    fn try_get_last_char(&self) -> String {
        if self.chunked > 0 && self.line_number == 0 {
            format!("{}~~", self.last_char_number)
        } else {
            self.last_char_number.to_string()
        }
    }

    /// Log error
    pub fn elog(&mut self, log: &str) -> RadResult<()> {
        self.error_count += 1;

        if self.assert {
            return Ok(());
        }
        let last_char = self.try_get_last_char();
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(
                        format!(
                            "error : {} -> {}:{}:{}{}",
                            log, self.current_input, self.last_line_number, last_char, LINE_ENDING
                        )
                        .as_bytes(),
                    )?;
                }
                WriteOption::Terminal => {
                    write!(
                        std::io::stderr(),
                        "{}: {} {} --> {}:{}:{}{}",
                        Utils::red("error"),
                        log,
                        LINE_ENDING,
                        self.current_input,
                        self.last_line_number,
                        last_char,
                        LINE_ENDING
                    )?;
                }
                WriteOption::Variable(var) => {
                    write!(
                        var,
                        "error : {} -> {}:{}:{}{}",
                        log, self.current_input, self.last_line_number, last_char, LINE_ENDING
                    )?;
                }
                WriteOption::Discard | WriteOption::Return => (),
            } // Match end
        }
        Ok(())
    }

    #[cfg(feature = "debug")]
    pub fn elog_no_prompt(&mut self, log: impl std::fmt::Display) -> RadResult<()> {
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(log.to_string().as_bytes())?;
                }
                WriteOption::Terminal => {
                    write!(std::io::stderr(), "{}", log)?;
                }
                WriteOption::Variable(var) => var.push_str(&log.to_string()),
                WriteOption::Discard | WriteOption::Return => (),
            } // match end
        }
        Ok(())
    }

    /// Log warning
    pub fn wlog(&mut self, log: &str, warning_type: WarningType) -> RadResult<()> {
        if self.suppresion_type == WarningType::Any || self.suppresion_type == warning_type {
            return Ok(());
        }

        self.warning_count += 1;

        if self.assert {
            return Ok(());
        }
        let last_char = self.try_get_last_char();
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(
                        format!(
                            "warning : {} -> {}:{}:{}{}",
                            log, self.current_input, self.last_line_number, last_char, LINE_ENDING
                        )
                        .as_bytes(),
                    )?;
                }
                WriteOption::Terminal => {
                    writeln!(
                        std::io::stderr(),
                        "{}: {} {} --> {}:{}:{}",
                        Utils::yellow("warning"),
                        log,
                        LINE_ENDING,
                        self.current_input,
                        last_char,
                        self.last_char_number
                    )?;
                }
                WriteOption::Variable(var) => {
                    write!(
                        var,
                        "error : {} -> {}:{}:{}{}",
                        log, self.current_input, self.last_line_number, last_char, LINE_ENDING
                    )?;
                }
                WriteOption::Discard | WriteOption::Return => (),
            } // match end
        }

        Ok(())
    }

    /// Assertion log
    pub fn alog(&mut self, success: bool) -> RadResult<()> {
        if success {
            self.assert_success += 1;
            return Ok(());
        }
        self.assert_fail += 1;
        let last_char = self.try_get_last_char();

        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.write_all(
                        format!(
                            "assert fail -> {}:{}:{}{}",
                            self.current_input, self.last_line_number, last_char, LINE_ENDING
                        )
                        .as_bytes(),
                    )?;
                }
                WriteOption::Terminal => {
                    writeln!(
                        std::io::stderr(),
                        "{} -> {}:{}:{}",
                        Utils::red("assert fail"),
                        self.current_input,
                        self.last_line_number,
                        last_char
                    )?;
                }
                WriteOption::Variable(var) => write!(
                    var,
                    "assert fail -> {}:{}:{}{}",
                    self.current_input, self.last_line_number, last_char, LINE_ENDING
                )?,
                WriteOption::Discard | WriteOption::Return => (),
            } // match end
        }

        Ok(())
    }

    /// Print result of logging of warnings and errors
    pub fn print_result(&mut self) -> RadResult<()> {
        if let Some(option) = &mut self.write_option {
            // There is either error or warning
            let error_result =
                format!("{}: found {} errors", Utils::red("error"), self.error_count);
            let warning_result = format!(
                "{}: found {} warnings",
                Utils::yellow("warning"),
                self.warning_count
            );
            let assert_result = format!(
                "
{}
SUCCESS : {}
FAIL: {}",
                Utils::green("Assert"),
                self.assert_success,
                self.assert_fail
            );
            match option {
                WriteOption::File(file) => {
                    if self.error_count > 0 {
                        file.write_all(error_result.as_bytes())?;
                    }
                    if self.warning_count > 0 {
                        file.write_all(warning_result.as_bytes())?;
                    }
                    if self.assert {
                        file.write_all(assert_result.as_bytes())?;
                    }
                }
                WriteOption::Terminal => {
                    if self.error_count > 0 {
                        writeln!(std::io::stderr(), "{}", error_result)?;
                    }
                    if self.warning_count > 0 {
                        writeln!(std::io::stderr(), "{}", warning_result)?;
                    }
                    if self.assert {
                        writeln!(std::io::stderr(), "{}", assert_result)?;
                    }
                }
                WriteOption::Discard | WriteOption::Variable(_) | WriteOption::Return => (),
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

    #[cfg(feature = "debug")]
    /// Get absolute last line position
    pub fn get_abs_last_line(&self) -> usize {
        self.last_line_number
    }

    #[cfg(feature = "debug")]
    /// Get absolute line position
    pub fn get_abs_line(&self) -> usize {
        if self.chunked > 0 {
            self.last_line_number + self.line_number - 1
        } else {
            self.line_number
        }
    }

    /// Log debug information
    #[cfg(feature = "debug")]
    pub fn dlog_print(&mut self, log: &str) -> RadResult<()> {
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::Terminal => {
                    write!(
                        std::io::stderr(),
                        "{}{}{}",
                        Utils::green(&format!("{}:log", self.last_line_number)),
                        LINE_ENDING,
                        log
                    )?;
                }
                WriteOption::File(file) => {
                    file.write_all(
                        format!("{}:log{}{}", self.last_line_number, LINE_ENDING, log).as_bytes(),
                    )?;
                }
                _ => (),
            }
        }
        Ok(())
    }
    // End of debug related methods
    // </DEBUG>
    // ----------
}

/// Type variant or warning
///
/// - None : Default value
/// - Security : Security related warning
/// - Sanity : Warning about possible errors
/// - Any : Both warnings
#[derive(PartialEq)]
pub enum WarningType {
    /// Default wrapping type
    None,
    /// About possibly dangerous behaviours
    Security,
    /// About possibly unintended behaviours
    Sanity,
    /// Both warnings
    Any,
}

impl std::str::FromStr for WarningType {
    type Err = RadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "none" => Self::None,
            "security" => Self::Security,
            "sanity" => Self::Sanity,
            "any" => Self::Any,
            _ => {
                return Err(RadError::InvalidConversion(format!(
                    "Cannot convert \"{}\" into WarningType",
                    s
                )))
            }
        })
    }
}
