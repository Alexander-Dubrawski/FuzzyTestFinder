use crate::errors::FztError;

pub mod python;

pub trait Runner {
    fn run(&self, history: bool, last: bool, verbose: bool, debug: bool) -> Result<(), FztError>;
    fn clear_cache(&self) -> Result<(), FztError>;
    fn clear_history(&self) -> Result<(), FztError>;
}
