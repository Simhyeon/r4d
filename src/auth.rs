//! Authorization(Permission)
//!
//! Permission should be given for some function macro types

use crate::consts::LINE_ENDING;
use std::fmt::Write;

#[derive(Debug)]
/// Struct that stores auth states
pub(crate) struct AuthFlags {
    auths: Vec<AuthState>,
}

impl AuthFlags {
    pub fn new() -> Self {
        let mut auths = Vec::new();
        for _ in 0..AuthType::LEN as usize {
            auths.push(AuthState::Restricted);
        }

        Self { auths }
    }

    pub fn set_state(&mut self, auth_type: &AuthType, auth_state: AuthState) {
        self.auths[*auth_type as usize] = auth_state;
    }

    pub fn get_state(&self, auth_type: &AuthType) -> &AuthState {
        &self.auths[*auth_type as usize]
    }

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

    pub fn clear(&mut self) {
        self.auths.clear();
    }
}

#[derive(Debug, Clone, Copy)]
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

impl AuthType {
    /// Convert str slice into AuthType
    pub fn from(string: &str) -> Option<Self> {
        match string.to_lowercase().as_str() {
            "env" => Some(Self::ENV),
            "fin" => Some(Self::FIN),
            "fout" => Some(Self::FOUT),
            "cmd" => Some(Self::CMD),
            _ => None,
        }
    }

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
