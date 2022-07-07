use crate::auth::{AuthFlags, AuthState, AuthType};
#[cfg(feature = "debug")]
use crate::debugger::DebugSwitch;
#[cfg(feature = "debug")]
use crate::debugger::Debugger;
use crate::error::RadError;
#[cfg(feature = "hook")]
use crate::hookmap::{HookMap, HookType};
use crate::lexor::*;
use crate::logger::{Logger, LoggerLines, WarningType};
#[cfg(feature = "debug")]
use crate::models::DiffOption;
#[cfg(not(feature = "wasm"))]
use crate::models::FileTarget;
use crate::models::RegexCache;
#[cfg(feature = "signature")]
use crate::models::SignatureType;
use crate::models::{
    CommentType, ErrorBehaviour, ExtMacroBuilder, ExtMacroType, FlowControl, Hygiene, LocalMacro,
    MacroFragment, MacroMap, MacroType, ProcessInput, RelayTarget, RuleFile, UnbalancedChecker,
    WriteOption,
};
use crate::models::{RadStorage, StorageOutput};
use crate::runtime_map::RuntimeMacro;
#[cfg(feature = "signature")]
use crate::sigmap::SignatureMap;
use crate::trim;
use crate::utils::Utils;
use crate::DefineParser;
use crate::{consts::*, RadResult};
use crate::{ArgParser, GreedyState};
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
    pub error_cache: Option<RadError>,
    // This is reserved for hygienic execution
    pub hygiene: Hygiene,
    pub pipe_truncate: bool,
    pipe_map: HashMap<String, String>,
    pub relay: Vec<RelayTarget>,
    pub sandbox: bool,
    pub behaviour: ErrorBehaviour,
    pub comment_type: CommentType,
    // Temp target needs to save both path and file
    // because file doesn't necessarily have path.
    // Especially in unix, this is not so an unique case
    #[cfg(not(feature = "wasm"))]
    pub temp_target: FileTarget,
    pub comment_char: Option<char>,
    pub macro_char: Option<char>,
    pub flow_control: FlowControl,
    pub deny_newline: bool,    // This deny next-next newline
    pub consume_newline: bool, // This consumes newline if the line was only empty
    pub escape_newline: bool,  // This escapes right next newline
    pub queued: Vec<String>,
    pub regex_cache: RegexCache,
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
            error_cache: None,
            hygiene: Hygiene::None,
            relay: vec![],
            behaviour: ErrorBehaviour::Strict,
            comment_type: CommentType::None,
            sandbox: false,
            #[cfg(not(feature = "wasm"))]
            temp_target: FileTarget::empty(),
            comment_char: None,
            macro_char: None,
            flow_control: FlowControl::None,
            deny_newline: false,
            consume_newline: false,
            escape_newline: false,
            queued: vec![],
            regex_cache: RegexCache::new(),
        }
    }

    #[cfg(not(feature = "wasm"))]
    /// Internal method for setting temp target
    pub(crate) fn set_temp_target(&mut self, path: &Path) -> RadResult<()> {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        self.temp_target.set_path(path);
        Ok(())
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
    write_option: WriteOption<'processor>,
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

        // You cannot use filesystem in wasm target
        // Since, temp_dir is always present it can't fail, therefore unwrap is safe
        #[cfg(not(feature = "wasm"))]
        {
            state
                .set_temp_target(&std::env::temp_dir().join("rad.txt"))
                .unwrap();
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
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&target_file);

        if let Ok(file) = file {
            self.write_option = WriteOption::File(file);
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
            .open(&target_file);

        if let Ok(file) = file {
            self.logger = Logger::new();
            self.logger.set_write_option(Some(WriteOption::File(file)));
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
            self.logger.assert();
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

    // This is not included in docs.rs documentation becuase... is it necessary?
    // I don't really remember when this method was added at the first time.
    /// Melt rule as literal input source, or say from byte array
    ///
    /// This always melt file into non-volatile form, which means hygiene doesn't affect melted
    /// macros.
    ///
    /// ```rust
    /// let source = b"Some frozen macro definition";
    /// let proc = r4d::Processor::empty()
    ///     .rule_literal(source);
    /// ```
    pub fn rule_literal(mut self, literal: &[u8]) -> RadResult<Self> {
        let mut rule_file = RuleFile::new(None);
        rule_file.melt_literal(literal)?;
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
            self.log_warning(&status_with_header, WarningType::Security)?;
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

        // Set logger's variable to fresh state to prevent confusion on line numbers
        self.logger.reset_everything();

        if self.state.input_stack.len() == 1 {
            // Warn unterminated relaying
            if !self.state.relay.is_empty() {
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

        // Clear input stack
        // This is necessary because operation can be contiguous
        self.state.input_stack.clear();

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
            let err = RadError::StrictPanic(format!(
                "Cannot register macros : \"{:?}\" in aseptic mode",
                rules.iter().map(|(s, _, _)| s.as_ref()).collect::<Vec<_>>()
            ));
            if self.state.behaviour == ErrorBehaviour::Strict {
                return Err(err);
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
            let err = RadError::StrictPanic(format!(
                "Cannot register macros : \"{:?}\" in aseptic mode",
                rules.iter().map(|(s, _)| s.as_ref()).collect::<Vec<_>>()
            ));
            self.log_strict(&err.to_string(), WarningType::Security)?;
            if self.state.behaviour == ErrorBehaviour::Strict {
                return Err(err);
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
        // Set name as string
        self.set_input_stdin()?;

        let mut reader = content.as_bytes();
        self.process_buffer(&mut reader, None, false)?;
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
            //let result = self.parse_line(&mut line_iter, &mut lexor, &mut frag)?;
            let result = match self.parse_line(&mut line_iter, &mut lexor, &mut frag) {
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
    ///
    /// This is used when step debug command is to be executed.
    /// Without chekcing locality, step will go inside local binding macros
    #[cfg(feature = "debug")]
    fn is_local(&self, mut level: usize, name: &str) -> bool {
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
            self.debugger.user_input_on_macro(frag, &mut self.logger)?;
        } else {
            self.debugger.user_input_on_step(frag, &mut self.logger)?;
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
                FlowControl::Exit => {
                    return Ok(ParseResult::Eoi);
                }
                FlowControl::None => (),
            }

            // Save to original
            #[cfg(feature = "debug")]
            self.debugger.write_to_original(&line)?;

            let remainder = self.parse(lexor, frag, &line, 0, MAIN_CALLER)?;

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
                    // Frag is empty
                    // Print everything
                    let mut remainder = remainder.as_str();

                    // Consume a newline only when Macro execution was not "multiline".
                    // e.g )
                    // 1 define(test=a
                    // b)
                    // 2
                    // !==
                    // 1 2
                    // ===
                    // 1
                    // 2
                    if self.state.consume_newline {
                        if remainder.ends_with('\n') || remainder.ends_with("\r\n") {
                            remainder = remainder.trim_end();
                        }
                        self.state.consume_newline = false;
                    }

                    if self.state.escape_newline {
                        if remainder.ends_with("\r\n") {
                            remainder = remainder.strip_suffix("\r\n").unwrap();
                        } else if remainder.ends_with('\n') {
                            remainder = remainder.strip_suffix('\n').unwrap();
                        }
                        self.state.escape_newline = false;
                    }

                    Ok(ParseResult::Printable(remainder.to_string()))
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
        let backup = self.logger.backup_lines();
        self.logger.set_chunk(true);
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
            let mut line_result = self.parse(&mut lexor, &mut frag, &line, level, caller)?;

            // Escape new line
            if self.state.escape_newline {
                self.state.escape_newline = false;
                if let Some(line) = line_result.strip_suffix("\r\n") {
                    line_result = line.to_owned();
                } else if let Some(line) = line_result.strip_suffix('\n') {
                    line_result = line.to_owned();
                };
            }
            result.push_str(&line_result);

            self.logger.add_line_number();
        }

        // If unexpanded texts remains
        // Add to result
        if !frag.whole_string.is_empty() {
            result.push_str(&frag.whole_string);
        }

        self.logger.set_chunk(false);
        self.logger.recover_lines(backup);
        Ok(result)
    } // parse_chunk_lines end

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

        // Check comment line
        // If it is a comment then return nothing and write nothing
        if self.state.comment_type != CommentType::None
            && line.trim().starts_with(self.get_comment_char())
        {
            return Ok(String::new());
        }

        for ch in line.chars() {
            self.logger.add_char_number();

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
                    if !hook_mainder.is_empty() {
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
        let (name, raw_args) = (&frag.name, &frag.args);

        let args: String;
        // Preprocess only when macro is not a deterred macro
        if !self.map.is_deterred_macro(name) {
            if frag.trim_input {
                let new_args = raw_args
                    .lines()
                    .map(|l| l.trim())
                    .collect::<Vec<_>>()
                    .join(&self.state.newline);
                args = self.parse_chunk_args(level, name, new_args.trim())?;
            } else {
                args = self.parse_chunk_args(level, name, raw_args)?;
            };
            // This parses and processes arguments
            // and macro should be evaluated after

            // Also update original arguments for better debugging
            #[cfg(feature = "debug")]
            {
                frag.processed_args = args.clone();
            }
        } else {
            // Even if deterred macro should
            // respect input_trim
            if frag.trim_input {
                args = raw_args
                    .lines()
                    .map(|l| l.trim())
                    .collect::<Vec<_>>()
                    .join(&self.state.newline)
                    .trim()
                    .to_string();
            } else {
                args = raw_args.to_string();
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

        // Find local macro
        // The macro can be  the one defined in parent macro
        let mut temp_level = level;
        while temp_level > 0 {
            if let Some(local) = self.map.local.get(&Utils::local_name(temp_level, name)) {
                return Ok(Some(local.body.to_owned()));
            }
            temp_level -= 1;
        }

        // Find runtime macro
        // runtime macro comes before function macro so that
        // user can override it
        if self.map.runtime.contains(name, self.state.hygiene) {
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
                let final_result = func(&args, level, self)?;
                // TODO
                // Make parse logic consistent, not defined by delarator
                // result = self.parse_chunk_args(level, caller, &result)?;
                return Ok(final_result);
            }
        }

        // Find function macro
        if self.map.function.contains(name) {
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
            Err(err)
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
                return Err(err);
            };

        for (idx, arg_type) in arg_types.iter().enumerate() {
            //Set arg to be substitued
            self.map.add_local_macro(level + 1, arg_type, &args[idx]);
        }

        // Process the rule body
        // NOTE
        // Previously, this was parse_chunk_body
        let result = self.parse_chunk_args(level, name, &rule.body)?;

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
                let err = RadError::StrictPanic(format!(
                    "Can't override exsiting macro : \"{}\"",
                    mac_name
                ));
                return Err(err);
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

        // Save to container if it has value then return
        // **IMPORTANT**
        // However this doesn't have priority over relaying
        if let Some(cont) = container.as_mut() {
            if self.state.relay.is_empty() {
                cont.push_str(content);
                return Ok(());
            }
        }

        // Redirect to cache if set
        if let Some(cache) = &mut self.cache_file {
            cache.write_all(content.as_bytes())?;
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
                    file.write_all(content.as_bytes())?;
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
            .user_input_before_macro(frag, &mut self.logger)?;

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
                if frag.name.is_empty() {
                    self.logger.freeze_number();
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
                self.state.consume_newline = true;
                let err = RadError::StrictPanic(format!(
                    "Cannot register a macro : \"{}\" in aseptic mode",
                    frag.name
                ));
                if self.state.behaviour == ErrorBehaviour::Strict {
                    return Err(err);
                } else {
                    self.log_warning(&err.to_string(), WarningType::Security)?;
                }
            } else {
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
                ErrorBehaviour::Assert => return Err(RadError::AssertFail),
                // Re-throw error
                // It is not captured in cli but it can be handled by library user.
                ErrorBehaviour::Strict => {
                    return Err(RadError::StrictPanic(
                        "Every error is panicking in strict mode".to_string(),
                    ));
                }
                // If purge mode is set, don't print anything
                // and don't print error
                ErrorBehaviour::Purge => (),
                ErrorBehaviour::Lenient => remainder.push_str(&frag.whole_string),
            }
        }
        self.state.consume_newline = true;
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
        if frag.name.is_empty() {
            let err =
                RadError::InvalidMacroName("Cannot invoke a macro with empty name".to_string());

            // Handle empty name error
            match self.state.behaviour {
                ErrorBehaviour::Assert => return Err(RadError::AssertFail),
                ErrorBehaviour::Strict => return Err(err), // Error
                ErrorBehaviour::Lenient => remainder.push_str(&frag.whole_string),
                ErrorBehaviour::Purge => (),
            }

            // Clear fragment regardless
            frag.clear();
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
            if frag.name.is_empty() {
                self.state.consume_newline = true;
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
        if self.state.error_cache.is_none() {
            self.log_error(&error.to_string())?;
            self.state.error_cache.replace(error);
        }
        match self.state.behaviour {
            ErrorBehaviour::Assert => return Err(RadError::AssertFail),
            // Re-throw error
            // It is not captured in cli but it can be handled by library user.
            ErrorBehaviour::Strict => {
                return Err(RadError::StrictPanic(
                    "Every error is panicking in strict mode".to_string(),
                ));
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
        if !self.is_local(level + 1, &frag.name) {
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
        &self.state.temp_target.path
    }

    #[cfg(not(feature = "wasm"))]
    /// Get temp file's "file" struct
    pub(crate) fn get_temp_file(&mut self) -> Option<&mut File> {
        self.state.temp_target.file.as_mut()
    }

    /// Backup information of current file before processing sandboxed input
    ///
    /// This backup current input source, declared local macros, logger lines information
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

    /// This prints error if strict mode else print warning
    pub(crate) fn log_strict(&mut self, log: &str, warning_type: WarningType) -> RadResult<()> {
        if self.state.behaviour == ErrorBehaviour::Strict {
            self.logger.elog(log)?;
        } else {
            self.logger.wlog(log, warning_type)?;
        }
        Ok(())
    }

    /// Log message
    pub(crate) fn log_message(&mut self, log: &str) -> RadResult<()> {
        self.logger.log(log)?;
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

    #[cfg(feature = "signature")]
    pub(crate) fn get_macro_manual(&self, macro_name: &str) -> Option<String> {
        self.map.get_signature(macro_name).map(|s| s.to_string())
    }

    /// This returns canonicalized absolute path
    ///
    /// It fails when the parent path cannot be canonicalized
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
    /// This is for internal macros logics
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// proc.get_split_arguments(2, "a,b").expect("Failed to split arguments");
    /// ```
    pub fn get_split_arguments(
        &self,
        target_length: usize,
        source: &str,
    ) -> RadResult<Vec<String>> {
        if let Some(args) = ArgParser::new().args_with_len(source, target_length) {
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

    /// Expand given text
    ///
    /// This exits for internal macro logic.
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// proc.expand(0, "argument").expect("Failed to expand a macro argument");
    /// ```
    pub fn expand(&mut self, level: usize, source: impl AsRef<str>) -> RadResult<String> {
        self.parse_chunk_args(level, MAIN_CALLER, source.as_ref())
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

    /// Replace macro's content
    ///
    /// This exits for internal macro logic.
    ///
    /// This will do nothing if macro doesn't exist
    ///
    /// ```rust
    /// let mut proc = r4d::Processor::new();
    /// if proc.contains_macro("name", r4d::MacroType::Runtime) {
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

    pub(crate) fn get_runtime_macro_body(&self, macro_name: &str) -> RadResult<&str> {
        let body = &self
            .map
            .runtime
            .get(macro_name, self.state.hygiene)
            .unwrap()
            .body;
        Ok(body)
    }

    // </EXT>
    // ----------
}

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
    logger_lines: LoggerLines,
}
