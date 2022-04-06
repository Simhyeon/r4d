//! # processor
//!
//! "processor" module is about processing of given input.
//!
//! Processor substitutes all macros only when the macros were already defined and returns
//! untouched string back if not found any.
//!
//! Processor can handle various types of inputs (string|stdin|file)
//!
//! # Detailed usage
//! ```rust
//! use rad::RadResult;
//! use rad::Processor;
//! use rad::AuthType;
//! use rad::CommentType;
//! use rad::DiffOption;
//! use rad::MacroType;
//! use rad::HookType; // This is behind hook feature
//! use rad::Hygiene;
//! use std::path::Path;
//!
//! // Builder
//! let mut processor = Processor::new()
//!     .set_comment_type(CommentType::Start)                // Use comment
//!     .custom_macro_char('~')?                             // use custom macro character
//!     .custom_comment_char('#')?                           // use custom comment character
//!     .purge(true)                                         // Purge undefined macro
//!     .silent(WarningType::Security)                       // Silents all warnings
//!     .nopanic(true)                                       // No panic in any circumstances
//!     .assert(true)                                        // Enable assertion mode
//!     .lenient(true)                                       // Disable strict mode
//!     .hygiene(Hygiene::Macro)                             // Enable hygiene mode
//!     .pipe_truncate(false)                                // Disable pipe truncate
//!     .write_to_file(Some(Path::new("out.txt")))?          // default is stdout
//!     .error_to_file(Some(Path::new("err.txt")))?          // default is stderr
//!     .unix_new_line(true)                                 // use unix new line for formatting
//!     .discard(true)                                       // discard all output
//!     .melt_files(vec![Path::new("source.r4d")])?          // Read runtime macros from frozen
//!     // Permission
//!     .allow(Some(vec![AuthType::ENV]))                    // Grant permission of authtypes
//!     .allow_with_warning(Some(vec![AuthType::CMD]))       // Grant permission of authypes with warning enabled
//!     // Debugging options
//!     .debug(true)                                         // Turn on debug mode
//!     .log(true)                                           // Use logging to terminal
//!     .diff(DiffOption::All)?                              // Print diff in final result
//!     .interactive(true);                                   // Use interactive mode
//!
//! // Comment char and macro char cannot be same
//! // Unallowed pattern for the characters are [a-zA-Z1-9\\_\*\^\|\(\)=,]
//!
//! // Use Processor::empty() instead of Processor::new()
//! // if you don't want any default macros
//!
//! // Print information about current processor permissions
//! // This is an warning and can be suppressed with silent option
//! processor.print_permission()?;
//!
//! // Register a hook macro
//! // Trigger and execution macro should be defined elsewhere
//! processor.register_hook(
//!     HookType::Macro,            // Macro type
//!     "trigger_macro",            // Macro that triggers
//!     "hook_div",                 // Macro to be executed
//!     1,                          // target count
//!     false                       // Resetable
//! )?;
//!
//! // Add runtime rules(in order of "name, args, body")
//! processor.add_runtime_rules(vec![("test","a_src a_link","$a_src() -> $a_link()")])?;
//!
//! // Add custom rules without any arguments
//! processor.add_static_rules(vec![("test","TEST"),("lul","kekw")])?;
//!
//! // Undefine only macro
//! processor.undefine_macro("name1", MacroType::Any);
//!
//! // Process with inputs
//! // This prints to desginated write destinations
//! processor.from_string(r#"$define(test=Test)"#)?;
//! processor.from_stdin()?;
//! processor.from_file(Path::new("from.txt"))?;
//!
//! processor.freeze_to_file(Path::new("out.r4f"))?; // Create frozen file
//!
//! // Print out result
//! // This will print counts of warning and errors.
//! // It will also print diff between source and processed if diff option was
//! // given as builder pattern.
//! processor.print_result()?;                       
//! ```

use crate::arg_parser::{ArgParser, GreedyState};
use crate::auth::{AuthFlags, AuthState, AuthType};
#[cfg(feature = "debug")]
use crate::debugger::DebugSwitch;
#[cfg(feature = "debug")]
use crate::debugger::Debugger;
use crate::define_parser::DefineParser;
use crate::error::RadError;
#[cfg(feature = "hook")]
use crate::hookmap::{HookMap, HookType};
use crate::lexor::*;
use crate::logger::{Logger, LoggerLines, WarningType};
#[cfg(feature = "debug")]
use crate::models::DiffOption;
#[cfg(feature = "signature")]
use crate::models::SignatureType;
use crate::models::{
    Behaviour, CommentType, ExtMacroBuilder, ExtMacroType, FileTarget, FlowControl, Hygiene,
    LocalMacro, MacroFragment, MacroMap, MacroType, ProcessInput, RelayTarget, RuleFile,
    UnbalancedChecker, WriteOption,
};
#[cfg(feature = "storage")]
use crate::models::{RadStorage, StorageOutput};
use crate::runtime_map::RuntimeMacro;
#[cfg(feature = "signature")]
use crate::sigmap::SignatureMap;
use crate::utils::Utils;
use crate::{consts::*, RadResult};
#[cfg(feature = "cindex")]
use cindex::Indexer;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, Write};
use std::path::{Path, PathBuf};

lazy_static! {
    // Source : https://stackoverflow.com/questions/17564088/how-to-form-a-regex-to-recognize-correct-declaration-of-variable-names/17564142
    static ref MAC_NAME : Regex = Regex::new(r#"^[_a-zA-Z]\w*$"#).expect("Failed to create regex expression");
}

// Methods of processor consists of multiple sections followed as <TAG>
// <BUILDER>            -> Builder pattern related
// <PROCESS>            -> User functions related
// <DEBUG>              -> Debug related functions
// <PARSE>              -> Parse rleated functions
//     <LEX>            -> sub section of parse, this is technically not a lexing but it's named as
// <MISC>               -> Miscellaenous
// <EXT>                -> Methods for extensions
//
// Find each section's start with <NAME> and find end of section with </NAME>
//
// e.g. <BUILDER> for builder section start and </BUILDER> for builder section end

pub(crate) struct ProcessorState {
    // Current_input is either "stdin" or currently being read file's name thus it should not be a
    // path derivative
    pub auth_flags: AuthFlags,
    pub current_input: ProcessInput,
    pub input_stack: HashSet<PathBuf>,
    pub newline: String,
    pub paused: bool,
    // This is reserved for hygienic execution
    pub hygiene: Hygiene,
    pub pipe_truncate: bool,
    pipe_map: HashMap<String, String>,
    pub relay: Vec<RelayTarget>,
    pub sandbox: bool,
    pub behaviour: Behaviour,
    pub comment_type: CommentType,
    // Temp target needs to save both path and file
    // because file doesn't necessarily have path.
    // Especially in unix, this is not so an unique case
    #[cfg(not(feature = "wasm"))]
    pub temp_target: FileTarget,
    pub comment_char: Option<char>,
    pub macro_char: Option<char>,
    pub flow_control: FlowControl,
    pub deny_newline: bool,
    pub escape_newline: bool,
    pub queued: Vec<String>,
}

impl ProcessorState {
    pub fn new() -> Self {
        Self {
            current_input: ProcessInput::Stdin,
            input_stack: HashSet::new(),
            auth_flags: AuthFlags::new(),
            newline: LINE_ENDING.to_owned(),
            pipe_truncate: true,
            pipe_map: HashMap::new(),
            paused: false,
            hygiene: Hygiene::None,
            relay: vec![],
            behaviour: Behaviour::Strict,
            comment_type: CommentType::None,
            sandbox: false,
            #[cfg(not(feature = "wasm"))]
            temp_target: FileTarget::empty(),
            comment_char: None,
            macro_char: None,
            flow_control: FlowControl::None,
            deny_newline: false,
            escape_newline: false,
            queued: vec![],
        }
    }

    #[cfg(not(feature = "wasm"))]
    /// Internal method for setting temp target
    pub(crate) fn set_temp_target(&mut self, path: &Path) {
        self.temp_target.set_path(path);
    }

    pub fn add_pipe(&mut self, name: Option<&str>, value: String) {
        if let Some(name) = name {
            self.pipe_map.insert(name.to_owned(), value);
        } else {
            self.pipe_map.insert("-".to_owned(), value);
        }
    }

    pub fn get_pipe(&mut self, key: &str) -> Option<String> {
        if self.pipe_truncate {
            self.pipe_map.remove(key)
        } else {
            self.pipe_map.get(key).map(|s| s.to_owned())
        }
    }
}

/// Processor that parses(lexes) given input and print out to desginated output
pub struct Processor<'processor> {
    map: MacroMap,
    define_parser: DefineParser,
    write_option: WriteOption<'processor>,
    logger: Logger<'processor>,
    cache: String,
    // -- Features --
    #[cfg(feature = "hook")]
    pub(crate) hook_map: HookMap,
    #[cfg(feature = "debug")]
    debugger: Debugger,
    checker: UnbalancedChecker,
    pub(crate) state: ProcessorState,
    #[cfg(feature = "storage")]
    pub storage: Option<Box<dyn RadStorage>>,
    #[cfg(feature = "cindex")]
    pub indexer: Indexer,
}

impl<'processor> Processor<'processor> {
    // ----------
    // Builder pattern methods
    // <BUILDER>
    /// Creates default processor with default macros
    pub fn new() -> Self {
        Self::new_processor(true)
    }

    /// Creates default processor without default macros
    pub fn empty() -> Self {
        Self::new_processor(false)
    }

    /// Internal function to create Processor struct
    ///
    /// This creates a complete processor that can parse and create output without any extra
    /// informations.
    fn new_processor(use_default: bool) -> Self {
        let mut state = ProcessorState::new();

        // You cannot use filesystem in wasm target
        #[cfg(not(feature = "wasm"))]
        {
            state.set_temp_target(&std::env::temp_dir().join("rad.txt"));
        }

        let mut logger = Logger::new();
        logger.set_write_option(Some(WriteOption::Terminal));

        let map = if use_default {
            MacroMap::new()
        } else {
            MacroMap::empty()
        };

        Self {
            map,
            cache: String::new(),
            write_option: WriteOption::Terminal,
            define_parser: DefineParser::new(),
            logger,
            state,
            #[cfg(feature = "hook")]
            hook_map: HookMap::new(),
            #[cfg(feature = "debug")]
            debugger: Debugger::new(),
            checker: UnbalancedChecker::new(),
            #[cfg(feature = "storage")]
            storage: None,
            #[cfg(feature = "cindex")]
            indexer: Indexer::new(),
        }
    }

    /// Set write option to yield output to the file
    pub fn write_to_file(mut self, target_file: Option<impl AsRef<Path>>) -> RadResult<Self> {
        if let Some(target_file) = target_file {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&target_file);

            if let Err(_) = file {
                return Err(RadError::InvalidCommandOption(format!(
                    "Could not create file \"{}\"",
                    target_file.as_ref().display()
                )));
            } else {
                self.write_option = WriteOption::File(file.unwrap());
            }
        }
        Ok(self)
    }

    /// Write to variable
    pub fn write_to_variable(mut self, value: &'processor mut String) -> Self {
        self.write_option = WriteOption::Variable(value);
        self
    }

    /// Yield error to the file
    pub fn error_to_file(mut self, target_file: Option<impl AsRef<Path>>) -> RadResult<Self> {
        if let Some(target_file) = target_file {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&target_file);

            if let Err(_) = file {
                return Err(RadError::InvalidCommandOption(format!(
                    "Could not create file \"{}\"",
                    target_file.as_ref().display()
                )));
            } else {
                self.logger = Logger::new();
                self.logger
                    .set_write_option(Some(WriteOption::File(file.unwrap())));
            }
        }
        Ok(self)
    }

    /// Yield error to the file
    pub fn error_to_variable(mut self, value: &'processor mut String) -> Self {
        self.logger
            .set_write_option(Some(WriteOption::Variable(value)));
        self
    }

    /// Custom comment character
    pub fn custom_comment_char(mut self, character: char) -> RadResult<Self> {
        // check if unallowed character
        if UNALLOWED_CHARS.is_match(&character.to_string()) {
            return Err(RadError::UnallowedChar(format!(
                "\"{}\" is not allowed",
                character
            )));
        } else if self.get_macro_char() == character {
            // macro char and comment char should not be equal
            return Err(RadError::UnallowedChar(format!(
                "\"{}\" is already defined for macro character",
                character
            )));
        }
        self.state.comment_char.replace(character);
        Ok(self)
    }

    /// Custom macro character
    pub fn custom_macro_char(mut self, character: char) -> RadResult<Self> {
        if UNALLOWED_CHARS.is_match(&character.to_string()) {
            return Err(RadError::UnallowedChar(format!(
                "\"{}\" is not allowed",
                character
            )));
        } else if self.get_comment_char() == character {
            // macro char and comment char should not be equal
            return Err(RadError::UnallowedChar(format!(
                "\"{}\" is already defined for comment character",
                character
            )));
        }
        self.state.macro_char.replace(character);
        Ok(self)
    }

    /// Use unix line ending instead of operating system's default one
    pub fn unix_new_line(mut self, use_unix_new_line: bool) -> Self {
        if use_unix_new_line {
            self.state.newline = "\n".to_owned();
        }
        self
    }

    /// Set purge option
    pub fn purge(mut self, purge: bool) -> Self {
        if purge {
            self.state.behaviour = Behaviour::Purge;
        }
        self
    }

    /// Set lenient
    pub fn lenient(mut self, lenient: bool) -> Self {
        if lenient {
            self.state.behaviour = Behaviour::Lenient;
        }
        self
    }

    /// Set hygiene variant
    pub fn hygiene(mut self, hygiene: Hygiene) -> Self {
        self.state.hygiene = hygiene;
        self
    }

    /// Set truncate option
    pub fn pipe_truncate(mut self, truncate: bool) -> Self {
        self.state.pipe_truncate = truncate;
        self
    }

    /// Set comment type
    pub fn set_comment_type(mut self, comment_type: CommentType) -> Self {
        self.state.comment_type = comment_type;
        self
    }

    /// Set silent option
    pub fn silent(mut self, silent_type: WarningType) -> Self {
        self.logger.suppress_warning(silent_type);
        self
    }

    /// Set nopanic
    pub fn nopanic(mut self, nopanic: bool) -> Self {
        if nopanic {
            self.state.behaviour = Behaviour::Nopanic;
        }
        self
    }

    /// Set assertion mode
    pub fn assert(mut self, assert: bool) -> Self {
        if assert {
            self.logger.assert();
            self.write_option = WriteOption::Discard;
            self.state.behaviour = Behaviour::Nopanic;
        }
        self
    }

    /// Add debug options
    #[cfg(feature = "debug")]
    pub fn debug(mut self, debug: bool) -> Self {
        self.debugger.debug = debug;
        self
    }

    /// Add debug log options
    #[cfg(feature = "debug")]
    pub fn log(mut self, log: bool) -> Self {
        self.debugger.log = log;
        self
    }

    /// Add diff option
    #[cfg(feature = "debug")]
    pub fn diff(mut self, diff: DiffOption) -> RadResult<Self> {
        self.debugger.enable_diff(diff)?;
        Ok(self)
    }

    /// Add debug interactive options
    #[cfg(feature = "debug")]
    pub fn interactive(mut self, interactive: bool) -> Self {
        if interactive {
            self.debugger.set_interactive();
        }
        self
    }

    /// Melt rule file
    ///
    /// This always melt file into non-volatile form
    pub fn melt_files(mut self, paths: Vec<impl AsRef<Path>>) -> RadResult<Self> {
        let mut rule_file = RuleFile::new(None);
        for p in paths.iter() {
            // File validity is checked by melt methods
            rule_file.melt(p.as_ref())?;
        }
        self.map.runtime.extend_map(rule_file.rules, Hygiene::None);

        Ok(self)
    }

    // This is not included in docs.rs documentation becuase... is it necessary?
    // I don't really remember when this method was added at the first time.
    /// Melt rule as literal input source, or say from byte array
    pub fn rule_literal(mut self, literal: &Vec<u8>) -> RadResult<Self> {
        let mut rule_file = RuleFile::new(None);
        rule_file.melt_literal(literal)?;
        self.map.runtime.extend_map(rule_file.rules, Hygiene::None);
        Ok(self)
    }

    /// Open authorization of processor
    pub fn allow(mut self, auth_types: Option<Vec<AuthType>>) -> Self {
        if let Some(auth_types) = auth_types {
            for auth in auth_types {
                self.state.auth_flags.set_state(&auth, AuthState::Open)
            }
        }
        self
    }

    /// Open authorization of processor but yield warning
    pub fn allow_with_warning(mut self, auth_types: Option<Vec<AuthType>>) -> Self {
        if let Some(auth_types) = auth_types {
            for auth in auth_types {
                self.state.auth_flags.set_state(&auth, AuthState::Warn)
            }
        }
        self
    }

    /// Discard output
    pub fn discard(mut self, discard: bool) -> Self {
        if discard {
            self.write_option = WriteOption::Discard;
        }
        self
    }

    /// Build with storage
    #[cfg(feature = "storage")]
    pub fn storage(mut self, storage: Box<dyn RadStorage>) -> Self {
        self.storage.replace(storage);
        self
    }

    // </BUILDER>
    // End builder methods
    // ----------

    // ----------
    // Processing methods
    // <PROCESS>
    //

    /// Set queue object
    pub fn insert_queue(&mut self, item: &str) {
        self.state.queued.push(item.to_owned());
    }

    /// Set hygiene type
    ///
    /// This also clears volatile runtime macro at the same time.
    pub fn set_hygiene(&mut self, hygiene: Hygiene) {
        if !self.map.runtime.volatile.is_empty() {
            self.map.clear_runtime_macros(true);
        }
        self.state.hygiene = hygiene;
    }

    /// Clear volatile macros
    pub fn clear_volatile(&mut self) {
        if !self.map.runtime.volatile.is_empty() {
            self.map.clear_runtime_macros(true);
        }
    }

    /// Toggle macro hygiene
    pub fn toggle_hygiene(&mut self, toggle: bool) {
        if toggle {
            if !self.map.runtime.volatile.is_empty() {
                self.map.clear_runtime_macros(true);
            }
            self.state.hygiene = Hygiene::Macro;
        } else {
            self.state.hygiene = Hygiene::None;
        }
    }

    /// Set write option in the process
    pub fn set_write_option(&mut self, write_option: WriteOption<'processor>) {
        self.write_option = write_option;
    }

    /// Swap write option
    pub fn swap_write_option(&mut self, write_option: WriteOption<'processor>) -> WriteOption {
        std::mem::replace(&mut self.write_option, write_option)
    }

    /// Set write option in the process
    pub fn reset_flow_control(&mut self) {
        self.state.flow_control = FlowControl::None;
    }

    /// Get macro signatrue map
    #[cfg(feature = "signature")]
    pub fn get_signature_map(&self, sig_type: SignatureType) -> RadResult<SignatureMap> {
        let signatures = match sig_type {
            SignatureType::All => self.map.get_signatures(),
            SignatureType::Default => self.map.get_default_signatures(),
            SignatureType::Runtime => self.map.get_runtime_signatures(),
        };
        Ok(SignatureMap::new(signatures))
    }

    /// Print current permission status
    #[allow(dead_code)]
    pub fn print_permission(&mut self) -> RadResult<()> {
        if let Some(status) = self.state.auth_flags.get_status_string() {
            let mut status_with_header = String::from("Permission granted");
            status_with_header.push_str(&status);
            self.log_warning(&status_with_header, WarningType::Security)?;
        }
        Ok(())
    }

    /// Print the result of a processing
    #[allow(dead_code)]
    pub fn print_result(&mut self) -> RadResult<()> {
        self.logger.print_result()?;

        #[cfg(feature = "debug")]
        self.debugger.yield_diff(&mut self.logger)?;

        Ok(())
    }

    /// Clear cached and organze multiple jobs
    ///
    /// * - clear volatile macors
    /// * - Check if there is any unterminated job
    /// Return cached string, if cache was not empty
    pub fn organize_and_clear_cache(&mut self) -> RadResult<Option<String>> {
        if self.state.hygiene == Hygiene::Input {
            self.map.clear_runtime_macros(true);
        }

        // If the last processing has ended
        if self.state.input_stack.len() == 1 {
            // Warn unterminated relaying
            if self.state.relay.len() != 0 {
                let relay = format!("{:?}", self.state.relay.last().unwrap());
                self.log_warning(&format!("There is unterminated relay target : \"{}\" which might not be an intended behaviour.", relay), WarningType::Sanity)?;
            }

            // Warn flow control
            match self.state.flow_control {
                FlowControl::None => (),
                FlowControl::Exit => self.log_warning("Process exited.", WarningType::Sanity)?,
                FlowControl::Escape => self.log_warning("Process escaped.", WarningType::Sanity)?,
            }

        }

        if self.cache.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(std::mem::replace(&mut self.cache, String::new())))
        }
    }

    /// Set storage
    #[cfg(feature = "storage")]
    pub fn set_storage(&mut self, storage: Box<dyn RadStorage>) {
        self.storage.replace(storage);
    }

    /// Extract from storage
    #[cfg(feature = "storage")]
    pub fn extract_storage(&mut self, serialize: bool) -> RadResult<Option<StorageOutput>> {
        if let Some(storage) = self.storage.as_mut() {
            match storage.extract(serialize) {
                Err(err) => Err(RadError::StorageError(format!("Extract error : {}", err))),
                Ok(value) => Ok(value),
            }
        } else {
            Ok(None)
        }
    }

    /// Freeze to a single file
    ///
    /// Frozen file is a bincode encoded binary format file.
    pub fn freeze_to_file(&self, path: impl AsRef<Path>) -> RadResult<()> {
        // File path validity is checked by freeze method
        RuleFile::new(Some(self.map.runtime.macros.clone())).freeze(path.as_ref())?;
        Ok(())
    }

    /// Add a new macro as an extension
    pub fn add_ext_macro(&mut self, ext: ExtMacroBuilder) {
        match ext.macro_type {
            ExtMacroType::Function => self.map.function.new_ext_macro(ext),
            ExtMacroType::Deterred => self.map.deterred.new_ext_macro(ext),
        }
    }

    /// Add runtime rules without builder pattern
    ///
    /// # Args
    ///
    /// The order of argument is "name, args, body"
    ///
    /// # Example
    ///
    /// ```rust
    /// processor.add_runtime_rules(vec![("macro_name","macro_arg1 macro_arg2","macro_body=$macro_arg1()")]);
    /// ```
    pub fn add_runtime_rules(
        &mut self,
        rules: Vec<(impl AsRef<str>, &str, &str)>,
    ) -> RadResult<()> {
        if self.state.hygiene == Hygiene::Aseptic {
            self.log_strict(
                "Runtime macro declaration is disabled in aseptic mode",
                WarningType::Security,
            )?;
            if self.state.behaviour == Behaviour::Strict {
                return Err(RadError::StrictPanic);
            }
        }
        for (name, args, body) in rules {
            let name = name.as_ref();
            if !MAC_NAME.is_match(name) {
                return Err(RadError::InvalidMacroName(format!(
                    "Name : \"{}\" is not a valid macro name",
                    name
                )));
            }
            self.map.runtime.macros.insert(
                name.to_owned(),
                RuntimeMacro {
                    name: name.to_owned(),
                    args: args
                        .split_whitespace()
                        .map(|s| s.to_owned())
                        .collect::<Vec<String>>(),
                    body: body.to_owned(),
                    desc: None,
                },
            );
        }
        Ok(())
    }

    /// Add static rules without builder pattern
    ///
    /// # Args
    ///
    /// The order of argument is "name, body"
    ///
    /// # Example
    ///
    /// ```rust
    /// processor.add_static_rules(vec![("macro_name","Macro body without arguments")]);
    /// ```
    pub fn add_static_rules(
        &mut self,
        rules: Vec<(impl AsRef<str>, impl AsRef<str>)>,
    ) -> RadResult<()> {
        if self.state.hygiene == Hygiene::Aseptic {
            self.log_warning(
                "Runtime macro declaration is disabled in aspectic mode",
                WarningType::Security,
            )?;
            return Ok(());
        }
        for (name, body) in rules {
            let name = name.as_ref();
            if !MAC_NAME.is_match(name) {
                return Err(RadError::InvalidMacroName(format!(
                    "Name : \"{}\" is not a valid macro name",
                    name
                )));
            }
            self.map.runtime.macros.insert(
                name.to_owned(),
                RuntimeMacro {
                    name: name.to_owned(),
                    args: vec![],
                    body: body.as_ref().to_owned(),
                    desc: None,
                },
            );
        }
        Ok(())
    }

    #[cfg(feature = "hook")]
    /// Register a hook
    pub fn register_hook(
        &mut self,
        hook_type: HookType,
        target_macro: &str,
        invoke_macro: &str,
        target_count: usize,
        resetable: bool,
    ) -> RadResult<()> {
        // Check target macro is empty
        if target_macro.len() == 0 {
            return Err(RadError::InvalidMacroName(format!(
                "Cannot register hook for macro \"{}\"",
                target_macro
            )));
        }

        // Check invoke macro is empty
        if invoke_macro.len() == 0 {
            return Err(RadError::InvalidMacroName(format!(
                "Cannot register hook which invokes a macro \"{}\"",
                target_macro
            )));
        }
        self.hook_map.add_hook(
            hook_type,
            target_macro,
            invoke_macro,
            target_count,
            resetable,
        )?;
        Ok(())
    }

    #[cfg(feature = "hook")]
    /// Deregister
    pub fn deregister_hook(&mut self, hook_type: HookType, target_macro: &str) -> RadResult<()> {
        // Check target macro is empty
        if target_macro.len() == 0 {
            return Err(RadError::InvalidMacroName(format!(
                "Cannot deregister hook for macro \"{}\"",
                target_macro
            )));
        }

        self.hook_map.del_hook(hook_type, target_macro)?;
        Ok(())
    }

    /// Read from string
    pub fn from_string(&mut self, content: &str) -> RadResult<Option<String>> {
        // Set name as string
        self.set_input_stdin()?;

        let mut reader = content.as_bytes();
        self.from_buffer(&mut reader, None, false)?;
        self.organize_and_clear_cache()
    }

    /// Read from standard input
    ///
    /// If debug mode is enabled this, doesn't read stdin line by line but by chunk because user
    /// input is also a standard input and processor cannot distinguish the two
    pub fn from_stdin(&mut self) -> RadResult<Option<String>> {
        #[allow(unused_imports)]
        use std::io::Read;
        let stdin = io::stdin();

        // Early return if debug
        // This read whole chunk of string
        #[cfg(feature = "debug")]
        if self.is_debug() {
            let mut input = String::new();
            stdin.lock().read_to_string(&mut input)?;
            // This is necessary to prevent unexpected output from being captured.
            self.from_buffer(&mut input.as_bytes(), None, false)?;
            return self.organize_and_clear_cache();
        }

        let mut reader = stdin.lock();
        self.from_buffer(&mut reader, None, false)?;
        self.organize_and_clear_cache()
    }

    /// Process contents from a file
    pub fn from_file(&mut self, path: impl AsRef<Path>) -> RadResult<Option<String>> {
        // Sandboxed environment, backup
        let backup = if self.state.sandbox {
            Some(self.backup())
        } else {
            None
        };
        // Set file as name of given path
        self.set_file(path.as_ref().to_str().unwrap())?;

        let file_stream = File::open(path)?;
        let mut reader = BufReader::new(file_stream);
        self.from_buffer(&mut reader, backup, false)?;
        self.organize_and_clear_cache()
    }

    /// Internal method that is executed by macro
    ///
    /// Target usages
    /// - include
    /// - temp_include
    ///
    pub(crate) fn from_file_as_chunk(
        &mut self,
        path: impl AsRef<Path>,
    ) -> RadResult<Option<String>> {
        // Sandboxed environment, backup
        let backup = if self.state.sandbox {
            Some(self.backup())
        } else {
            None
        };
        // Set file as name of given path
        self.set_file(path.as_ref().to_str().unwrap())?;

        let file_stream = File::open(path)?;
        let mut reader = BufReader::new(file_stream);
        let chunk = self.from_buffer(&mut reader, backup, true);
        chunk
    }

    /// Internal method for processing buffers line by line
    fn from_buffer(
        &mut self,
        buffer: &mut impl std::io::BufRead,
        backup: Option<SandboxBackup>,
        use_container: bool,
    ) -> RadResult<Option<String>> {
        let mut line_iter = Utils::full_lines(buffer).peekable();
        let mut lexor = Lexor::new(
            self.get_macro_char(),
            self.get_comment_char(),
            &self.state.comment_type,
        );
        let mut frag = MacroFragment::new();

        // Container can be used when file include is nested inside macro definition
        // Without container, namely read macro, will not preserve the order
        // of definition and simply print everything before evaluation
        let container = String::new(); // Don't remove this!
        let mut cont = if use_container { Some(container) } else { None };

        #[cfg(feature = "debug")]
        self.debugger
            .user_input_on_start(&self.state.current_input.to_string(), &mut self.logger)?;
        loop {
            #[cfg(feature = "debug")]
            if let Some(line) = line_iter.peek() {
                let line = line.as_ref().unwrap();
                // Update line cache
                self.debugger.add_line_cache(line);
                // Only if debug switch is nextline
                self.debugger.user_input_on_line(&frag, &mut self.logger)?;
            }
            let result = self.parse_line(&mut line_iter, &mut lexor, &mut frag)?;
            match result {
                // This means either macro is not found at all
                // or previous macro fragment failed with invalid syntax
                ParseResult::Printable(remainder) => {
                    self.write_to(&remainder, &mut cont)?;

                    // Test if this works
                    #[cfg(feature = "debug")]
                    self.debugger.clear_line_cache();

                    // Reset fragment
                    if &frag.whole_string != "" {
                        frag = MacroFragment::new();
                    }
                }
                ParseResult::FoundMacro(remainder) => {
                    self.write_to(&remainder, &mut cont)?;
                }
                // This happens only when given macro involved text should not be printed
                ParseResult::NoPrint => {}
                // End of input, end loop
                ParseResult::EOI => {
                    // THis is necessary somehow, its kinda hard to explain
                    // but chunk read makes trailing new line and it should be deleted
                    if use_container {
                        Utils::pop_newline(cont.as_mut().unwrap());
                    }
                    break;
                }
            }
            // Increaing number should be followed after evaluation
            // To ensure no panick occurs during user_input_on_line, which is caused by
            // out of index exception from getting current line_cache
            // Increase absolute line number
            #[cfg(feature = "debug")]
            self.debugger.inc_line_number();

            // Execute queued object
            let queued = std::mem::replace(&mut self.state.queued, vec![]); // Queue should be emptied after
            for item in queued {
                // This invokes parse method
                let result = self.parse_chunk_args(0, MAIN_CALLER, &item)?;
                self.write_to(&result, &mut cont)?;
            }
        } // Loop end

        // Recover previous state from sandboxed processing
        if let Some(backup) = backup {
            self.recover(backup)?;
            self.state.sandbox = false;
        }

        if use_container {
            Ok(cont)
        } else {
            Ok(None)
        }
    }

    // End of process methods
    // </PROCESS>
    // ----------

    // ===========
    // Debug related methods
    // <DEBUG>

    /// Check if given macro is local macro or not
    ///
    /// This is used when step debug command is to be executed.
    /// Without chekcing locality, step will go inside local binding macros
    #[cfg(feature = "debug")]
    fn is_local(&self, mut level: usize, name: &str) -> bool {
        while level > 0 {
            if self
                .map
                .local
                .contains_key(&Utils::local_name(level, &name))
            {
                return true;
            }
            level = level - 1;
        }
        false
    }

    /// Check if debug macro should be executed
    #[cfg(feature = "debug")]
    fn check_debug_macro(&mut self, frag: &mut MacroFragment, level: usize) -> RadResult<()> {
        // Early return if not in a debug mode
        if !self.is_debug() {
            return Ok(());
        }

        // If debug switch target is next macro
        // Stop and wait for input
        // Only on main level macro
        if level == 0 {
            self.debugger.user_input_on_macro(&frag, &mut self.logger)?;
        } else {
            self.debugger.user_input_on_step(&frag, &mut self.logger)?;
        }

        // Clear line_caches
        if level == 0 {
            self.debugger.clear_line_cache();
        }
        Ok(())
    }

    // </DEBUG>
    // End of debug methods
    // ----------

    // ----------
    // Parse related methods
    // <PARSE>
    /// Parse line is called only by the main loop thus, caller name is special name of @MAIN@
    ///
    /// This parses given input as line by line with an iterator of lines including trailing new
    /// line chracter.
    fn parse_line(
        &mut self,
        lines: &mut impl std::iter::Iterator<Item = std::io::Result<String>>,
        lexor: &mut Lexor,
        frag: &mut MacroFragment,
    ) -> RadResult<ParseResult> {
        self.logger.add_line_number();
        if let Some(line) = lines.next() {
            let line = line?;

            // Deny newline
            if self.state.deny_newline {
                self.state.deny_newline = false;
                if line == "\n" || line == "\r\n" {
                    return Ok(ParseResult::NoPrint);
                }
            }

            match self.state.flow_control {
                FlowControl::Escape => return Ok(ParseResult::Printable(line)),
                FlowControl::Exit => return Err(RadError::Exit),
                FlowControl::None => (),
            }

            // Save to original
            #[cfg(feature = "debug")]
            self.debugger.write_to_original(&line)?;

            let remainder = self.parse(lexor, frag, &line, 0, MAIN_CALLER)?;

            // Clear local variable macros
            self.map.clear_local();

            // Clear volatile variables when macro hygiene is enabled
            if self.state.hygiene == Hygiene::Macro {
                self.map.clear_runtime_macros(true);
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

    /// Parse chunk args by separating it into lines which implements BufRead
    pub(crate) fn parse_chunk_args(
        &mut self,
        level: usize,
        _caller: &str,
        chunk: &str,
    ) -> RadResult<String> {
        let mut lexor = Lexor::new(
            self.get_macro_char(),
            self.get_comment_char(),
            &self.state.comment_type,
        );
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

        // If unexpanded texts remains
        // Add to result
        if !frag.whole_string.is_empty() {
            result.push_str(&frag.whole_string);
        }

        self.logger.set_chunk(false);
        self.logger.recover_lines(backup);
        return Ok(result);
    } // parse_chunk_lines end

    /// Parse chunk body without separating lines
    ///
    /// In contrast to parse_chunk_lines, parse_chunk doesn't create lines iterator but parses the
    /// chunk as a single entity or line.
    fn parse_chunk_body(&mut self, level: usize, caller: &str, chunk: &str) -> RadResult<String> {
        let mut lexor = Lexor::new(
            self.get_macro_char(),
            self.get_comment_char(),
            &self.state.comment_type,
        );
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
    ///
    /// This calles lexor.lex to validate characters and decides next behaviour
    fn parse(
        &mut self,
        lexor: &mut Lexor,
        frag: &mut MacroFragment,
        line: &str,
        level: usize,
        caller: &str,
    ) -> RadResult<String> {
        // Initiate values
        // Reset character number
        self.logger.reset_char_number();
        // Local values
        let mut remainder = String::new();

        // Reset lexor's escape_nl
        lexor.reset_escape();

        // If escape_nl is set as global attribute, set escape_newline
        if self.state.escape_newline {
            lexor.escape_next_newline();
            self.state.escape_newline = false;
        }

        // Check comment line
        // If it is a comment then return nothing and write nothing
        if self.state.comment_type != CommentType::None && line.starts_with(self.get_comment_char())
        {
            return Ok(String::new());
        }

        for ch in line.chars() {
            self.logger.add_char_number();

            let lex_result = lexor.lex(ch)?;
            // Either add character to remainder or fragments
            match lex_result {
                LexResult::CommentExit => {
                    self.lex_branch_comment_exit(frag, &mut remainder);
                    return Ok(remainder);
                }
                LexResult::Discard => (),
                LexResult::Ignore => frag.whole_string.push(ch),
                // If given result is literal
                LexResult::Literal(cursor) => {
                    self.lex_branch_literal(ch, frag, &mut remainder, cursor);
                }
                LexResult::StartFrag => {
                    self.lex_branch_start_frag(ch, frag, &mut remainder, lexor)?;
                }
                LexResult::RestartName => {
                    // This restart frags
                    remainder.push_str(&frag.whole_string);
                    frag.clear();
                    frag.whole_string.push(self.get_macro_char());
                }
                LexResult::EmptyName => {
                    self.lex_branch_empty_name(ch, frag, &mut remainder, lexor);
                }
                LexResult::AddToRemainder => {
                    self.lex_branch_add_to_remainder(ch, &mut remainder)?;
                }
                LexResult::AddToFrag(cursor) => {
                    self.lex_branch_add_to_frag(ch, frag, cursor)?;
                }
                LexResult::EndFrag => {
                    self.lex_branch_end_frag(ch, frag, &mut remainder, lexor, level, caller)?;
                }
                // Remove fragment and set to remainder
                LexResult::ExitFrag => {
                    self.lex_branch_exit_frag(ch, frag, &mut remainder);
                }
            }

            // Character hook macro evaluation
            // This only works on plain texts with level 0
            #[cfg(feature = "hook")]
            if frag.is_empty() && level == 0 {
                if let Some(mac_name) = self.hook_map.add_char_count(ch) {
                    // If no fragment information is saved then
                    let mut hook_frag = MacroFragment::new();
                    hook_frag.name = mac_name;
                    let mut hook_mainder = String::new();
                    let mut hook_lexor = Lexor::new(
                        self.get_macro_char(),
                        self.get_comment_char(),
                        &self.state.comment_type,
                    );
                    self.lex_branch_end_invoke(
                        &mut hook_lexor,
                        &mut hook_frag,
                        &mut hook_mainder,
                        level,
                        &frag.name,
                    )?;

                    // If there is content to add
                    if hook_mainder.len() != 0 {
                        remainder.push_str(&hook_mainder);
                    }
                } // End if let of macro name
            }
        } // End Character iteration
        Ok(remainder)
    }

    // Evaluate can be nested deeply
    // Disable caller for temporary
    /// Evaluate detected macro usage
    ///
    /// Evaluation order is followed
    /// - Keyword macro
    /// - Local bound macro
    /// - Custom macro
    /// - Basic macro
    fn evaluate(
        &mut self,
        level: usize,
        caller: &str,
        frag: &mut MacroFragment,
    ) -> RadResult<EvalResult> {
        // Increase level to represent nestedness
        let level = level + 1;
        let (name, raw_args) = (&frag.name, &frag.args);

        let mut args: String = raw_args.to_owned();
        // Preprocess only when macro is not a deterred macro
        if !self.map.is_deterred_macro(name) {
            // This parses and processes arguments
            // and macro should be evaluated after
            args = self.parse_chunk_args(level, name, raw_args)?;

            // Also update original arguments for better debugging
            #[cfg(feature = "debug")]
            {
                frag.processed_args = args.clone();
            }
        }

        // Possibly inifinite loop so warn user
        if caller == name {
            self.log_warning(
                &format!(
                    "Calling self, which is \"{}\", can possibly trigger infinite loop. This is also occured when argument's name is equal to macro's name.",
                    name,
                ),
                WarningType::Sanity,
            )?;
        }

        // Find deterred macro
        if self.map.is_deterred_macro(name) {
            if let Some(func) = self.map.deterred.get_deterred_macro(name) {
                let final_result = func(&args, level, self)?;
                // TODO
                // Make parse logic consistent, not defined by delarator
                // result = self.parse_chunk_args(level, caller, &result)?;
                return Ok(EvalResult::Eval(final_result));
            }
        }

        // Find local macro
        // The macro can be  the one defined in parent macro
        let mut temp_level = level;
        while temp_level > 0 {
            if let Some(local) = self.map.local.get(&Utils::local_name(temp_level, &name)) {
                return Ok(EvalResult::Eval(Some(local.body.to_owned())));
            }
            temp_level = temp_level - 1;
        }
        // Find runtime macro
        // runtime macro comes before function macro so that
        // user can override it
        if self.map.runtime.contains(name, self.state.hygiene) {
            // Prevent invocation if relaying to
            if let Some(RelayTarget::Macro(mac)) = &self.state.relay.last() {
                if mac == name {
                    return Err(RadError::UnallowedMacroExecution(format!(
                        "Cannot execute macro \"{}\" when it is being relayed to",
                        mac
                    )));
                }
            }

            if let Some(result) = self.invoke_rule(level, name, &args)? {
                return Ok(EvalResult::Eval(Some(result)));
            } else {
                return Ok(EvalResult::InvalidArg);
            }
        }
        // Find function macro
        else if self.map.function.contains(&name) {
            // Func always exists, because contains succeeded.
            let func = self.map.function.get_func(name).unwrap();
            let final_result = func(&args, self)?;
            return Ok(EvalResult::Eval(final_result));
        }
        // No macros found to evaluate
        else {
            return Ok(EvalResult::InvalidName);
        }
    }

    /// Invoke a runtime rule and get a result
    ///
    /// Invoke rule evaluates body of macro rule because body is not evaluated on register process
    fn invoke_rule(
        &mut self,
        level: usize,
        name: &str,
        arg_values: &str,
    ) -> RadResult<Option<String>> {
        // Get rule
        // Invoke is called only when key exists, thus unwrap is safe
        let rule = self
            .map
            .runtime
            .get(name, self.state.hygiene)
            .unwrap()
            .clone();

        let arg_types = &rule.args;
        let args: Vec<String>;
        // Set variable to local macros
        if let Some(content) = ArgParser::new().args_with_len(arg_values, arg_types.len()) {
            args = content;
        } else {
            // Necessary arg count is bigger than given arguments
            self.log_error(&format!(
                "{}'s arguments are not sufficient. Given {}, but needs {}",
                name,
                ArgParser::new()
                    .args_to_vec(arg_values, ',', GreedyState::Never)
                    .len(),
                arg_types.len()
            ))?;
            return Ok(None);
        }

        for (idx, arg_type) in arg_types.iter().enumerate() {
            //Set arg to be substitued
            self.map.add_local_macro(level + 1, arg_type, &args[idx]);
        }
        // Process the rule body
        let result = self.parse_chunk_body(level, &name, &rule.body)?;

        // Clear lower locals to prevent local collisions
        self.map.clear_lower_locals(level);

        Ok(Some(result))
    }

    /// Add runtime rule to macro map
    ///
    /// This doesn't clear fragment
    fn add_rule(&mut self, frag: &MacroFragment, remainder: &mut String) -> RadResult<()> {
        if let Some((name, args, body)) = self.define_parser.parse_define(&frag.args) {
            // Strict mode
            // Overriding is prohibited
            if self.state.behaviour == Behaviour::Strict
                && self
                    .map
                    .contains_macro(&name, MacroType::Any, self.state.hygiene)
            {
                self.log_error("Can't override exsiting macro on strict mode")?;
                return Err(RadError::StrictPanic);
            }

            self.map
                .register_runtime(&name, &args, &body, self.state.hygiene)?;
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
    fn write_to(&mut self, content: &str, container: &mut Option<String>) -> RadResult<()> {
        // Don't try to write empty string, because it's a waste
        if content.len() == 0 {
            return Ok(());
        }

        // Save to container if it has value then return
        if let Some(cont) = container.as_mut() {
            cont.push_str(content);
            return Ok(());
        }

        // Save to "source" file for debuggin
        #[cfg(feature = "debug")]
        self.debugger.write_to_processed(content)?;

        match self
            .state
            .relay
            .last_mut()
            .unwrap_or(&mut RelayTarget::None)
        {
            RelayTarget::Macro(mac) => {
                if !self
                    .map
                    .contains_macro(mac, MacroType::Runtime, self.state.hygiene)
                {
                    return Err(RadError::InvalidMacroName(format!(
                        "Cannot relay to non-exsitent macro \"{}\"",
                        mac
                    )));
                }
                self.map.append(&mac, content, self.state.hygiene);
            }
            RelayTarget::File(target) => {
                // NOTE
                // Pracically this cannot be set in wasm target because multiple code barriers
                // yet panic can theorically happen.
                target
                    .file
                    .as_mut()
                    .unwrap()
                    .write_all(content.as_bytes())?;
            }
            #[cfg(not(feature = "wasm"))]
            RelayTarget::Temp => {
                if let Some(file) = self.get_temp_file() {
                    file.write(content.as_bytes())?;
                }
            }
            RelayTarget::None => {
                match &mut self.write_option {
                    WriteOption::File(f) => f.write_all(content.as_bytes())?,
                    WriteOption::Terminal => std::io::stdout().write_all(content.as_bytes())?,
                    WriteOption::Variable(var) => var.push_str(content),
                    WriteOption::Return => self.cache.push_str(content),
                    WriteOption::Discard => (), // Don't print anything
                }
            }
        }

        Ok(())
    }

    // ==========
    // <LEX>
    // Start of lex branch methods
    // These are parse's sub methods for eaiser reading
    fn lex_branch_comment_exit(&mut self, frag: &mut MacroFragment, remainder: &mut String) {
        remainder.push_str(&frag.whole_string);
        remainder.push_str(&self.state.newline);
        frag.clear();
    }
    fn lex_branch_literal(
        &mut self,
        ch: char,
        frag: &mut MacroFragment,
        remainder: &mut String,
        cursor: Cursor,
    ) {
        match cursor {
            // Exit frag
            // If literal is given on names
            Cursor::Name => {
                frag.whole_string.push(ch);
                remainder.push_str(&frag.whole_string);
                frag.clear();
            }
            // Simply push if none or arg
            Cursor::None => {
                remainder.push(ch);
            }
            Cursor::Arg => {
                frag.args.push(ch);
                frag.whole_string.push(ch);
            }
        }
    }

    fn lex_branch_start_frag(
        &mut self,
        ch: char,
        frag: &mut MacroFragment,
        remainder: &mut String,
        lexor: &mut Lexor,
    ) -> RadResult<()> {
        #[cfg(feature = "debug")]
        self.debugger
            .user_input_before_macro(&frag, &mut self.logger)?;

        frag.whole_string.push(ch);

        // If paused and not pause, then reset lexor context
        if self.state.paused && frag.name != "pause" {
            lexor.reset();
            remainder.push_str(&frag.whole_string);
            frag.clear();
        }

        Ok(())
    }

    fn lex_branch_empty_name(
        &mut self,
        ch: char,
        frag: &mut MacroFragment,
        remainder: &mut String,
        lexor: &mut Lexor,
    ) {
        // THis is necessary because whole string should be whole anyway
        frag.whole_string.push(ch);
        // Freeze needed for logging
        self.logger.freeze_number();
        // If paused, then reset lexor context to remove cost
        if self.state.paused {
            lexor.reset();
            remainder.push_str(&frag.whole_string);
            frag.clear();
        }
    }

    fn lex_branch_add_to_remainder(&mut self, ch: char, remainder: &mut String) -> RadResult<()> {
        if !self.checker.check(ch) && !self.state.paused {
            self.logger.freeze_number();
            self.log_warning("Unbalanced parenthesis detected.", WarningType::Sanity)?;
        }
        remainder.push(ch);

        Ok(())
    }

    fn lex_branch_add_to_frag(
        &mut self,
        ch: char,
        frag: &mut MacroFragment,
        cursor: Cursor,
    ) -> RadResult<()> {
        match cursor {
            Cursor::Name => {
                if frag.name.len() == 0 {
                    self.logger.freeze_number();
                }
                match ch {
                    '|' => frag.pipe = true,
                    '*' => frag.yield_literal = true,
                    '^' => frag.trimmed = true,
                    _ => {
                        if frag.has_attribute() {
                            self.log_error("Received non attribute character after macro attributes which is not allowed")?;
                            return Err(RadError::InvalidMacroName(
                                "Invalid macro attribute".to_owned(),
                            ));
                        }
                        frag.name.push(ch);
                    }
                }
            }
            Cursor::Arg => frag.args.push(ch),
            _ => unreachable!(),
        }
        frag.whole_string.push(ch);
        Ok(())
    }

    fn lex_branch_end_frag(
        &mut self,
        ch: char,
        frag: &mut MacroFragment,
        remainder: &mut String,
        lexor: &mut Lexor,
        level: usize,
        caller: &str,
    ) -> RadResult<()> {
        // Push character to whole string anyway
        frag.whole_string.push(ch);

        if frag.name == DEFINE_KEYWORD {
            // Within aseptic circumstances you cannot define runtime macros
            if self.state.hygiene == Hygiene::Aseptic {
                self.log_strict(
                    "Runtime macro declaration is disabled in aseptic mode",
                    WarningType::Security,
                )?;
                frag.clear();
                lexor.escape_next_newline();
                if self.state.behaviour == Behaviour::Strict {
                    return Err(RadError::StrictPanic);
                }
            } else {
                self.lex_branch_end_frag_define(
                    lexor,
                    frag,
                    remainder,
                    #[cfg(feature = "debug")]
                    level,
                )?;
            }
        } else {
            // Invoke macro
            self.lex_branch_end_invoke(lexor, frag, remainder, level, caller)?;
        }

        // Last return
        Ok(())
    }

    // Level is necessary for debug feature
    fn lex_branch_end_frag_define(
        &mut self,
        lexor: &mut Lexor,
        frag: &mut MacroFragment,
        remainder: &mut String,
        #[cfg(feature = "debug")] level: usize,
    ) -> RadResult<()> {
        self.add_rule(frag, remainder)?;
        lexor.escape_next_newline();
        #[cfg(feature = "debug")]
        self.check_debug_macro(frag, level)?;

        frag.clear();
        Ok(())
    }

    // TODO
    // This should be renamed, because now it is not only used by lex branches
    // but also used by external logic functions such as hook macro executions.
    fn lex_branch_end_invoke(
        &mut self,
        lexor: &mut Lexor,
        frag: &mut MacroFragment,
        remainder: &mut String,
        level: usize,
        caller: &str,
    ) -> RadResult<()> {
        // Name is empty
        if frag.name.len() == 0 {
            self.log_error("Name is empty")?;
            remainder.push_str(&frag.whole_string);
            frag.clear();
            return Ok(());
        }

        // Debug
        #[cfg(feature = "debug")]
        {
            // Print a log information
            self.debugger
                .print_log(&frag.name, &frag.args, frag, &mut self.logger)?;

            // If debug switch target is break point
            // Set switch to next line.
            self.debugger.break_point(frag, &mut self.logger)?;
            // Break point is true , continue
            if frag.name.len() == 0 {
                lexor.escape_next_newline();
                return Ok(());
            }
        }

        let evaluation_result = self.evaluate(level, caller, frag);

        match evaluation_result {
            // If panicked, this means unrecoverable error occured.
            Err(error) => {
                self.lex_branch_end_frag_eval_result_error(error)?;
            }
            Ok(eval_variant) => {
                self.lex_branch_end_frag_eval_result_ok(
                    eval_variant,
                    frag,
                    remainder,
                    lexor,
                    level,
                )?;
            }
        }
        // Clear fragment regardless of success
        frag.clear();

        Ok(())
    }

    fn lex_branch_end_frag_eval_result_error(&mut self, error: RadError) -> RadResult<()> {
        // this is equlvalent to conceptual if let not pattern
        // Log error when panic occured
        if let RadError::Panic = error {
            // Do nothing
            ();
        } else {
            self.log_error(&format!("{}", error))?;
        }

        // If nopanic don't panic
        // If not, re-throw err to caller
        if self.state.behaviour == Behaviour::Nopanic {
            Ok(())
        } else {
            Err(RadError::Panic)
        }
    }

    // Level is needed for feature debug & hook codes
    fn lex_branch_end_frag_eval_result_ok(
        &mut self,
        variant: EvalResult,
        frag: &mut MacroFragment,
        remainder: &mut String,
        lexor: &mut Lexor,
        #[allow(unused_variables)] level: usize,
    ) -> RadResult<()> {
        match variant {
            EvalResult::Eval(content) => {
                // Debug
                // Debug command after macro evaluation
                // This goes to last line and print last line
                #[cfg(feature = "debug")]
                if !self.is_local(level + 1, &frag.name) {
                    // Only when macro is not a local macro
                    self.check_debug_macro(frag, level)?;
                }

                // If content is none
                // Ignore new line after macro evaluation until any character
                if let None = content {
                    lexor.escape_next_newline();
                } else {
                    // else it is ok to proceed.
                    // thus it is safe to unwrap it
                    let mut content = content.unwrap();
                    if frag.trimmed {
                        content = Utils::trim(&content);
                    }
                    if frag.yield_literal {
                        content = format!("\\*{}*\\", content);
                    }

                    // TODO
                    // Check unpredicted duplicate hook
                    // Macro hook check
                    #[cfg(feature = "hook")]
                    if let Some(mac_name) = self.hook_map.add_macro_count(&frag.name) {
                        let mut hook_frag = MacroFragment::new();
                        hook_frag.name = mac_name;
                        let mut hook_mainder = String::new();
                        let mut hook_lexor = Lexor::new(
                            self.get_macro_char(),
                            self.get_comment_char(),
                            &self.state.comment_type,
                        );
                        self.lex_branch_end_invoke(
                            &mut hook_lexor,
                            &mut hook_frag,
                            &mut hook_mainder,
                            level,
                            &frag.name,
                        )?;

                        // Add hook remainder into current content
                        // which will be added to main remainder
                        if hook_mainder.len() != 0 {
                            content.push_str(&hook_mainder);
                        }
                    }

                    // NOTE
                    // This should come later!!
                    // because pipe should respect all other macro attributes
                    // not the other way
                    if frag.pipe {
                        self.state.add_pipe(None, content);
                        lexor.escape_next_newline();
                    } else {
                        remainder.push_str(&content);
                    }
                }
            }
            EvalResult::InvalidArg => {
                if self.state.behaviour == Behaviour::Strict {
                    return Err(RadError::StrictPanic);
                } else {
                    remainder.push_str(&frag.whole_string);
                }
            }
            EvalResult::InvalidName => {
                // Failed to invoke
                // because macro doesn't exist

                match self.state.behaviour {
                    Behaviour::Strict => {
                        self.log_error(&format!("Failed to invoke a macro : \"{}\"", frag.name))?;
                        return Err(RadError::StrictPanic);
                    }
                    // If purge mode is set, don't print anything
                    // and don't print error
                    Behaviour::Purge | Behaviour::Nopanic => lexor.escape_next_newline(),
                    _ => {
                        self.log_error(&format!("Failed to invoke a macro : \"{}\"", frag.name))?;
                        remainder.push_str(&frag.whole_string);
                    }
                }
            }
        } // End match
        Ok(())
    }

    fn lex_branch_exit_frag(&mut self, ch: char, frag: &mut MacroFragment, remainder: &mut String) {
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
    /// Get comment chararacter
    ///
    /// This will return custom character if existent
    pub(crate) fn get_comment_char(&self) -> char {
        comment_start(self.state.comment_char)
    }

    /// Get macro chararacter
    ///
    /// This will return custom character if existent
    pub(crate) fn get_macro_char(&self) -> char {
        macro_start(self.state.macro_char)
    }

    /// Bridge method to get auth state
    pub(crate) fn get_auth_state(&self, auth_type: &AuthType) -> AuthState {
        *self.state.auth_flags.get_state(auth_type)
    }

    /// Change temp file target
    ///
    /// This will create a new temp file if not existent
    #[cfg(not(feature = "wasm"))]
    pub(crate) fn set_temp_file(&mut self, path: &Path) {
        self.state.set_temp_target(path);
    }

    /// Set pipe value manually
    #[allow(dead_code)]
    pub(crate) fn set_pipe(&mut self, value: &str) {
        self.state
            .pipe_map
            .insert("-".to_owned(), value.to_string());
    }

    /// Set debug flag
    #[cfg(feature = "debug")]
    pub(crate) fn set_debug(&mut self, debug: bool) {
        self.debugger.debug = debug;
    }

    /// Turn on sandbox
    ///
    /// This is an explicit state change method for non-processor module's usage
    ///
    /// Sandbox means that current state(cursor) of processor should not be applied for following
    /// independent processing
    pub(crate) fn set_sandbox(&mut self) {
        self.state.sandbox = true;
    }

    #[cfg(not(feature = "wasm"))]
    /// Get temp file's path
    pub(crate) fn get_temp_path(&self) -> &Path {
        &self.state.temp_target.path
    }

    #[cfg(not(feature = "wasm"))]
    /// Get temp file's "file" struct
    pub(crate) fn get_temp_file(&mut self) -> Option<&mut File> {
        self.state.temp_target.file.as_mut()
    }

    /// Backup information of current file before processing sandboxed input
    fn backup(&self) -> SandboxBackup {
        SandboxBackup {
            current_input: self.state.current_input.clone(),
            local_macro_map: self.map.local.clone(),
            logger_lines: self.logger.backup_lines(),
        }
    }

    /// Recover backup information into the processor
    fn recover(&mut self, backup: SandboxBackup) -> RadResult<()> {
        // NOTE ::: Set file should come first becuase set_file override line number and character number
        self.logger.set_input(&backup.current_input);
        self.state.current_input = backup.current_input;
        self.map.local = backup.local_macro_map;
        self.logger.recover_lines(backup.logger_lines);

        // Also recover env values
        self.set_file_env(&self.state.current_input.to_string())?;
        Ok(())
    }

    pub(crate) fn track_assertion(&mut self, success: bool) -> RadResult<()> {
        self.logger.alog(success)?;
        Ok(())
    }

    /// This prints error if strict mode else warning
    pub(crate) fn log_strict(&mut self, log: &str, warning_type: WarningType) -> RadResult<()> {
        if self.state.behaviour == Behaviour::Strict {
            self.logger.elog(log)?;
        } else {
            self.logger.wlog(log, warning_type)?;
        }
        Ok(())
    }

    /// Log error
    pub(crate) fn log_error(&mut self, log: &str) -> RadResult<()> {
        self.logger.elog(log)?;
        Ok(())
    }

    /// Log warning
    pub(crate) fn log_warning(&mut self, log: &str, warning_type: WarningType) -> RadResult<()> {
        self.logger.wlog(log, warning_type)?;
        Ok(())
    }

    // This is not a backup but fresh set of file information
    /// Set(update) current processing file information
    fn set_file(&mut self, file: &str) -> RadResult<()> {
        let path = Path::new(file);
        if !path.exists() {
            Err(RadError::InvalidCommandOption(format!(
                "File, \"{}\" doesn't exist, therefore cannot be read by r4d.",
                path.display()
            )))
        } else {
            let path = PathBuf::from(file);
            self.state.input_stack.insert(path.canonicalize()?);
            let input = ProcessInput::File(path);
            self.state.current_input = input.clone();
            self.logger.set_input(&input);
            self.set_file_env(file)?;
            Ok(())
        }
    }

    /// Set some useful env values
    fn set_file_env(&self, file: &str) -> RadResult<()> {
        let path = Path::new(file);
        std::env::set_var("RAD_FILE", file);
        std::env::set_var(
            "RAD_FILE_DIR",
            std::fs::canonicalize(path)?.parent().unwrap(),
        );
        Ok(())
    }

    /// Set input as string not as &path
    ///
    /// This is conceptualy identical to set_file but doesn't validate if given input is existent
    fn set_input_stdin(&mut self) -> RadResult<()> {
        self.state.current_input = ProcessInput::Stdin;
        self.logger.set_input(&ProcessInput::Stdin);
        // Why no set_file_env like set_file?
        Ok(())
    }

    #[cfg(feature = "debug")]
    pub(crate) fn is_debug(&self) -> bool {
        self.debugger.debug
    }

    /// Get debug switch
    #[cfg(feature = "debug")]
    pub(crate) fn get_debug_switch(&self) -> &DebugSwitch {
        &self.debugger.debug_switch
    }

    /// Set custom prompt log
    #[cfg(feature = "debug")]
    pub(crate) fn set_prompt_log(&mut self, prompt: &str) {
        self.debugger.set_prompt_log(prompt);
    }

    // End of miscellaenous methods
    // </MISC>
    // ----------

    // ----------
    // Function that is exposed for better end user's qualify of life
    // <EXT>

    pub fn set_documentation(&mut self, macro_name: &str, content: &str) -> bool {
        if let Some(mac) = self.map.runtime.get_mut(macro_name, Hygiene::None) {
            mac.desc = Some(content.to_owned());
            true
        } else {
            false
        }
    }

    pub fn get_current_dir(&self) -> RadResult<PathBuf> {
        let path = match &self.state.current_input {
            ProcessInput::Stdin => std::env::current_dir()?,
            ProcessInput::File(path) => path
                .parent()
                .unwrap_or(&std::env::current_dir()?)
                .to_owned(),
        };
        Ok(path)
    }

    pub fn print_error(&mut self, error: &str) -> RadResult<()> {
        self.log_error(error)?;
        Ok(())
    }

    pub fn get_split_arguments(
        &self,
        target_length: usize,
        source: &str,
    ) -> RadResult<Vec<String>> {
        if let Some(args) = ArgParser::new().args_with_len(source, target_length) {
            Ok(args)
        } else {
            Err(RadError::InvalidArgument(format!(
                "Insufficient arguments."
            )))
        }
    }

    /// Check auth information
    ///
    /// This will print log_error if auth was enabled with warning
    ///
    /// @return If auth is enabled or not
    pub fn check_auth(&mut self, auth_type: AuthType) -> RadResult<bool> {
        let variant = match self.state.auth_flags.get_state(&auth_type) {
            AuthState::Warn => {
                self.log_warning(
                    &format!("{} was enabled with warning", auth_type),
                    WarningType::Security,
                )?;
                true
            }
            AuthState::Open => true,
            AuthState::Restricted => false,
        };

        Ok(variant)
    }

    /// Expand given text
    pub fn expand(&mut self, level: usize, source: impl AsRef<str>) -> RadResult<String> {
        self.parse_chunk_args(level, MAIN_CALLER, source.as_ref())
    }

    /// Check if given macro exists
    pub fn contains_macro(&self, macro_name: &str, macro_type: MacroType) -> bool {
        self.map
            .contains_macro(macro_name, macro_type, self.state.hygiene)
    }

    /// Try undefine macro
    pub fn undefine_macro(&mut self, macro_name: &str, macro_type: MacroType) {
        self.map
            .undefine(macro_name, macro_type, self.state.hygiene);
    }

    /// Rename macro
    pub fn rename_macro(&mut self, macro_name: &str, target_name: &str, macro_type: MacroType) {
        self.map
            .rename(macro_name, target_name, macro_type, self.state.hygiene);
    }

    /// Append content into a macro
    pub fn append_macro(&mut self, macro_name: &str, target: &str) {
        self.map.append(macro_name, target, self.state.hygiene);
    }

    /// Replace macro's content
    pub fn replace_macro(&mut self, macro_name: &str, target: &str) -> bool {
        self.map.replace(macro_name, target, self.state.hygiene)
    }

    /// Add new local macro
    pub fn add_new_local_macro(&mut self, level: usize, macro_name: &str, body: &str) {
        self.map.add_local_macro(level, macro_name, body);
    }

    /// Check if given text is boolean-able
    pub fn is_true(&self, src: &str) -> RadResult<bool> {
        Utils::is_arg_true(src)
    }

    // </EXT>
    // ----------
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
    current_input: ProcessInput,
    local_macro_map: HashMap<String, LocalMacro>,
    logger_lines: LoggerLines,
}

enum EvalResult {
    Eval(Option<String>),
    InvalidName,
    InvalidArg,
}
