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
    pub(crate) tracker_stack: TrackerStack,
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
            tracker_stack: TrackerStack::new(),
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

    /// Create new tracker instance
    ///
    /// This creates a context of tracking
    pub fn start_new_tracker(&mut self, track_type: TrackType) {
        self.tracker_stack.increase_level(track_type);
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
        self.start_new_tracker(TrackType::Input(self.current_input.to_string()));
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
    pub fn append_track(&mut self, record: String) {
        self.tracker_stack
            .tracker_mut()
            .set_milestone(TrackType::Record(record));
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

    fn construct_log_position(&self) -> RadResult<String> {
        let track = self.get_last_track();
        let (last_line, last_char) = (track.line_index, track.char_index);

        // Set last position first,
        // which is the first trigger macro's position
        let mut position = format!(
            "[INPUT = {}]:{}:{}",
            self.current_input, last_line, last_char
        );

        // Then append current macro's position which is the direct source of an error
        let last_distance = self.tracker_stack.tracker().get_distance();
        if let TrackType::Body(macro_name) = &last_distance.milestone {
            write!(
                position,
                " >> (MACRO = {}):{}:{}{}",
                macro_name, last_distance.line_index, last_distance.char_index, LINE_ENDING
            )?;
        } else {
            position.push_str(LINE_ENDING);
        }

        Ok(position)
    }

    fn write_formatted_log_msg(
        &mut self,
        prompt: &str,
        log_msg: &str,
        #[cfg(feature = "clap")] color_func: ColorDisplayFunc,
    ) -> RadResult<()> {
        let log_pos = self.construct_log_position()?;
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.inner().write_all(
                        format!("{} : {} -> {}", prompt, log_msg, log_pos,).as_bytes(),
                    )?;
                }
                WriteOption::Terminal => {
                    let mut prompt = prompt.to_string();
                    #[cfg(feature = "clap")]
                    {
                        prompt = color_func(&prompt, self.is_logging_to_file()).to_string();
                    }
                    write!(
                        std::io::stderr(),
                        "{}: {} {} --> {}",
                        prompt,
                        log_msg,
                        LINE_ENDING,
                        log_pos
                    )?;
                }
                WriteOption::Variable(var) => {
                    write!(var, "{} : {} -> {}", prompt, log_msg, log_pos)?;
                }
                WriteOption::Discard | WriteOption::Return => (),
            } // Match end
        }
        Ok(())
    }

    /// Log message
    pub fn log(&mut self, log_msg: &str) -> RadResult<()> {
        self.write_formatted_log_msg(
            "log",
            log_msg,
            #[cfg(feature = "clap")]
            Utils::green,
        )
    }

    /// Log error
    pub fn elog(&mut self, log_msg: &str) -> RadResult<()> {
        self.error_count += 1;

        if self.assert {
            return Ok(());
        }
        if std::env::var("PRINT_STACK").is_ok() {
            println!("{:#?}", self.tracker_stack)
        }
        self.write_formatted_log_msg(
            "error",
            log_msg,
            #[cfg(feature = "clap")]
            Utils::red,
        )
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
    pub fn wlog(&mut self, log_msg: &str, warning_type: WarningType) -> RadResult<()> {
        if self.suppresion_type == WarningType::Any || self.suppresion_type == warning_type {
            return Ok(());
        }

        self.warning_count += 1;

        if self.assert {
            return Ok(());
        }

        self.write_formatted_log_msg(
            "warning",
            log_msg,
            #[cfg(feature = "clap")]
            Utils::yellow,
        )
    }

    /// Assertion log
    pub fn alog(&mut self, success: bool) -> RadResult<()> {
        if success {
            self.assert_success += 1;
            return Ok(());
        }
        self.assert_fail += 1;
        self.write_formatted_log_msg(
            "assert fail",
            "",
            #[cfg(feature = "clap")]
            Utils::red,
        )
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
        let track = self.get_last_track();
        let (last_line, last_char) = (track.line_index, track.char_index);
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::Terminal => {
                    write!(
                        std::io::stderr(),
                        "{}{}{}",
                        Utils::green(&format!("{}:log", last_line), self.is_logging_to_file()),
                        LINE_ENDING,
                        log
                    )?;
                }
                WriteOption::File(file) => {
                    file.inner()
                        .write_all(format!("{}:log{}{}", last_line, LINE_ENDING, log).as_bytes())?;
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
pub struct TrackerStack {
    pub(crate) stack: Vec<Tracker<TrackType>>,
}

impl TrackerStack {
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn count(&self) -> usize {
        self.stack.len()
    }

    pub fn previous_tracker(&self) -> &Tracker<TrackType> {
        &self.stack[self.stack.len() - 2]
    }

    pub fn tracker(&self) -> &Tracker<TrackType> {
        self.stack.last().unwrap()
    }

    pub fn tracker_mut(&mut self) -> &mut Tracker<TrackType> {
        self.stack.last_mut().unwrap()
    }

    pub fn increase_level(&mut self, track_type: TrackType) {
        let tracker = Tracker::new(track_type);
        self.stack.push(tracker);
    }

    pub fn decrease_level(&mut self) {
        self.stack.pop();
    }
}

#[derive(Debug)]
pub enum TrackType {
    Record(String),
    Input(String),
    Argumnet(String),
    Body(String),
}
