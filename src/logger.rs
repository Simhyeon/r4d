//! # Logger
//!
//! Logger handles all kinds of logging logics. Such log can be warning, error or debug logs.

use crate::models::{ProcessInput, RadResult, WriteOption};
use crate::utils::Utils;
use crate::{consts::*, RadError};
use std::fmt::Write;
use std::io::Write as _;
use tracks::{Track, Tracker};

/// Logger that controls logging
pub(crate) struct Logger<'logger> {
    suppresion_type: WarningType,
    current_input: ProcessInput,
    pub(crate) tracker_stack: NestedTracker,
    pub(crate) write_option: Option<WriteOption<'logger>>,
    error_count: usize,
    warning_count: usize,
    assert_success: usize,
    assert_fail: usize,
    pub(crate) assert: bool,
}

/// TODO
/// Apply this struct to Logger struct
pub struct LoggerStat {
    error_count: usize,
    warning_count: usize,
    assert_success: usize,
    assert_fail: usize,
    pub(crate) assert: bool,
}

impl<'logger> Logger<'logger> {
    pub fn new() -> Self {
        Self {
            suppresion_type: WarningType::None,
            current_input: ProcessInput::Stdin,
            write_option: None,
            tracker_stack: NestedTracker::new(),
            error_count: 0,
            warning_count: 0,
            assert_success: 0,
            assert_fail: 0,
            assert: false,
        }
    }

    pub fn set_assert(&mut self) {
        self.assert = true;
    }

    pub fn suppress_warning(&mut self, warning_type: WarningType) {
        self.suppresion_type = warning_type;
    }

    pub fn set_write_option(&mut self, write_option: Option<WriteOption<'logger>>) {
        self.write_option = write_option;
    }

    // ----Tracker methods----

    pub fn start_new_tracker(&mut self) {
        self.tracker_stack.increase_level();
    }

    pub fn stop_last_tracker(&mut self) {
        self.tracker_stack.decrease_level();
    }

    pub fn get_track_count(&self) -> usize {
        self.tracker_stack.tracker().get_track_counts()
    }

    /// Set file's logging information and reset state
    pub fn set_input(&mut self, input: &ProcessInput) {
        self.current_input = input.clone();
        self.tracker_stack.increase_level();
    }

    /// Increase line number
    pub fn inc_line_number(&mut self) {
        self.tracker_stack.tracker_mut().forward_line();
    }
    /// Increase char number
    pub fn inc_char_number(&mut self) {
        self.tracker_stack.tracker_mut().forward_char();
    }

    /// Add new tracks inside tracker
    pub fn append_track(&mut self) {
        self.tracker_stack.tracker_mut().set_milestone(());
    }

    /// Merge last tracks
    pub fn merge_track(&mut self) {
        self.tracker_stack.tracker_mut().connect_track();
    }

    fn get_last_track(&self) -> Track<()> {
        let mut out_track = Track::new(());
        for tracker in &self.tracker_stack.stack {
            out_track.line_index += tracker.get_distance().line_index;
            out_track.char_index = tracker.get_distance().char_index;
        }
        out_track
    }

    /// Log message
    pub fn log(&mut self, log: &str) -> RadResult<()> {
        let track = self.get_last_track();
        let (last_line, last_char) = (track.line_index, track.char_index);
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.inner().write_all(
                        format!(
                            "Log : {} -> {}:{}:{}{}",
                            log, self.current_input, last_line, last_char, LINE_ENDING
                        )
                        .as_bytes(),
                    )?;
                }
                WriteOption::Terminal => {
                    write!(
                        std::io::stderr(),
                        "{}: {} {} --> {}:{}:{}{}",
                        Utils::green("log", self.is_logging_to_file()),
                        log,
                        LINE_ENDING,
                        self.current_input,
                        last_line,
                        last_char,
                        LINE_ENDING
                    )?;
                }
                WriteOption::Variable(var) => {
                    write!(
                        var,
                        "log : {} -> {}:{}:{}{}",
                        log, self.current_input, last_line, last_char, LINE_ENDING
                    )?;
                }
                WriteOption::Discard | WriteOption::Return => (),
            } // Match end
        }
        Ok(())
    }

    /// Log error
    pub fn elog(&mut self, log: &str) -> RadResult<()> {
        self.error_count += 1;

        if self.assert {
            return Ok(());
        }
        let track = self.get_last_track();
        let (last_line, last_char) = (track.line_index, track.char_index);
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.inner().write_all(
                        format!(
                            "error : {} -> {}:{}:{}{}",
                            log, self.current_input, last_line, last_char, LINE_ENDING
                        )
                        .as_bytes(),
                    )?;
                }
                WriteOption::Terminal => {
                    write!(
                        std::io::stderr(),
                        "{}: {} {} --> {}:{}:{}{}",
                        Utils::red("error", self.is_logging_to_file()),
                        log,
                        LINE_ENDING,
                        self.current_input,
                        last_line,
                        last_char,
                        LINE_ENDING
                    )?;
                }
                WriteOption::Variable(var) => {
                    write!(
                        var,
                        "error : {} -> {}:{}:{}{}",
                        log, self.current_input, last_line, last_char, LINE_ENDING
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
                    file.inner().write_all(log.to_string().as_bytes())?;
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
        let track = self.get_last_track();
        let (last_line, last_char) = (track.line_index, track.char_index);
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.inner().write_all(
                        format!(
                            "warning : {} -> {}:{}:{}{}",
                            log, self.current_input, last_line, last_char, LINE_ENDING
                        )
                        .as_bytes(),
                    )?;
                }
                WriteOption::Terminal => {
                    writeln!(
                        std::io::stderr(),
                        "{}: {} {} --> {}:{}:{}",
                        Utils::yellow("warning", self.is_logging_to_file()),
                        log,
                        LINE_ENDING,
                        self.current_input,
                        last_line,
                        last_char
                    )?;
                }
                WriteOption::Variable(var) => {
                    write!(
                        var,
                        "error : {} -> {}:{}:{}{}",
                        log, self.current_input, last_line, last_char, LINE_ENDING
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
        let track = self.get_last_track();
        let (last_line, last_char) = (track.line_index, track.char_index);

        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.inner().write_all(
                        format!(
                            "assert fail -> {}:{}:{}{}",
                            self.current_input, last_line, last_char, LINE_ENDING
                        )
                        .as_bytes(),
                    )?;
                }
                WriteOption::Terminal => {
                    writeln!(
                        std::io::stderr(),
                        "{} -> {}:{}:{}",
                        Utils::red("assert fail", self.is_logging_to_file()),
                        self.current_input,
                        last_line,
                        last_char
                    )?;
                }
                WriteOption::Variable(var) => write!(
                    var,
                    "assert fail -> {}:{}:{}{}",
                    self.current_input, last_line, last_char, LINE_ENDING
                )?,
                WriteOption::Discard | WriteOption::Return => (),
            } // match end
        }

        Ok(())
    }

    /// Print result of logging of warnings and errors
    pub fn print_result(&mut self) -> RadResult<()> {
        let log_to_file = self.is_logging_to_file();
        if let Some(option) = &mut self.write_option {
            // There is either error or warning
            let error_result = format!(
                "{}: found {} errors",
                Utils::red("error", log_to_file),
                self.error_count
            );
            let warning_result = format!(
                "{}: found {} warnings",
                Utils::yellow("warning", log_to_file),
                self.warning_count
            );
            let assert_result = format!(
                "
{}
SUCCESS : {}
FAIL: {}",
                Utils::green("Assert", log_to_file),
                self.assert_success,
                self.assert_fail
            );
            match option {
                WriteOption::File(file) => {
                    if self.error_count > 0 {
                        file.inner().write_all(error_result.as_bytes())?;
                    }
                    if self.warning_count > 0 {
                        file.inner().write_all(warning_result.as_bytes())?;
                    }
                    if self.assert {
                        file.inner().write_all(assert_result.as_bytes())?;
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

    /// Check if logger is logging to file or not
    pub fn is_logging_to_file(&self) -> bool {
        matches!(self.write_option, Some(WriteOption::File(_)))
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
                        Utils::green(
                            &format!("{}:log", self.get_last_line()),
                            self.is_logging_to_file()
                        ),
                        LINE_ENDING,
                        log
                    )?;
                }
                WriteOption::File(file) => {
                    file.inner().write_all(
                        format!("{}:log{}{}", self.get_last_line(), LINE_ENDING, log).as_bytes(),
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

#[derive(Debug)]
pub struct NestedTracker {
    pub(crate) stack: Vec<Tracker<()>>,
}

impl NestedTracker {
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn count(&self) -> usize {
        self.stack.len()
    }

    pub fn previous_tracker(&self) -> &Tracker<()> {
        &self.stack[self.stack.len() - 2]
    }

    pub fn tracker(&self) -> &Tracker<()> {
        self.stack.last().unwrap()
    }

    pub fn tracker_mut(&mut self) -> &mut Tracker<()> {
        self.stack.last_mut().unwrap()
    }

    pub fn increase_level(&mut self) {
        let tracker = Tracker::new(());
        self.stack.push(tracker);
    }

    pub fn decrease_level(&mut self) {
        self.stack.pop();
    }
}
