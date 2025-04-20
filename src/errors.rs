use std::{fmt::Display, str::Utf8Error, time::SystemTimeError};

use rustpython_parser::ParseError;

#[derive(Debug)]
pub enum FztError {
    IoError(std::io::Error),
    StringParsing(Utf8Error),
    GeneralParsingError(String),
    PythonParsingError(ParseError),
    TimeError(SystemTimeError),
    DictionaryWalking(walkdir::Error),
    Regex(regex::Error),
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
