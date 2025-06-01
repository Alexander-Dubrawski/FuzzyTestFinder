use crate::errors::FztError;

pub mod java;
pub mod python;

pub trait Test {
    fn runtime_argument(&self) -> String;
    fn name(&self) -> String;
}

pub trait Tests {
    fn to_json(&self) -> Result<String, FztError>;
    fn tests(self) -> Vec<impl Test>;
    fn update(&mut self) -> Result<bool, FztError>;
}
