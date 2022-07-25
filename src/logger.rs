//! # Logger
//!
//! Logger handles all kinds of logging logics&&&. Such log can be warning, error or debug logs.

use crate::common::{ProcessInput, RadResult, WriteOption};
use crate::utils::Utils;
use crate::{consts::*, RadError};
use std::fmt::Write;
use std::io::Write as _;
use trexter::{Track, Tracker};

/// Logger that controls logging
pub(crate) struct Logger<'logger> {
    suppresion_type: WarningType,
    current_input: ProcessInput,
    pub(crate) tracker_stack: TrackerStack,
    pub(crate) write_option: Option<WriteOption<'logger>>,
    pub(crate) assert: bool,
    stat: LoggerStat,
}

#[derive(Default)]
pub struct LoggerStat {
    error_count: usize,
    warning_count: usize,
    assert_success: usize,
    assert_fail: usize,
}

impl<'logger> Logger<'logger> {
    pub fn new() -> Self {
        Self {
            suppresion_type: WarningType::None,
            current_input: ProcessInput::Stdin,
            write_option: None,
            tracker_stack: TrackerStack::new(),
            assert: false,
            stat: LoggerStat::default(),
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
            .new_track(TrackType::Record(record));
    }

    /// Merge last tracks
    pub fn merge_track(&mut self) {
        self.tracker_stack.tracker_mut().connect_track();
    }

    /// Get current input track without generic milestone
    pub fn get_current_input_track(&self) -> Track<()> {
        let mut out_track = Track::new(());
        for tracker in self.tracker_stack.stack.iter().rev() {
            if let TrackType::Input(_) = tracker.get_distance().milestone {
                out_track.line_index = tracker.get_distance().line_index;
                out_track.char_index = tracker.get_distance().char_index;

                // Only get the latet input information
                break;
            }
        }
        out_track
    }

    /// Get first track
    #[cfg(feature = "debug")]
    pub fn get_first_track(&self) -> Track<()> {
        let mut out_track = Track::new(());
        let tracker = self.tracker_stack.stack.first().unwrap();
        out_track.line_index = tracker.get_distance().line_index;
        out_track.char_index = tracker.get_distance().char_index;
        out_track
    }

    #[cfg(feature = "debug")]
    pub fn get_last_line(&self) -> usize {
        let mut last_line = 0;
        for tracker in &self.tracker_stack.stack {
            let distance = tracker.get_distance();
            last_line += distance.line_index;
        }
        last_line
    }

    fn construct_log_position(&self) -> RadResult<String> {
        #[cfg(debug_assertions)]
        if std::env::var("DEBUG_TRACE").is_ok() {
            eprintln!("{:#?}", self.tracker_stack);
        }
        if std::env::var("RAD_BACKTRACE").is_ok() {
            let mut track_iter = self.tracker_stack.stack.iter();
            let input = track_iter.next().unwrap().get_distance();
            let mut trace = format!(
                "[INPUT = {}]:{}:{}",
                self.current_input, input.line_index, input.char_index
            );
            for track in track_iter {
                let dist = track.get_distance();
                write!(
                    trace,
                    " >> ({}):{}:{}",
                    dist.milestone, dist.line_index, dist.char_index
                )?;
            }
            return Ok(trace);
        }
        let track = self.get_current_input_track();
        let (last_line, last_char) = (track.line_index, track.char_index);

        // Set last position first,
        // which is the first trigger macro's position
        let mut position = format!(
            "[INPUT = {}]:{}:{}",
            self.current_input, last_line, last_char
        );

        // Then append current macro's position which is the direct source of an error
        let last_distance = self.tracker_stack.tracker().get_distance();
        match &last_distance.milestone {
            TrackType::Body(name) | TrackType::Argument(name) => {
                write!(
                    position,
                    " >> (MACRO = {}):{}:{}",
                    name,
                    last_distance.line_index + 1, // THis is because inner tracks starts from line "0"
                    last_distance.char_index,
                )?;
            }
            _ => (),
        }

        Ok(position)
    }

    fn write_formatted_log_msg_without_line(
        &mut self,
        prompt: &str,
        log_msg: &str,
        #[cfg(feature = "color")] color_func: ColorDisplayFunc,
    ) -> RadResult<()> {
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.inner()
                        .write_all(format!("{} : {}{}", prompt, log_msg, LINE_ENDING).as_bytes())?;
                }
                WriteOption::Terminal => {
                    #[allow(unused_mut)]
                    let mut prompt = prompt.to_string();
                    #[cfg(feature = "color")]
                    {
                        prompt = color_func(&prompt, self.is_logging_to_file()).to_string();
                    }
                    write!(std::io::stderr(), "{}: {}{}", prompt, log_msg, LINE_ENDING)?;
                }
                WriteOption::Variable(var) => {
                    write!(var, "{} : {}{}", prompt, log_msg, LINE_ENDING)?;
                }
                WriteOption::Discard | WriteOption::Return => (),
            } // Match end
        }
        Ok(())
    }

    fn write_formatted_log_msg(
        &mut self,
        prompt: &str,
        log_msg: &str,
        #[cfg(feature = "color")] color_func: ColorDisplayFunc,
    ) -> RadResult<()> {
        let log_pos = self.construct_log_position()?;
        if let Some(option) = &mut self.write_option {
            match option {
                WriteOption::File(file) => {
                    file.inner().write_all(
                        format!("{} : {} -> {}{}", prompt, log_msg, log_pos, LINE_ENDING)
                            .as_bytes(),
                    )?;
                }
                WriteOption::Terminal => {
                    #[allow(unused_mut)]
                    let mut prompt = prompt.to_string();
                    #[cfg(feature = "color")]
                    {
                        prompt = color_func(&prompt, self.is_logging_to_file()).to_string();
                    }
                    write!(
                        std::io::stderr(),
                        "{}: {} {} --> {}{}",
                        prompt,
                        log_msg,
                        LINE_ENDING,
                        log_pos,
                        LINE_ENDING
                    )?;
                }
                WriteOption::Variable(var) => {
                    write!(
                        var,
                        "{} : {} -> {}{}",
                        prompt, log_msg, log_pos, LINE_ENDING
                    )?;
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
            #[cfg(feature = "color")]
            Utils::green,
        )
    }

    /// Log error
    pub fn elog(&mut self, log_msg: &str) -> RadResult<()> {
        self.stat.error_count += 1;

        if self.assert {
            return Ok(());
        }
        self.write_formatted_log_msg(
            "error",
            log_msg,
            #[cfg(feature = "color")]
            Utils::red,
        )
    }

    pub fn elog_no_line(&mut self, log_msg: impl std::fmt::Display) -> RadResult<()> {
        self.stat.error_count += 1;

        if self.assert {
            return Ok(());
        }
        self.write_formatted_log_msg_without_line(
            "error",
            &log_msg.to_string(),
            #[cfg(feature = "color")]
            Utils::red,
        )?;
        Ok(())
    }

    /// Log warning
    pub fn wlog(&mut self, log_msg: &str, warning_type: WarningType) -> RadResult<()> {
        if self.suppresion_type == WarningType::Any || self.suppresion_type == warning_type {
            return Ok(());
        }

        self.stat.warning_count += 1;

        if self.assert {
            return Ok(());
        }

        self.write_formatted_log_msg(
            "warning",
            log_msg,
            #[cfg(feature = "color")]
            Utils::yellow,
        )
    }

    /// Log warning within line
    pub fn wlog_no_line(&mut self, log_msg: &str, warning_type: WarningType) -> RadResult<()> {
        if self.suppresion_type == WarningType::Any || self.suppresion_type == warning_type {
            return Ok(());
        }

        self.stat.warning_count += 1;

        if self.assert {
            return Ok(());
        }

        self.write_formatted_log_msg_without_line(
            "warning",
            log_msg,
            #[cfg(feature = "color")]
            Utils::yellow,
        )?;

        Ok(())
    }

    /// Assertion log
    pub fn alog(&mut self, success: bool) -> RadResult<()> {
        if success {
            self.stat.assert_success += 1;
            return Ok(());
        }
        self.stat.assert_fail += 1;
        self.write_formatted_log_msg(
            "assert fail",
            "",
            #[cfg(feature = "color")]
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
                self.stat.error_count
            );
            let warning_result = format!(
                "{}: found {} warnings",
                Utils::yellow("warning", log_to_file),
                self.stat.warning_count
            );
            let assert_result = format!(
                "
{}
SUCCESS : {}
FAIL: {}",
                Utils::green("Assert", log_to_file),
                self.stat.assert_success,
                self.stat.assert_fail
            );
            match option {
                WriteOption::File(file) => {
                    if self.stat.error_count > 0 {
                        file.inner().write_all(error_result.as_bytes())?;
                    }
                    if self.stat.warning_count > 0 {
                        file.inner().write_all(warning_result.as_bytes())?;
                    }
                    if self.assert {
                        file.inner().write_all(assert_result.as_bytes())?;
                    }
                }
                WriteOption::Terminal => {
                    if self.stat.error_count > 0 {
                        writeln!(std::io::stderr(), "{}", error_result)?;
                    }
                    if self.stat.warning_count > 0 {
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
        let track = self.get_first_track();
        let last_line = track.line_index;
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
    Argument(String),
    Body(String),
}

impl std::fmt::Display for TrackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Record(cont) => write!(f, "RECORD = {}", cont),
            Self::Input(cont) => write!(f, "INPUT = {}", cont),
            Self::Argument(cont) => write!(f, "MACRO = {}", cont),
            Self::Body(cont) => write!(f, "MACRO = {}", cont),
        }
    }
}
