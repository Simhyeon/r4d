use super::ProcessorState;
use crate::auth::{AuthState, AuthType};
#[cfg(feature = "debug")]
use crate::common::DiffOption;
#[cfg(feature = "signature")]
use crate::common::SignatureType;
use crate::common::{
    CommentType, ErrorBehaviour, FlowControl, Hygiene, LocalMacro, MacroFragment, MacroType,
    ProcessInput, ProcessType, RelayTarget, WriteOption,
};
#[cfg(feature = "debug")]
use crate::debugger::DebugSwitch;
#[cfg(feature = "debug")]
use crate::debugger::Debugger;
use crate::error::RadError;
use crate::extension::{ExtMacroBuilder, ExtMacroType};
#[cfg(feature = "hook")]
use crate::hookmap::{HookMap, HookType};
use crate::lexor::*;
use crate::logger::TrackType;
use crate::logger::{Logger, WarningType};
use crate::map::MacroMap;
use crate::package::StaticScript;
use crate::runtime_map::RuntimeMacro;
#[cfg(feature = "signature")]
use crate::sigmap::SignatureMap;
use crate::storage::{RadStorage, StorageOutput};
use crate::trim;
use crate::utils::Utils;
use crate::DefineParser;
use crate::{consts::*, RadResult};
use crate::{ArgParser, GreedyState};
#[cfg(feature = "cindex")]
use cindex::Indexer;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, Read, Write};
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

/// Central macro logic processor
///
/// - Processor parses given input and expands detected macros
/// - Processor substitutes all macros only when the macros were already defined. The fallback
/// behaviour can be configured.
/// - Processor can handle various types of inputs (string|stdin|file)
///
/// # Detailed usage
/// ```no_run
/// use r4d::{
///     RadResult, Processor, AuthType, CommentType,
///     WarningType, MacroType, Hygiene
/// };
/// #[cfg(feature = "debug")]
/// use r4d::DiffOption;
/// #[cfg(feature = "hook")]
/// use r4d::HookType; // This is behind hook feature
/// use std::path::Path;
///
/// fn main() -> RadResult<()> {
///     // Builder
///     let mut processor = Processor::new()
///         .set_comment_type(CommentType::Start)                // Use comment
///         .custom_macro_char('~')?                             // use custom macro character
///         .custom_comment_char('#')?                           // use custom comment character
///         .purge(true)                                         // Purge undefined macro
///         .silent(WarningType::Security)                       // Silents all warnings
///         .assert(true)                                        // Enable assertion mode
///         .lenient(true)                                       // Disable strict mode
///         .hygiene(Hygiene::Macro)                             // Enable hygiene mode
///         .pipe_truncate(false)                                // Disable pipe truncate
///         .write_to_file(Path::new("out.txt"))?                // default is stdout
///         .error_to_file(Path::new("err.txt"))?                // default is stderr
///         .unix_new_line(true)                                 // use unix new line for formatting
///         .discard(true)                                       // discard all output
///         .melt_files(&[Path::new("source.r4d")])?             // Read runtime macros from frozen
///         // Permission
///         .allow(&[AuthType::ENV])                             // Grant permission of authtypes
///         .allow_with_warning(&[AuthType::CMD]);               // Grant permission of authypes with warning enabled
///
///
///         // Debugging options
///         #[cfg(feature = "debug")]
///         {
///             processor = processor
///                 .diff(DiffOption::All)?                      // Print diff in final result
///                 .debug(true)                                 // Turn on debug mode
///                 .interactive(true)                           // Use interactive mode
///                 .log(true);                                  // Log all macro invocations
///         }
///
///     // Comment char and macro char cannot be same
///     // Unallowed pattern for the characters are [a-zA-Z1-9\\_\*\^\|\(\)=,]
///
///     // Use Processor::empty() instead of Processor::new()
///     // if you don't want any default macros
///
///     // Print information about current processor permissions
///     // This is an warning and can be suppressed with silent option
///     processor.print_permission()?;
///
///     // Register a hook macro
///     // Trigger and execution macro should be defined elsewhere
///     #[cfg(feature = "hook")]
///     processor.register_hook(
///         HookType::Macro,            // Macro type
///         "trigger_macro",            // Macro that triggers
///         "hook_div",                 // Macro to be executed
///         1,                          // target count
///         false                       // Resetable
///     )?;
///
///     // Add runtime rules(in order of "name, args, body")
///     processor.add_runtime_rules(&[("test","a_src a_link","$a_src() -> $a_link()")])?;
///
///     // Add custom rules without any arguments
///     processor.add_static_rules(&[("test","TEST"),("lul","kekw")])?;
///
///     // Undefine only macro
///     processor.undefine_macro("name1", MacroType::Any);
///
///     // Process with inputs
///     // This prints to desginated write destinations
///     processor.process_string(r#"$define(test=Test)"#)?;
///     processor.process_stdin()?;
///     processor.process_file(Path::new("from.txt"))?;
///
///     processor.freeze_to_file(Path::new("out.r4f"))?; // Create frozen file
///
///     // Print out result
///     // This will print counts of warning and errors.
///     // It will also print diff between source and processed if diff option was
///     // given as builder pattern.
///     processor.print_result()?;                       
///     Ok(())
/// }
/// ```
pub struct Processor<'processor> {
    map: MacroMap,
    define_parser: DefineParser,
    pub(crate) write_option: WriteOption<'processor>,
    cache_file: Option<File>,
    logger: Logger<'processor>,
    cache: String,
    // -- Features --
    #[cfg(feature = "hook")]
    pub(crate) hook_map: HookMap,
    #[cfg(feature = "debug")]
    debugger: Debugger,
    checker: UnbalancedChecker,
    pub(crate) state: ProcessorState,
    pub(crate) storage: Option<Box<dyn RadStorage>>,
    #[cfg(feature = "cindex")]
    pub(crate) indexer: Indexer,
}

impl<'processor> Default for Processor<'processor> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'processor> Processor<'processor> {
    // ----------
    // Builder pattern methods
    // <BUILDER>
    /// Creates default processor with built-in macros
    ///
    /// You can chain builder methods to further configure processor
    ///
    /// ```rust
    /// use std::path::Path;
    /// let proc = r4d::Processor::new()
    ///     .lenient(true)
    ///     .pipe_truncate(false)
    ///     .unix_new_line(true)
    ///     .write_to_file(Path::new("cache.txt"))
    ///     .expect("Failed to open a file");
    /// ```
    pub fn new() -> Self {
        Self::new_processor(true)
    }

    /// Creates default processor without built-in macros
    ///
    /// You can chain builder methods to further configure processor
    ///
    /// ```rust
    /// use std::path::Path;
    /// let proc = r4d::Processor::empty()
    ///     .lenient(true)
    ///     .pipe_truncate(false)
    ///     .unix_new_line(true)
    ///     .write_to_file(Path::new("cache.txt"))
    ///     .expect("Failed to open a file");
    /// ```
    pub fn empty() -> Self {
        Self::new_processor(false)
    }

    /// Internal function to create Processor struct
    ///
    /// This creates a complete processor that can parse and create output without any extra
    /// informations.
    fn new_processor(use_default: bool) -> Self {
        #[allow(unused_mut)] // Mut is required on feature codes
        let mut state = ProcessorState::new();

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
            cache_file: None,
            define_parser: DefineParser::new(),
            logger,
            state,
            #[cfg(feature = "hook")]
            hook_map: HookMap::new(),
            #[cfg(feature = "debug")]
            debugger: Debugger::new(),
            checker: UnbalancedChecker::new(),
            storage: None,
            #[cfg(feature = "cindex")]
            indexer: Indexer::new(),
        }
    }

    /// Set write option to yield output to the file
    ///
    /// ```rust
    /// use std::path::Path;
    /// let proc = r4d::Processor::empty()
    ///     .write_to_file(Path::new("cache.txt"))
    ///     .expect("Failed to open a file");
    /// ```
    pub fn write_to_file<P: AsRef<Path>>(mut self, target_file: P) -> RadResult<Self> {
        let open_option = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .to_owned();

        if let Ok(option) = WriteOption::file(target_file.as_ref(), open_option) {
            self.write_option = option;
        } else {
            return Err(RadError::InvalidCommandOption(format!(
                "Could not create file \"{}\"",
                target_file.as_ref().display()
            )));
        }
        Ok(self)
    }

    /// Write to variable
    ///
    /// ```rust
    /// let mut acc = String::new();
    /// let proc = r4d::Processor::empty()
    ///     .write_to_variable(&mut acc);
    /// ```
    pub fn write_to_variable(mut self, value: &'processor mut String) -> Self {
        self.write_option = WriteOption::Variable(value);
        self
    }

    /// Yield error to the file
    ///
    /// ```rust
    /// use std::path::Path;
    /// let proc = r4d::Processor::empty()
    ///     .error_to_file(Path::new("err.txt"))
    ///     .expect("Failed to open a file");
    /// ```
    pub fn error_to_file<F: AsRef<Path>>(mut self, target_file: F) -> RadResult<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .to_owned();

        if let Ok(file) = WriteOption::file(target_file.as_ref(), file) {
            self.logger.set_write_option(Some(file));
        } else {
            return Err(RadError::InvalidCommandOption(format!(
                "Could not create file \"{}\"",
                target_file.as_ref().display()
            )));
        }
        Ok(self)
    }

    /// Yield error to the file
    ///
    /// ```rust
    /// let mut acc = String::new();
    /// let proc = r4d::Processor::empty()
    ///     .error_to_variable(&mut acc);
    /// ```
    pub fn error_to_variable(mut self, value: &'processor mut String) -> Self {
        self.logger
            .set_write_option(Some(WriteOption::Variable(value)));
        self
    }

    /// Custom comment character
    ///
    /// Every character that consists of valid macro name cannot be a custom comment character.
    /// Unallowed characters are ```[a-zA-Z1-9\\_\*\^\|\(\)=,]```
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .custom_comment_char('&');
    /// ```
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

    /// Set custom characters
    ///
    /// Every character that consists of valid macro name cannot be a custom macro character.
    /// Unallowed characters are ```[a-zA-Z1-9\\_\*\^\|\(\)=,]```
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .custom_chars('&', '%');
    /// ```
    pub fn custom_chars(mut self, macro_character: char, comment_char: char) -> RadResult<Self> {
        if macro_character == comment_char {
            return Err(RadError::UnallowedChar(
                "Cannot set a same character for macro and comment".to_string(),
            ));
        }
        if UNALLOWED_CHARS.is_match(&macro_character.to_string())
            || UNALLOWED_CHARS.is_match(&comment_char.to_string())
        {
            return Err(RadError::UnallowedChar(format!(
                "\"{}\" is not allowed",
                macro_character
            )));
        } else {
            self.state.macro_char.replace(macro_character);
            self.state.comment_char.replace(comment_char);
        }
        Ok(self)
    }

    /// Custom macro character
    ///
    /// Every character that consists of valid macro name cannot be a custom macro character.
    /// Unallowed characters are ```[a-zA-Z1-9\\_\*\^\|\(\)=,]```
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .custom_macro_char('&');
    /// ```
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
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .unix_new_line(true);
    /// ```
    pub fn unix_new_line(mut self, use_unix_new_line: bool) -> Self {
        if use_unix_new_line {
            self.state.newline = "\n".to_owned();
        }
        self
    }

    /// Set purge option
    ///
    /// Purge mode removed failed macro expression.
    ///
    /// This overrides lenient option
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .purge(true);
    /// ```
    pub fn purge(mut self, purge: bool) -> Self {
        if purge {
            self.state.behaviour = ErrorBehaviour::Purge;
        }
        self
    }

    /// Set lenient
    ///
    /// Lenient mode left macro expression in place.
    ///
    /// This overrides purge option
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .lenient(true);
    /// ```
    pub fn lenient(mut self, lenient: bool) -> Self {
        if lenient {
            self.state.behaviour = ErrorBehaviour::Lenient;
        }
        self
    }

    /// Set hygiene variant
    ///
    /// Hygiene decides the processor's behaviour toward runtime macros
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .hygiene(r4d::Hygiene::Macro);
    /// ```
    pub fn hygiene(mut self, hygiene: Hygiene) -> Self {
        self.state.hygiene = hygiene;
        self
    }

    /// Set pipe truncate option
    ///
    /// By default, ```pipe``` truncates original value and leave empty value in place.
    ///
    /// If pipe_truncate is set to false, it will retain the original value.
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .pipe_truncate(false);
    /// ```
    pub fn pipe_truncate(mut self, truncate: bool) -> Self {
        self.state.pipe_truncate = truncate;
        self
    }

    /// Set comment type
    ///
    /// By default, comment is disabled for better compatibility.
    ///
    /// There are two types of comments other than none
    /// - Start : Treats a line comment only if a comment character was placed in the
    /// first index.
    /// - Any   : Treats any text chunk followed as a comment whenever a comment character is detected.
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .set_comment_type(r4d::CommentType::Start);
    /// ```
    pub fn set_comment_type(mut self, comment_type: CommentType) -> Self {
        self.state.comment_type = comment_type;
        self
    }

    /// Set silent option
    ///
    /// By default, every warning types are enabled>
    ///
    /// There are three types of warnings
    /// - Sanity   : Rrelated to possibly unintended behaviours.
    /// - Security : Related to possibly dangerous behaviour related to a file system.
    /// - Any      : Both of the warnings
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .silent(r4d::WarningType::Sanity);
    /// ```
    pub fn silent(mut self, silent_type: WarningType) -> Self {
        self.logger.suppress_warning(silent_type);
        self
    }

    /// Set assertion mode
    ///
    /// Assert mode will not print the output by default and treat assertion fallable not
    /// panicking.
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .assert(true);
    /// ```
    pub fn assert(mut self, assert: bool) -> Self {
        if assert {
            self.logger.set_assert();
            self.state.behaviour = ErrorBehaviour::Purge; // Default is purge
            self.write_option = WriteOption::Discard;
        }
        self
    }

    /// Add debug options
    ///
    /// This toggles debugger.
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .debug(true);
    /// ```
    #[cfg(feature = "debug")]
    pub fn debug(mut self, debug: bool) -> Self {
        self.debugger.debug = debug;
        self
    }

    /// Add debug log options
    ///
    /// This toggles loggig. When logging is enabled every macro invocation is saved into a log
    /// file.
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .log(true);
    /// ```
    #[cfg(feature = "debug")]
    pub fn log(mut self, log: bool) -> Self {
        self.debugger.log = log;
        self
    }

    /// Add diff option
    ///
    /// This toggles diffing. When diffing is enabled diff_file is created after macro execution.
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .diff(true);
    /// ```
    #[cfg(feature = "debug")]
    pub fn diff(mut self, diff: DiffOption) -> RadResult<Self> {
        self.debugger.enable_diff(diff)?;
        Ok(self)
    }

    /// Add debug interactive options
    ///
    /// This toggles interactive mode. When interactive is set, smooth terminal interaction is
    /// enabled.
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .interactive(true);
    /// ```
    #[cfg(feature = "debug")]
    pub fn interactive(mut self, interactive: bool) -> Self {
        if interactive {
            self.debugger.set_interactive();
        }
        self
    }

    /// Melt rule file
    ///
    /// This always melt file into non-volatile form, which means hygiene doesn't affect melted
    /// macros.
    ///
    /// ```rust
    /// use std::path::Path;
    /// let proc = r4d::Processor::empty()
    ///     .melt_files(&[Path::new("a.r4f"), Path::new("b.r4f")]);
    /// ```
    pub fn melt_files(mut self, paths: &[impl AsRef<Path>]) -> RadResult<Self> {
        let mut rule_file = RuleFile::new(None);
        for p in paths.iter() {
            // File validity is checked by melt methods
            rule_file.melt(p.as_ref())?;
        }
        self.map.runtime.extend_map(rule_file.rules, Hygiene::None);

        Ok(self)
    }
    /// Open authorization of processor
    ///
    /// Some macros require authorization be granted by user. There are several authorization types
    /// as follows.
    /// - ENV  : Get or set environment variable
    /// - CMD  : System command
    /// - FIN  : File read
    /// - FOUT : File write
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .allow(&[r4d::AuthType::CMD]);
    /// ```
    pub fn allow(mut self, auth_types: &[AuthType]) -> Self {
        for auth in auth_types {
            self.state.auth_flags.set_state(auth, AuthState::Open)
        }
        self
    }

    /// Open authorization of processor but yield warning
    ///
    /// Some macros require authorization be granted by user. This grants authorization to
    /// processor but yields warning on every authorized macro invocation.
    ///
    /// There are several authorization types
    /// as follows.
    /// - ENV  : Get or set environment variable
    /// - CMD  : System command
    /// - FIN  : File read
    /// - FOUT : File write
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .allow_with_warning(&[r4d::AuthType::CMD]);
    /// ```
    pub fn allow_with_warning(mut self, auth_types: &[AuthType]) -> Self {
        for auth in auth_types {
            self.state.auth_flags.set_state(auth, AuthState::Warn)
        }
        self
    }

    /// Discard output
    ///
    /// Set write option to discard. Nothing will be printed or redirected.
    ///
    /// ```rust
    /// let proc = r4d::Processor::empty()
    ///     .discard(true);
    /// ```
    pub fn discard(mut self, discard: bool) -> Self {
        if discard {
            self.write_option = WriteOption::Discard;
        }
        self
    }

    /// Build with storage
    ///
    /// Append Storage to a processor. Storage should implment [RadStorage](RadStorage) trait.
    ///
    /// Storage interaction is accessed with update and extract macro.
    ///
    /// ```ignore
    /// let proc = r4d::Processor::empty()
    ///     .storage(Box::new(CustomStorage));
    /// ```
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

    // This is not included in docs.rs documentation becuase... is it necessary?
    // I don't really remember when this method was added at the first time.
    /// Melt rule as literal input source, or say from byte array
    ///
    /// This always melt file into non-volatile form, which means hygiene doesn't affect melted
    /// macros.
    ///
    /// ```rust
    /// let source = b"Some frozen macro definition";
    /// let proc = r4d::Processor::empty();
    /// proc.melt_literal(source);
    /// ```
    pub fn melt_literal(&mut self, literal: &[u8]) -> RadResult<()> {
        let mut rule_file = RuleFile::new(None);
        rule_file.melt_literal(literal)?;
        self.map.runtime.extend_map(rule_file.rules, Hygiene::None);
        Ok(())
    }

    pub fn package_sources<T: AsRef<Path>>(
        &self,
        sources: &[T],
        out_file: Option<T>,
    ) -> RadResult<()> {
        let mut body = Vec::new();
        for file in sources {
            body.extend(std::fs::read(file)?);
        }

        let mut static_script = StaticScript::new(self, body)?;
        let path = if let Some(file) = out_file {
            file.as_ref().to_owned()
        } else {
            PathBuf::from("out.r4c")
        };
        static_script.package(Some(&path))?;
        Ok(())
    }

    /// Import(melt) a frozen file
    ///
    /// This always melt file into non-volatile form
    ///
    /// ```rust
    /// use std::path::Path;
    /// let mut proc = r4d::Processor::empty();
    /// proc.import_frozen_file(Path::new("file.r4f")).expect("Failed to import a frozen file");
    /// ```
    pub fn import_frozen_file(&mut self, path: &Path) -> RadResult<()> {
        let mut rule_file = RuleFile::new(None);
        rule_file.melt(path)?;
        self.map.runtime.extend_map(rule_file.rules, Hygiene::None);

        Ok(())
    }

    /// Set queue object
    ///
    /// This is intended for macro logics not processor logics.
    ///
    /// Queued objects are not executed immediately but executed only after currently aggregated
    /// macro fragments are fully expanded.
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::empty();
    /// proc.insert_queue("$halt()");
    /// ```
    pub fn insert_queue(&mut self, item: &str) {
        self.state.queued.push(item.to_owned());
    }

    /// Set hygiene type
    ///
    /// Set hygiene type and also clears volatile runtime macro.
    ///
    /// Queued objects are not executed immediately but executed only after currently aggregated
    /// macro fragments are fully expanded.
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::empty();
    /// proc.set_hygiene(r4d::Hygiene::Macro);
    /// ```
    pub fn set_hygiene(&mut self, hygiene: Hygiene) {
        if !self.map.runtime.volatile.is_empty() {
            self.map.clear_runtime_macros(true);
        }
        self.state.hygiene = hygiene;
    }

    /// Set to dry run mode
    pub fn set_dry_mode(&mut self) {
        self.write_option = WriteOption::Discard;
        self.state.process_type = ProcessType::Dry;
        self.state.auth_flags.clear();
    }

    /// Set to freeze mode
    pub fn set_freeze_mode(&mut self) {
        self.write_option = WriteOption::Discard;
        self.state.process_type = ProcessType::Freeze;
        self.state.auth_flags.clear();
    }

    /// Clear volatile macros
    ///
    /// This removes runtime macros which are not melted from.
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::empty();
    /// proc.clear_volatile();
    /// ```
    pub fn clear_volatile(&mut self) {
        if !self.map.runtime.volatile.is_empty() {
            self.map.clear_runtime_macros(true);
        }
    }

    /// Toggle macro hygiene
    pub(crate) fn toggle_hygiene(&mut self, toggle: bool) {
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
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::empty();
    /// proc.set_write_option(r4d::WriteOption::Discard);
    /// ```
    pub fn set_write_option(&mut self, write_option: WriteOption<'processor>) {
        self.write_option = write_option;
    }

    /// Reset flow control
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::empty();
    /// proc.reset_flow_control();
    /// ```
    pub fn reset_flow_control(&mut self) {
        self.state.flow_control = FlowControl::None;
    }

    /// Get macro signatrue map
    #[cfg(feature = "signature")]
    pub(crate) fn get_signature_map(&self, sig_type: SignatureType) -> RadResult<SignatureMap> {
        let signatures = match sig_type {
            SignatureType::All => self.map.get_signatures(),
            SignatureType::Default => self.map.get_default_signatures(),
            SignatureType::Runtime => self.map.get_runtime_signatures(),
        };
        Ok(SignatureMap::new(signatures))
    }

    /// Print current permission status
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::empty();
    /// proc.print_permission().expect("Failed to print permission data");
    /// ```
    #[allow(dead_code)]
    pub fn print_permission(&mut self) -> RadResult<()> {
        if let Some(status) = self.state.auth_flags.get_status_string() {
            let mut status_with_header = String::from("Permission granted");
            status_with_header.push_str(&status);
            self.log_warning_no_line(&status_with_header, WarningType::Security)?;
        }
        Ok(())
    }

    /// Print the result of a processing
    ///
    /// This will also print diff file if debug and diff feature is enabled.
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::empty();
    /// proc.print_result().expect("Failed to print result");
    /// ```
    pub fn print_result(&mut self) -> RadResult<()> {
        self.logger.print_result()?;

        #[cfg(feature = "debug")]
        self.debugger.yield_diff(&mut self.logger)?;

        Ok(())
    }

    /// Clear cached and organze multiple jobs
    ///
    /// * clear volatile macors
    /// * Check if there is any unterminated job
    /// Return cached string, if cache was not empty
    pub(crate) fn organize_and_clear_cache(&mut self) -> RadResult<Option<String>> {
        if self.state.hygiene == Hygiene::Input {
            self.map.clear_runtime_macros(true);
        }

        if self.state.input_stack.len() == 1 {
            // Warn unterminated relaying
            if !self.state.relay.is_empty() {
                let relay = format!("{:?}", self.state.relay.last().unwrap());
                self.log_warning(&format!("There is unterminated relay target : \"{}\" which might not be an intended behaviour.", relay), WarningType::Sanity)?;
            }
            // Warn flow control
            match self.state.flow_control {
                FlowControl::None => (),
                FlowControl::Exit => self
                    .logger
                    .wlog_no_line("Process exited.", WarningType::Sanity)?,
                FlowControl::Escape => self
                    .logger
                    .wlog_no_line("Process escaped.", WarningType::Sanity)?,
            }
        }

        // Clear input stack
        // This is necessary because operation can be contiguous
        self.state.input_stack.clear();
        self.logger.stop_last_tracker();

        if self.cache.is_empty() {
            Ok(None)
        } else {
            Ok(Some(std::mem::take(&mut self.cache)))
        }
    }

    /// Set storage
    ///
    /// Storage should implment [RadStorage](RadStorage) trait.
    ///
    /// ```ignore
    /// let mut proc = r4d::Processor::empty();
    /// proc.set_storage(Box::new(some_storage_struct));
    /// ```
    pub fn set_storage(&mut self, storage: Box<dyn RadStorage>) {
        self.storage.replace(storage);
    }

    /// Extract from storage
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::empty();
    /// let serialize = false;
    /// proc.extract_storage(serialize).expect("Failed to extract storage information");
    /// ```
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
    ///
    /// ```rust
    /// use std::path::Path;
    /// let mut proc = r4d::Processor::empty();
    /// proc.freeze_to_file(Path::new("file.r4f")).expect("Failed to freeze to a file");
    /// ```
    pub fn freeze_to_file(&mut self, path: impl AsRef<Path>) -> RadResult<()> {
        // File path validity is checked by freeze method
        RuleFile::new(Some(self.map.runtime.macros.clone())).freeze(path.as_ref())?;
        Ok(())
    }

    /// Serialize rule files into a bincode
    pub fn serialize_rules(&self) -> RadResult<Vec<u8>> {
        // File path validity is checked by freeze method
        RuleFile::new(Some(self.map.runtime.macros.clone())).serialize()
    }

    /// Add a new macro as an extension
    ///
    /// Register a function as macro. To make a register process eaiser, use template feature if
    /// possible.
    ///
    /// Refer [doc](https://github.com/Simhyeon/r4d/blob/master/docs/ext.md) for detailed usage.
    ///
    /// ```rust
    /// let mut processor = r4d::Processor::empty();
    /// #[cfg(feature = "template")]
    /// processor.add_ext_macro(r4d::ExtMacroBuilder::new("macro_name")
    ///     .args(&["a1","b2"])
    ///     .function(r4d::function_template!(
    ///         let args = r4d::split_args!(2)?;
    ///         let result = format!("{} + {}", args[0], args[1]);
    ///         Ok(Some(result))
    /// )));
    /// ```
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
    /// let mut processor = r4d::Processor::empty();
    /// processor.add_runtime_rules(&[("macro_name","macro_arg1 macro_arg2","macro_body=$macro_arg1()")]);
    /// ```
    pub fn add_runtime_rules(&mut self, rules: &[(impl AsRef<str>, &str, &str)]) -> RadResult<()> {
        if self.state.hygiene == Hygiene::Aseptic {
            let err = RadError::UnallowedMacroExecution(format!(
                "Cannot register macros : \"{:?}\" in aseptic mode",
                rules.iter().map(|(s, _, _)| s.as_ref()).collect::<Vec<_>>()
            ));
            if self.state.behaviour == ErrorBehaviour::Strict {
                self.log_error(&err.to_string())?;
                return Err(RadError::StrictPanic);
            } else {
                self.log_warning(&err.to_string(), WarningType::Security)?;
            }
        }
        for (name, args, body) in rules {
            let name = name.as_ref().trim();
            if !MAC_NAME.is_match(name) {
                let err = RadError::InvalidMacroName(format!(
                    "Name : \"{}\" is not a valid macro name",
                    name
                ));
                return Err(err);
            }
            self.map.runtime.macros.insert(
                name.to_owned(),
                RuntimeMacro {
                    name: name.to_owned(),
                    args: args
                        .split_whitespace()
                        .map(|s| s.to_owned())
                        .collect::<Vec<String>>(),
                    body: body.to_string(),
                    desc: None,
                    is_static: false,
                },
            );
        }
        Ok(())
    }

    /// Add static rules without builder pattern
    ///
    /// **NOTE** that this method doesn't expand body, but needs to be handled before invoking this method
    ///
    /// # Args
    ///
    /// The order of argument is "name, body"
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut processor = r4d::Processor::new();
    /// processor.add_static_rules(&[("macro_name","Macro body without arguments")]);
    /// ```
    pub fn add_static_rules(
        &mut self,
        rules: &[(impl AsRef<str>, impl AsRef<str>)],
    ) -> RadResult<()> {
        if self.state.hygiene == Hygiene::Aseptic {
            let err = RadError::UnallowedMacroExecution(format!(
                "Cannot register macros : \"{:?}\" in aseptic mode",
                rules.iter().map(|(s, _)| s.as_ref()).collect::<Vec<_>>()
            ));
            if self.state.behaviour == ErrorBehaviour::Strict {
                self.log_error(&err.to_string())?;
                return Err(err);
            } else {
                self.log_warning(&err.to_string(), WarningType::Sanity)?;
            }
        }
        for (name, body) in rules {
            let name = name.as_ref().trim();
            if !MAC_NAME.is_match(name) {
                let err = RadError::InvalidMacroName(format!(
                    "Name : \"{}\" is not a valid macro name",
                    name
                ));
                return Err(err);
            }
            self.map.runtime.macros.insert(
                name.to_owned(),
                RuntimeMacro {
                    name: name.to_owned(),
                    args: vec![],
                    body: body.as_ref().to_owned(),
                    desc: None,
                    is_static: true,
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
        if target_macro.is_empty() {
            let err = RadError::InvalidMacroName(format!(
                "Cannot register hook for macro \"{}\"",
                target_macro
            ));
            return Err(err);
        }

        // Check invoke macro is empty
        if invoke_macro.is_empty() {
            let err = RadError::InvalidMacroName(format!(
                "Cannot register hook which invokes a macro \"{}\"",
                target_macro
            ));
            return Err(err);
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
    /// Deregister a hook
    pub fn deregister_hook(&mut self, hook_type: HookType, target_macro: &str) -> RadResult<()> {
        // Check target macro is empty
        if target_macro.is_empty() {
            let err = RadError::InvalidMacroName(format!(
                "Cannot deregister hook for macro \"{}\"",
                target_macro
            ));
            return Err(err);
        }

        self.hook_map.del_hook(hook_type, target_macro)?;
        Ok(())
    }

    /// Read from string
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::empty();
    /// proc.process_string("$define(new=NEW)")
    ///     .expect("Failed to process a string");
    /// ```
    pub fn process_string(&mut self, content: &str) -> RadResult<Option<String>> {
        self.logger
            .start_new_tracker(TrackType::Input("String".to_string()));
        let mut reader = content.as_bytes();
        self.process_buffer(&mut reader, None, false)?;

        // This method stops last tracker, thus explicit one is not needed
        self.organize_and_clear_cache()
    }

    /// Read from standard input
    ///
    /// If debug mode is enabled this, doesn't read stdin line by line but by chunk because user
    /// input is also a standard input and processor cannot distinguish the two
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::empty();
    /// proc.process_stdin()
    ///     .expect("Failed to process a standard input");
    /// ```
    pub fn process_stdin(&mut self) -> RadResult<Option<String>> {
        #[allow(unused_imports)]
        use std::io::Read;
        let stdin = io::stdin();

        self.set_input_stdin()?;

        // Early return if debug
        // This read whole chunk of string
        #[cfg(feature = "debug")]
        if self.is_debug() {
            let mut input = String::new();
            stdin.lock().read_to_string(&mut input)?;
            // This is necessary to prevent unexpected output from being captured.
            self.process_buffer(&mut input.as_bytes(), None, false)?;
            return self.organize_and_clear_cache();
        }

        let mut reader = stdin.lock();
        self.process_buffer(&mut reader, None, false)?;
        self.organize_and_clear_cache()
    }

    /// Process contents from a file
    ///
    /// ```no_run
    /// use std::path::Path;
    /// let mut proc = r4d::Processor::empty();
    /// proc.process_file(Path::new("source.txt"))
    ///     .expect("Failed to process a file");
    /// ```
    pub fn process_file(&mut self, path: impl AsRef<Path>) -> RadResult<Option<String>> {
        if path.as_ref().is_dir() {
            return Err(RadError::InvalidFile(format!(
                "File \"{}\" is not a readable file",
                path.as_ref().display()
            )));
        }

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
        self.process_buffer(&mut reader, backup, false)?;
        self.organize_and_clear_cache()
    }

    /// Process contents from a static script
    ///
    /// ```no_run
    /// use std::path::Path;
    /// let mut proc = r4d::Processor::empty();
    /// proc.process_static_script(Path::new("source.r4c"))
    ///     .expect("Failed to process a script");
    /// ```
    pub fn process_static_script(&mut self, path: impl AsRef<Path>) -> RadResult<Option<String>> {
        if path.as_ref().is_dir() {
            return Err(RadError::InvalidFile(format!(
                "File \"{}\" is not a readable file",
                path.as_ref().display()
            )));
        }

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
        let mut source = vec![];
        reader.read_to_end(&mut source)?;
        let static_script = StaticScript::unpack(source)?;
        let mut reader = BufReader::new(&static_script.body[..]);
        self.melt_literal(&static_script.header[..])?;
        self.process_buffer(&mut reader, backup, false)?;
        self.organize_and_clear_cache()
    }

    /// Internal method that is executed by macro
    ///
    /// Target usages
    /// - include
    /// - temp_include
    ///
    pub(crate) fn process_file_as_chunk(
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
        self.process_buffer(&mut reader, backup, true)
    }

    /// Internal method for processing buffers line by line
    fn process_buffer(
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

        // when processing has to return a value rather than modify in-place
        let container = String::new();
        let mut cont = if use_container { Some(container) } else { None };

        #[cfg(feature = "debug")]
        self.debugger.user_input_on_start(
            &self.state.current_input.to_string(),
            self.logger.get_last_line(),
        )?;

        loop {
            #[cfg(feature = "debug")]
            if let Some(line) = line_iter.peek() {
                let line = line.as_ref().unwrap();
                // Update line cache
                self.debugger.add_line_cache(line);
                // Only if debug switch is nextline
                self.debugger
                    .user_input_on_line(&frag, self.logger.get_last_line())?;
            }

            let result = match self.process_line(&mut line_iter, &mut lexor, &mut frag) {
                Ok(oo) => oo,
                Err(err) => {
                    return Err(err);
                }
            };
            match result {
                // This means either macro is not found at all
                // or previous macro fragment failed with invalid syntax
                ParseResult::Printable(remainder) => {
                    self.write_to(&remainder, &mut cont)?;

                    // Test if this works
                    #[cfg(feature = "debug")]
                    self.debugger.clear_line_cache();

                    // Reset fragment
                    if !frag.whole_string.is_empty() {
                        frag = MacroFragment::new();
                    }
                }
                ParseResult::FoundMacro(remainder) => {
                    self.write_to(&remainder, &mut cont)?;
                }
                // This happens only when given macro involved text should not be printed
                ParseResult::NoPrint => {}
                // End of input, end loop
                ParseResult::Eoi => {
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
        } // Loop end

        // Recover previous state from sandboxed processing
        if let Some(backup) = backup {
            self.recover(backup)?;
            self.state.sandbox = false;
        }

        if use_container {
            Ok(cont.filter(|t| !t.is_empty()))
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
    fn is_local_macro(&self, mut level: usize, name: &str) -> bool {
        while level > 0 {
            if self.map.local.contains_key(&Utils::local_name(level, name)) {
                return true;
            }
            level -= 1;
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
            self.debugger
                .user_input_on_macro(frag, self.logger.get_last_line())?;
        } else {
            self.debugger
                .user_input_on_step(frag, self.logger.get_last_line())?;
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
    fn process_line(
        &mut self,
        lines: &mut impl std::iter::Iterator<Item = std::io::Result<String>>,
        lexor: &mut Lexor,
        frag: &mut MacroFragment,
    ) -> RadResult<ParseResult> {
        if let Some(line) = lines.next() {
            self.logger.inc_line_number();
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
                FlowControl::Exit => {
                    return Ok(ParseResult::Eoi);
                }
                FlowControl::None => (),
            }

            // Save to original
            #[cfg(feature = "debug")]
            self.debugger.write_diff_original(&line)?;

            let remainder = self.parse_line(lexor, frag, &line, 0, MAIN_CALLER)?;

            // Clear local variable macros
            self.map.clear_local();
            // Reset error cache
            self.state.error_cache.take();

            // Clear volatile variables when macro hygiene is enabled
            if self.state.hygiene == Hygiene::Macro {
                self.map.clear_runtime_macros(true);
            }

            // Non macro string is included
            if !remainder.is_empty() {
                // Fragment is not empty
                if !frag.is_empty() {
                    // Consume new line shold not work on fragment extension
                    if self.state.consume_newline {
                        self.state.consume_newline = false;
                    }
                    Ok(ParseResult::FoundMacro(remainder))
                } else {
                    Ok(ParseResult::Printable(remainder))
                }
            }
            // Nothing to print
            else {
                Ok(ParseResult::NoPrint)
            }
        } else {
            Ok(ParseResult::Eoi)
        }
    } // parse_line end

    fn parse_chunk_body(&mut self, level: usize, caller: &str, chunk: &str) -> RadResult<String> {
        let mut lexor = Lexor::new(
            self.get_macro_char(),
            self.get_comment_char(),
            &self.state.comment_type,
        );
        let mut frag = MacroFragment::new();
        let mut result = String::new();
        self.logger
            .start_new_tracker(TrackType::Body(caller.to_string()));

        for line in Utils::full_lines(chunk.as_bytes()) {
            let line = line?;

            // Deny newline
            if self.state.deny_newline {
                self.state.deny_newline = false;
                if line == "\n" || line == "\r\n" {
                    continue;
                }
            }

            let line_result = self.parse_line(&mut lexor, &mut frag, &line, level, caller)?;
            // Increase line number
            self.logger.inc_line_number();

            result.push_str(&line_result);
        }

        // Frag has not been cleared which means unterminated string has been not picked up yet.
        if !frag.is_empty() {
            result.push_str(&frag.whole_string);
        }

        self.logger.stop_last_tracker();
        Ok(result)
    } // parse_chunk end

    /// Parse chunk args by separating it into lines which implements BufRead
    pub(crate) fn parse_chunk_args(
        &mut self,
        level: usize,
        caller: &str,
        chunk: &str,
    ) -> RadResult<String> {
        let mut lexor = Lexor::new(
            self.get_macro_char(),
            self.get_comment_char(),
            &self.state.comment_type,
        );
        // Set inner parsing logic
        lexor.set_inner();
        let mut frag = MacroFragment::new();
        let mut result = String::new();
        self.logger
            .start_new_tracker(TrackType::Argument(caller.to_owned()));
        for line in Utils::full_lines(chunk.as_bytes()) {
            let line = line?;

            // Deny newline
            if self.state.deny_newline {
                self.state.deny_newline = false;
                if line == "\n" || line == "\r\n" {
                    continue;
                }
            }

            // NOTE
            // Parse's final argument is some kind of legacy of previous logics
            // However it can detect self calling macros in some cases
            // parse_chunk_body needs this caller but, parse_chunk_args doesn't need because
            // this methods only parses arguments thus, infinite loop is unlikely to happen
            let line_result = self.parse_line(&mut lexor, &mut frag, &line, level, caller)?;
            result.push_str(&line_result);

            self.logger.inc_line_number();
        }

        // If unexpanded texts remains
        // Add to result
        if !frag.whole_string.is_empty() {
            result.push_str(&frag.whole_string);
        }

        self.logger.stop_last_tracker();
        Ok(result)
    } // parse_chunk_lines end

    /// Parse a given line
    ///
    /// This calles lexor.lex to validate characters and decides next behaviour
    fn parse_line(
        &mut self,
        lexor: &mut Lexor,
        frag: &mut MacroFragment,
        line: &str,
        level: usize,
        caller: &str,
    ) -> RadResult<String> {
        // Initiate values
        // Reset character number
        // self.logger.reset_char_number();
        // Local values
        let mut remainder = String::new();

        // Check comment line
        // If it is a comment then return nothing and write nothing
        if self.state.comment_type != CommentType::None
            && line.trim().starts_with(self.get_comment_char())
        {
            return Ok(String::new());
        }

        let mut ch_iter = line.chars().peekable();

        while let Some(ch) = ch_iter.next() {
            // Escape new charater is not respected
            if self.state.escape_newline {
                if ch == '\r' && ch_iter.peek().unwrap_or(&'0') == &'\n' {
                    continue;
                } else if ch == '\n' {
                    self.state.escape_newline = false;
                    continue;
                }
            }
            self.logger.inc_char_number();

            let lex_result = lexor.lex(ch);
            // Either add character to remainder or fragments
            match lex_result {
                LexResult::CommentExit => {
                    self.lex_branch_comment_exit(frag, &mut remainder);
                    return Ok(remainder);
                }
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
                    frag.is_processed = true;
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

                    // Escape blank check is ok to be in here because
                    // escape blanks can only invoked by macro ( for now at least )
                    if self.state.lexor_escape_blanks {
                        lexor.consume_blank();
                        self.state.lexor_escape_blanks = false;
                    }
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

                    // Char marcro execute
                    self.lex_branch_end_invoke(
                        &mut hook_lexor,
                        &mut hook_frag,
                        &mut hook_mainder,
                        level,
                        &frag.name,
                    )?;

                    // If there is content to add
                    if !hook_mainder.is_empty() {
                        remainder.push_str(&hook_mainder);
                    }
                } // End if let of macro name
            }

            if frag.is_processed {
                self.logger.merge_track();
                frag.is_processed = false;
            }
        }

        // Don't print if current was empty and consume_newline was set( No print was called )
        if self.state.consume_newline && remainder.trim().is_empty() {
            remainder.clear();
            self.state.consume_newline = false;
        } else if self.state.escape_newline {
            if remainder.ends_with("\r\n") {
                remainder = remainder.strip_suffix("\r\n").unwrap().to_string();
            } else if remainder.ends_with('\n') {
                remainder = remainder.strip_suffix('\n').unwrap().to_string();
            }
            self.state.escape_newline = false;
        }

        // Consume newline should be negated by the end of parse
        if self.state.consume_newline {
            self.state.consume_newline = false;
        }
        Ok(remainder)
    }

    // Evaluate can be nested deeply
    // Disable caller for temporary
    /// Evaluate detected macro usage
    ///
    /// Evaluation order is followed
    /// - Local bound macro
    /// - Runtime macro
    /// - Deterred macro
    /// - Function macro
    fn evaluate(
        &mut self,
        level: usize,
        caller: &str,
        frag: &mut MacroFragment,
    ) -> RadResult<Option<String>> {
        // Increase level to represent nestedness
        let level = level + 1;
        let (name, mut raw_args) = (&frag.name, frag.args.clone());

        if frag.trim_input {
            raw_args = raw_args
                .lines()
                .map(|l| l.trim())
                .collect::<Vec<_>>()
                .join(&self.state.newline);
        }
        let args: String;
        // Preprocess only when macro is not a deterred macro
        if !self.map.is_deterred_macro(name) || self.state.process_type == ProcessType::Dry {
            args = self.parse_chunk_args(level, name, &raw_args)?;
            // This parses and processes arguments
            // and macro should be evaluated after

            // Also update original arguments for better debugging
            #[cfg(feature = "debug")]
            {
                frag.processed_args = args.clone();
            }
        } else {
            // Is deterred macro

            // Deterred macro is not allowed in freeze mode
            if self.state.process_type == ProcessType::Freeze {
                self.log_warning(
                    "Deterred macro is not expanded in freeze mode.",
                    WarningType::Sanity,
                )?;
                frag.clear();
                return Ok(None);
            }

            // Set raw args as literal
            args = raw_args;

            // Set processed_args some helpful message
            #[cfg(feature = "debug")]
            {
                frag.processed_args =
                    String::from("It is impossible to retrieve args from deterred macro")
            }
        }

        // Find local macro
        // The macro can be  the one defined in parent macro
        let mut temp_level = level;
        while temp_level > 0 {
            if let Some(local) = self.map.local.get(&Utils::local_name(temp_level, name)) {
                // IMPORTANT
                // Local body should not be expanded
                return Ok(Some(local.body.to_owned()));
            }
            temp_level -= 1;
        }

        // Find runtime macro
        // runtime macro comes before function macro so that
        // user can override it
        if self.map.runtime.contains(name, self.state.hygiene) {
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

            // Prevent invocation if relaying to
            if let Some(RelayTarget::Macro(mac)) = &self.state.relay.last() {
                if mac == name {
                    let err = RadError::UnallowedMacroExecution(format!(
                        "Cannot execute a macro \"{}\" when it is being relayed to",
                        mac
                    ));
                    // You have clear whole string because whole string is append to final output
                    // when leneient mode
                    frag.whole_string.clear();
                    return Err(err);
                }
            }

            let result = self.invoke_rule(level, name, &args)?;
            return Ok(result);
        }
        // Find deterred macro
        else if self.map.is_deterred_macro(name) {
            if let Some(func) = self.map.deterred.get_deterred_macro(name) {
                // On dry run, macro is not expanded but only checks if found macro exists
                if self.state.process_type == ProcessType::Dry {
                    return Ok(None);
                }

                let final_result = func(&args, level, self)?;
                // TODO
                // Make parse logic consistent, not defined by delarator
                // result = self.parse_chunk_args(level, caller, &result)?;
                return Ok(final_result);
            }
        }

        // Find function macro
        if self.map.function.contains(name) {
            // On dry run, macro is not expanded but only checks if found macro exists
            if self.state.process_type == ProcessType::Dry {
                return Ok(None);
            }

            // Func always exists, because contains succeeded.
            let func = self.map.function.get_func(name).unwrap();
            //let final_result = func(&args, self)?;
            let final_result = match func(&args, self) {
                Ok(e) => e,
                Err(err) => {
                    return Err(err);
                }
            };
            Ok(final_result)
        }
        // No macros found to evaluate
        else {
            let err = RadError::InvalidMacroName(format!("No such macro name : \"{}\"", &name));

            // On Dry mode, invalid macro name is not an error but a warning.
            // Because macros are not expanded, it is unsure if it is an error or not, thus rad
            // simply prints warning
            if self.state.process_type == ProcessType::Dry {
                self.log_warning(&err.to_string(), WarningType::Sanity)?;
                Ok(None)
            } else {
                Err(err)
            }
        }
    }

    /// Invoke a runtime rule and get a result
    ///
    /// Invoke rule evaluates body of macro rule because the body is not evaluated on register process
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

        // If static macro, return body without expansion
        if rule.is_static {
            return Ok(Some(rule.body));
        }

        let arg_types = &rule.args;
        // Set variable to local macros
        let args =
            if let Some(content) = ArgParser::new().args_with_len(arg_values, arg_types.len()) {
                content
            } else {
                // Necessary arg count is bigger than given arguments
                let err = RadError::InvalidArgument(format!(
                    "{}'s arguments are not sufficient. Given {}, but needs {}",
                    name,
                    ArgParser::new()
                        .args_to_vec(arg_values, ',', GreedyState::Never)
                        .len(),
                    arg_types.len()
                ));
                if self.state.process_type == ProcessType::Dry {
                    self.log_warning(&err.to_string(), WarningType::Sanity)?;
                    return Ok(None);
                } else {
                    return Err(err);
                }
            };

        for (idx, arg_type) in arg_types.iter().enumerate() {
            //Set arg to be substitued
            self.map.add_local_macro(level + 1, arg_type, &args[idx]);
        }

        // Process the rule body
        // NOTE
        // Previously, this was parse_chunk_body
        let result = self.parse_chunk_body(level, name, &rule.body)?;

        // Clear lower locals to prevent local collisions
        self.map.clear_lower_locals(level);

        Ok(Some(result))
    }

    /// Add runtime rule to macro map
    ///
    /// This doesn't clear fragment
    fn add_rule(&mut self, frag: &MacroFragment) -> RadResult<()> {
        if let Some((name, args, mut body)) = self.define_parser.parse_define(&frag.args) {
            if name.is_empty() {
                let err = RadError::InvalidMacroName("Cannot define a empty macro".to_string());
                return Err(err);
            }
            // Strict mode
            // Overriding is prohibited
            if self.state.behaviour == ErrorBehaviour::Strict
                && self
                    .map
                    .contains_macro(&name, MacroType::Any, self.state.hygiene)
            {
                let mac_name = if frag.args.contains(',') {
                    frag.args.split(',').collect::<Vec<&str>>()[0]
                } else {
                    frag.args.split('=').collect::<Vec<&str>>()[0]
                };
                let err = RadError::UnallowedMacroExecution(format!(
                    "Can't override exsiting macro : \"{}\"",
                    mac_name
                ));
                self.log_error(&err.to_string())?;
                return Err(RadError::StrictPanic);
            }

            if frag.trim_input {
                body = trim!(&body
                    .lines()
                    .map(|l| trim!(l))
                    .collect::<Vec<_>>()
                    .join(&self.state.newline))
                .to_string();
            }

            self.map
                .register_runtime(&name, &args, &body, self.state.hygiene)?;

            // Dry run if such mode is set
            if self.state.process_type == ProcessType::Dry {
                let err = RadError::InvalidArgument(format!("Macro \"{}\" has invalid body", name));
                let res = self
                    .process_string(&format!("${}({})", name, args.replace(' ', ",")))
                    .map_err(|_| &err);

                if res.is_err() {
                    self.log_warning(&err.to_string(), WarningType::Sanity)?;
                }
            }
        } else {
            let name = if frag.args.contains(',') {
                frag.args.split(',').collect::<Vec<&str>>()[0]
            } else {
                frag.args.split('=').collect::<Vec<&str>>()[0]
            };
            let err = RadError::InvalidMacroName(format!(
                "Invalid macro definition format for a macro : \"{}\"",
                name
            ));
            return Err(err);
        }
        Ok(())
    }

    /// Write text to either file or standard output according to processor's write option
    fn write_to(&mut self, content: &str, container: &mut Option<String>) -> RadResult<()> {
        // Don't try to write empty string, because it's a waste
        if content.is_empty() {
            return Ok(());
        }

        // Save to container
        if let Some(cont) = container.as_mut() {
            cont.push_str(content);
            return Ok(());
        }

        // Redirect to cache if set
        if let Some(cache) = &mut self.cache_file {
            cache.write_all(content.as_bytes())?;
            return Ok(());
        }

        // Save to "source" file for debuggin
        #[cfg(feature = "debug")]
        self.debugger.write_diff_processed(content)?;

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
                    let err = RadError::InvalidMacroName(format!(
                        "Cannot relay to non-exsitent macro \"{}\"",
                        mac
                    ));
                    return Err(err);
                }
                self.map.append(mac, content, self.state.hygiene);
            }
            RelayTarget::File(target) => {
                // NOTE
                // Pracically this cannot be set in wasm target because multiple code barriers
                // yet panic can theoreoically happen.
                target.inner().write_all(content.as_bytes())?;
            }
            #[cfg(not(feature = "wasm"))]
            RelayTarget::Temp => {
                if let Some(file) = self.get_temp_file() {
                    file.write_all(content.as_bytes())?;
                }
            }
            RelayTarget::None => {
                match &mut self.write_option {
                    WriteOption::File(f) => f.inner().write_all(content.as_bytes())?,
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
        frag.is_processed = true;
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
                frag.is_processed = true;
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
            .user_input_before_macro(frag, self.logger.get_last_line())?;

        frag.whole_string.push(ch);

        // If paused and not pause, then reset lexor context
        if self.state.paused && frag.name != "pause" {
            lexor.reset();
            remainder.push_str(&frag.whole_string);
            frag.clear();
            frag.is_processed = true;
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
        self.logger.append_track(String::from("empty name"));
        // If paused, then reset lexor context to remove cost
        if self.state.paused {
            lexor.reset();
            remainder.push_str(&frag.whole_string);
            frag.clear();
            frag.is_processed = true;
        }
    }

    fn lex_branch_add_to_remainder(&mut self, ch: char, remainder: &mut String) -> RadResult<()> {
        if !self.checker.check(ch) && !self.state.paused {
            self.logger
                .append_track(String::from("Unbalanced parenthesis"));
            self.log_warning("Unbalanced parenthesis detected.", WarningType::Sanity)?;
            self.logger.merge_track();
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
                if frag.name.is_empty() {
                    self.logger.append_track(String::from("Macro start"));
                }
                match ch {
                    '|' => frag.pipe = true,
                    '*' => frag.yield_literal = true,
                    '=' => frag.trim_input = true,
                    '^' => frag.trimmed = true,
                    _ => {
                        // This is mostly not reached because it is captured as non-exsitent name
                        if frag.has_attribute() {
                            let err = RadError::InvalidMacroName(format!(
                                "Invalid macro attribute : \"{}\"",
                                ch
                            ));
                            return Err(err);
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
                frag.clear();
                frag.is_processed = true;
                self.state.consume_newline = true;
                let err = RadError::UnallowedMacroExecution(format!(
                    "Cannot register a macro : \"{}\" in aseptic mode",
                    frag.name
                ));
                if self.state.behaviour == ErrorBehaviour::Strict {
                    self.log_error(&err.to_string())?;
                    return Err(RadError::StrictPanic);
                } else {
                    self.log_warning(&err.to_string(), WarningType::Security)?;
                }
            } else {
                if level != 0 && self.state.process_type == ProcessType::Freeze {
                    self.log_warning(
                        "Only first level define is allowed in freeze mode",
                        WarningType::Sanity,
                    )?;
                    frag.clear();
                    remainder.clear();
                    return Ok(());
                }
                self.lex_branch_end_frag_define(
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
        frag: &mut MacroFragment,
        remainder: &mut String,
        #[cfg(feature = "debug")] level: usize,
    ) -> RadResult<()> {
        if let Err(err) = self.add_rule(frag) {
            self.log_error(&err.to_string())?;
            match self.state.behaviour {
                ErrorBehaviour::Interrupt => return Err(err),
                ErrorBehaviour::Assert => return Err(RadError::AssertFail),
                // Re-throw error
                // It is not captured in cli but it can be handled by library user.
                ErrorBehaviour::Strict => {
                    return Err(RadError::StrictPanic);
                }
                // If purge mode is set, don't print anything
                // and don't print error
                ErrorBehaviour::Purge => (),
                ErrorBehaviour::Lenient => remainder.push_str(&frag.whole_string),
            }
        }
        // Set states
        self.state.consume_newline = true;
        #[cfg(feature = "debug")]
        self.check_debug_macro(frag, level)?;

        frag.clear();
        frag.is_processed = true;
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
        if frag.name.is_empty() {
            let err =
                RadError::InvalidMacroName("Cannot invoke a macro with empty name".to_string());
            self.log_error(&err.to_string())?;

            // Handle empty name error
            match self.state.behaviour {
                ErrorBehaviour::Assert => return Err(RadError::AssertFail),
                ErrorBehaviour::Strict | ErrorBehaviour::Interrupt => {
                    return Err(RadError::StrictPanic);
                } // Error
                ErrorBehaviour::Lenient => remainder.push_str(&frag.whole_string),
                ErrorBehaviour::Purge => (),
            }

            // Clear fragment regardless
            frag.clear();
            frag.is_processed = true;
        }

        // Debug
        #[cfg(feature = "debug")]
        {
            // Print a log information
            self.debugger
                .print_log(&frag.name, &frag.args, frag, &mut self.logger)?;

            // If debug switch target is break point
            // Set switch to next line.
            self.debugger.break_point(frag)?;
            // Break point is true , continue
            if frag.name.is_empty() {
                // Clear fragment regardless of success
                frag.clear();
                frag.is_processed = true;
                self.state.consume_newline = true;
                self.debugger.set_prompt("\"BR\"");
                return Ok(());
            }
        }

        let evaluation_result = self.evaluate(level, caller, frag);

        match evaluation_result {
            // If panicked, this means unrecoverable error occured.
            Err(error) => {
                self.lex_branch_end_frag_eval_result_error(error, frag, remainder)?;
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
        frag.is_processed = true;

        // Execute queues
        // Execute queued object
        let queued = std::mem::take(&mut self.state.queued); // Queue should be emptied after
        for item in queued {
            // This invokes parse method
            let result = self.parse_chunk_args(0, MAIN_CALLER, &item)?;
            remainder.push_str(&result);
        }

        Ok(())
    }

    /// When evaluation failed for various reasons.
    fn lex_branch_end_frag_eval_result_error(
        &mut self,
        error: RadError,
        frag: &MacroFragment,
        remainder: &mut String,
    ) -> RadResult<()> {
        // UnsoundExecution is critical error and should not be permitted
        if let RadError::UnsoundExecution(_) = error {
            return Err(error);
        }

        if self.state.error_cache.is_none() {
            self.log_error(&error.to_string())?;
            self.state.error_cache.replace(error);
        }

        match self.state.behaviour {
            ErrorBehaviour::Interrupt => return Err(RadError::Interrupt),
            ErrorBehaviour::Assert => return Err(RadError::AssertFail),
            // Re-throw error
            // It is not captured in cli but it can be handled by library user.
            ErrorBehaviour::Strict => {
                return Err(RadError::StrictPanic);
            }
            // If purge mode is set, don't print anything
            // and don't print error
            ErrorBehaviour::Purge => (),
            ErrorBehaviour::Lenient => remainder.push_str(&frag.whole_string),
        }

        Ok(())
    }

    // Level is needed for feature debug & hook codes
    fn lex_branch_end_frag_eval_result_ok(
        &mut self,
        content: Option<String>,
        frag: &mut MacroFragment,
        remainder: &mut String,
        _lexor: &mut Lexor,
        #[allow(unused_variables)] level: usize,
    ) -> RadResult<()> {
        // Debug
        // Debug command after macro evaluation
        // This goes to last line and print last line
        #[cfg(feature = "debug")]
        if !self.is_local_macro(level + 1, &frag.name) {
            // Only when macro is not a local macro
            self.check_debug_macro(frag, level)?;
        }

        // If content is none
        // Ignore new line after macro evaluation until any character
        if let Some(mut content) = content {
            // else it is ok to proceed.
            // thus it is safe to unwrap it
            if frag.trimmed {
                content = trim!(&content).to_string();
                if content.is_empty() {
                    self.state.consume_newline = true;
                }
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
                // Macro hook execute
                self.lex_branch_end_invoke(
                    &mut hook_lexor,
                    &mut hook_frag,
                    &mut hook_mainder,
                    level,
                    &frag.name,
                )?;

                // Add hook remainder into current content
                // which will be added to main remainder
                if !hook_mainder.is_empty() {
                    content.push_str(&hook_mainder);
                }
            }

            // NOTE
            // This should come later!!
            // because pipe should respect all other macro attributes
            // not the other way
            if frag.pipe {
                self.state.add_pipe(None, content);
                self.state.consume_newline = true;
            } else {
                remainder.push_str(&content);
            }
        } else {
            self.state.consume_newline = true;
        }
        Ok(())
    }

    fn lex_branch_exit_frag(&mut self, ch: char, frag: &mut MacroFragment, remainder: &mut String) {
        frag.whole_string.push(ch);
        remainder.push_str(&frag.whole_string);
        frag.clear();
        frag.is_processed = true;
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

    pub(crate) fn get_logger_write_option(&self) -> Option<&WriteOption> {
        self.logger.write_option.as_ref()
    }

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
    pub(crate) fn set_temp_file(&mut self, path: &Path) -> RadResult<()> {
        self.state.set_temp_target(path)
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
    /// Sandbox means that current state(cursor) of processor should not be applied for following independent processing
    /// This mostly means loggers lines information is separate from sandboxed input and main input.
    pub(crate) fn set_sandbox(&mut self, sandbox: bool) {
        self.state.sandbox = sandbox;
    }

    #[cfg(not(feature = "wasm"))]
    /// Get temp file's path
    pub(crate) fn get_temp_path(&self) -> &Path {
        self.state.temp_target.name()
    }

    #[cfg(not(feature = "wasm"))]
    /// Get temp file's "file" struct
    pub(crate) fn get_temp_file(&mut self) -> Option<&mut File> {
        Some(self.state.temp_target.inner())
    }

    /// Backup information of current file before processing sandboxed input
    ///
    /// This backup current input source, declared local macros
    fn backup(&self) -> SandboxBackup {
        SandboxBackup {
            current_input: self.state.current_input.clone(),
            local_macro_map: self.map.local.clone(),
        }
    }

    /// Recover backup information into the processor
    fn recover(&mut self, backup: SandboxBackup) -> RadResult<()> {
        // NOTE ::: Set file should come first becuase set_file override line number and character number
        self.logger.set_input(&backup.current_input);
        self.state.current_input = backup.current_input;
        self.map.local = backup.local_macro_map;
        self.logger.stop_last_tracker();

        Ok(())
    }

    /// Log assertion message
    pub(crate) fn track_assertion(&mut self, success: bool) -> RadResult<()> {
        self.logger.alog(success)?;
        Ok(())
    }

    /// Log message
    pub(crate) fn log_message(&mut self, log: &str) -> RadResult<()> {
        self.logger.log(log)?;
        Ok(())
    }

    /// Log error message
    pub(crate) fn log_error(&mut self, log: &str) -> RadResult<()> {
        self.logger.elog(log)?;
        Ok(())
    }

    /// Log warning message without line number
    pub(crate) fn log_warning_no_line(
        &mut self,
        log: &str,
        warning_type: WarningType,
    ) -> RadResult<()> {
        self.logger.wlog_no_line(log, warning_type)?;
        Ok(())
    }

    /// Log warning message
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
            // Input stack should always guarantee that path is canonicalized
            self.state.input_stack.insert(path.canonicalize()?);
            let input = ProcessInput::File(path);
            self.state.current_input = input.clone();
            self.logger.set_input(&input);
            Ok(())
        }
    }

    /// Set input as string not as &path
    ///
    /// This is conceptualy identical to set_file but doesn't validate if given input is existent
    fn set_input_stdin(&mut self) -> RadResult<()> {
        self.state.current_input = ProcessInput::Stdin;
        self.logger.set_input(&ProcessInput::Stdin);
        Ok(())
    }

    /// Check if processor has debug mode enabled
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
    pub(crate) fn set_prompt(&mut self, prompt: &str) {
        self.debugger.set_prompt(prompt);
    }

    /// Set documentation for a macro
    ///
    /// This sets a description for a macro. This will fail silent if macro doesn't exist
    ///
    /// # Return
    ///
    /// - Boolean value represents a success of an operation
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// proc.set_documentation("macro_name", "this is a new macro");
    /// ```
    pub fn set_documentation(&mut self, macro_name: &str, content: &str) -> bool {
        if let Some(mac) = self.map.runtime.get_mut(macro_name, Hygiene::None) {
            mac.desc = Some(content.to_owned());
            true
        } else {
            false
        }
    }

    /// This returns canonicalized absolute path
    ///
    /// It fails when the parent path cannot be canonicalized
    #[cfg(not(feature = "wasm"))]
    pub(crate) fn get_current_dir(&self) -> RadResult<PathBuf> {
        let path = match &self.state.current_input {
            ProcessInput::Stdin => std::env::current_dir()?,
            ProcessInput::File(path) => {
                let path = match path.parent() {
                    Some(empty) if empty == Path::new("") => PathBuf::from("./"),
                    Some(other) => other.to_owned(),
                    None => std::env::current_dir()?,
                };
                path
            }
        };
        Ok(path)
    }

    /// Parse chunk and strip quotes
    ///
    /// This is for internal logic
    pub(crate) fn parse_and_strip(
        &mut self,
        ap: &mut ArgParser,
        level: usize,
        src: &str,
    ) -> RadResult<String> {
        Ok(ap.strip(&self.parse_chunk_args(level, "", src)?))
    }

    /// Get a local macro's raw body
    pub(crate) fn get_local_macro_body(&self, macro_name: &str) -> RadResult<&str> {
        let body = &self
            .map
            .local
            .get(macro_name)
            .ok_or_else(|| RadError::InvalidMacroName("No such macro".to_string()))?
            .body;
        Ok(body)
    }

    /// Get a runtime macro's raw body
    pub(crate) fn get_runtime_macro_body(&self, macro_name: &str) -> RadResult<&str> {
        let body = &self
            .map
            .runtime
            .get(macro_name, self.state.hygiene)
            .ok_or_else(|| RadError::InvalidMacroName("No such macro".to_string()))?
            .body;
        Ok(body)
    }

    // End of miscellaenous methods
    // </MISC>
    // ----------

    // ----------
    // Function that is exposed for better end user's qualify of life
    // <EXT>

    /// Get a macro manual string
    #[cfg(feature = "signature")]
    pub(crate) fn get_macro_manual(&self, macro_name: &str) -> Option<String> {
        self.map.get_signature(macro_name).map(|s| s.to_string())
    }

    /// Try getting a regex or newly compile
    ///
    /// # Return
    ///
    /// this returns reference to a existing or compiled regex
    pub fn try_get_or_insert_regex(&mut self, expression: &str) -> RadResult<&Regex> {
        if self.state.regex_cache.contains(expression) {
            Ok(self.state.regex_cache.get(expression).unwrap())
        } else {
            self.state.regex_cache.append(expression)
        }
    }

    /// Expand chunk and strip quotes
    ///
    /// This is intended for end user
    pub fn expand(&mut self, level: usize, src: &str, strip: bool) -> RadResult<String> {
        let parsed = self.parse_chunk_args(level, "", src)?;
        if strip {
            Ok(ArgParser::new().strip(&parsed))
        } else {
            Ok(parsed)
        }
    }

    /// Print error using a processor's error logger wihtout line information
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// proc.print_error_no_line("Error occured").expect("Failed to write error");
    /// ```
    pub fn print_error_no_line(&mut self, error: &str) -> RadResult<()> {
        self.logger.elog_no_line(error)?;
        Ok(())
    }

    /// Print error using a processor's error logger
    ///
    /// This utilizes logger's line number tracking and colored prompt
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// proc.print_error("Error occured right now").expect("Failed to write error");
    /// ```
    pub fn print_error(&mut self, error: &str) -> RadResult<()> {
        self.log_error(error)?;
        Ok(())
    }

    /// Split arguments as vector
    ///
    /// This is designed for end user
    ///
    /// This doesn't strip literal quotes from vector
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// proc.split_arguments("a,b", 2,false).expect("Failed to split arguments");
    /// ```
    pub fn split_arguments(
        &self,
        source: &str,
        target_length: usize,
        no_strip: bool,
    ) -> RadResult<Vec<String>> {
        let mut ap = ArgParser::new();
        ap.set_strip(!no_strip);
        if let Some(args) = ap.args_with_len(source, target_length) {
            Ok(args)
        } else {
            Err(RadError::InvalidArgument(
                "Insufficient arguments.".to_string(),
            ))
        }
    }

    /// Check auth information
    ///
    /// This exits for internal macro logic.
    ///
    /// This will print log_error if auth was enabled with warning
    ///
    /// # Return
    ///
    /// Whether auth is enabled or not
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// proc.check_auth(r4d::AuthType::CMD).expect("Failed to get auth information");
    /// ```
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

    /// Check if given macro exists
    ///
    /// This exits for internal macro logic.
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// if !proc.contains_macro("name", r4d::MacroType::Runtime) {
    ///     proc.print_error("Error").expect("Failed to write error");
    /// }
    /// ```
    pub fn contains_macro(&self, macro_name: &str, macro_type: MacroType) -> bool {
        self.map
            .contains_macro(macro_name, macro_type, self.state.hygiene)
    }

    /// Check if given local macro exists
    ///
    /// This exits for internal macro logic.
    ///
    /// # Return
    ///
    /// - Return a local_name if exists
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// let level = 2;
    /// if !proc.contains_local_macro("name", 2) {
    ///     proc.print_error("Error").expect("Failed to write error");
    /// }
    /// ```
    pub fn contains_local_macro(&self, mut level: usize, macro_name: &str) -> Option<String> {
        while level > 0 {
            let local_name = Utils::local_name(level, macro_name);
            if self.map.contains_local_macro(&local_name) {
                return Some(local_name);
            }
            level -= 1;
        }
        None
    }

    /// Try undefine a macro
    ///
    /// This exits for internal macro logic.
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// if proc.contains_macro("name", r4d::MacroType::Runtime) {
    ///     proc.undefine_macro("name", r4d::MacroType::Runtime);
    /// }
    /// ```
    pub fn undefine_macro(&mut self, macro_name: &str, macro_type: MacroType) {
        self.map
            .undefine(macro_name, macro_type, self.state.hygiene);
    }

    /// Rename macro
    ///
    /// This exits for internal macro logic.
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// if proc.contains_macro("name", r4d::MacroType::Runtime) {
    ///     proc.rename_macro("name", "new_name",r4d::MacroType::Runtime);
    /// }
    /// ```
    pub fn rename_macro(&mut self, macro_name: &str, target_name: &str, macro_type: MacroType) {
        self.map
            .rename(macro_name, target_name, macro_type, self.state.hygiene);
    }

    /// Append content into a macro
    ///
    /// This exits for internal macro logic.
    ///
    /// This will do nothing if macro doesn't exist
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// if proc.contains_macro("name", r4d::MacroType::Runtime) {
    ///     proc.append_macro("name", "added text");
    /// }
    /// ```
    pub fn append_macro(&mut self, macro_name: &str, target: &str) {
        self.map.append(macro_name, target, self.state.hygiene);
    }

    /// Append content into a local macro
    ///
    /// This exits for internal macro logic.
    ///
    /// This will do nothing if macro doesn't exist
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// if proc.contains_macro("name", r4d::MacroType::Runtime) {
    ///     proc.append_local_macro("name", "added text");
    /// }
    /// ```
    pub fn append_local_macro(&mut self, macro_name: &str, target: &str) {
        self.map.append_local(macro_name, target);
    }

    /// Replace macro's content
    ///
    /// This exits for internal macro logic.
    ///
    /// This will do nothing if macro doesn't exist
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// let level = 2
    /// if proc.contains_local_macro("name", level) {
    ///     proc.replace_macro("name", "new macro content");
    /// }
    /// ```
    pub fn replace_macro(&mut self, macro_name: &str, target: &str) -> bool {
        self.map.replace(macro_name, target, self.state.hygiene)
    }

    /// Add new local macro
    ///
    /// This exits for internal macro logic.
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// proc.add_new_local_macro(0,"a_macro", "macro body");
    /// ```
    pub fn add_new_local_macro(&mut self, level: usize, macro_name: &str, body: &str) {
        self.map.add_local_macro(level, macro_name, body);
    }

    /// Remove local macro
    ///
    /// This exits for internal macro logic.
    ///
    /// This will do nothing if macro doesn't exist
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// proc.remove_local_macro(0,"a_macro");
    /// ```
    pub fn remove_local_macro(&mut self, level: usize, macro_name: &str) {
        self.map.remove_local_macro(level, macro_name);
    }

    /// Check if given text is boolean-able
    ///
    /// This exits for internal macro logic.
    ///
    /// This panics when the value is neither true nor false
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// assert_eq!(false,proc.is_true("0").expect("Failed to convert"));
    /// assert_eq!(true,proc.is_true("true").expect("Failed to convert"));
    /// ```
    pub fn is_true(&self, src: &str) -> RadResult<bool> {
        Utils::is_arg_true(src)
    }

    // </EXT>
    // ----------
}

/// Result of a parsing logic
#[derive(Debug)]
enum ParseResult {
    FoundMacro(String),
    Printable(String),
    NoPrint,
    Eoi,
}

/// Struct for backing current file and logging information
///
/// This is necessary because some macro processing should be executed in sandboxed environment.
/// e.g. when include macro is called, outer file's information is not helpful at all.
struct SandboxBackup {
    current_input: ProcessInput,
    local_macro_map: HashMap<String, LocalMacro>,
}

/// Struct designed to check unbalanced parenthesis
pub(crate) struct UnbalancedChecker {
    paren: usize,
}

impl UnbalancedChecker {
    /// Create an empty checker
    pub fn new() -> Self {
        Self { paren: 0 }
    }

    /// Main logic for checking
    pub fn check(&mut self, ch: char) -> bool {
        match ch {
            '(' => self.paren += 1,
            ')' => {
                if self.paren > 0 {
                    self.paren -= 1;
                } else {
                    return false;
                }
            }
            _ => {
                return true;
            }
        }
        true
    }
}

/// Readable, writeable struct that holds information of runtime macros
#[derive(Serialize, Deserialize)]
pub struct RuleFile {
    pub rules: HashMap<String, RuntimeMacro>,
}

impl RuleFile {
    /// Create an empty rule file from runtime macros
    pub fn new(rules: Option<HashMap<String, RuntimeMacro>>) -> Self {
        if let Some(content) = rules {
            Self { rules: content }
        } else {
            Self {
                rules: HashMap::new(),
            }
        }
    }

    /// Read from rule file and make it into hash map
    pub fn melt(&mut self, path: &Path) -> RadResult<()> {
        Utils::is_real_path(path)?;
        let result = bincode::deserialize::<Self>(&std::fs::read(path)?);
        if let Err(err) = result {
            Err(RadError::BincodeError(format!(
                "Failed to melt the file : {} \n {}",
                path.display(),
                err
            )))
        } else {
            self.rules.extend(result.unwrap().rules.into_iter());
            Ok(())
        }
    }

    /// Melt from byte array not a file
    pub fn melt_literal(&mut self, literal: &[u8]) -> RadResult<()> {
        let result = bincode::deserialize::<Self>(literal);
        if let Ok(rule_file) = result {
            self.rules.extend(rule_file.rules.into_iter());
            Ok(())
        } else {
            Err(RadError::BincodeError(
                "Failed to melt the literal value".to_string(),
            ))
        }
    }

    /// Convert runtime rules into a single binary file
    pub(crate) fn freeze(&self, path: &std::path::Path) -> RadResult<()> {
        let result = bincode::serialize(self);
        if result.is_err() {
            Err(RadError::BincodeError(format!(
                "Failed to freeze to a file : {}",
                path.display()
            )))
        } else if std::fs::write(path, result.unwrap()).is_err() {
            Err(RadError::InvalidArgument(format!(
                "Failed to create a file : {}",
                path.display()
            )))
        } else {
            Ok(())
        }
    }

    /// Serialize a rule file into a byte array
    pub(crate) fn serialize(&self) -> RadResult<Vec<u8>> {
        let result = bincode::serialize(self);
        if result.is_err() {
            return Err(RadError::BincodeError(
                "Failed to serialize a rule".to_string(),
            ));
        }
        Ok(result.unwrap())
    }
}
