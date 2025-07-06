use crate::errors::FztError;

pub mod mod_resolver;
pub mod rust_test;
pub mod rust_test_parser;

pub trait ParseRustTest {
    fn parse_tests() -> Result<Vec<(Vec<String>, String)>, FztError>;
}
