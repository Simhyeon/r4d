//! Authorization(Permission)
//!
//! Permission should be given for some function macro types
//!
//! Currently there are four types of authorization
//!
//! - fin
//! - fout
//! - cmd
//! - env

use crate::consts::LINE_ENDING;
use serde::{Deserialize, Serialize};

use crate::RadError;
use std::fmt::Write;

#[derive(Debug)]
/// Struct that stores auth states
pub(crate) struct AuthFlags {
    auths: Vec<AuthState>,
}

impl AuthFlags {
    /// Create a new instance
    pub fn new() -> Self {
        let mut auths = Vec::new();
        for _ in 0..AuthType::LEN as usize {
            auths.push(AuthState::Restricted);
        }

        Self { auths }
    }

    /// Set auth state
    pub fn set_state(&mut self, auth_type: &AuthType, auth_state: AuthState) {
        self.auths[*auth_type as usize] = auth_state;
    }

    /// Get auth state
    pub fn get_state(&self, auth_type: &AuthType) -> &AuthState {
        &self.auths[*auth_type as usize]
    }

    /// Get auth state but in string
    pub fn get_status_string(&self) -> Option<String> {
        let mut format = String::new();
        for index in 0..AuthType::LEN as usize {
            let auth_type = AuthType::from_usize(index).unwrap();
            match self.get_state(&auth_type) {
                &AuthState::Warn | &AuthState::Open => {
                    // Add newline before
                    format.push_str(LINE_ENDING);
                    // This is mostly ok since, auth_type is always valid utf8 character
                    write!(format, "Auth : {:?} is open.", auth_type).ok();
                }
                &AuthState::Restricted => (),
            }
        }
        if !format.is_empty() {
            Some(format)
        } else {
            None
        }
    }

    /// Clear all auth states
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
/// Authorization type
///
/// ```text
/// Each variants means
/// - ENV  : environment variable permission
/// - FIN  : File in(read) permission
/// - FOUT : File out(write) permission
/// - CMD  : System command permission
/// - LEN  : This is a functional variant not a real value, namely a length
/// ```
pub enum AuthType {
    /// Environment variable permission
    ENV = 0,
    /// File in(read) permission
    FIN = 1,
    /// File out(write) permission
    FOUT = 2,
    /// System command permission
    CMD = 3,
    /// This is a functional variant not a real value
    LEN = 4,
}

impl std::fmt::Display for AuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::ENV => "ENV",
            Self::FIN => "FIN",
            Self::FOUT => "FOUT",
            Self::CMD => "CMD",
            Self::LEN => "LEN",
        };

        write!(f, "{}", string)
    }
}

impl std::str::FromStr for AuthType {
    type Err = RadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "env" => Ok(Self::ENV),
            "fin" => Ok(Self::FIN),
            "fout" => Ok(Self::FOUT),
            "cmd" => Ok(Self::CMD),
            _ => Err(RadError::InvalidArgument(format!(
                "Given type \"{}\" is not a valid auth type",
                s
            ))),
        }
    }
}

impl AuthType {
    /// Convert usize integer into a auth type
    pub fn from_usize(number: usize) -> Option<Self> {
        match number {
            0 => Some(Self::ENV),
            1 => Some(Self::FIN),
            2 => Some(Self::FOUT),
            3 => Some(Self::CMD),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
/// Current state authorization
pub(crate) enum AuthState {
    /// Not allowed
    Restricted,
    /// Allowed but wans user
    Warn,
    /// Openly allowed
    Open,
}
