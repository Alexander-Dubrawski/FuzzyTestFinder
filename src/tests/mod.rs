use crate::errors::FztError;

pub mod java;
pub mod python;

pub trait Test {
    fn runtime_argument(&self) -> String;
    fn search_item_name(&self) -> String;
}

pub trait Tests {
    fn to_json(&self) -> Result<String, FztError>;
    fn tests(self) -> Vec<impl Test>;
    fn update(&mut self, only_check_for_update: bool) -> Result<bool, FztError>;
}
