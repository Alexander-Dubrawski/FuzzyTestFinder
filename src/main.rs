use FzT::{cli::cli_parser::parse_cli, errors::FztError};

fn main() -> Result<(), FztError> {
    let mut runner = parse_cli()?;
    runner.run()
}
