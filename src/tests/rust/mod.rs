use crate::errors::FztError;

mod helper;
pub mod mod_resolver;
pub mod rust_test;
pub mod rust_test_parser;

pub trait ParseRustTest {
    fn parse_tests(&self) -> Result<Vec<(Vec<String>, String)>, FztError>;
}
