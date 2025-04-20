use crate::errors::FztError;

pub mod python;

pub trait Runner {
    fn run(&self) -> Result<(), FztError>;
}
