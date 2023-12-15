//! State struct of a processing

use crate::auth::AuthFlags;
#[cfg(not(feature = "wasm"))]
use crate::common::FileTarget;
use crate::common::RadResult;
use crate::common::{
    CommentType, ErrorBehaviour, FlowControl, Hygiene, ProcessInput, ProcessType, RelayTarget,
};
use crate::consts::LINE_ENDING;
use crate::RadError;
use regex::Regex;
use std::collections::{HashMap, HashSet};
#[cfg(not(feature = "wasm"))]
use std::path::Path;
use std::path::PathBuf;

/// Processors processing state
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
    pub pipe_map: HashMap<String, String>,
    pub relay: Vec<RelayTarget>,
    pub sandbox: bool,
    pub behaviour: ErrorBehaviour,
    pub process_type: ProcessType,
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
    pub lexor_escape_blanks: bool,
}

impl ProcessorState {
    /// Create a new instance
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
            process_type: ProcessType::Expand,
            comment_type: CommentType::None,
            sandbox: false,
            #[cfg(not(feature = "wasm"))]
            temp_target: FileTarget::with_truncate(&std::env::temp_dir().join("rad.txt")).unwrap(),
            comment_char: None,
            macro_char: None,
            flow_control: FlowControl::None,
            deny_newline: false,
            consume_newline: false,
            escape_newline: false,
            queued: vec![],
            regex_cache: RegexCache::new(),
            lexor_escape_blanks: false,
        }
    }

    #[cfg(not(feature = "wasm"))]
    /// Internal method for setting temp target
    pub(crate) fn set_temp_target(&mut self, path: &Path) -> RadResult<()> {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        let new_target = FileTarget::with_truncate(path)?;
        self.temp_target = new_target;
        Ok(())
    }

    /// Add a pipe with name
    pub fn add_pipe(&mut self, name: Option<&str>, value: String) {
        if let Some(name) = name {
            self.pipe_map.insert(name.to_owned(), value);
        } else {
            self.pipe_map.insert("-".to_owned(), value);
        }
    }

    /// Get a pipe with key
    pub fn get_pipe(&mut self, key: &str, ignore_truncate: bool) -> Option<String> {
        if self.pipe_truncate && !ignore_truncate {
            self.pipe_map.remove(key)
        } else {
            self.pipe_map.get(key).map(|s| s.to_owned())
        }
    }
}

/// Cache for regex compilation
pub(crate) struct RegexCache {
    cache: HashMap<String, Regex>,
    register: HashMap<String, Regex>,
}

impl RegexCache {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            register: HashMap::new(),
        }
    }

    /// Check if cache contains a key
    pub fn contains(&self, name: &str) -> bool {
        self.cache.contains_key(name)
    }

    /// Register a regex
    ///
    /// Registered regex is not cleared
    pub fn register(&mut self, name: &str, source: &str) -> RadResult<()> {
        self.cache.insert(name.to_string(), Regex::new(source)?);
        Ok(())
    }

    /// Append a regex to cache
    pub fn append(&mut self, src: &str) -> RadResult<&Regex> {
        // Set hard capacity of 100
        if self.cache.len() > 100 {
            self.cache.clear();
        }
        self.cache.insert(src.to_string(), Regex::new(src)?);
        Ok(self.get(src).unwrap())
    }

    /// Get a regex with name
    pub fn get(&self, src: &str) -> Option<&Regex> {
        if self.register.get(src).is_some() {
            self.register.get(src)
        } else {
            self.cache.get(src)
        }
    }
}
