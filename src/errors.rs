use std::{fmt::Display, str::Utf8Error, string::FromUtf8Error, time::SystemTimeError};

use rustpython_parser::ParseError;

#[derive(Debug)]
pub enum FztError {
    IoError(std::io::Error),
    StringFromUtf8(FromUtf8Error),
    StringParsing(Utf8Error),
    GeneralParsingError(String),
    PythonParsingError(ParseError),
    TimeError(SystemTimeError),
    DictionaryWalking(walkdir::Error),
    Regex(regex::Error),
    Json(serde_json::Error),
    UserError(String),
    JavaParser(String),
    PythonParser(String),
    RustParser(syn::Error),
    InvalidArgument(String),
    RustError(String),
    PythonError(String),
    InternalError(String),
    RuntumeError(String),
}

impl std::error::Error for FztError {}

impl Display for FztError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FztError::IoError(io_error) => write!(f, "{}", io_error),
            FztError::StringParsing(utf8_error) => write!(f, "{}", utf8_error),
            FztError::GeneralParsingError(error) => write!(f, "{}", error),
            FztError::TimeError(system_time_error) => write!(f, "{}", system_time_error),
            FztError::PythonParsingError(base_error) => write!(f, "{}", base_error),
            FztError::DictionaryWalking(error) => write!(f, "{}", error),
            FztError::Regex(error) => write!(f, "{}", error),
            FztError::Json(error) => write!(f, "{}", error),
            FztError::UserError(error) => write!(f, "{}", error),
            FztError::JavaParser(error) => write!(f, "{}", error),
            FztError::PythonParser(error) => write!(f, "{}", error),
            FztError::RustParser(syn_error) => write!(f, "{}", syn_error),
            FztError::InvalidArgument(error) => write!(f, "{}", error),
            FztError::RustError(error) => write!(f, "{}", error),
            FztError::PythonError(error) => write!(f, "{}", error),
            FztError::StringFromUtf8(error) => write!(f, "{}", error),
            FztError::InternalError(error) => write!(f, "{}", error),
            FztError::RuntumeError(error) => write!(f, "{}", error),
        }
    }
}

impl From<std::io::Error> for FztError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<Utf8Error> for FztError {
    fn from(err: Utf8Error) -> Self {
        Self::StringParsing(err)
    }
}

impl From<SystemTimeError> for FztError {
    fn from(value: SystemTimeError) -> Self {
        Self::TimeError(value)
    }
}

impl From<ParseError> for FztError {
    fn from(value: ParseError) -> Self {
        Self::PythonParsingError(value)
    }
}

impl From<walkdir::Error> for FztError {
    fn from(value: walkdir::Error) -> Self {
        Self::DictionaryWalking(value)
    }
}

impl From<regex::Error> for FztError {
    fn from(value: regex::Error) -> Self {
        Self::Regex(value)
    }
}

impl From<serde_json::Error> for FztError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<syn::Error> for FztError {
    fn from(value: syn::Error) -> Self {
        Self::RustParser(value)
    }
}

impl From<FromUtf8Error> for FztError {
    fn from(value: FromUtf8Error) -> Self {
        Self::StringFromUtf8(value)
    }
}
